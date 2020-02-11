# soundcloud-archive-site

A website that provides a simple interface to store an archive of your SoundCloud user data, allowing you both to browse / retrieve the data and to view various statistics.

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

After that's done:

* `cargo run` to run the webserver
* `cargo test` to run tests

## Tips for Rust development

* Use the `rust-analyzer` extension for VSCode
