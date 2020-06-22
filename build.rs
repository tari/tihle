fn main() {
    rerun_if_changed("os/");
    std::process::Command::new("make")
        .arg("-C")
        .arg("os")
        .status()
        .expect("Failed to run make to build OS");

    rerun_if_changed("third_party/redcode/");
    cc::Build::new()
        .file("third_party/redcode/Z80.c")
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
