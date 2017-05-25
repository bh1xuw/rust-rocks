extern crate gcc;

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=./");
    println!("cargo:rerun-if-changed=./rocks");

    println!("cargo:rustc-link-lib=rocksdb");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    println!("cargo:rustc-link-lib=bz2");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=snappy");

    env::set_var("CXXFLAGS", "-std=c++11");

    gcc::Config::new()
        .cpp(true)
        .include("/usr/local/include")
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
        .file("rocks/aux.cc")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-g")
        .compile("librocksdb_wrap.a");
}
