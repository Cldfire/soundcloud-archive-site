use serde_derive::{Deserialize, Serialize};

/// Post this from the web client to provide credentials with which to get data
/// from SoundCloud with for a specific user.
// TODO: document how to get these credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCredentials {
    pub oauth_token: String,
    pub client_id: String
}
