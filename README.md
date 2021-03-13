# ttyper

Ttyper is a terminal-based typing test built with Rust and tui-rs.

![Recording](./resources/recording.gif)

## installation

With Cargo:

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

## to-do

- Write unit tests.
- Use per-word frequency data for more realistic tests.
- Add keywise data to the results UI.
- Add WPM graph to results UI.
