# lssub

Rust TUI for searching and downloading subtitles via the OpenSubtitles API.

What it does:
- Search subtitles by typing a query (debounced; queries shorter than 3 characters return no results).
- Configure languages (persisted to a config file).
- Log in/out and show basic account/download quota info (token stored via Secret Service / libsecret).
- Download the selected subtitle to disk.


## Requirements
- Rust toolchain (rustup, cargo)
- Network access to OpenSubtitles
- Build-time OpenSubtitles API key in `OSBK` (required; see below)
- Secret Service + libsecret (for login/logout and user info)
  - Linux: e.g., GNOME Keyring or KeePassXC Secret Service, plus libsecret development headers to compile
  - Other OSes: unverified


## Build And Run
`OSBK` is read by [`build.rs`](build.rs) at build time and embedded as a compile-time env var.

```sh
export OSBK=YOUR_OPENSUBTITLES_API_KEY
cargo build

# Run the TUI
cargo run -- [PATH]
```

`PATH` is optional:
- If `PATH` is a directory: downloads are saved into that directory.
- If `PATH` is a file: downloads are saved into the file’s parent directory and the file stem is used as:
  - the initial search query
  - the output subtitle base name (extension is derived from the downloaded subtitle file name; falls back to `.srt`)
- If `PATH` is omitted: the current working directory is used for downloads.


## Configuration
Config is stored at `~/.config/lssub/config.toml` (XDG base dirs with prefix `lssub`).

If the file does not exist, it is created automatically with defaults.

Current schema:
```toml
languages = ["en"]
```


## Logging
Logs are written to `/tmp/lssub.log` and truncated on each run.
