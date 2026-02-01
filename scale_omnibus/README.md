# The Scale Omnibus

[![Crates.io](https://img.shields.io/crates/v/scale_omnibus?color=blue
)](https://crates.io/crates/scale_omnibus)
[![Docs](https://img.shields.io/badge/Docs-red)](https://docs.rs/scale_omnibus)
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://enigmacurry.github.io/scale_omnibus/coverage/master/)
[![Matrix chat](https://img.shields.io/badge/Matrix-Join_Chat-%234fb99a)](https://matrix.to/#/#blog.rymcg.tech:enigmacurry.com)

[The Scale Omnibus](http://www.saxopedia.com/the-scale-omnibus/)
([wayback](https://web.archive.org/web/20200220013047/http://www.saxopedia.com/the-scale-omnibus/))
is a book written by Francesco Balena, which is a catalouge of musical
scales and their intervals. The `scale_omnibus` crate provides this
data in the form of a Rust library.

This library contains YAML data compiled by Corey Hoard: [ioanszilagyi/scale_omnibus](https://github.com/ioanszilagyi/scale_omnibus)

## Features

- More than 1000 musical scales.
- Retrieve scales directly by name.
- Search for scales based on any criteria, such as origin, name
  substring match, or the number of intervals.
- CLI tool for exploring scales from the command line.

## CLI

Build and install the CLI with the `cli` feature:

```bash
cargo install scale_omnibus --features cli
```

### Options

| Option | Description |
|--------|-------------|
| `--json` | Output in JSON format |

### Commands

| Command | Description |
|---------|-------------|
| `get <name>` | Get a scale by name (case-insensitive) |
| `list` | List all scale names |
| `search <pattern>` | Search scales by name pattern |
| `origin <origin>` | Find scales by origin/culture |
| `intervals <min>` | Find scales with more than N intervals |
| `asymmetric` | Find scales with different ascending/descending intervals |
| `count` | Count total number of scales |
| `origins` | List all unique origins |

### Examples

```bash
# Get details about a specific scale
scale_omnibus get major

# Search for scales by name
scale_omnibus search pentatonic

# Find scales from a specific origin
scale_omnibus origin india

# List all scales with different ascending/descending intervals
scale_omnibus asymmetric

# Count total scales
scale_omnibus count
```

