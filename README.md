# ordiseq

[![Crates.io](https://img.shields.io/crates/v/ordiseq?color=blue
)](https://crates.io/crates/ordiseq)
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://EnigmaCurry.github.io/ordiseq/coverage/master/)

ordiseq is a MIDI sequencer library for Rust, as well as a collection
of related sub-crates:

 * [ordiseq](ordiseq) - a MIDI sequenc_er.
 * [scale_omnibus](scale_omnibus) - a library of musical scales.
 
## Development

### Install host dependencies

```
# Fedora:
sudo dnf install git openssh rustup just
sudo dnf install @development-tools @development-libs
```

### Install Rust and cargo

```
rustup-init  ## Enter the default prompts
rustup toolchain install nightly
. "$HOME/.cargo/env"
```

### Clone the source repository

```
git clone git@github.com:EnigmaCurry/ordiseq.git \
  ~/git/vendor/EnigmaCurry/ordiseq
cd ~/git/vendor/EnigmaCurry/ordiseq
```

### Install the development dependencies

```
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

 * [scale_omnibus](scale_omnibus) includes data from [The Scale
   Omnibus](https://www.saxopedia.com/the-scale-omnibus) - a massive
   book of musical scales by Francesco Balena -
   ([wayback](https://web.archive.org/web/20200220013047/http://www.saxopedia.com/the-scale-omnibus/)).

   * The scale data was copied from the book (v1.02) by Corey Hoard
     and saved as a YAML file published at
     [ioanszilagyi/scale_omnibus](https://github.com/ioanszilagyi/scale_omnibus)
     and reproduced in
     [scale_omnibus/data/scales.yaml](scale_omnibus/data/scales.yaml)

 * [ordiseq](ordiseq) includes reexports from the following crates:

   * [kord](https://crates.io/crates/kord) (klib) - used for Pitches,
     Notes, Chords and more. Created by Aaron Roney.

 * [ordiseq_plug](ordiseq_plug) uses
   [nih-plug](https://github.com/robbert-vdh/nih-plug) and started as
   a copy of [one of the nih-plug
   examples](https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/midi_inverter/src/lib.rs).

 * Many other libraries from the Rust ecosystem have been used as
   listed in each crate's Cargo.toml. Thank you to all the library
   authors and supporters.
