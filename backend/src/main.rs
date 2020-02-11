#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rocket_contrib::{json::Json, serve::StaticFiles};
use rocket::response::NamedFile;
use scas_json::AuthCredentials;

use std::path::Path;

/// The error type used throughout the binary
#[derive(Debug)]
enum Error {
    IoErr(std::io::Error)
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
    dbg!(auth_creds);
    Ok(())
}

/// Create a Rocket instance.
fn rocket() -> Result<rocket::Rocket, Error> {
    Ok(
        rocket::ignite()
            // TODO: if the directory you put frontend static files in is different
            // then change this accordingly
            .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/public")))
            .mount("/api", routes![auth_creds])
            .register(catchers![not_found])
    )
}

fn main() -> Result<(), Error> {
    // Rocket pretty prints the error on drop if one occurs
    let _ = rocket()?.launch();

    Ok(())
}

#[cfg(test)]
mod test {
    use scas_json::AuthCredentials;
    use rocket::local::Client;
    use crate::{rocket, Error};
    use rocket::http::{StatusClass, ContentType};

    #[test]
    fn auth_creds() -> Result<(), Error> {
        let client = Client::new(rocket()?).unwrap();

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
}
