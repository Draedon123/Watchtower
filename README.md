Software to monitor multiple linux machines. Consists of an agent that runs on each machine that communicates with a central server. The server can then be queried and interfaced through a webpage. The agent and server both hinge on a custom-built HTTP library.

# Usage

## Required Software

- [Node.js](https://nodejs.org/en)
- [pnpm](https://pnpm.io)
- [Rust](https://rust-lang.org)
- [Linux](https://www.linux.org) (for the agents)

## Building from Source

### Server

1. `cd` into `apps/server`
2. Run `cargo build -r`
3. Locate your built executable at `target/release/watchtower-server`

### Agent

Note: the agent only runs on Linux devices

1. `cd` into `apps/agent`
2. Run `cargo build -r`
3. Locate your built executable at `target/release/watchtower-agent`

### Web Interface

1. `cd` into `apps/web`
2. Change the server address in `apps/web/src/lib/request.ts` to match the address and port of the server
3. Run `pnpm i && pnpm build`
4. Locate your built webpage at `apps/web/build`

## Running

1. Download the precompiled executables and webpage from the latest release, or build it from source, as instructed above. Unzip the webpage if downloaded from the release
2. `cd` into the location of the `watchtower-server` executable
3. Run `./watchtower-server --password "YOUR_PASSWORD" --threads THREAD_COUNT`. `--threads THREAD_COUNT` is optional. `THREAD_COUNT` must be an integer greater than 0 and defaults to 4. For more help, run `./watchtower-server -h`
4. `cd` into the location of the `watchtower-agent` executable
5. Run `./watchtower-agent -r -- --password "YOUR_PASSWORD" --watchtower-address "ADDRESS:PORT"` where `ADDRESS:PORT` is the address and port of the server. This option is mandatory. By default, the server will run on `127.0.0.1:9824`. For more help, run `./watchtower-agent -h`
6. `cd` into the location of the webpage
7. Host the webpage located with any method. Possible methods include (but are not limited to) running `pnpm preview` or hosting it on Vercel/other static site hosters
