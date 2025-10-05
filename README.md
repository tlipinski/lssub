# subster

A Rust CLI and TUI application for interacting with the OpenSubtitles API.

The project provides:
- A command-line interface to authenticate, query features (titles), and search for subtitles.
- A terminal user interface (TUI) to browse files and search for subtitles interactively.
- A small internal `osb` library crate that wraps HTTP calls to the OpenSubtitles API.

Note: This README documents what is present in the repository without inventing unknown details. Items that need more information are marked as TODO.


## Stack
- Language: Rust (Edition 2024 for the main crate; Edition 2021 for `osb` crate)
- Package manager & build: Cargo
- Async runtime: tokio
- CLI: clap
- TUI: ratatui (with crossterm and ratatui-explorer)
- HTTP client: reqwest
- Config: TOML via `toml` crate and XDG dirs via `xdg`
- Secrets storage: libsecret (Secret Service API) via `libsecret` crate
- Logging: env_logger (configured to write to `logs.txt`)


## Features (high-level)
- Login and store an API token securely using the system’s Secret Service (libsecret).
- Query OpenSubtitles for:
  - Features (titles) by text query.
  - Subtitles by filename and selected languages.
- Fetch and display basic user info once authenticated.
- TUI mode to:
  - Browse local files in a pane.
  - Search for subtitles for the selected filename.


## Requirements
- Rust toolchain (rustup, cargo)
- Network access to OpenSubtitles API
- System libraries for libsecret (Secret Service):
  - Linux: a Secret Service implementation (e.g., GNOME Keyring, KeePassXC’s secret service), plus development headers if building from source.
  - Other OSes: TODO: Verify support and provide instructions.
- A valid OpenSubtitles API key available at build time via environment variable `OSBK` (see Configuration & Environment Variables).
- A terminal that supports crossterm/ratatui for the TUI mode.
- Optional: `urxvt` if you want to use the provided `gui-term.sh` helper script (otherwise you can run the TUI directly with cargo).


## Configuration & Environment Variables
- Build-time API key (required):
  - `OSBK` — Your OpenSubtitles API key. It is consumed by the Cargo build script (`build.rs`) and embedded as a compile-time environment value in the `osb` crate.
  - Example:
    - `export OSBK=YOUR_OPENSUBTITLES_API_KEY`

- Runtime configuration file:
  - Path: `~/.config/subster/config.toml` (XDG base directories with prefix `subster`).
  - Current schema: The `Config` struct in the code is empty, but the application tries to read and parse this file on startup. Create an empty file to avoid errors until configuration fields are defined.
  - Minimal setup:
    - `mkdir -p ~/.config/subster`
    - `: > ~/.config/subster/config.toml`  (an empty file)
  - TODO: Document configuration fields once they are introduced.

- Secrets storage:
  - After a successful login, the API token is stored in the system Secret Service using schema `com.subster` with `username` attribute. The token is later retrieved for authenticated operations (e.g., `userinfo`).

- Logging:
  - Logs are written to `logs.txt` in the repository’s working directory at runtime, with log level set to `Debug` by default in code. The setup currently does not honor `RUST_LOG` since the logger is configured programmatically.


## Installation
1. Ensure `OSBK` is set in your environment (see above).
2. Ensure you have a working Secret Service (libsecret) setup on your system (Linux) if you intend to use login/userinfo.
3. Build:
   - `cargo build`

Optionally install the binary:
- `cargo install --path .`


## Usage
All commands are subcommands of the `subster` binary.

- Show help and version:
  - `cargo run -- --help`
  - After installation: `subster --help`

- Login (stores token in secret storage):
  - `cargo run -- login`
  - You will be prompted for username and password.

- Logout (clears stored token):
  - `cargo run -- logout`

- Show user info (requires stored token):
  - `cargo run -- userinfo`

- Search for subtitles (by filename and languages):
  - `cargo run -- search <file_path> <lang1> <lang2> ...`
  - Example:
    - `cargo run -- search "/path/to/Movie.2024.1080p.mkv" en pl`

- Search features (titles) by query string:
  - `cargo run -- features "The Matrix"`

- Launch TUI mode (optional file path to focus file name in search):
  - `cargo run -- gui [file_path]`
  - Example:
    - `cargo run -- gui` (no initial file)
    - `cargo run -- gui "/media/Movies/Movie.2024.1080p.mkv"`


## TUI Controls (basic)
- ESC — go back/exit context; in main screens it exits TUI.
- F10 — exit application from the main screen.
- s — switch focus to the search input.
- Tab — cycle between panes (Explorer, Search, Results table).
- Explorer pane: Enter — select current file and send its name to the search field.
- Note: Additional widget-specific key handling exists; this section only includes what’s evident from the code. TODO: Document full keybindings.


## Scripts
- `gui-term.sh` — Launches the TUI in a `urxvt` terminal:
  - `./gui-term.sh`
  - Requirements: `urxvt` in PATH, Cargo installed.
  - You can run the same directly without `urxvt` via `cargo run -- gui`.


## Project Structure
- `Cargo.toml` — Main crate manifest (`subster`).
- `build.rs` — Build script that requires `OSBK` environment variable.
- `src/main.rs` — Application entry point (tokio async main) that initializes logging, loads config, and dispatches CLI subcommands.
- `src/cli/` — CLI module:
  - `command.rs` — clap `Subcommand` enum: `login`, `logout`, `userinfo`, `search`, `features`, `gui`.
  - `login_cmd.rs`, `logout_cmd.rs`, `search_cmd.rs`, `features_cmd.rs`, `gui_cmd.rs` — corresponding handlers.
- `src/ui/` — TUI components built on ratatui and crossterm:
  - `app.rs` — Application state, event loop, layout, and key handling.
  - `explorer_widget.rs` — File explorer pane using `ratatui-explorer`.
  - `search_widget.rs`, `subs_widget.rs`, `features_fetcher.rs`, `events.rs`, `input_handler.rs` — components for search input, results, features fetching, event routing, and input handling.
- `src/bin/test.rs` — A small, separate binary demonstrating a minimal ratatui usage (Hello World on key press).
- `osb/` — Internal library crate wrapping OpenSubtitles API calls:
  - `osb/src/login.rs` — Login and token handling.
  - `osb/src/user_info.rs` — Fetch user info using stored token.
  - `osb/src/subtitles.rs` — Search subtitles by filename and languages.
  - `osb/src/features.rs` — Search features (titles) by query.
  - `osb/src/values.rs` — API constants: `API_URL`, `USER_AGENT`, and `KEY` derived from build-time env `OSBK`.
  - `osb/src/guess_search.rs` — Helper for guess-based queries (used by UI or future features).


## Development
- Run clippy and fmt (if desired):
  - `cargo fmt`
  - `cargo clippy`
- Run in watch mode (if you use cargo-watch):
  - `cargo watch -x run`  (requires installing cargo-watch; optional)


## Tests
- No tests were found in the repository at the time of writing.
- TODO: Add unit tests and integration tests for CLI commands, TUI logic (as feasible), and `osb` HTTP layer with mocked responses.


## License
- No license file was found in the repository.
- TODO: Add a LICENSE file and specify the project’s license in `Cargo.toml`.


## Troubleshooting
- Build error: `OSBK not set` — Ensure `export OSBK=...` is set before `cargo build`.
- Runtime error loading config — Create an empty file at `~/.config/subster/config.toml` until configuration fields are defined.
- Login issues — Verify your OpenSubtitles credentials; token is stored via libsecret; ensure a Secret Service is running.
- TUI display/input issues — Ensure your terminal supports crossterm; try another terminal emulator.
