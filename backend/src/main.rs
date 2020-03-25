#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

mod database;

use rocket_contrib::{json::Json, serve::StaticFiles};
use rocket::{response::{status, NamedFile}, State, http::{Cookie, Status, Cookies}};
use rocket::response::Responder;
use json_structs::*;
use dotenv::dotenv;
use postgres::{Client, NoTls};

use database::*;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc};
use std::env;
use std::thread;

struct ArgonSecretKey(String);

// Returns a path to the directory where the frontend files are located
fn frontend_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().join("frontend/")
}

/// The error type used throughout the binary
#[derive(Debug)]
pub enum Error {
    IoErr(std::io::Error),
    /// An error encountered while working with password hashes
    HashError(argonautica::Error),
    PostgresErr(postgres::error::Error),
    JsonErr(serde_json::Error),
    OrangeZestErr(orange_zest::Error),
    /// An attempt was made to create a user given details that already exist
    UserAlreadyExists,
    /// The backend did not have the auth tokens to do scraping for the requested user with
    ScAuthTokensNotPresent,
    /// Could not log in with the given `LoginInfo`
    LoginFailed
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _req: &rocket::request::Request) -> rocket::response::Result<'r> {
            eprintln!("Response was a non-`Responder` `Err`: {:?}.", self);
            Err(Status::InternalServerError)
    }
}

impl From<argonautica::Error> for Error {
    fn from(err: argonautica::Error) -> Self {
        Self::HashError(err)
    }
}

impl From<postgres::error::Error> for Error {
    fn from(err: postgres::error::Error) -> Self {
        Self::PostgresErr(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonErr(err)
    }
}

impl From<orange_zest::Error> for Error {
    fn from(err: orange_zest::Error) -> Self {
        Self::OrangeZestErr(err)
    }
}

/// Route used to create a new user
// TODO: right now my error type does not implement responder so returning an error
// here returns a 500 to the client and logs the error to the console
#[post("/register", format = "json", data = "<reg_info>")]
fn register(
    mut cookies: Cookies,
    reg_info: Json<RegisterInfo>,
    db: State<DbClient>,
    key: State<ArgonSecretKey>
) -> Result<Json<UserInfo>, Error> {
    let mut client = db.lock().unwrap();
    let user_id = User::create_new(&mut client, &reg_info, &key.0)?;

    let user_info = UserInfo {
        user_id: user_id,
        username: reg_info.username.clone()
    };

    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
    Ok(Json(user_info))
}

#[post("/login", format = "json", data = "<login_info>")]
fn login(
    mut cookies: Cookies,
    login_info: Json<LoginInfo>,
    db: State<DbClient>,
    key: State<ArgonSecretKey>
) -> Result<Json<UserInfo>, Error> {
    let mut client = db.lock().unwrap();
    let user = User::load_username(&mut client, &login_info.username)?;

    if user.auth(&login_info, &key.0)? {
        cookies.add_private(Cookie::new("user_id", user.user_id.to_string()));
        Ok(Json(user.into()))
    } else {
        Err(Error::LoginFailed)
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Status {
    cookies.remove_private(Cookie::named("user_id"));
    Status::Ok
}

#[get("/me")]
fn me_authed(user: User) -> Json<UserInfo> {
    Json(user.into())
}

#[get("/me", rank = 2)]
fn me() -> status::Custom<()> {
    status::Custom(Status::Unauthorized, ())
}

#[get("/<_whatever..>", rank = 20)]
fn not_logged_in_get(_whatever: std::path::PathBuf) -> status::Custom<()> {
    status::Custom(Status::Unauthorized, ())
}

#[post("/<_whatever..>", rank = 20)]
fn not_logged_in_post(_whatever: std::path::PathBuf) -> status::Custom<()> {
    status::Custom(Status::Unauthorized, ())
}

/// A "catch-all" to redirect path requests to the index since we are building a SPA
// TODO: if you need requests to be directed at a different file, change this accordingly
#[catch(404)]
fn not_found() -> NamedFile {
    NamedFile::open(frontend_dir().join("index.html")).unwrap()
}

/// Route used to provide auth credentials (OAuth token and Client ID).
///
/// You have to be logged in with an account to access this route (applies to
/// any route with a `User` parameter).
#[post("/auth-creds", format = "json", data = "<auth_creds>")]
fn auth_creds(user: User, db: State<DbClient>, auth_creds: Json<AuthCredentials>) -> Result<(), Error> {
    let mut client = db.lock().unwrap();
    user.store_sc_credentials(&mut client, &auth_creds)
}

/// Tell the backend do scrape all available data from SoundCloud for the logged-in
/// user.
///
/// Make sure you have posted valid credentials to `/auth-creds` before you post
/// to this route.
///
/// The `num_recent_*` query parameters allow you to specify how many recent likes
/// or playlists should be scraped. For instance, a request such as
/// `/do-scraping?num_recent_likes=5&num_recent_playlists=0` will scrape 5 likes
/// and will not scrape playlists at all.
///
/// If the query parameters are not specified, this route scrapes all available
/// info.
#[post("/do-scraping?<num_recent_likes>&<num_recent_playlists>")]
fn do_scraping(
    user: User,
    db: State<DbClient>,
    num_recent_likes: Option<u64>,
    num_recent_playlists: Option<u64>
) -> Result<(), Error> {
    let db = db.clone();
    let num_recent_likes = num_recent_likes.unwrap_or(std::u64::MAX);
    let num_recent_playlists = num_recent_playlists.unwrap_or(std::u64::MAX);

    if let (Some(oauth_token), Some(client_id)) = (user.sc_oauth_token, user.sc_client_id) {
        thread::spawn(move || -> Result<(), Error> {
            let zester = orange_zest::Zester::new(oauth_token, client_id)?;

            let likes = zester.likes(num_recent_likes, |_| {})?;
            let playlists = zester.playlists(num_recent_playlists, |_| {})?;

            {
                let mut conn = db.lock().unwrap();

                for track in likes.collections.iter().map(|c| &c.track) {
                    Track::from(track).create_new(
                        &mut conn,
                        &SoundCloudUser::from(track.user.as_ref().unwrap())
                    )?;
                }

                for playlist in playlists.playlists.iter() {
                    Playlist::from(playlist).create_new(
                        &mut conn,
                        &SoundCloudUser::from(playlist.user.as_ref().unwrap()),
                        &playlist.tracks.as_ref().unwrap().into_iter().map(|t| Track::from(t)).collect()
                    )?;
                }
            }

            Ok(())
        });

        Ok(())
    } else {
        Err(Error::ScAuthTokensNotPresent)
    }
}

/// Create a Rocket instance given a PostgreSQL client.
fn rocket(client: Client) -> Result<rocket::Rocket, Error> {
    Ok(
        rocket::ignite()
            .manage(Arc::new(Mutex::new(client)))
            .manage(ArgonSecretKey(env::var("ARGON_SECRET_KEY").unwrap()))
            .mount("/", StaticFiles::from(frontend_dir()))
            .mount("/api", routes![
                auth_creds,
                do_scraping,
                register,
                login,
                logout,
                me,
                me_authed,
                not_logged_in_get,
                not_logged_in_post
            ])
            .register(catchers![not_found])
    )
}

fn main() -> Result<(), Error> {
    dotenv().ok();

    // Rocket pretty prints the error on drop if one occurs
    let _ = rocket(postgresql_client()?)?.launch();

    Ok(())
}

#[cfg(test)]
mod test {
    use json_structs::{AuthCredentials, RegisterInfo, UserInfo};
    use rocket::local::Client as HttpClient;
    use postgres::{Client, NoTls};
    use crate::{rocket, Error};
    use crate::create_tables;
    use rocket::http::{Status, StatusClass, ContentType};
    use std::process::Command;
    use dotenv::dotenv;
    use super::*;

    fn test_client() -> Result<Client, Error> {
        dotenv().ok();

        let output = Command::new("pg_tmp").arg("-t").output().unwrap();
        let mut client = Client::connect(&String::from_utf8(output.stdout).unwrap(), NoTls)?;
        create_tables(&mut client)?;

        Ok(client)
    }

    #[test]
    fn database_tables() -> Result<(), Error> {
        let mut db_client = test_client()?;
        create_tables(&mut db_client)?;

        let track1 = Track {
            track_id: 847238,
            sc_user_id: 102832,
            length_ms: 4039482,
            created_at: "2019-09-10T16:07:05Z".into(),
            title: "Database Testing Track".into(),
            description: "This is a track for testing the database".into(),
            likes_count: 4838,
            playback_count: 30248,
            artwork_url: Some("https://thislinkisinvalid.com".into()),
            permalink_url: "https://thislinkisalsoinvalid.com".into(),
            download_url: Some("https://thetrack/download.mp3".into())
        };
    
        let track2 = Track {
            track_id: 1028438,
            sc_user_id: 102832,
            length_ms: 2294884,
            created_at: "2019-09-17T06:29:59Z".into(),
            title: "Sick Banger".into(),
            description: "Does it need explanation??".into(),
            likes_count: 53828,
            playback_count: 9928732,
            artwork_url: Some("https://amazingbanger.dev".into()),
            permalink_url: "https://bangbang.com".into(),
            download_url: None
        };
    
        let sc_user = SoundCloudUser {
            sc_user_id: 102832,
            avatar_url: Some("https://anotherbadurl.net".into()),
            full_name: "John Bayer".into(),
            username: "superdude".into(),
            permalink_url: "https://ohnoalinkthatdoesntwork.com".into()
        };

        let playlist = Playlist {
            playlist_id: 82334,
            sc_user_id: 102832,
            track_ids: vec![847238, 1028438],
            length_ms: 6334366,
            created_at: "2019-09-17T06:29:59Z".into(),
            title: "My Killer Tunes".into(),
            permalink_url: "https://sadfacefakelink.cupcake".into(),
            description: "This playlist slays dude. Play it in the car".into(),
            likes_count: 9238,
            is_album: false
        };

        sc_user.create_new(&mut db_client)?;
        track1.create_new(&mut db_client, &sc_user)?;
        track2.create_new(&mut db_client, &sc_user)?;
        playlist.create_new(&mut db_client, &sc_user, &vec![track1.clone(), track2.clone()])?;

        let loaded_sc_user = SoundCloudUser::load_id(&mut db_client, sc_user.sc_user_id)?;
        let loaded_track1 = Track::load_id(&mut db_client, track1.track_id)?;
        let loaded_track2 = Track::load_id(&mut db_client, track2.track_id)?;
        let loaded_playlist = Playlist::load_id(&mut db_client, playlist.playlist_id)?;

        assert_eq!(sc_user, loaded_sc_user);
        assert_eq!(track1, loaded_track1);
        assert_eq!(track2, loaded_track2);
        assert_eq!(playlist, loaded_playlist);

        Ok(())
    }

    #[test]
    fn auth_creds() -> Result<(), Error> {
        let client = HttpClient::new(rocket(test_client()?)?).unwrap();
        let db = client.rocket().state::<DbClient>().unwrap();

        let response = client
            .post("/api/auth-creds")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&AuthCredentials {
                oauth_token: "bla".into(),
                client_id: "bla".into()
            }).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::ClientError);

        let rinfo = RegisterInfo {
            username: "testusername".into(),
            password: "testpass".into()
        };

        let response = client
            .post("/api/register")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&rinfo).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        let auth_creds = AuthCredentials {
            oauth_token: "bla".into(),
            client_id: "bla2".into()
        };

        let response = client
            .post("/api/auth-creds")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&auth_creds).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        {
            let mut conn = db.lock().unwrap();
            let user = User::load_id(&mut conn, 1)?;

            assert_eq!(user.sc_oauth_token.unwrap(), auth_creds.oauth_token);
            assert_eq!(user.sc_client_id.unwrap(), auth_creds.client_id);
        }

        Ok(())
    }

    #[test]
    fn login_flow() -> Result<(), Error> {
        let client = HttpClient::new(rocket(test_client()?)?).unwrap();

        let rinfo = RegisterInfo {
            username: "testusername".into(),
            password: "testpass".into()
        };

        // First, we register a user
        let mut response = client
            .post("/api/register")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&rinfo).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        let user_info: UserInfo = serde_json::from_str(&response.body_string().unwrap())?;
        assert_eq!(&user_info.username, &rinfo.username);

        // We observe that requests to authorized routes work
        let mut response = client
            .get("/api/me")
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        let user_info_from_me: UserInfo = serde_json::from_str(&response.body_string().unwrap())?;
        assert_eq!(&user_info_from_me, &user_info);

        // We logout
        let response = client
            .get("/api/logout")
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        // We observe that requests to authorized routes fail
        let response = client
            .get("/api/me")
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        Ok(())
    }

    #[test]
    fn cannot_create_with_same_username() -> Result<(), Error> {
        let client = HttpClient::new(rocket(test_client()?)?).unwrap();

        let rinfo = RegisterInfo {
            username: "testusername".into(),
            password: "testpass".into()
        };

        let rinfo2 = RegisterInfo {
            username: "testusername".into(),
            password: "testpass2".into()
        };

        // Register first user
        let response = client
            .post("/api/register")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&rinfo).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        // Attempt to register second
        let response = client
            .post("/api/register")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&rinfo2).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::ServerError);

        Ok(())
    }

    #[test]
    #[ignore]
    fn do_scraping() -> Result<(), Error> {
        let client = HttpClient::new(rocket(test_client()?)?).unwrap();
        let db = client.rocket().state::<DbClient>().unwrap();

        let rinfo = RegisterInfo {
            username: "testusername".into(),
            password: "testpass".into()
        };

        let response = client
            .post("/api/register")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&rinfo).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        let auth_creds = AuthCredentials {
            oauth_token: env::var("SC_OAUTH_TOKEN").unwrap(),
            client_id: env::var("SC_CLIENT_ID").unwrap()
        };

        let response = client
            .post("/api/auth-creds")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&auth_creds).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        let num_recent_likes = 10;
        let num_recent_playlists = 2;

        let response = client
            .post(format!(
                "/api/do-scraping?num_recent_likes={}&num_recent_playlists={}",
                num_recent_likes,
                num_recent_playlists
            ))
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

        thread::sleep(std::time::Duration::from_secs(10));

        {
            let mut conn = db.lock().unwrap();
            let track_count: i64 = conn.query_one("SELECT COUNT(track_id) FROM tracks", &[])?.get(0);
            let playlist_count: i64 = conn.query_one("SELECT COUNT(playlist_id) FROM playlists", &[])?.get(0);

            assert!(track_count >= num_recent_likes);
            assert!(playlist_count == num_recent_playlists);
        }

        Ok(())
    }
}
