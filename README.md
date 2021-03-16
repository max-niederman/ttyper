# ttyper

[![Crates.io](https://img.shields.io/crates/v/ttyper)](https://crates.io/crates/ttyper)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/max-niederman/ttyper/Rust)](https://github.com/max-niederman/ttyper/actions)
[![Dependency Status](https://img.shields.io/librariesio/release/cargo/ttyper)](https://libraries.io/cargo/ttyper/tree?kind=normal)
[![GitHub issues](https://img.shields.io/github/issues/max-niederman/ttyper)](https://github.com/max-niederman/ttyper/issues)
[![License](https://img.shields.io/crates/l/ttyper)](./LICENSE.md)

Ttyper is a terminal-based typing test built with Rust and tui-rs.

![Recording](./resources/recording.gif)

## installation

With [Cargo](https://crates.io):

```bash
cargo install ttyper
```

## usage
For usage instructions, you can run `ttyper --help`. Currently available languages are `english100`, `english200`, and `english1000`.

### examples

| command                         | test contents                               |
| :------------------------------ | ------------------------------------------: |
| `ttyper`                        | 50 of the 200 most common english words     |
| `ttyper -w 100`                 | 100 of the 200 most common English words    |
| `ttyper -w 100 -l english1000`  | 100 of the 1000 most common English words   |
| `ttyper text.txt`               | contents of `text.txt` split at whitespace  |
