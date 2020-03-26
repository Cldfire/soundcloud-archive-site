use json_structs::{AuthCredentials, RegisterInfo, UserInfo};
use rocket::local::Client as HttpClient;
use postgres::{Client, NoTls};
use crate::{rocket, Error};
use crate::create_tables;
use rocket::http::{Status, StatusClass, ContentType};
use std::process::Command;
use dotenv::dotenv;
use sse_client::EventSource;
use crate::*;

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
        .post("/api/auth-creds")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&auth_creds).unwrap())
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    Ok(rinfo)
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
fn entire_flow_live_site() -> Result<(), Error> {
    let client = HttpClient::new(rocket(test_client()?)?).unwrap();
    let db = client.rocket().state::<DbClient>().unwrap();
    SSE.spawn("[::1]:3000".parse().unwrap());
    let rinfo = setup_test_user(&client)?;

    let mut response = client
        .get("/api/me")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);
    let user_id = serde_json::from_str::<UserInfo>(&response.body_string().unwrap())?.user_id;

    let mut response = client
        .get("/api/sse-auth-token")
        .dispatch();
    assert_eq!(response.status().class(), StatusClass::Success);

    let es = EventSource::new(
        &format!("http://[::1]:3000/push/{}?{}", user_id, response.body_string().unwrap())
    ).unwrap();

    es.add_event_listener("update", |msg| {
        let sse_event: serde_json::Value = serde_json::from_str(&msg.data).unwrap();
        println!("Event: {:?}", sse_event);
    });

    let num_recent_likes: i64 = 10;
    let num_recent_playlists: i64 = 2;

    let response = client
        .post(format!(
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

    // Likes and tracks stuff
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

    // Playlist stuff
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
