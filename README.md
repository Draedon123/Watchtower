Software to monitor multiple linux machines

# Usage

## Required Software

- [Node.js](https://nodejs.org/en)
- [pnpm](https://pnpm.io)
- [Rust](https://rust-lang.org)

## Building & Running from Source

### Server

1. `cd` into `apps/server`
2. Run `cargo run -r -- --password "YOUR_PASSWORD"`

### Agent

Note: the agent only runs on Linux devices

1. `cd` into `apps/agent`
2. Run `cargo run -r -- --password "YOUR_PASSWORD" --watchtower-address "ADDRESS:PORT"` where `ADDRESS:PORT` is the address and port of the server. This option is mandatory. By default, the server will run on `127.0.0.1:9824`

### Web Interface

1. `cd` into `apps/web`
2. Run `pnpm i && pnpm build`
3. Host the webpage located in `apps/web/build` with any method. One such method is to run `pnpm preview`
