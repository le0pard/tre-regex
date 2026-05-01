fn main() {
    let mut build = cc::Build::new();

    // Bypass the need for TRE's autotools `./configure` script
    build.define("TRE_USE_SYSTEM_REGEX_H", "0")
        .define("TRE_USE_SYSTEM_WCTYPE_H", "0")
        .define("HAVE_MEMMOVE", "1");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Translate your Windows-specific tweaks!
    if target_os == "windows" {
        // Equivalent to your '--disable-nls'
        build.define("ENABLE_NLS", "0");

        // If the user happens to be compiling via MinGW instead of MSVC
        if target_env == "gnu" {
            // Equivalent to your '-fno-stack-protector'
            build.flag("-fno-stack-protector");

            // Equivalent to '-Wl,-Bstatic -lpthread -Wl,-Bdynamic'
            println!("cargo:rustc-link-arg=-Wl,-Bstatic");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-arg=-Wl,-Bdynamic");
        }
    }

    // Compile the C code directly (Statically linking it into the .node module)
    build.file("tre/lib/tre-compile.c")
        .file("tre/lib/tre-match-approx.c")
        .file("tre/lib/tre-match-parallel.c")
        .file("tre/lib/tre-mem.c")
        .file("tre/lib/regcomp.c")
        .file("tre/lib/regexec.c")
        .file("tre/lib/regerror.c")
        .include("tre/include")
        .include("tre/lib")
        .compile("tre"); // Generates libtre.a and bakes it into Rust
}
