# TIE
Trassenfinder Infrastructure Explorer (TIE)

## Build
Minimal Rust Version 1.39 (`edition = "2018"`)
1. Install [Rust](https://www.rust-lang.org) (via [rustup.rs](https://rustup.rs))
2. Clone the repository
3. Build `cargo build --release`

## Run
```bash
./target/release/tie
```

## Usage
Keys
* `q`: Exit
* `b`: Stations
* `s`: Segments

## Command Line

```bash
$ ./target/debug/tie --help

Trassenfinder Infrastructure Explorer 0.1.0

USAGE:
    tie [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --api-url <api-url>     [default: https://www.trassenfinder.de/api/web/infrastrukturen]
```

## License
MIT