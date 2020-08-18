fn main() {
    built::write_built_file().expect("Failed to collect build information");

    let os_sources = ["os/page00.asm", "os/page01.asm", "os/page1b.asm"];
    // TODO this should probably emit binaries to the cargo build dir
    if let Err(e) = spasm_multipage::autobuild(&os_sources, &["os/", "programs/include/"]) {
        println!(
            "cargo:warning=Failed to invoke spasm to build OS image; build may fail: {:?}",
            e
        );
    }
    rerun_if_changed("os/tihle-os.inc");
    for src in &os_sources {
        rerun_if_changed(src);
    }

    rerun_if_changed("src/z80/redcode/");
    cc::Build::new()
        .file("src/z80/redcode/Z80.c")
        .define("CPU_Z80_STATIC", None)
        .define("CPU_Z80_USE_LOCAL_HEADER", None)
        .define("CPU_Z80_DEPENDENCIES_H", Some("\"z80bits.h\""))
        .compile("z80_redcode");

    embed_resource::compile("dist/win/tihle.rc");
}

fn rerun_if_changed<P: AsRef<std::path::Path>>(path: P) {
    for entry in walkdir::WalkDir::new(path) {
        println!("cargo:rerun-if-changed={}", entry.unwrap().path().display());
    }
}
