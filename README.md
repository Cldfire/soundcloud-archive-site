# soundcloud-archive-site

A website that provides a simple interface to store an archive of your [SoundCloud](https://soundcloud.com) user data, allowing you both to browse / retrieve the data and to view various statistics.

## Frontend (started by Jarek, will be finished by Ian)

The frontend is written in JS with the Svelte 3 framework. From the `frontend` folder, `npm install` and then `npm run watch`.

(Make sure you also start the backend seperately. Live reload should be working.)

## Backend (by Jarek)

The backend is in `backend` and is written in Rust. It requires Rust nightly due to the usage of Rocket. Make sure you have Rust [installed](https://www.rust-lang.org/tools/install), and then do the following from the repo root:

```bash
rustup override set nightly
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

Also, optionally provide the following for use by some tests (run via `cargo test -- --test-threads 1 --ignored`):

```
SC_CLIENT_ID="..."
SC_OAUTH_TOKEN="..."
```

See the [orange-zest README](https://github.com/Cldfire/orange-zest#obtaining-soundcloud-auth-credentials) for details on obtaining these values.

Finally, you'll need [pg_tmp](https://github.com/eradman/ephemeralpg) installed and available for unit tests to be able to run.

After all of this is done:

* `cargo run` to run the webserver
* `cargo test -- --test-threads 1` to run tests (tests must be run sequentially due to usage of pg_tmp)
* `cargo test -- --test-threads 1 --ignored` to run tests that involve scraping data from soundcloud (note that you must set some environment variables first, see above)

Note: we unfortunately are forced to use Rocket from the repo master branch due to v0.4 of Rocket requiring an older version of `ring` than what a dependency of `orange-zest` requires, and `ring` not supporting cross-version linking. The Rocket master branch has an updated `ring` dependency.

## Tips for Rust development

* Use the `rust-analyzer` extension for VSCode
