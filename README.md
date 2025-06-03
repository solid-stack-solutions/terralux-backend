# Terralux Backend
### Powered by [SunriseSunset.io](https://sunrisesunset.io)

## Installation

### Using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
> If this is your first time working with Rust, set it up using [rustup](https://rustup.rs)
```sh
# fetch the source
git clone https://github.com/solid-stack-solutions/terralux-backend
cd terralux-backend
# build and run
cargo run
# build and run with more logging
RUST_LOG=terralux_backend=trace cargo run
```
### Using [Nix Flakes](https://wiki.nixos.org/wiki/Flakes)
```sh
# option 1: fully automatic
nix run github:solid-stack-solutions/terralux-backend
# option 2: fetch source, build and run
git clone https://github.com/solid-stack-solutions/terralux-backend
cd terralux-backend
nix run
```
