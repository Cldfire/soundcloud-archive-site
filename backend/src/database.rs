use rocket::{request::{self, FromRequest}, Request, State, Outcome, http::Status};
use json_structs::*;
use postgres::Client;
use argonautica::{Hasher, Verifier};
use orange_zest::api::common::{Track as ScTrack, User as ScUser};
use orange_zest::api::playlists::Playlist as ScPlaylist;

use super::*;

pub type DbClient = Arc<Mutex<Client>>;

/// Creates a PostgreSQL client based off of environment variables.
pub fn postgresql_client() -> Result<Client, Error> {
    let mut client = Client::configure()
        .port(env::var("POSTGRES_PORT").unwrap().parse().unwrap())
        .user(&env::var("POSTGRES_USER").unwrap())
        .dbname(&env::var("POSTGRES_DBNAME").unwrap())
        .host(&env::var("POSTGRES_HOST").unwrap())
        .password(&env::var("POSTGRES_PASSWORD").unwrap_or("".into()))
        .connect(NoTls)?;

    create_tables(&mut client)?;
    Ok(client)
}

/// Creates any tables required by the backend if they do not exist already.
pub fn create_tables(client: &mut Client) -> Result<(), Error> {
    User::create_table(client)?;
    SoundCloudUser::create_table(client)?;
    Track::create_table(client)?;
    Playlist::create_table(client)?;

    Ok(())
}

/// Request guard to validate request is not coming from an authenticated user.
#[derive(Debug)]
pub struct NotLoggedIn;

impl<'a, 'r> FromRequest<'a, 'r> for NotLoggedIn {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<NotLoggedIn, Self::Error> {
        match request.cookies().get_private("user_id") {
            Some(_) => Outcome::Forward(()),
            None => Outcome::Success(NotLoggedIn)
        }
    }
}

/// Representation of a user in the database.
///
/// Users have both an id (numeric) and a username (alphanumeric). The username
/// is for the user to login with and view, and the id is for the backend to
/// use.
#[derive(Debug, PartialEq)]
pub struct User {
    /// The user's unique, numeric ID.
    ///
    /// This number starts at 1 (the first user) and is incremented for each
    /// user that signs up.
    pub user_id: i32,
    /// Argon stores the salt alongside the hash and other info
    ///
    /// (To be clear this is the hash of the user's password.)
    pub hash: String,
    /// The user's name
    ///
    /// This is both their login and their displayname
    pub username: String,
    /// The OAuth token that this user has provided for us to use to access their
    /// SoundCloud account
    pub sc_oauth_token: Option<String>,
    /// The client id that corresponds to this user's account on SoundCloud
    pub sc_client_id: Option<String>,
    /// A vector of ids for tracks that this user has liked on SoundCloud
    pub liked_track_ids: Vec<i64>,
    /// A vector of ids for playlists that this user has made or liked on
    /// SoundCloud
    pub playlist_ids: Vec<i64>
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
    pub fn create_table(client: &mut Client) -> Result<(), Error> {
        Ok(client.execute(
            "CREATE TABLE IF NOT EXISTS users (
                user_id             SERIAL PRIMARY KEY,
                username            TEXT NOT NULL UNIQUE,
                hash                TEXT NOT NULL,
                sc_oauth_token      TEXT,
                sc_client_id        TEXT,
                liked_track_ids     BIGINT[] NOT NULL,
                playlist_ids        BIGINT[] NOT NULL
            )",
            &[]
        ).map(|_| ())?)
    }

    /// Checks to see if a user with the given username exists and returns true
    /// if one does.
    pub fn exists(client: &mut Client, username: &str) -> Result<bool, Error> {
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
    pub fn create_new(client: &mut Client, rinfo: &RegisterInfo, key: &str) -> Result<i32, Error> {
        if User::exists(client, &rinfo.username)? {
            return Err(Error::UserAlreadyExists);
        }

        let mut hasher = Hasher::default();
        let hash = hasher
            .with_password(&rinfo.password)
            .with_secret_key(key)
            .hash()
            .unwrap();

        // Needed to make types work out
        let empty_vec: Vec<i64> = vec![];

        Ok(client.query_one(
            "INSERT INTO users (
                hash, username, liked_track_ids, playlist_ids
            ) VALUES ($1, $2, $3, $4) RETURNING user_id",
            &[&hash, &rinfo.username, &empty_vec, &empty_vec],
        )?.get(0))
    }

    /// Loads the user specified by the given id from the database
    pub fn load_id(client: &mut Client, id: i32) -> Result<Self, Error> {
        let row = client.query_one("
            SELECT
                user_id,
                hash,
                username,
                sc_oauth_token,
                sc_client_id,
                liked_track_ids,
                playlist_ids
            FROM users WHERE user_id = $1", &[&id])?;

        Ok(Self {
            user_id: row.get(0),
            hash: row.get(1),
            username: row.get(2),
            sc_oauth_token: row.get(3),
            sc_client_id: row.get(4),
            liked_track_ids: row.get(5),
            playlist_ids: row.get(6)
        })
    }

    /// Loads the user specified by the given username from the database
    pub fn load_username(client: &mut Client, username: &str) -> Result<Self, Error> {
        let maybe_row = client.query_opt("
            SELECT
                user_id,
                hash,
                username,
                sc_oauth_token,
                sc_client_id,
                liked_track_ids,
                playlist_ids
            FROM users WHERE username = $1", &[&username])?;

        if let Some(row) = maybe_row {
            Ok(Self {
                user_id: row.get(0),
                hash: row.get(1),
                username: row.get(2),
                sc_oauth_token: row.get(3),
                sc_client_id: row.get(4),
                liked_track_ids: row.get(5),
                playlist_ids: row.get(6)
            })
        } else {
            Err(Error::LoginFailed)
        }
    }

    /// Stores the given `AuthCredentials` in the databse for this user.
    pub fn store_sc_credentials(
        &self,
        client: &mut Client,
        credentials: &AuthCredentials
    ) -> Result<(), Error> {
        Ok(client.execute(
            "UPDATE users SET sc_oauth_token = $1, sc_client_id = $2 WHERE user_id = $3",
            &[&credentials.oauth_token, &credentials.client_id, &self.user_id]
        ).map(|_| ())?)
    }

    /// Set this user's liked_track_ids to the values produced by the given iterator.
    pub fn update_liked_track_ids<I: IntoIterator<Item = i64>>(
        &self,
        client: &mut Client,
        ids: I
    ) -> Result<(), Error> {
        Ok(client.execute(
            "UPDATE users SET liked_track_ids = $1 WHERE user_id = $2",
            &[&ids.into_iter().collect::<Vec<i64>>(), &self.user_id]
        ).map(|_| ())?)
    }

    /// Set this user's playlist_ids to the values produced by the given iterator.
    pub fn update_playlist_ids<I: IntoIterator<Item = i64>>(
        &self,
        client: &mut Client,
        ids: I
    ) -> Result<(), Error> {
        Ok(client.execute(
            "UPDATE users SET playlist_ids = $1 WHERE user_id = $2",
            &[&ids.into_iter().collect::<Vec<i64>>(), &self.user_id]
        ).map(|_| ())?)
    }

    /// Returns true if this user matches the given `LoginInfo`
    ///
    /// This means that the usernames are equivalent and the password the user
    /// entered hashed to the correct value.
    ///
    /// The `key` parameter is the secret key given to argon for hashing
    pub fn auth(&self, login_info: &LoginInfo, key: &str) -> Result<bool, Error> {
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

/// Representation of a track in the database.
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    /// A unique numeric id for the track
    pub track_id: i64,
    /// The id of the SoundCloud user that uploaded this track
    pub sc_user_id: i64,
    /// The length of the track in milliseconds
    pub length_ms: i64,
    /// When the track was uploaded to SoundCloud as a date-time string
    pub created_at: String,
    /// The name of the track
    pub title: String,
    /// A description of the track written by the user who posted it
    pub description: String,
    /// The number of times the track was liked on SoundCloud
    pub likes_count: i64,
    /// The number of times the track was played on SoundCloud
    pub playback_count: i64,
    /// A URL to the track's album art
    pub artwork_url: Option<String>,
    /// A URL to the track on SoundCloud
    pub permalink_url: String,
    /// A URL via which the audio data for this track can be downloaded on the backend.
    ///
    /// This may not exist for every track.
    pub download_url: Option<String>
}

impl From<&ScTrack> for Track {
    fn from(track: &ScTrack) -> Self {
        Track {
            track_id: track.id.unwrap(),
            sc_user_id: track.user_id.unwrap(),
            length_ms: track.duration.unwrap(),
            created_at: track.created_at.clone().unwrap(),
            title: track.title.clone().unwrap_or("".into()),
            description: track.description.clone().unwrap_or("".into()),
            likes_count: track.likes_count.clone().unwrap_or(0),
            playback_count: track.playback_count.unwrap_or(0),
            artwork_url: track.artwork_url.clone(),
            permalink_url: track.permalink_url.clone().unwrap(),
            download_url: None
        }
    }
}

impl Track {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    pub fn create_table(client: &mut Client) -> Result<(), Error> {
        Ok(client.execute(
            "CREATE TABLE IF NOT EXISTS tracks (
                track_id        BIGINT PRIMARY KEY,
                sc_user_id      BIGINT NOT NULL references soundcloudusers(sc_user_id),
                length_ms       BIGINT NOT NULL,
                created_at      TEXT NOT NULL,
                title           TEXT NOT NULL,
                description     TEXT NOT NULL,
                likes_count     BIGINT NOT NULL,
                playback_count  BIGINT NOT NULL,
                artwork_url     TEXT,
                permalink_url   TEXT NOT NULL,
                download_url    TEXT
            )",
            &[]
        ).map(|_| ())?)
    }

    /// Creates a new track in the database based on an instance of the struct
    /// and a reference to the user that owns the track.
    pub fn create_new(&self, client: &mut Client, user: &SoundCloudUser) -> Result<(), Error> {
        user.create_new(client)?;
        Ok(client.execute(
            "INSERT INTO tracks VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT DO NOTHING",
            &[
                &self.track_id,
                &self.sc_user_id,
                &self.length_ms,
                &self.created_at,
                &self.title,
                &self.description,
                &self.likes_count,
                &self.playback_count,
                &self.artwork_url,
                &self.permalink_url,
                &self.download_url
            ],
        ).map(|_| ())?)
    }

    /// Loads the track specified by the given id from the database
    pub fn load_id(client: &mut Client, track_id: i64) -> Result<Self, Error> {
        let row = client.query_one("
            SELECT
                track_id,
                sc_user_id,
                length_ms,
                created_at,
                title,
                description,
                likes_count,
                playback_count,
                artwork_url,
                permalink_url,
                download_url
            FROM tracks
            WHERE track_id = $1",
            &[&track_id]
        )?;

        Ok(Self {
            track_id: row.get(0),
            sc_user_id: row.get(1),
            length_ms: row.get(2),
            created_at: row.get(3),
            title: row.get(4),
            description: row.get(5),
            likes_count: row.get(6),
            playback_count: row.get(7),
            artwork_url: row.get(8),
            permalink_url: row.get(9),
            download_url: row.get(10)
        })
    }
}

/// Representation of a SoundCloud user in the database
#[derive(Debug, PartialEq, Clone)]
pub struct SoundCloudUser {
    /// The id of the SoundCloud user
    pub sc_user_id: i64,
    /// A URL to the user's profile image on SoundCloud
    pub avatar_url: Option<String>,
    /// The user's full name
    pub full_name: String,
    /// The user's display name
    pub username: String,
    /// A URL to the user on SoundCloud
    pub permalink_url: String
}

impl From<&ScUser> for SoundCloudUser {
    fn from(u: &ScUser) -> Self {
        Self {
            sc_user_id: u.id.unwrap(),
            avatar_url: u.avatar_url.clone(),
            full_name: u.full_name.clone().unwrap_or("".into()),
            username: u.username.clone().unwrap(),
            permalink_url: u.permalink_url.clone().unwrap()
        }
    }
}

impl SoundCloudUser {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    pub fn create_table(client: &mut Client) -> Result<(), Error> {
        Ok(client.execute(
            "CREATE TABLE IF NOT EXISTS soundcloudusers (
                sc_user_id      BIGINT PRIMARY KEY,
                avatar_url      TEXT,
                full_name       TEXT NOT NULL,
                username        TEXT NOT NULL,
                permalink_url   TEXT NOT NULL
            )",
            &[]
        ).map(|_| ())?)
    }

    /// Creates a new SoundCloud user in the database based on an instance of
    /// the struct.
    pub fn create_new(&self, client: &mut Client) -> Result<(), Error> {
        Ok(client.execute(
            "INSERT INTO soundcloudusers VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING",
            &[
                &self.sc_user_id,
                &self.avatar_url,
                &self.full_name,
                &self.username,
                &self.permalink_url
            ],
        ).map(|_| ())?)
    }

    /// Loads the user specified by the given id from the database
    pub fn load_id(client: &mut Client, sc_user_id: i64) -> Result<Self, Error> {
        let row = client.query_one("
            SELECT
                sc_user_id,
                avatar_url,
                full_name,
                username,
                permalink_url
            FROM soundcloudusers
            WHERE sc_user_id = $1",
            &[&sc_user_id]
        )?;

        Ok(Self {
            sc_user_id: row.get(0),
            avatar_url: row.get(1),
            full_name: row.get(2),
            username: row.get(3),
            permalink_url: row.get(4)
        })
    }
}

/// Representation of a playlist in the database.
#[derive(Debug, PartialEq, Clone)]
pub struct Playlist {
    /// A unique numeric id for the playlist
    pub playlist_id: i64,
    /// The id of the soundcloud user that created this playlist
    pub sc_user_id: i64,
    /// IDs of tracks that are a part of this playlist
    pub track_ids: Vec<i64>,
    /// The number of tracks in the playlist
    pub num_tracks: i64,
    /// The total length of all tracks in the playlist combined in milliseconds
    pub length_ms: i64,
    /// When the playlist was created on SoundCloud as a date-time string
    pub created_at: String,
    /// The name of the playlist
    pub title: String,
    /// A URL to the playlist on SoundCloud
    pub permalink_url: String,
    /// The playlist's description
    pub description: String,
    /// The number of times the playlist was liked on SoundCloud
    pub likes_count: i64,
    /// Whether or not this playlist is an album
    pub is_album: bool
}

impl From<&ScPlaylist> for Playlist {
    fn from(playlist: &ScPlaylist) -> Self {
        Self {
            playlist_id: playlist.id.unwrap(),
            sc_user_id: playlist.user_id.unwrap(),
            track_ids: playlist.tracks.as_ref().unwrap().iter().map(|t| t.id.unwrap()).collect(),
            num_tracks: playlist.tracks.as_ref().unwrap().len() as i64,
            length_ms: playlist.duration.unwrap_or(0),
            created_at: playlist.created_at.clone().unwrap(),
            title: playlist.title.clone().unwrap_or("".into()),
            permalink_url: playlist.permalink_url.clone().unwrap(),
            description: playlist.description.clone().unwrap_or("".into()),
            likes_count: playlist.likes_count.unwrap_or(0),
            is_album: playlist.is_album.unwrap_or(false)
        }
    }
}

impl Playlist {
    /// Creates a table in the given database for storing this struct.
    ///
    /// The table will only be created if it does not already exist.
    pub fn create_table(client: &mut Client) -> Result<(), Error> {
        Ok(client.execute(
            "CREATE TABLE IF NOT EXISTS playlists (
                playlist_id     BIGINT PRIMARY KEY,
                sc_user_id      BIGINT NOT NULL references soundcloudusers(sc_user_id),
                track_ids       BIGINT[] NOT NULL,
                num_tracks      BIGINT NOT NULL,
                length_ms       BIGINT NOT NULL,
                created_at      TEXT NOT NULL,
                title           TEXT NOT NULL,
                permalink_url   TEXT NOT NULL,
                description     TEXT NOT NULL,
                likes_count     BIGINT NOT NULL,
                is_album        BOOLEAN NOT NULL
            )",
            &[]
        ).map(|_| ())?)
    }

    /// Creates a new playlist in the database based on an instance of the struct
    /// and a reference to the info about the playlist.
    pub fn create_new(
        &self,
        client: &mut Client,
        playlist_info: &ScPlaylist
    ) -> Result<(), Error> {
        SoundCloudUser::from(playlist_info.user.as_ref().unwrap()).create_new(client)?;

        for track in playlist_info.tracks.as_ref().unwrap() {
            Track::from(track).create_new(client, &SoundCloudUser::from(track.user.as_ref().unwrap()))?;
        }

        Ok(client.execute(
            "INSERT INTO playlists VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT DO NOTHING",
            &[
                &self.playlist_id,
                &self.sc_user_id,
                &self.track_ids,
                &self.num_tracks,
                &self.length_ms,
                &self.created_at,
                &self.title,
                &self.permalink_url,
                &self.description,
                &self.likes_count,
                &self.is_album
            ],
        ).map(|_| ())?)
    }

    /// Loads the playlist specified by the given id from the database
    pub fn load_id(client: &mut Client, playlist_id: i64) -> Result<Self, Error> {
        let row = client.query_one("
            SELECT
                playlist_id,
                sc_user_id,
                track_ids,
                num_tracks,
                length_ms,
                created_at,
                title,
                permalink_url,
                description,
                likes_count,
                is_album
            FROM playlists
            WHERE playlist_id = $1",
            &[&playlist_id]
        )?;

        Ok(Self {
            playlist_id: row.get(0),
            sc_user_id: row.get(1),
            track_ids: row.get(2),
            num_tracks: row.get(3),
            length_ms: row.get(4),
            created_at: row.get(5),
            title: row.get(6),
            permalink_url: row.get(7),
            description: row.get(8),
            likes_count: row.get(9),
            is_album: row.get(10)
        })
    }
}
