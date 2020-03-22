#[cfg(not(feature = "static-link"))]
mod imp {
    pub fn build() {
        #[cfg(feature = "snappy")]
        snappy();

        #[cfg(feature = "zlib")]
        zlib();

        #[cfg(feature = "bzip2")]
        bzip2();

        #[cfg(feature = "lz4")]
        lz4();

        #[cfg(feature = "zstd")]
        zstd();

        rocksdb();
    }

    #[cfg(feature = "snappy")]
    fn snappy() {
        let _ = pkg_config::Config::new().probe("snappy").map_err(|_| {
            println!("cargo:rustc-link-lib=dylib=snappy");
        });
    }

    #[cfg(feature = "zlib")]
    fn zlib() {
        let _ = pkg_config::Config::new().probe("zlib").map_err(|_| {
            println!("cargo:rustc-link-lib=dylib=z");
        });
    }

    #[cfg(feature = "bzip2")]
    fn bzip2() {
        println!("cargo:rustc-link-lib=dylib=bz2");
    }

    #[cfg(feature = "lz4")]
    fn lz4() {
        let _ = pkg_config::Config::new().probe("liblz4").map_err(|_| {
            println!("cargo:rustc-link-lib=dylib=lz4");
        });
    }

    #[cfg(feature = "zstd")]
    fn zstd() {
        let _ = pkg_config::Config::new().probe("libzstd").map_err(|_| {
            println!("cargo:rustc-link-lib=dylib=zstd");
        });
    }

    fn rocksdb() {
        println!("cargo:rustc-link-lib=rocksdb");
    }
}

#[cfg(feature = "static-link")]
mod imp {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    pub fn build() {
        println!("cargo:warning=static link feature enabled, it'll take minutes to finish compiling...");

        #[cfg(feature = "snappy")]
        snappy();

        #[cfg(feature = "zlib")]
        zlib();

        #[cfg(feature = "bzip2")]
        bzip2();

        #[cfg(feature = "lz4")]
        lz4();

        #[cfg(feature = "zstd")]
        zstd();

        rocksdb();
    }

    #[cfg(feature = "snappy")]
    fn snappy() {
        if !Path::new("snappy/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init", "snappy"])
                .status();
        }

        // cmake can only have 1 run in 1 crate
        // or else the `build` directory will be overwritten
        let out_dir = env::var("OUT_DIR").unwrap();
        let this_out_dir = Path::new(&out_dir).join("snappy_out");
        let _ = fs::create_dir(&this_out_dir);

        let dst = cmake::Config::new("snappy")
            .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
            .build_target("snappy")
            .out_dir(this_out_dir)
            .very_verbose(true)
            .uses_cxx11()
            .build();

        println!("cargo:rustc-link-search=native={}/build/", dst.display());
        println!("cargo:rustc-link-lib=static=snappy");
    }

    #[cfg(feature = "zlib")]
    fn zlib() {
        if !Path::new("zlib/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init", "zlib"])
                .status();
        }

        Command::new(env::current_dir().unwrap().join("zlib/configure"))
            .current_dir(env::current_dir().unwrap().join("zlib"))
            .arg("--static")
            .output()
            .expect("failed to execute ./configure");

        let mut cfg = ::cc::Build::new();
        cfg.warnings(false);
        cfg.include("zlib");

        // TODO: borrow following list form Makefile
        let filez = "adler32.c crc32.c deflate.c infback.c inffast.c inflate.c inftrees.c trees.c zutil.c";
        let fileg = "compress.c uncompr.c gzclose.c gzlib.c gzread.c gzwrite.c";

        for file in filez.split(" ") {
            cfg.file(format!("zlib/{}", file));
        }
        for file in fileg.split(" ") {
            cfg.file(format!("zlib/{}", file));
        }

        cfg.pic(true);
        cfg.opt_level(2);
        cfg.compile("libz.a");
    }

    // Yes, copied from alexcrichton/bzip2-rs
    #[cfg(feature = "bzip2")]
    fn bzip2() {
        if !Path::new("bzip2/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init", "bzip2"])
                .status();
        }

        let mut cfg = ::cc::Build::new();

        if cfg!(windows) {
            cfg.define("_WIN32", None);
            cfg.define("BZ_EXPORT", None);
        }

        cfg.include("bzip2")
            .define("_FILE_OFFSET_BITS", Some("64"))
            .define("BZ_NO_STDIO", None)
            .opt_level(2)
            .warnings(false)
            .file("bzip2/blocksort.c")
            .file("bzip2/huffman.c")
            .file("bzip2/crctable.c")
            .file("bzip2/randtable.c")
            .file("bzip2/compress.c")
            .file("bzip2/decompress.c")
            .file("bzip2/bzlib.c")
            .compile("libbz2.a");
    }

    #[cfg(feature = "lz4")]
    fn lz4() {
        if !Path::new("lz4/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init", "lz4"])
                .status();
        }

        ::cc::Build::new()
            .warnings(false)
            .include("lz4/lib")
            .opt_level(2)
            .pic(true)
            .file("lz4/lib/lz4.c")
            .file("lz4/lib/lz4frame.c")
            .file("lz4/lib/lz4hc.c")
            .file("lz4/lib/xxhash.c")
            .compile("liblz4.a");
    }

    #[cfg(feature = "zstd")]
    fn zstd() {
        if !Path::new("zstd/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init", "zstd"])
                .status();
        }

        let files = [
            "zstd/lib/common/entropy_common.c",
            "zstd/lib/common/error_private.c",
            "zstd/lib/common/fse_decompress.c",
            "zstd/lib/common/pool.c",
            "zstd/lib/common/threading.c",
            "zstd/lib/common/xxhash.c",
            "zstd/lib/common/zstd_common.c",
            "zstd/lib/compress/fse_compress.c",
            "zstd/lib/compress/huf_compress.c",
            "zstd/lib/compress/zstd_compress.c",
            "zstd/lib/compress/zstdmt_compress.c",
            "zstd/lib/decompress/huf_decompress.c",
            "zstd/lib/decompress/zstd_decompress.c",
            "zstd/lib/dictBuilder/cover.c",
            "zstd/lib/dictBuilder/divsufsort.c",
            "zstd/lib/dictBuilder/zdict.c",
        ];

        ::cc::Build::new()
            .define("ZSTD_LEGACY_SUPPORT", Some("0"))
            .include("zstd/lib")
            .include("zstd/lib/common")
            .opt_level(3)
            .warnings(false)
            .files(&files)
            .compile("libzstd.a");
    }

    fn rocksdb() {
        if !Path::new("rocksdb/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init", "rocksdb"])
                .status();
        }

        let mut cfg = cmake::Config::new("rocksdb");

        #[cfg(feature = "snappy")]
        {
            let src = std::env::current_dir().unwrap();
            let out_dir = env::var("OUT_DIR").unwrap();
            let snappy_out_dir = Path::new(&out_dir).join("snappy_out");

            // FIXME: how to use cmake's define?
            cfg.cxxflag("-DSNAPPY");
            cfg.cxxflag(format!("-I{}", src.join("snappy").display()));
            // snappy-stubs-public.h
            cfg.cxxflag(format!("-I{}", snappy_out_dir.join("build").display()));
        }

        #[cfg(feature = "zlib")]
        {
            cfg.cxxflag("-DZLIB");
            cfg.cxxflag("-Izlib");
        }

        #[cfg(feature = "bzip2")]
        {
            cfg.cxxflag("-DBZIP2");
            cfg.cxxflag("-Ibzip2");
        }

        #[cfg(feature = "lz4")]
        {
            cfg.cxxflag("-DLZ4");
            cfg.cxxflag("-Ilz4/lib");
        }

        #[cfg(feature = "zstd")]
        {
            cfg.cxxflag("-DZSTD");
            cfg.cxxflag("-Izstd/lib");
        }

        let dst = cfg
            // .define("CMAKE_BUILD_TYPE", "Release") //  RelWithDebInfo
            .define("WITH_GFLAGS", "OFF")
            .define("WITH_CORE_TOOLS", "OFF")
            .define("WITH_TOOLS", "OFF")
            .build_target("rocksdb")
            .build();

        println!("cargo:rustc-link-search=native={}/build/", dst.display());
        println!("cargo:rustc-link-lib=static=rocksdb");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./");
    println!("cargo:rerun-if-changed=./rocks/");

    imp::build();

    let mut build = ::cc::Build::new();

    #[cfg(feature = "static-link")]
    {
        build.include("rocksdb/include");
    }

    build
        .cpp(true)
        .pic(true)
        .opt_level(2)
        .warnings(false)
        .flag("-std=c++11")
        .include(".")
        .file("rocks/cache.cc")
        .file("rocks/comparator.cc")
        .file("rocks/convenience.cc")
        .file("rocks/db.cc")
        .file("rocks/db_dump_tool.cc")
        .file("rocks/env.cc")
        .file("rocks/filter_policy.cc")
        .file("rocks/iostats_context.cc")
        .file("rocks/iterator.cc")
        .file("rocks/merge_operator.cc")
        .file("rocks/metadata.cc")
        .file("rocks/options.cc")
        .file("rocks/perf_context.cc")
        .file("rocks/perf_level.cc")
        .file("rocks/rate_limiter.cc")
        .file("rocks/slice.cc")
        .file("rocks/snapshot.cc")
        .file("rocks/sst_file_writer.cc")
        .file("rocks/statistics.cc")
        .file("rocks/status.cc")
        .file("rocks/table.cc")
        .file("rocks/table_properties.cc")
        .file("rocks/transaction_log.cc")
        .file("rocks/universal_compaction.cc")
        .file("rocks/util.cc")
        .file("rocks/write_batch.cc")
        .file("rocks/write_buffer_manager.cc")
        .file("rocks/debug.cc")
        .file("rocks/listener.cc")
        .file("rocks/compaction_job_stats.cc")
        .file("rocks/thread_status.cc")
        .compile("librocksdb_wrap.a");
}
