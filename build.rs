use std::env;
use std::fs;
use std::path::Path;

const BLAS_SRC_LINK_NAME: &str = "BLAS_LIB";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // requested configure by features
    let lapack_int_64_request = match (cfg!(feature = "lp64"), cfg!(feature = "ilp64")) {
        (true, true) => panic!("Features `lp64` and `ilp64` are mutually exclusive."),
        (true, false) => Some(false),
        (false, true) => Some(true),
        (false, false) => None,
    };

    let (lapack_int_64, prefix, suffix) = if cfg!(feature = "blas-src") {
        // auto configure with blas-src info
        println!(
            "cargo:rerun-if-env-changed=DEP_{}_LAPACK",
            BLAS_SRC_LINK_NAME
        );
        println!(
            "cargo:rerun-if-env-changed=DEP_{}_LAPACK_PREFIX",
            BLAS_SRC_LINK_NAME
        );
        println!(
            "cargo:rerun-if-env-changed=DEP_{}_LAPACK_SUFFIX",
            BLAS_SRC_LINK_NAME
        );
        println!(
            "cargo:rerun-if-env-changed=DEP_{}_LAPACK_64",
            BLAS_SRC_LINK_NAME
        );
        println!(
            "cargo:rerun-if-env-changed=DEP_{}_LAPACK_64_PREFIX",
            BLAS_SRC_LINK_NAME
        );
        println!(
            "cargo:rerun-if-env-changed=DEP_{}_LAPACK_64_SUFFIX",
            BLAS_SRC_LINK_NAME
        );
        let lapack = if env::var(format!("DEP_{}_LAPACK", BLAS_SRC_LINK_NAME)).is_ok() {
            let prefix = env::var(format!("DEP_{}_LAPACK_PREFIX", BLAS_SRC_LINK_NAME))
                .unwrap_or_else(|_| {
                    panic!(
                        "internal error: DEP_{}_LAPACK_PREFIX must be set if DEP_{}_LAPACK is set",
                        BLAS_SRC_LINK_NAME, BLAS_SRC_LINK_NAME
                    )
                });
            let suffix = env::var(format!("DEP_{}_LAPACK_SUFFIX", BLAS_SRC_LINK_NAME))
                .unwrap_or_else(|_| {
                    panic!(
                        "internal error: DEP_{}_LAPACK_SUFFIX must be set if DEP_{}_LAPACK is set",
                        BLAS_SRC_LINK_NAME, BLAS_SRC_LINK_NAME
                    )
                });
            Some((prefix, suffix))
        } else {
            None
        };
        let lapack_64 = if env::var(format!("DEP_{}_LAPACK_64", BLAS_SRC_LINK_NAME)).is_ok() {
            let prefix = env::var(format!("DEP_{}_LAPACK_64_PREFIX", BLAS_SRC_LINK_NAME))
                .unwrap_or_else(|_| {
                    panic!(
                    "internal error: DEP_{}_LAPACK_64_PREFIX must be set if DEP_{}_LAPACK_64 is set",
                    BLAS_SRC_LINK_NAME, BLAS_SRC_LINK_NAME
                )
                });
            let suffix = env::var(format!("DEP_{}_LAPACK_64_SUFFIX", BLAS_SRC_LINK_NAME))
                .unwrap_or_else(|_| {
                    panic!(
                    "internal error: DEP_{}_LAPACK_64_SUFFIX must be set if DEP_{}_LAPACK_64 is set",
                    BLAS_SRC_LINK_NAME, BLAS_SRC_LINK_NAME
                )
                });
            Some((prefix, suffix))
        } else {
            None
        };

        match lapack_int_64_request {
            Some(true) => {
                if let Some((prefix, suffix)) = lapack_64 {
                    (true, prefix, suffix)
                } else {
                    panic!("Features `ilp64` and `blas-src` are enabled, but crate `blas-src` does not provide lapack 64-bit interface.");
                }
            }
            Some(false) => {
                if let Some((prefix, suffix)) = lapack {
                    (false, prefix, suffix)
                } else {
                    panic!("Features `lp64` and `blas-src` are enabled, but crate `blas-src` does not provide lapack 32-bit interface.");
                }
            }
            None => {
                // prefer lapack 64-bit if both available
                if let Some((prefix, suffix)) = lapack_64 {
                    (true, prefix, suffix)
                } else if let Some((prefix, suffix)) = lapack {
                    (false, prefix, suffix)
                } else {
                    panic!("Feature `blas-src` is enabled, but blas-src does not provide lapack interface.");
                }
            }
        }
    } else {
        // manual configure require lp64/ilp64 feature
        let lapack_int_64 = lapack_int_64_request.expect(
                "When feature `blas-src` is not enabled, either feature `lp64` or `ilp64` must be enabled to configure lapack integer size."
            );

        // use requested prefix/suffix by environment variables
        // if not set use default prefix/suffix "" and "_"

        println!("cargo:rerun-if-env-changed=LAPACK_SYS_PREFIX");
        println!("cargo:rerun-if-env-changed=LAPACK_SYS_SUFFIX");
        let prefix = env::var("LAPACK_SYS_PREFIX").unwrap_or("".to_string());
        let suffix = env::var("LAPACK_SYS_SUFFIX").unwrap_or("_".to_string());
        (lapack_int_64, prefix, suffix)
    };
    configure(lapack_int_64, &prefix, &suffix);
}

fn configure(lapack_int_64: bool, prefix: &str, suffix: &str) {
    if lapack_int_64 {
        println!("cargo:rustc-cfg=lapack_int_64");
    }
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("lapack_func.rs");

    fs::write(
        dest_path,
        format!(
            "macro_rules! lapack_func {{ ($func:literal) => {{ concat!(r##\"{}\"##, $func, r##\"{}\"##) }};}}\n",
            prefix,suffix
        ),
    )
    .unwrap();
}
