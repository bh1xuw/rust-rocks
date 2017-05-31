extern crate gcc;

use std::process::Command;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

#[cfg(feature = "snappy")]
fn snappy() {
    if !Path::new("snappy/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init", "snappy"])
            .status();
    }

    Command::new("./autogen.sh")
        .current_dir("snappy")
        .output()
        .expect("failed to execute process");

    Command::new("./configure")
        .current_dir("snappy")
        .arg("--with-pic")
        .arg("--enable-static")
        .output()
        .expect("failed to execute process");

    gcc::Config::new()
        .include("snappy")
        .file("snappy/snappy.cc")
        .file("snappy/snappy-sinksource.cc")
        .file("snappy/snappy-stubs-internal.cc")
        .file("snappy/snappy-c.cc")
        .cpp(true)
        .flag("-std=c++11")
        .flag("-O2")
        .compile("libsnappy.a");
}

#[cfg(feature = "zlib")]
fn zlib() {
    if !Path::new("zlib/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init", "zlib"])
            .status();
    }

    Command::new("./configure")
        .current_dir("zlib")
        .arg("--static")
        .output()
        .expect("failed to execute process");

    let mut cfg = gcc::Config::new();
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

    cfg.flag("-fPIC");
    cfg.flag("-O2");
    cfg.compile("libz.a");
}

// Yes, copied from alexcrichton/bzip2-rs
#[cfg(feature = "bzip2")]
fn bzip2() {
    if !Path::new("bzip2/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init", "bzip2"])
            .status();
    }

    let mut cfg = gcc::Config::new();

    if cfg!(windows) {
        cfg.define("_WIN32", None);
        cfg.define("BZ_EXPORT", None);
    }

    cfg.define("_FILE_OFFSET_BITS", Some("64"));

    cfg.include("bzip2")
        .define("BZ_NO_STDIO", None)
        .file("bzip2/blocksort.c")
        .file("bzip2/huffman.c")
        .file("bzip2/crctable.c")
        .file("bzip2/randtable.c")
        .file("bzip2/compress.c")
        .file("bzip2/decompress.c")
        .file("bzip2/bzlib.c")
        .flag("-O2")
        .compile("libbz2.a");
}

#[cfg(feature = "lz4")]
fn lz4() {
    if !Path::new("lz4/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init", "lz4"])
            .status();
    }

    gcc::Config::new()
        .include("lz4/lib")
        .file("lz4/lib/lz4.c")
        .file("lz4/lib/lz4frame.c")
        .file("lz4/lib/lz4hc.c")
        .file("lz4/lib/xxhash.c")
        .flag("-O3")
        .compile("liblz4.a");
}

#[cfg(feature = "zstd")]
fn zstd() {
    if !Path::new("zstd/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init", "zstd"])
            .status();
    }

    gcc::Config::new()
        .define("ZSTD_LEGACY_SUPPORT", Some("0"))
        .include("zstd/lib")
        .file("zstd/lib/zstd_compress.c")
        .file("zstd/lib/zstd_decompress.c")
        .file("zstd/lib/zstd_buffered.c")
        .file("zstd/lib/fse.c")
        .file("zstd/lib/huff0.c")
        .flag("-O3")
        .flag("-std=c99")
        .compile("libzstd.a");
}

fn rocksdb() {
    if !Path::new("rocksdb/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init", "rocksdb"])
            .status();
    }

    let mut cfg = gcc::Config::new();

    cfg.include("rocksdb/include");
    cfg.include("rocksdb");

    cfg.define("NDEBUG", Some("1"));

    #[cfg(feature = "snappy")]
    {
        cfg.define("SNAPPY", Some("1"));
        cfg.include("snappy");
    }

    #[cfg(feature = "zlib")]
    {
        cfg.define("ZLIB", Some("1"));
        cfg.include("zlib");
    }

    #[cfg(feature = "lz4")]
    {
        cfg.define("LZ4", Some("1"));
        cfg.include("lz4/lib");
    }

    #[cfg(feature = "zstd")]
    {
        cfg.define("ZSTD", Some("1"));
        cfg.include("zstd/lib");
        cfg.include("zstd/lib/legacy");
    }

    #[cfg(feature = "jemalloc")]
    {
        cfg.define("JEMALLOC", Some("1"));
    }

    for s in include_str!("sources.txt").lines() {
        let f = s.trim();
        if !f.is_empty() {
            cfg.file(format!("rocksdb/{}", f));
        }
    }

    // Borrowed from rust-rocksdb
    if cfg!(target_os = "macos") {
        cfg.define("OS_MACOSX", Some("1"));
        cfg.define("ROCKSDB_PLATFORM_POSIX", Some("1"));
        cfg.define("ROCKSDB_LIB_IO_POSIX", Some("1"));

    }
    if cfg!(target_os = "linux") {
        cfg.define("OS_LINUX", Some("1"));
        cfg.define("ROCKSDB_PLATFORM_POSIX", Some("1"));
        cfg.define("ROCKSDB_LIB_IO_POSIX", Some("1"));
        // COMMON_FLAGS="$COMMON_FLAGS -fno-builtin-memcmp"
    }
    if cfg!(target_os = "freebsd") {
        cfg.define("OS_FREEBSD", Some("1"));
        cfg.define("ROCKSDB_PLATFORM_POSIX", Some("1"));
        cfg.define("ROCKSDB_LIB_IO_POSIX", Some("1"));
    }

    if cfg!(windows) {
        for s in include_str!("sources_win32.txt").lines() {
            let f = s.trim();
            if !f.is_empty() {
                cfg.file(format!("rocksdb/{}", f));
            }
        }
    } else {
        cfg.flag("-std=c++11");
        // POSIX systems
        for s in include_str!("sources_posix.txt").lines() {
            let f = s.trim();
            if !f.is_empty() {
                cfg.file(format!("rocksdb/{}", f));
            }
        }
    }

    let git_sha = "88724cc719784c78df0c0ff87cca7cafd7abbe37";
    let compile_date = "2017-02-29";
    File::create("./build_version.cc")
        .and_then(|mut f| {
            write!(&mut f,
                   "const char* rocksdb_build_git_sha = \"{}\";\n\
                    const char* rocksdb_build_compile_date = \"{}\";\n",
                   git_sha, compile_date)
        })
        .unwrap();
    cfg.file("build_version.cc");

    cfg.cpp(true);
    cfg.compile("librocksdb.a");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./");
    println!("cargo:rerun-if-changed=./rocks");

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

    gcc::Config::new()
        .cpp(true)
        .include("rocksdb/include")
        .include(".")
        .file("rocks/db.cc")
        .file("rocks/env.cc")
        .file("rocks/options.cc")
        .file("rocks/rate_limiter.cc")
        .file("rocks/snapshot.cc")
        .file("rocks/status.cc")
        .file("rocks/iterator.cc")
        .file("rocks/write_batch.cc")
        .file("rocks/cache.cc")
        .file("rocks/merge_operator.cc")
        .file("rocks/sst_file_writer.cc")
        .file("rocks/comparator.cc")
        .file("rocks/db_dump_tool.cc")
        .file("rocks/perf_level.cc")
        .file("rocks/iostats_context.cc")
        .file("rocks/perf_context.cc")
        .file("rocks/statistics.cc")
        .file("rocks/table.cc")
        .file("rocks/filter_policy.cc")
        .file("rocks/metadata.cc")
        .file("rocks/aux.cc")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-g")
        .flag("-std=c++11")
        .compile("librocksdb_wrap.a");
}
