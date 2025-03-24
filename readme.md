[WEB_CLIENT]

wasm-pack build --target web
cargo run --bin server

serve index file (client)
python -m http.server 8000

[WEB_SERVER]
This will start websocket server on 127.0.0.1:8080
cargo run --bin server --features server

[NATIVE_CLIENT_macOS]

Run Natively
cargo run --bin native_client --features native

This will compile the code in release mode, optimizing the executable for performance.
cargo build --bin native_client --features native --release

To install cargo-bundle:
cargo install cargo-bundle

This will package your project into a macOS application bundle (.app), which you can distribute as a standard macOS application.
cargo bundle --bin native_client --features native --release
