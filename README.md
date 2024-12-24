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


## Credits

 * This library includes data from [The Scale
   Omnibus](https://www.saxopedia.com/the-scale-omnibus) - a
   collection of musical scales by Francesco Balena - (the link may
   not be valid anymore - see [wayback
   link](https://web.archive.org/web/20200220013047/http://www.saxopedia.com/the-scale-omnibus/)).

 * The scales are directly copied from
   [ioanszilagyi/scale_omnibus](https://github.com/ioanszilagyi/scale_omnibus)
   compiled by Corey Hoard - this is a YAML translation of The Scale
   Omnibus v1.02.

 * This library includes reexports from the following crates:

  * [kord](https://crates.io/crates/kord) (klib) - used for Pitches,
    Notes, Chords and more. Created by Aaron Roney.



