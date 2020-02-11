# soundcloud-archive-site

A website that provides a simple interface to store an archive of your [SoundCloud](https://soundcloud.com) user data, allowing you both to browse / retrieve the data and to view various statistics.

## Frontend (by Ian)

TODO: Ian will put relevant info here as he builds the frontend

(Temporary info): The frontend should go in the `frontend` folder. Feel free to use whatever you want.

If you decide to use a Rust framework that compiles to WASM, I've split the struct definitions for the JSON that will be used to communicate between the client and the server out into the library portion of the crate (`backend/src/lib.rs`). You can depend on it and `serde` / `serde_json` to quickly and easily deserialize the JSON into typed structs.

If not, just work with the JSON however you would normally.

If you want to see an example of something I've done in the past with Svelte, see [here](https://github.com/Cldfire/self-host-social).

## Backend (by Jarek)

The backend is in `backend` and is written in Rust. It requires Rust nightly due to the usage of Rocket. Make sure you have Rust [installed](https://www.rust-lang.org/tools/install), and then do the following from the repo root:

```bash
rustup set override nightly
```

You will need to set up a local [PostgreSQL](https://www.postgresql.org/) server for the backend to connect to. Connection info (port, host, user, database name) are provided through environment variables.

* `POSTGRES_PORT` specifies the database port
* `POSTGRES_USER` specifies the database user
* `POSTGRES_DBNAME` specifies the particular database name to connect to
* `POSTGRES_HOST` specifies host (should be localhost)

You will also need to provide a value for the environment variable `ARGON_SECRET_KEY` that is used for password hashing. You can get a suitable value by doing something like `openssl rand -base64 32` (although for development purposes it doesn't really matter).

All of these environment variables can be provided in a **`.env` file**. Create a file named `.env` in the `soundcloud-archive-site` directory with the following:

```
POSTGRES_PORT="..."
POSTGRES_USER="..."
POSTGRES_DBNAME="..."
POSTGRES_HOST="..."
ARGON_SECRET_KEY="..."
```

See [dotenv](https://github.com/dotenv-rs/dotenv) for more.

Finally, you'll need [pg_tmp](https://github.com/eradman/ephemeralpg) installed and available for unit tests to be able to run.

After all of this is done:

* `cargo run` to run the webserver
* `cargo test -- --test-threads 1` to run tests (tests must be run sequentially due to usage of pg_tmp)

## Tips for Rust development

* Use the `rust-analyzer` extension for VSCode
