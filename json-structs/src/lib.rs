use serde_derive::{Deserialize, Serialize};
use orange_zest::events::*;

/// The events sent via Server Sent Events are the only things not in this file.
///
/// To see their definitions, go to https://github.com/Cldfire/orange-zest/blob/master/src/events.rs
/// 
/// Those structs contain some different layouts for tracks and playlists compared
/// to what I have in this file. Two other things you'll have to look at:
/// 
/// * Track, https://github.com/Cldfire/orange-zest/blob/master/src/api/common.rs
/// * PlaylistMeta, https://github.com/Cldfire/orange-zest/blob/master/src/api/playlists.rs
/// 
/// It's a huge hassle to convert those types to the types in this file, so I
/// left them as-is. I may change that in the future.
/// 
/// Basically, get the events, dump the json, look at it, figure out how to
/// access it. I'm always available for questions.
/// 
/// Note that all events are sent with an event name of "update". This means you
/// will need to add a listener for that event. See
/// https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
#[derive(Serialize, Debug)]
pub enum SseEvent<'a> {
    LikesScrapingEvent(LikesZestingEvent),
    PlaylistsScrapingEvent(PlaylistsZestingEvent<'a>)
}

/// Post this from the web client to provide credentials with which to get data
/// from SoundCloud with for a specific user.
// TODO: document how to get these credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCredentials {
    pub oauth_token: String,
    pub client_id: String
}

/// Post this from the web client to create a new user.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterInfo {
    pub password: String,
    pub username: String
}

/// Web client posts this to login.
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginInfo {
    pub username: String,
    pub password: String
}

/// Information about the requested user sent to the web client
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UserInfo {
    pub user_id: i32,
    pub username: String
}

/// Summarized information about a track.
/// 
/// Useful for displaying a long list of tracks on the frontend.
#[derive(Serialize, Deserialize, Debug)]
pub struct TrackInfoBrief {
    /// A unique numeric id for the track
    pub track_id: i64,
    /// The length of the track in milliseconds
    pub length_ms: i64,
    /// When the track was uploaded to SoundCloud as a date-time string
    pub created_at: String,
    /// The name of the track
    pub title: String,
    /// The number of times the track was played on SoundCloud
    pub playback_count: i64,

    /// The id of the SoundCloud user that uploaded this track
    pub sc_user_id: i64,
    /// The user's display name
    pub username: String
}

/// Detailed information about a track.
/// 
/// Useful for displaying detailed information about a single track on
/// the frontend.
#[derive(Serialize, Deserialize, Debug)]
pub struct TrackInfoLong {
    /// Everything that was in the brief version of the struct
    pub brief_info: TrackInfoBrief,

    /// A description of the track written by the user who posted it
    pub description: String,
    /// The number of times the track was liked on SoundCloud
    pub likes_count: i64,
    /// A URL to the track's album art
    pub artwork_url: Option<String>,
    /// A URL to the track on SoundCloud
    pub track_permalink_url: String,

    /// A URL to the profile image of the user that uploaded this track on
    /// SoundCloud
    pub avatar_url: Option<String>,
    /// The user's full name
    pub full_name: String,
    /// A URL to the user on SoundCloud
    pub user_permalink_url: String
}

/// Summarized information about a playlist.
/// 
/// Useful for displaying a long list of playlists on the frontend.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlaylistInfoBrief {
    /// A unique numeric id for the playlist
    pub playlist_id: i64,
    /// The total length of all tracks in the playlist combined in milliseconds
    pub length_ms: i64,
    /// When the playlist was created on SoundCloud as a date-time string
    pub created_at: String,
    /// The name of the playlist
    pub title: String,
    /// Whether or not this playlist is an album
    pub is_album: bool,
    /// The number of tracks in this playlist
    pub num_tracks: i64,

    /// The id of the soundcloud user that created this playlist
    pub sc_user_id: i64,
    /// The user's display name
    pub username: String
}

/// Detailed information about a playlist.
/// 
/// Useful for displaying detailed information about a single playlist on
/// the frontend.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlaylistInfoLong {
    /// Everything that was in the brief version of the struct
    pub brief_info: PlaylistInfoBrief,

    /// IDs of tracks that are a part of this playlist
    pub track_ids: Vec<i64>,
    /// A URL to the playlist on SoundCloud
    pub playlist_permalink_url: String,
    /// The playlist's description
    pub description: String,
    /// The number of times the playlist was liked on SoundCloud
    pub likes_count: i64,

    /// A URL to the profile image of the user that uploaded this track on
    /// SoundCloud
    pub avatar_url: Option<String>,
    /// The user's full name
    pub full_name: String,
    /// A URL to the user on SoundCloud
    pub user_permalink_url: String
}
