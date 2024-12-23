# ordiseq

[![Crates.io](https://img.shields.io/crates/v/ordiseq?color=blue
)](https://crates.io/crates/ordiseq)
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://EnigmaCurry.github.io/ordiseq/coverage/master/)


## Install

[Download the latest release for your platform.](https://github.com/EnigmaCurry/ordiseq/releases)

Or install via cargo ([crates.io/crates/ordiseq](https://crates.io/crates/ordiseq)):

```
cargo install ordiseq
```

### Tab completion

To install tab completion support, put this in your `~/.bashrc` (assuming you use Bash):

```
### Bash completion for ordiseq (Put this in ~/.bashrc)
source <(ordiseq completions bash)
```

If you don't like to type out the full name `ordiseq`, you can make
a shorter alias (`h`), as well as enable tab completion for the alias
(`h`):

```
### Alias ordiseq as h (Put this in ~/.bashrc):
alias h=ordiseq
complete -F _ordiseq -o bashdefault -o default h
```

Completion for Zsh and/or Fish has also been implemented, but the
author has not tested this:

```
### Zsh completion for ordiseq (Put this in ~/.zshrc):
autoload -U compinit; compinit; source <(ordiseq completions zsh)

### Fish completion for ordiseq (Put this in ~/.config/fish/config.fish):
ordiseq completions fish | source
```

## Usage

```
$ ordiseq

Usage: ordiseq [OPTIONS] [COMMAND]

Commands:

Options:
  -h, --help                  Print help
  -V, --version               Print version
```

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md)
