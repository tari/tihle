fn main() {
    cc::Build::new()
        .file("third_party/redcode/Z80.c")
        .define("CPU_Z80_STATIC", None)
        .define("CPU_Z80_USE_LOCAL_HEADER", None)
        .define("CPU_Z80_DEPENDENCIES_H", Some("\"z80bits.h\""))
        .compile("z80_redcode");
}
