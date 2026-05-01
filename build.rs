use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

fn main() {
  let tre_version = "5ac28057f648debda76f9bf4d39dfdfa85b0df18";
  let expected_sha256 = "528a8f8a4672cd3a0e5354629323a17d0cfa98b3792a57d764b64db30e2d5e9a";

  let vendors_dir = "vendors";
  let tre_dir = format!("{}/tre-{}", vendors_dir, tre_version);
  let tarball = format!("{}/tre.tar.gz", vendors_dir);

  std::fs::create_dir_all(vendors_dir).expect("Failed to create vendors directory");

  // Automatically Download, Verify, and Extract TRE
  if !Path::new(&tre_dir).exists() {
    println!("cargo:warning=Downloading TRE source...");
    let url = format!(
      "https://github.com/laurikari/tre/archive/{}.tar.gz",
      tre_version
    );

    let curl_status = Command::new("curl")
      .args(&["-L", &url, "-o", &tarball])
      .status()
      .expect("Failed to run curl to download TRE");
    assert!(curl_status.success(), "Failed to download TRE tarball");

    println!("cargo:warning=Verifying SHA256 checksum...");
    let mut file = File::open(&tarball).expect("Failed to open tarball for hashing");
    let mut buffer = Vec::new();
    file
      .read_to_end(&mut buffer)
      .expect("Failed to read tarball");

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash = hasher.finalize();

    let actual_sha256: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

    if actual_sha256 != expected_sha256 {
      panic!(
        "SECURITY ERROR: Checksum mismatch for TRE source!\nExpected: {}\nActual:   {}",
        expected_sha256, actual_sha256
      );
    }

    println!("cargo:warning=Extracting TRE source...");
    let tar_status = Command::new("tar")
      .args(&["-xzf", &tarball, "-C", vendors_dir])
      .status()
      .expect("Failed to run tar to extract TRE");
    assert!(tar_status.success(), "Failed to extract TRE tarball");
  }

  // Generate tre-config.h
  let config_h_path = format!("{}/lib/tre-config.h", tre_dir);
  if !Path::new(&config_h_path).exists() {
    println!("cargo:warning=Generating tre-config.h...");
    std::fs::write(
      &config_h_path,
      "
#ifndef TRE_CONFIG_H
#define TRE_CONFIG_H

#define TRE_APPROX 1
#define TRE_WCHAR 1
#define TRE_MULTIBYTE 1
#define TRE_VERSION \"0.8.0\"
#define TRE_VERSION_1 0
#define TRE_VERSION_2 8
#define TRE_VERSION_3 0

/* Tell TRE the name of the internal pointer field in regex_t */
#define TRE_REGEX_T_FIELD value

/* Cross-platform fixes */
#include <stddef.h>
#include <sys/types.h>

#if defined(_MSC_VER)
#include <BaseTsd.h>
typedef SSIZE_T ssize_t;
#include <malloc.h>
#define alloca _alloca
#else
#include <alloca.h>
#endif

#endif
        ",
    )
    .expect("Failed to write tre-config.h");
  }

  // 2. Configure the C Compiler with maximum performance & Unicode macros
  let mut build = cc::Build::new();

  build
    .define("HAVE_MEMMOVE", "1")
    .define("TRE_USE_ALLOCA", "1")
    // --- FIXED: Multi-byte missing dependencies ---
    .define("HAVE_MBSTATE_T", "1")
    .define("HAVE_MBRTOWC", "1")
    // ----------------------------------------------
    .define("HAVE_WCHAR_H", "1")
    .define("HAVE_WCTYPE_H", "1")
		.define("HAVE_WCTYPE", "1")
    .define("HAVE_TOWLOWER", "1")
    .define("HAVE_TOWUPPER", "1")
    .define("HAVE_ISWUPPER", "1")
    .define("HAVE_ISWLOWER", "1")
    .define("HAVE_ISWCTYPE", "1");

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

  if target_os == "windows" {
    build.define("ENABLE_NLS", "0");

    if target_env == "gnu" {
      build.flag("-fno-stack-protector");
      println!("cargo:rustc-link-arg=-Wl,-Bstatic");
      println!("cargo:rustc-link-lib=pthread");
      println!("cargo:rustc-link-arg=-Wl,-Bdynamic");
    }
  }

  // 3. Compile ALL the core C files directly into the module
  build
    .file(format!("{}/lib/tre-ast.c", tre_dir))
    .file(format!("{}/lib/tre-compile.c", tre_dir))
    .file(format!("{}/lib/tre-match-approx.c", tre_dir))
    .file(format!("{}/lib/tre-match-backtrack.c", tre_dir))
    .file(format!("{}/lib/tre-match-parallel.c", tre_dir))
    .file(format!("{}/lib/tre-mem.c", tre_dir))
    .file(format!("{}/lib/tre-parse.c", tre_dir))
    .file(format!("{}/lib/tre-stack.c", tre_dir))
    .file(format!("{}/lib/regcomp.c", tre_dir))
    .file(format!("{}/lib/regexec.c", tre_dir))
    .file(format!("{}/lib/regerror.c", tre_dir))
    .include(format!("{}/local_includes", tre_dir))
    .include(format!("{}/lib", tre_dir))
    .compile("tre");
}
