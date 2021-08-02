# ttyper

[![Crates.io](https://img.shields.io/crates/v/ttyper)](https://crates.io/crates/ttyper)
[![GitHub Stars](https://img.shields.io/github/stars/max-niederman/ttyper)](https://github.com/max-niederman/ttyper)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/max-niederman/ttyper/Rust)](https://github.com/max-niederman/ttyper/actions)
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

For usage instructions, you can run `ttyper --help`.

### examples

| command                        |                              test contents |
| :----------------------------- | -----------------------------------------: |
| `ttyper`                       |    50 of the 200 most common english words |
| `ttyper -w 100`                |   100 of the 200 most common English words |
| `ttyper -w 100 -l english1000` |  100 of the 1000 most common English words |
| `ttyper text.txt`              | contents of `text.txt` split at whitespace |

## languages

The following languages are available by default:

| name          |                         description |
| :------------ | ----------------------------------: |
| `english100`  |       100 most common English words |
| `english200`  |       200 most common English words |
| `english1000` |      1000 most common English words |
| `c`           |          The C programming language |
| `csharp`      |         The C# programming language |
| `go`          |         The Go programming language |
| `html`        |           HyperText Markup Language |
| `java`        |       The Java programming language |
| `javascript`  | The Javascript programming language |
| `norwegian`   |     200 most common Norwegian words |
| `german`      |        207 most common German words |
| `php`         |        The PHP programming language |
| `python`      |     The Python programming language |
| `ruby`        |       The Ruby programming language |
| `rust`        |       The Rust programming language |
| `qt`          |          Common QT framework tokens |

Additional languages can be added by creating a file in `TTYPER_CONFIG_DIR/language` with a word on each line. On Linux, the config directory is `$HOME/.config/ttyper`; on Windows, it's `C:\Users\user\AppData\Roaming\ttyper`; and on macOS it's `$HOME/Library/Application Support/ttyper`.
