# owfs-rs

Rust API for [OWFS](https://owfs.org) (1-Wire file system)

Example:

```rust,no_run
// OWFS guard object, automatically calls `owcapi::OW_finish` on drop
let _og = owfs::init("localhost:4304").unwrap();
let devices = owfs::scan(owfs::ScanOptions::default()).unwrap();
for d in devices {
dbg!(&d.info());
if d.attrs().contains(&"PIO.1") {
    d.set("PIO.1", "1").unwrap();
}
}
```

Requires `libow` and `libowcapi` to be installed on the host. It is also
possible to tell the crate to compile the library from source (specify
**vendored** feature).
