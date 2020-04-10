use json_structs::{AuthCredentials, RegisterInfo, UserInfo};
use rocket::local::Client as HttpClient;
use postgres::{Client, NoTls};
use crate::{rocket, Error};
use crate::create_tables;
use rocket::http::{Status, StatusClass, ContentType};
use std::process::Command;
use dotenv::dotenv;
use serde_json::Value;
use sse_client::EventSource;
use crate::*;

impl Default for Track {
    fn default() -> Self {
        Track {
            track_id: 3234,
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
        }
    }
}

impl Default for SoundCloudUser {
    fn default() -> Self {
        SoundCloudUser {
            sc_user_id: 102832,
            avatar_url: Some("https://anotherbadurl.net".into()),
            full_name: "John Bayer".into(),
            username: "superdude".into(),
            permalink_url: "https://ohnoalinkthatdoesntwork.com".into()
        }
    }
}

fn test_client() -> Result<Client, Error> {
    dotenv().ok();

    let output = Command::new("pg_tmp").arg("-t").output().unwrap();
    let mut client = Client::connect(&String::from_utf8(output.stdout).unwrap(), NoTls)?;
    create_tables(&mut client)?;

    Ok(client)
}

// Quickly set up a test user with SC auth if credentials are available
//
// Returns the RegisterInfo used to set the user up with.
fn setup_test_user(client: &HttpClient) -> Result<RegisterInfo, Error> {
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
        .post("/api/set-auth-creds")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&auth_creds).unwrap())
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    Ok(rinfo)
}

#[test]
fn clear_liked_tracks() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    let db = client.rocket().state::<DbClient>().unwrap();
    let rinfo = setup_test_user(&client)?;

    {
        let mut conn = db.lock().unwrap();
        let mut user = User::load_username(&mut conn, &rinfo.username)?;
    
        let ids = vec![1, 2, 3];
        user.update_liked_track_ids(&mut conn, ids.clone())?;
        user = User::load_username(&mut conn, &rinfo.username)?;
        assert_eq!(user.liked_track_ids, ids);
    }

    let response = client
        .get("/api/clear-liked-tracks")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    {
        let mut conn = db.lock().unwrap();
        let user = User::load_username(&mut conn, &rinfo.username)?;

        let empty_vec: Vec<i64> = vec![];
        assert_eq!(user.liked_track_ids, empty_vec);
    }

    
    Ok(())
}

#[test]
fn clear_playlists() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    let db = client.rocket().state::<DbClient>().unwrap();
    let rinfo = setup_test_user(&client)?;

    {
        let mut conn = db.lock().unwrap();
        let mut user = User::load_username(&mut conn, &rinfo.username)?;
    
        let ids = vec![1, 2, 3];
        user.update_playlist_ids(&mut conn, ids.clone())?;
        user = User::load_username(&mut conn, &rinfo.username)?;
        assert_eq!(user.playlist_ids, ids);
    }

    let response = client
        .get("/api/clear-playlists")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    {
        let mut conn = db.lock().unwrap();
        let user = User::load_username(&mut conn, &rinfo.username)?;

        let empty_vec: Vec<i64> = vec![];
        assert_eq!(user.playlist_ids, empty_vec);
    }

    
    Ok(())
}

#[test]
fn most_liked_artist() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    let db = client.rocket().state::<DbClient>().unwrap();
    let rinfo = setup_test_user(&client)?;

    let mut tracks: Vec<_> = std::iter::repeat(Track::default()).take(5).collect();

    tracks[0].sc_user_id = 1;
    tracks[1].sc_user_id = 1;
    tracks[2].sc_user_id = 2;
    tracks[3].sc_user_id = 2;
    tracks[4].sc_user_id = 2;

    tracks[0].track_id = 1;
    tracks[1].track_id = 2;
    tracks[2].track_id = 3;
    tracks[3].track_id = 4;
    tracks[4].track_id = 5;

    let mut users: Vec<_> = std::iter::repeat(SoundCloudUser::default()).take(2).collect();

    users[0].sc_user_id = 1;
    users[1].sc_user_id = 2;
    {
        let mut conn = db.lock().unwrap();
        let user = User::load_username(&mut conn, &rinfo.username)?;
        user.update_liked_track_ids(&mut conn, tracks.iter().map(|t| t.track_id))?;

        for user in users.clone() {
            user.create_new(&mut conn)?;
        }
        for track in tracks {
            track.create_new(
                &mut conn,
                users.iter().find(|u| u.sc_user_id == track.sc_user_id).unwrap()
            )?;
        }
    }

    let mut response = client
        .get("/api/statistics/most-liked-artist")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    let user_info: ScUserInfo = serde_json::from_str(&response.body_string().unwrap())?;
    assert_eq!(user_info.sc_user_id, users[1].sc_user_id);

    Ok(())
}

#[test]
fn average_playback_count() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    let db = client.rocket().state::<DbClient>().unwrap();
    let rinfo = setup_test_user(&client)?;

    let mut tracks: Vec<_> = std::iter::repeat(Track::default()).take(5).collect();
    let sc_user = SoundCloudUser::default();

    tracks[0].track_id = 1;
    tracks[1].track_id = 2;
    tracks[2].track_id = 3;
    tracks[3].track_id = 4;
    tracks[4].track_id = 5;

    {
        let mut conn = db.lock().unwrap();
        let user = User::load_username(&mut conn, &rinfo.username)?;
        user.update_liked_track_ids(&mut conn, tracks.iter().map(|t| t.track_id))?;
        sc_user.create_new(&mut conn)?;

        for track in tracks.clone() {
            track.create_new(&mut conn, &sc_user)?;
        }
    }

    let mut response = client
        .get("/api/statistics/average-playback-count")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    let playback_count: i64 = serde_json::from_str(&response.body_string().unwrap())?;
    assert_eq!(playback_count, tracks[0].playback_count);

    Ok(())
}

#[test]
fn error_json() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    setup_test_user(&client)?;

    let auth_creds = AuthCredentials {
        oauth_token: "bla".into(),
        client_id: "bla2".into()
    };

    // Set sc credentials to something invalid
    let response = client
        .post("/api/set-auth-creds")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&auth_creds).unwrap())
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    // Try to do scraping, observe 500 status code and JSON error payload
    let mut response = client
        .get("/api/do-scraping?num_recent_likes=1&num_recent_playlists=1")
        .dispatch();

    assert_eq!(response.status().class(), StatusClass::ServerError);
    let err: Value = serde_json::from_str(&response.body_string().unwrap())?;
    assert_eq!(err["OrangeZestErr"]["HttpError"].as_i64().unwrap(), 401);

    Ok(())
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
        track_ids: vec![track1.track_id, track2.track_id],
        num_tracks: 2,
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
        .post("/api/set-auth-creds")
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
        .post("/api/set-auth-creds")
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

// TODO: add way to create tests that do not set up database connection
#[test]
fn non_existent_api_route() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    setup_test_user(&client)?;

    // Make request to API get route that doesn't exist
    let mut response = client
        .get("/api/something-that-doesnt-exist")
        .dispatch();

    assert_eq!(response.status().class(), StatusClass::ClientError);
    let err: Value = serde_json::from_str(&response.body_string().unwrap())?;
    assert_eq!(err.as_str().unwrap(), "NonExistentApiRoute");

    // Make request to API post route that doesn't exist
    let mut response = client
        .post("/api/something-that-doesnt-exist")
        .dispatch();

    assert_eq!(response.status().class(), StatusClass::ClientError);
    let err: Value = serde_json::from_str(&response.body_string().unwrap())?;
    assert_eq!(err.as_str().unwrap(), "NonExistentApiRoute");

    Ok(())
}

// Test trying to log in with a username that doesn't exist
#[test]
fn login_nonexistent_username() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();

    let mut response = client
        .post("/api/login")
        .header(ContentType::JSON)
        .body(
            serde_json::to_string(&RegisterInfo {
                username: "test".into(),
                password: "whatever".into()
            }).unwrap()
        )
        .dispatch();
    // TODO: This should technically be a StatusClass::ClientError
    assert_eq!(response.status().class(), StatusClass::ServerError);
    let err: Value = serde_json::from_str(&response.body_string().unwrap())?;
    assert_eq!(err.as_str().unwrap(), "LoginFailed");

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
fn can_create_with_same_password() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    
    let common_password = "testpass";
    let rinfo = RegisterInfo {
        username: "someusername".into(),
        password: common_password.into()
    };
    let rinfo2 = RegisterInfo {
        username: "someotherusername".into(),
        password: common_password.into()
    };

    // Register first user
    let response = client
        .post("/api/register")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&rinfo).unwrap())
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    // Register second
    let response = client
        .post("/api/register")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&rinfo2).unwrap())
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    Ok(())
}

#[test]
#[ignore]
fn entire_flow_live_site() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    let db = client.rocket().state::<DbClient>().unwrap();
    SSE.spawn("[::1]:3000".parse().unwrap());
    let rinfo = setup_test_user(&client)?;

    // This route can be used to get the user_id of the logged in user (something
    // you will need to store in the frontend for later use)
    let mut response = client
        .get("/api/me")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);
    let user_id = serde_json::from_str::<UserInfo>(&response.body_string().unwrap())?.user_id;

    // This route gives you an auth token with which to create an EventSource to
    // receive SSE events about likes and playlist download progress
    let mut response = client
        .get("/api/sse-auth-token")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    // Here I'm creating an EventSource to test with. You'll do something similar
    // in JS
    let es = EventSource::new(
        &format!("http://[::1]:3000/push/{}?{}", user_id, response.body_string().unwrap())
    ).unwrap();

    es.add_event_listener("update", |msg| {
        let sse_event: serde_json::Value = serde_json::from_str(&msg.data).unwrap();
        println!("Event: {:?}", sse_event);
    });

    // I'm limiting the amount of data scraped here to make the test faster
    //
    // You might want to start out doing this on the frontend to avoid
    // over-scraping soundcloud. Have it scrape everything when you're done
    // and you know things are working
    //
    // Or better yet, let the user choose ;)
    let num_recent_likes: i64 = 10;
    let num_recent_playlists: i64 = 2;

    // This route allows you to kick off the scraping process on the backend
    //
    // Use SSE to know when scraping is done. This route responds immediately
    let response = client
        .get(format!(
            "/api/do-scraping?num_recent_likes={}&num_recent_playlists={}",
            num_recent_likes,
            num_recent_playlists
        ))
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    thread::sleep(std::time::Duration::from_secs(10));
    
    // Database stuff, not relevant to frontend
    {
        let mut conn = db.lock().unwrap();
        let track_count: i64 = conn.query_one("SELECT COUNT(track_id) FROM tracks", &[])?.get(0);
        let playlist_count: i64 = conn.query_one("SELECT COUNT(playlist_id) FROM playlists", &[])?.get(0);

        assert!(track_count >= num_recent_likes);
        assert_eq!(playlist_count, num_recent_playlists);

        let loaded_user = User::load_username(&mut conn, &rinfo.username)?;
        assert_eq!(loaded_user.liked_track_ids.len() as i64, num_recent_likes);
        assert_eq!(loaded_user.playlist_ids.len() as i64, num_recent_playlists);
    }

    // This is how you access likes / track information after scraping has finished
    let mut response = client
        .get("/api/liked-tracks")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);
    
    let tracks_brief: Vec<TrackInfoBrief> = serde_json::from_reader(response.body().unwrap().into_inner())?;
    assert_eq!(tracks_brief.len() as i64, num_recent_likes);

    let mut response = client   
        .get(format!("/api/track-info/{}", tracks_brief[0].track_id))
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    let track_info: TrackInfoLong = serde_json::from_reader(response.body().unwrap().into_inner())?;
    assert_eq!(track_info.brief_info.track_id, tracks_brief[0].track_id);

    // This is how you access playlist information after scraping has finished
    let mut response = client
        .get("/api/liked-and-owned-playlists")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    let playlists_brief: Vec<PlaylistInfoBrief> = serde_json::from_reader(response.body().unwrap().into_inner())?;
    assert_eq!(playlists_brief.len() as i64, num_recent_playlists);

    let mut response = client   
        .get(format!("/api/playlist-info/{}", playlists_brief[0].playlist_id))
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    let playlist_info: PlaylistInfoLong = serde_json::from_reader(response.body().unwrap().into_inner())?;
    assert_eq!(playlist_info.brief_info.playlist_id, playlists_brief[0].playlist_id);

    Ok(())
}
