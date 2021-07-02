mod example;
mod util;

#[linkme::distributed_slice]
static ENTRY_POINTS: [(&str, fn())] = [..];

/**
Entry points can be defined like this:
```
crate::entry_point!("hello", hello);
fn hello() { ... }
```

If you define more than one entry point in one module,
for technical reasons you need to specify unique identifier names:
```
crate::entry_point!("hello1", hello1, _EP_HELLO1);
fn hello1() { ... }

crate::entry_point!("hello2", hello2, _EP_HELLO2);
fn hello2() { ... }
```
*/
#[macro_export]
macro_rules! entry_point {
    ($name:expr, $f:expr) => {
        $crate::entry_point!($name, $f, _ENTRY_POINT);
    };
    ($name:expr, $f:expr, $static_name:ident) => {
        #[linkme::distributed_slice($crate::ENTRY_POINTS)]
        static $static_name: (&'static str, fn()) = ($name, $f);
    };
}

fn ensure_entry_points_unique() {
    for (i, (name, _)) in ENTRY_POINTS.iter().enumerate() {
        for (name2, _) in &ENTRY_POINTS[..i] {
            assert_ne!(name, name2, "duplicate entry point names");
        }
    }
}

#[test]
fn entry_points_unique() {
    ensure_entry_points_unique();
}

fn main() {
    ensure_entry_points_unique();

    if let Some(entry_point) = std::env::args().nth(1) {
        let p = ENTRY_POINTS.iter().find(|(name, _)| name == &entry_point);
        if let Some((_, f)) = p {
            f();
            return;
        } else {
            eprintln!("no entry point {:?}", entry_point);
        }
    } else {
        eprintln!("entry point not specified");
        eprintln!("usage:");
        eprintln!("  cargo run <entry point>");
    }
    eprintln!("possible entry points:");
    for (name, _) in ENTRY_POINTS {
        eprintln!("- {}", name);
    }
    std::process::exit(1);
}
