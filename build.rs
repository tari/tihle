fn main() {
    rerun_if_changed("os/");
    if let Err(e) = std::process::Command::new("make")
        .arg("-C")
        .arg("os")
        .status()
    {
        println!(
            "cargo:warning=Failed to run make to build OS; compilation may fail: {}",
            e
        );
    }

    rerun_if_changed("src/z80/redcode/");
    cc::Build::new()
        .file("src/z80/redcode/Z80.c")
        .define("CPU_Z80_STATIC", None)
        .define("CPU_Z80_USE_LOCAL_HEADER", None)
        .define("CPU_Z80_DEPENDENCIES_H", Some("\"z80bits.h\""))
        .compile("z80_redcode");
}

fn rerun_if_changed<P: AsRef<std::path::Path>>(path: P) {
    for entry in walkdir::WalkDir::new(path) {
        println!("cargo:rerun-if-changed={}", entry.unwrap().path().display());
    }
}
