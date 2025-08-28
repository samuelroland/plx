<center>

![logo of PLX](imgs/logo.svg)
</center>

### **P**ractice programming exos in a deliberate **L**earning e**X**perience

###### Useful links
[Website](https://plx.rs) -
[WHY ?](https://plx.rs/book/why.html) -
[Git repository of PLX website](https://github.com/plx-pdg/plx-pdg.github.io)

### Introduction

PLX is a project developed to enhance the learning of programming languages, with a focus on a smooth and optimized learning experience. The goal of this project is to reduce the usual friction involved in completing coding exercises (such as manual compilation, running, testing, and result verification) by automating these steps.

PLX offers a terminal user interface (TUI) developed in Rust and supports multiple languages (currently C and C++). It enables automatic compilation as soon as a file is saved, automated checks to compare program outputs, and instant display of errors and output differences. The solution code can also be displayed. The project draws inspiration from [Rustlings](https://rustlings.cool/) and aims to create a more efficient learning experience, particularly for programming courses at HEIG-VD.

### Docs

We deploy documentations on [our website](https://plx.rs/book).

## Installation

You need Git and the Rust toolchain 1.87 ([See installation via Rustup](https://rustup.rs/)). You also need a C or C++ compiler, depending on what programming language you want to use.

```sh
git clone https://github.com/samuelroland/plx.git
cd plx
```

### Developing on the core library only
```sh
cargo build
```

Look at `src/lib.rs` for now and generate the doc with `cargo doc` to understand more about the available data structures.

How to run tests
```sh
cargo test
```

How to run tests and include ignored tests (they are marked as `#[ignored]` because they need network access or are slow to run)
```sh
cargo test -- --include-ignored
```

### The PLX CLI

Build the CLI. This will install all necessary dependencies and build the program in release mode.
```bash
cd cli
cargo build --release
```

To run it
```bash
cd cli
cargo run
```

Install the CLI globally
```bash
cd cli
cargo install --path .
```

Now you can try to run `plx`

### The desktop app
1. Make sure you have the Tauri prequisites so all build dependencies will be present: [Tauri prequisites](https://tauri.app/start/prerequisites/)
1. The frontend is built using [NodeJS v22+](https://nodejs.org) and [Pnpm 10+](https://pnpm.io/), make sure you have both of them

#### Running the desktop app for development
Just run
```sh
cd desktop
pnpm install
pnpm tauri dev
```

#### Building the desktop app for production
**WARNING: This is working mostly on Linux, installers for Windows are generated as `.msi` and for MacOS as `.dmg` but some features have not been tested or do not work**.
1. To build and generate a bundle for your platform
    ```sh
    cd app
    pnpm install
    pnpm tauri build && pnpm tauri bundle
    ```
1. On Fedora
   ```sh
   sudo dnf install src-tauri/target/release/bundle/deb/plx_0.1.0_amd64.deb
   ```
1. Or on Ubuntu
   ```sh
   sudo apt install src-tauri/target/release/bundle/rpm/plx-0.1.0-1.x86_64.rpm
   ```
1. On Windows: look at the generated `.msi` under `src-tauri/target/release/bundle/msi`.
1. On MacOs: look at the generated `.dmg` under `src-tauri/target/release/bundle/dmg`.

### Testing using a demo course

Once you have plx installed, you can try it on this repo's example folder

> [!IMPORTANT] 
> Set the $EDITOR environment variable if you wish for your editor to be opened when starting an exo

> [!WARNING] 
> The open editor feature is currently unstable, using a terminal based editor causes problems
> The following editors were tested and work fine: `code`, `clion` and `codium`

> [!IMPORTANT] 
> Only C and C++ exercises are valid for now, java and other languages support is comming soon™

On Linux and MacOS, you can easily change `EDITOR` just for PLX, here is an example for `VSCode`.
```sh
EDITOR=code plx-desktop
```

You might also find PLX in your start menu. To test PLX with a demo course use this repository `https://github.com/samuelroland/plx-demo/` via the `Add course` button.

### License

We are currently waiting for our school's approval before applying an open source license.
