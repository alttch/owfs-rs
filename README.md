# owfs-rs

Rust SDK for [OWFS](https://owfs.org) (1-Wire file system)

Example:

```rust
unsafe {
    owfs::init("localhost:4304").unwrap();
    let devices = owfs::scan(owfs::ScanOptions::default()).unwrap();
    for d in devices {
        dbg!(&d.info());
        if d.attrs().contains(&"PIO.1") {
            d.set("PIO.1", "1").unwrap();
        }
    }
    owfs::finish();
}
```

Requires linking with owcapi lib, build.rs example:

```rust
fn main() {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match os.as_str() {
        "linux" => println!("cargo:rustc-link-lib=owcapi"),
        _ => unimplemented!(),
    };
    println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
}
```
