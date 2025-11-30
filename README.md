# winternight

<img height="300" alt="cover art" src="https://github.com/user-attachments/assets/c4853d4b-56b1-4fa9-9d00-b2a4775f2384" />
<img height="300" alt="Screenshot 2025-11-30 192100" src="https://github.com/user-attachments/assets/0eeadaf5-6a86-43dc-82ac-db622cc4cc34" />


When a sudden snowstorm strikes the town, you have to take care of the cold and lost people who show up at your house! A very cute and wholesome game to lighten spirits in these dark winter weeks :>

Made in Rust for Hackclub's Siege, week 13 with the theme winter.

As usual, all assets done by myself.

## build

Project is written in rust, so you'll need that installed. You can just run it with `cargo run`.

If you want to build for web, serving with for instance `basic-http-server`, do:
```bash
 cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/winternight.wasm web/ && basic-http-server web/
```
