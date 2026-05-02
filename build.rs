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

  let local_includes = format!("{}/local_includes", tre_dir);
  std::fs::create_dir_all(&local_includes).unwrap();

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

  // Generate the master config.h exactly like autotools does,
  let config_h_path = format!("{}/config.h", local_includes);
  let mut config_h = String::from(
    r#"
#ifndef TRE_CONFIG_H_ROOT
#define TRE_CONFIG_H_ROOT

#define HAVE_ALLOCA 1
#define HAVE_ALLOCA_H 1
#define HAVE_DLFCN_H 1
#define HAVE_GETOPT_H 1
#define HAVE_GETOPT_LONG 1
#define HAVE_INTTYPES_H 1
#define HAVE_ISASCII 1
#define HAVE_ISBLANK 1
#define HAVE_ISWASCII 1
#define HAVE_ISWBLANK 1
#define HAVE_ISWCTYPE 1
#define HAVE_ISWLOWER 1
#define HAVE_ISWUPPER 1
#define HAVE_MBRTOWC 1
#define HAVE_MBSTATE_T 1
#define HAVE_STDINT_H 1
#define HAVE_STDIO_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRINGS_H 1
#define HAVE_STRING_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_TOWLOWER 1
#define HAVE_TOWUPPER 1
#define HAVE_UNISTD_H 1
#define HAVE_WCHAR_H 1
#define HAVE_WCHAR_T 1
#define HAVE_WCSCHR 1
#define HAVE_WCSCPY 1
#define HAVE_WCSLEN 1
#define HAVE_WCSNCPY 1
#define HAVE_WCSRTOMBS 1
#define HAVE_WCTYPE 1
#define HAVE_WCTYPE_H 1
#define HAVE_WINT_T 1
#define STDC_HEADERS 1
#define TRE_APPROX 1
#define TRE_MULTIBYTE 1
#define TRE_REGEX_T_FIELD value
#define TRE_USE_ALLOCA 1
#define TRE_VERSION "0.9.0"
#define TRE_WCHAR 1
#define USE_LOCAL_TRE_H 1

#ifndef _ALL_SOURCE
# define _ALL_SOURCE 1
#endif
#ifndef __EXTENSIONS__
# define __EXTENSIONS__ 1
#endif
#ifndef _POSIX_PTHREAD_SEMANTICS
# define _POSIX_PTHREAD_SEMANTICS 1
#endif
"#,
  );

  // Inject OS-Specific Macros safely
  if target_os == "macos" {
    config_h.push_str("#define HAVE_CFLOCALECOPYCURRENT 1\n");
    config_h.push_str("#define HAVE_CFPREFERENCESCOPYAPPVALUE 1\n");
    config_h.push_str("#ifndef _DARWIN_C_SOURCE\n# define _DARWIN_C_SOURCE 1\n#endif\n");
  } else if target_os == "linux" {
    config_h.push_str("#ifndef _GNU_SOURCE\n# define _GNU_SOURCE 1\n#endif\n");
  } else if target_os == "windows" {
    // Windows MSVC requires slightly different tuning, unistd.h doesn't exist
    config_h = config_h.replace("#define HAVE_UNISTD_H 1", "/* #undef HAVE_UNISTD_H */");
    // Tell TRE not to look for the Mac/Linux alloca header on Windows
    config_h = config_h.replace("#define HAVE_ALLOCA_H 1", "/* #undef HAVE_ALLOCA_H */");
  }

  config_h.push_str("\n#endif\n");
  std::fs::write(&config_h_path, config_h).unwrap();

  // Generate the internal API tre-config.h
  let tre_config_h_path = format!("{}/tre-config.h", local_includes);
  std::fs::write(
    &tre_config_h_path,
    r#"
#ifndef TRE_CONFIG_H
#define TRE_CONFIG_H

#define HAVE_SYS_TYPES_H 1
#define HAVE_WCHAR_H 1
#define TRE_APPROX 1
#define TRE_MULTIBYTE 1
#define TRE_WCHAR 1
#define TRE_VERSION "0.9.0"
#define TRE_VERSION_1 0
#define TRE_VERSION_2 9
#define TRE_VERSION_3 0
#define TRE_REGEX_T_FIELD value

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
"#,
  )
  .unwrap();

  // 2. Configure the C Compiler with maximum performance & Unicode macros
  let mut build = cc::Build::new();

  build.define("HAVE_CONFIG_H", "1");

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
    .include(format!("{}/local_includes", tre_dir))
    .include(format!("{}/lib", tre_dir))
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
    .compile("tre");
}
