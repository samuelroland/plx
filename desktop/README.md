# PLX desktop

## Introduction
This is the desktop app, replacing the previous TUI, currently in construction.

## Build

Make sure your have, Rust, NodeJS, PNPM and Git

Install `typeshare`
```sh
cargo install typeshare-cli
```

```sh
pnpm tauri dev
```

To generate installers for different plateforms
```sh
pnpm tauri bundle
```
