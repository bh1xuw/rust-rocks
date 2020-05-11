extern crate rocks;

use rocks::prelude::*;
use rocks::types::FullKey;
use std::str;

const DB_PATH: &str = "/tmp/rocksdb_simple_example";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optimize RocksDB. This is the easiest way to get RocksDB to perform well
    // NOTE: Is rust, Options is splited into 2 parts.
    let options = Options::default()
        .map_db_options(|db| db.create_if_missing(true).increase_parallelism(8))
        .map_cf_options(|cf| cf.optimize_level_style_compaction(512 * 1024 * 1024));

    // open DB
    let db = DB::open(&options, DB_PATH)?;
    println!("db => {:?}", db);

    // Put key-value
    db.put(WriteOptions::default_instance(), b"key1", b"value")?;

    let seq = db.get_latest_sequence_number();
    println!("latest seq = {}", seq);

    db.put(WriteOptions::default_instance(), b"key1", b"xxxxxxxxxx")?;
    db.put(WriteOptions::default_instance(), b"key2", b"xxasdfxxxx")?;
    db.put(WriteOptions::default_instance(), b"key3", b"xxxagaasdxxxx")?;
    db.put(WriteOptions::default_instance(), b"key4", b"xxxxxasdfx")?;
    db.delete(WriteOptions::default_instance(), b"key3")?;

    for (k, v) in db.new_iterator(&ReadOptions::default().iter_start_seqnum(seq)) {
        let fk = FullKey::parse(k).unwrap();
        println!("{:?} => {:?}", fk, str::from_utf8(v).unwrap());
    }

    Ok(())
}
