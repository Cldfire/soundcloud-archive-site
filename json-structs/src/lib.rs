use serde_derive::{Deserialize, Serialize};

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
