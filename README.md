# ordiseq

[![Crates.io](https://img.shields.io/crates/v/ordiseq?color=blue
)](https://crates.io/crates/ordiseq)
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://EnigmaCurry.github.io/ordiseq/coverage/master/)

ordiseq is a MIDI sequencer library for Rust.

## Development

### Install host dependencies

```
# Fedora:
sudo dnf install git openssh rustup
sudo dnf install @development-tools @development-libs
```

### Install Rust and cargo

```
rustup-init ## just press enter when prompted for default selection
. "$HOME/.cargo/env"
```

### Clone source repository

```
git clone git@github.com:EnigmaCurry/ordiseq.git \
  ~/git/vendor/EnigmaCurry/ordiseq
cd ~/git/vendor/EnigmaCurry/ordiseq
```

### Install development dependencies

```
cargo install just
just deps
```

## Run examples

Get list of examples:

```
just run --example
```

Run an example:

```
just run --example test
```
