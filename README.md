### Installing Rust

You need relatively recent Rust nightly toolchain.

On any Linux:

1. ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
   sh rustup.sh --default-toolchain nightly --profile minimal -y
   rm rustup.sh
   ```
2. ```
   sudo apt install -y lld
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

You can also compile and run in release mode:
```
cargo run --release hello
```

### How to test

All tests:
```
cargo test
```

To run individual test:
```
cargo test -- --nocapture test_name
```

In addition, try to fix or silence compiler warnings.
Also please run `cargo clippy` from time to time,
and fix or silence its warnings too (most Clippy lints
are good, but not all, so use common sense).

### Git workflow

We push everything to the main branch.
There are no mandatory code reviews.

Prefer fast-forward or rebase to merge.
Though infrequent merge commits are fine.

Push as often as possible while
avoiding breaking compilation or tests.

Pre-push hook that runs `cargo test` helps with that:
```
cp git_hooks/pre-push .git/hooks/
```

### Data directories

* `data/` is for stuff like example inputs from the problem statement.
  Produced by humans for programs.
  Everything here is under version control.
* `cache/` is for generated files that could be helpful to have around,
  but should be safe to delete at any time
  (it's possible to regenerate them if needed).
  Produced by programs for programs.
  It's in gitignore.
* `outputs/` is for logs, images rendered by visualizers, etc.
  In other word, stuff produced by programs for humans.
  It's in gitignore.

By the way, there is an utility to get paths relative to the project root:
`project_path("data/example.txt")`.

### How to run visualizer

1. Install TypeScript:
   ```
   npm install -g typescript@latest
   ```
2. Start `tsc` in the watch mode and let it run
   (it will generate js files from ts files as they change):
   ```
   cd icfpc2021-tbd
   tsc --watch
   ```
3. Start the Web server:
   ```
   cargo run viz_server
   ```
4. Navigate to http://127.0.0.1:8000/src/viz/static/viz.html#42
   (where 42 is the problem number)
