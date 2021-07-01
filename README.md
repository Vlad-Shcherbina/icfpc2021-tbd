### Installing Rust

You need relatively recent Rust nightly toolchain.

On any Linux:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
sh rustup.sh --default-toolchain nightly --profile minimal -y
rm rustup.sh
```

On Windows:

1. Download `rustup.exe` from https://rustup.rs/
2. Run it.
3. Select "toolchain: nightly", "profile: minimal", leave everything else as is.
4. Select "Install".
5. You also need to install Visual Studio or Visual Studio Build Tools (C++).

To check it's working:

```
$ cargo --version
cargo 1.54.0-nightly (44456677b 2021-06-12)
$ rustc --version
rustc 1.55.0-nightly (e82b65026 2021-06-20)
```

### How to run

The whole project is a single executable compilation unit,
because dealing with multiple compilation units is annoying.
It is very likely that we'll need to implement many tools.
All these tools will be part of this executable,
accessed by different entry points.
See `src/main.rs` for how this is implemented and how to define entry points.
Which one to run is decided by the first command line argument:
```
cargo run hello
```
