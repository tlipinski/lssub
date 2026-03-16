# lssub

`lssub` is a terminal app for finding and downloading movie/TV subtitles from OpenSubtitles.

## What it does
- Search subtitles by title
- Filter by language(s)
- Optionally log in to your OpenSubtitles account (for account details and quota info)
- Save selected subtitles directly to disk

## Requirements
- Linux terminal
- Rust toolchain (`cargo`, `rustc`) to build
- Internet connection
- Secret Service available on your desktop session (for login token storage), for example:
  - GNOME Keyring
  - KeePassXC with Secret Service integration
- `libsecret` development package installed to compile

## Install and run
From the project directory:

```bash
cargo build --release
cargo run --release -- [PATH]
```

You can also use the built binary:

```bash
./target/release/lssub [PATH]
```

`PATH` is optional:
- no `PATH`: subtitles are downloaded to your current directory
- `PATH` is a directory: subtitles are downloaded there
- `PATH` is a file path: its parent directory is used as download directory, and file stem is used as initial search query and output base name

Example:

```bash
lssub "/media/videos/Example.Movie.2014.mkv"
```

This pre-fills search with `Example.Movie.2014` and saves subtitles under `/media/videos`.

## How to use
Main navigation:
- `F2` Search
- `F3` Account
- `F4` Languages
- `F12` About
- `F10` or `Ctrl+C` Exit
- `Esc` Back to Search

Search screen:
- Type to search
- `Up` / `Down` / `PageUp` / `PageDown` to move in results
- `Enter` to download selected subtitle
- `F1` to open/close search help popup
- `Ctrl+S` toggle "single title" narrowing (based on selected row)
- `Ctrl+T` include/exclude AI-translated subtitles

Languages screen:
- Enter comma-separated language codes (for example: `en,es`)
- Press `Enter` to apply and save

Account screen:
- If logged out: enter OpenSubtitles username/password, press `Enter` to log in
- If logged in: press `Ctrl+O` to log out

## Downloaded file names
Downloaded subtitles include the subtitle language in file name.

Examples:
- base name from input file: `Movie.en.srt`
- base name from subtitle result title: `Some.Release.en.srt`

Final extension is taken from OpenSubtitles response when available; otherwise `.srt` is used.

## Configuration
Config file is created automatically at:

`~/.config/lssub/config.toml`

Current format:

```toml
languages = ["en"]
```

## Logs
Run log is written to:

`/tmp/lssub.log`

The file is overwritten on each start.

## Troubleshooting
- Login/token errors: ensure a Secret Service provider is running in your session.
- Build fails on `libsecret`: install your distro's `libsecret` dev package.
- No results: check internet connection and use at least 3 characters in search query.
