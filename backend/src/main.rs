#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rocket_contrib::{json::Json, serve::StaticFiles};
use rocket::{request::{self, FromRequest}, response::{status, NamedFile}, Request, State, Outcome, http::{Cookie, Status, Cookies}};
use json_structs::*;
use dotenv::dotenv;
use postgres::{Client, NoTls};
use argonautica::{Hasher, Verifier};

use std::path::Path;
use std::sync::Mutex;
use std::env;

type DbClient = Mutex<Client>;
struct ArgonSecretKey(String);

/// The error type used throughout the binary
#[derive(Debug)]
enum Error {
    IoErr(std::io::Error),
    /// An error encountered while working with password hashes
    HashError(argonautica::Error),
    PostgresErr(postgres::error::Error),
    JsonErr(serde_json::Error),
    /// An attempt was made to create a user given details that already exist
    UserAlreadyExists,
    /// Could not log in with the given `LoginInfo`
    LoginFailed
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

/// Representation of a user in the database.
///
/// Users have both an id (numeric) and a username (alphanumeric). The username
/// is for the user to login with and view, and the id is for the backend to
/// use.
#[derive(Debug, PartialEq)]
struct User {
    /// The user's unique, numeric ID.
    ///
    /// This number starts at 1 (the first user) and is incremented for each
    /// user that signs up.
    user_id: i32,
    /// Argon stores the salt alongside the hash and other info
    ///
    /// (To be clear this is the hash of the user's password.)
    hash: String,
    /// The user's name
    ///
    /// This is both their login and their displayname
    username: String,
}

impl From<User> for UserInfo {
    fn from(u: User) -> Self {
        Self {
            user_id: u.user_id,
            username: u.username
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, Self::Error> {
        let db = request.guard::<State<DbClient>>().unwrap();
        let mut client = db.lock().unwrap();

        let res = request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User::load_id(&mut client, id));

        match res {
            Some(Ok(r)) => Outcome::Success(r),
            Some(Err(err)) => Outcome::Failure((Status::InternalServerError, err)),
            None => Outcome::Forward(())
        }
    }
}

impl User {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    fn create_table(client: &mut Client) -> Result<(), Error> {
        Ok(client.execute(
            "CREATE TABLE IF NOT EXISTS users (
                        user_id       SERIAL PRIMARY KEY,
                        username      TEXT NOT NULL,
                        hash          TEXT NOT NULL
                    )",
                    &[]
        ).map(|_| ())?)
    }

    /// Checks to see if a user with the given username exists and returns true
    /// if one does.
    fn exists(client: &mut Client, username: &str) -> Result<bool, Error> {
        Ok(
            if client.query("SELECT * FROM users WHERE username = $1", &[&username])?.len() > 0 {
                true
            } else {
                false
            }
        )
    }

    /// Inserts a new user into the database based on the given registration info,
    /// returning the id of the new user.
    ///
    /// The `key` parameter is the secret key given to argon for hashing
    ///
    /// Errors if the user cannot be created.
    fn create_new(client: &mut Client, rinfo: &RegisterInfo, key: &str) -> Result<i32, Error> {
        if User::exists(client, &rinfo.username)? {
            return Err(Error::UserAlreadyExists);
        }

        let mut hasher = Hasher::default();
        let hash = hasher
            .with_password(&rinfo.password)
            .with_secret_key(key)
            .hash()
            .unwrap();


        Ok(client.query_one(
            "INSERT INTO users (hash, username) VALUES ($1, $2) RETURNING user_id",
            &[&hash, &rinfo.username],
        )?.get(0))
    }

    /// Loads the user specified by the given id from the database
    fn load_id(client: &mut Client, id: i32) -> Result<Self, Error> {
        let row = client.query_one("SELECT user_id, hash, username FROM users WHERE user_id = $1", &[&id])?;

        Ok(User {
            user_id: row.get(0),
            hash: row.get(1),
            username: row.get(2)
        })
    }

    /// Loads the user specified by the given username from the database
    fn load_username(client: &mut Client, username: &str) -> Result<Self, Error> {
        let row = client.query_one("SELECT user_id, hash, username FROM users WHERE username = $1", &[&username])?;

        Ok(User {
            user_id: row.get(0),
            hash: row.get(1),
            username: row.get(2)
        })
    }

    /// Returns true if this user matches the given `LoginInfo`
    ///
    /// This means that the usernames are equivalent and the password the user
    /// entered hashed to the correct value.
    ///
    /// The `key` parameter is the secret key given to argon for hashing
    fn auth(&self, login_info: &LoginInfo, key: &str) -> Result<bool, Error> {
        let mut verifier = Verifier::default();
        Ok(
            login_info.username == self.username &&
            verifier
                .with_hash(&self.hash)
                .with_password(&login_info.password)
                .with_secret_key(key)
                .verify()?
        )
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

/// A "catch-all" to redirect path requests to the index since we are building a SPA
// TODO: if you need requests to be directed at a different file, change this accordingly
#[catch(404)]
fn not_found() -> NamedFile {
    NamedFile::open(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/frontend/public/index.html")
    )).unwrap()
}

/// Route used to provide auth credentials (OAuth token and Client ID).
#[post("/auth-creds", format = "json", data = "<auth_creds>")]
fn auth_creds(auth_creds: Json<AuthCredentials>) -> Result<(), Error> {
    // TODO: this
    Ok(())
}

/// Create a Rocket instance given a PostgreSQL client.
fn rocket(client: Client) -> Result<rocket::Rocket, Error> {
    Ok(
        rocket::ignite()
            .manage(Mutex::new(client))
            .manage(ArgonSecretKey(env::var("ARGON_SECRET_KEY").unwrap()))
            // TODO: if the directory you put frontend static files in is different
            // then change this accordingly
            .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/public")))
            .mount("/api", routes![
                auth_creds,
                register,
                login,
                logout,
                me,
                me_authed
            ])
            .register(catchers![not_found])
    )
}

/// Creates a PostgreSQL client based off of environment variables.
fn postgresql_client() -> Result<Client, Error> {
    let mut client = Client::configure()
        .port(env::var("POSTGRES_PORT").unwrap().parse().unwrap())
        .user(&env::var("POSTGRES_USER").unwrap())
        .dbname(&env::var("POSTGRES_DBNAME").unwrap())
        .host(&env::var("POSTGRES_HOST").unwrap())
        .connect(NoTls)?;

    create_tables(&mut client)?;
    Ok(client)
}

/// Creates any tables required by the backend if they do not exist already.
fn create_tables(client: &mut Client) -> Result<(), Error> {
    User::create_table(client)?;

    Ok(())
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

    fn test_client() -> Result<Client, Error> {
        dotenv().ok();

        let output = Command::new("pg_tmp").arg("-t").output().unwrap();
        let mut client = Client::connect(&String::from_utf8(output.stdout).unwrap(), NoTls)?;
        create_tables(&mut client)?;

        Ok(client)
    }

    #[test]
    fn auth_creds() -> Result<(), Error> {
        let client = HttpClient::new(rocket(test_client()?)?).unwrap();

        let response = client
            .post("/api/auth-creds")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&AuthCredentials {
                oauth_token: "bla".into(),
                client_id: "bla".into()
            }).unwrap())
            .dispatch();
        assert_eq!(response.status().class(), StatusClass::Success);

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
}
