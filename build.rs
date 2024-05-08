#[cfg(feature = "vendored")]
fn compile() {
    use std::env;
    use std::fs::File;
    use std::path::Path;
    use std::process::Command;
    use ureq::Agent;
    const VENDORED_OWFS_VERSION: &str = "3.2p4";
    let num_cpus = env::var("CARGO_BUILD_JOBS").unwrap_or_else(|_| num_cpus::get().to_string());
    let target = env::var("TARGET").unwrap_or_default();
    let (gcc_compiler, host) = match target.as_str() {
        "x86_64-unknown-linux-gnu" => ("gcc", "x86_64-linux"),
        "x86_64-unknown-linux-musl" => ("x86_64-linux-musl-gcc", "x86_64-linux"),
        "arm-unknown-linux-musleabihf" => ("arm-linux-musleabihf-gcc", "arm-linux"),
        "arm-unknown-linux-gnueabihf" => ("arm-unknown-linux-gnueabihf-gcc", "arm-linux"),
        "aarch64-unknown-linux-gnu" => ("aarch64-linux-gnu-gcc", "arm-linux"),
        "aarch64-unknown-linux-musl" => ("aarch64-linux-musl-gcc", "arm-linux"),
        _ => ("gcc", ""),
    };
    println!("cargo:rustc-env=CC={}", gcc_compiler);
    let url = format!(
        "https://github.com/owfs/owfs/releases/download/v{}/owfs-{}.tar.gz",
        VENDORED_OWFS_VERSION, VENDORED_OWFS_VERSION
    );
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let tar_path = Path::new(&out_dir).join(format!("owfs-{}.tar.gz", VENDORED_OWFS_VERSION));
    let extract_path = Path::new(&out_dir);
    let work_path = extract_path.join(format!("owfs-{}", VENDORED_OWFS_VERSION));

    let agent = Agent::new();
    let response = agent.get(&url).call().unwrap();
    let mut file = File::create(&tar_path).unwrap();
    std::io::copy(&mut response.into_reader(), &mut file).unwrap();
    Command::new("tar")
        .args([
            "-xzf",
            tar_path.to_str().unwrap(),
            "-C",
            extract_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(Command::new("./configure")
        .arg(format!("--host={}", host))
        .args([
            "--with-fuseinclude=/usr/include",
            "--with-fuselib=/usr/lib",
            "--disable-owtcl",
            "--disable-owphp",
            "--disable-owpython",
            "--disable-zero",
            "--disable-owshell",
            "--disable-owhttpd",
            "--disable-owftpd",
            "--disable-owserver",
            "--disable-owperl",
            "--disable-owtap",
            "--disable-owmon",
            "--disable-owexternal",
            "--disable-usb",
            "--enable-static",
        ])
        .env("CC", gcc_compiler)
        .current_dir(&work_path)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("make")
        .arg("-j")
        .arg(num_cpus)
        .env("CC", gcc_compiler)
        .current_dir(&work_path)
        .status()
        .unwrap()
        .success());
    for (n, lib_n) in [("owlib", "libow.a"), ("owcapi", "libowcapi.a")] {
        let lib_path = work_path.join("module").join(n).join("src/c/.libs");
        assert!(Command::new("ar")
            .args(["rcs", lib_n])
            .args(objs(&lib_path))
            .current_dir(&lib_path)
            .status()
            .unwrap()
            .success());
        println!("cargo:rustc-link-search={}", lib_path.display());
    }
}

#[cfg(feature = "vendored")]
fn objs(path: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("o") {
                path.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(feature = "vendored")]
fn lib() {
    compile();
    println!("cargo:rustc-link-lib=static=owcapi");
    println!("cargo:rustc-link-lib=static=ow");
}

#[cfg(not(feature = "vendored"))]
fn lib() {
    println!("cargo:rustc-link-lib=ow");
    println!("cargo:rustc-link-lib=owcapi");
}

fn main() {
    lib();
}
