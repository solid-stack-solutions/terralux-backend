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

## Development

```sh
# build and run while mocking connection to smart plug
cargo run -F mock_plug

### build and run with more logging
# in posix-compliant shells like bash you can do the following
# to set the environment variable RUST_LOG to a value like
# "terralux_backend=debug" just for executing one command (cargo run).
# on windows using cmd or powershell you might need different syntax.
RUST_LOG=terralux_backend=debug cargo run # more logging
RUST_LOG=terralux_backend=trace cargo run # too much logging
```
