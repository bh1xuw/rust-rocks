extern crate rocks;

use rocks::prelude::*;

const DB_PATH: &str = "/tmp/rocksdb_simple_example";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optimize RocksDB. This is the easiest way to get RocksDB to perform well
    // NOTE: Is rust, Options is splited into 2 parts.
    let options = Options::default()
        .map_db_options(|db| db.create_if_missing(true).increase_parallelism(16))
        .map_cf_options(|cf| cf.optimize_level_style_compaction(512 * 1024 * 1024));

    // open DB
    let db = DB::open(&options, DB_PATH)?;

    // Put key-value
    db.put(WriteOptions::default_instance(), b"key1", b"value")?;

    // get value
    let value = db.get(ReadOptions::default_instance(), b"key1")?;
    assert_eq!(value, b"value");

    // atomically apply a set of updates
    {
        let mut batch = WriteBatch::default();
        batch.delete(b"key1");
        batch.put(b"key2", &value);

        db.write(WriteOptions::default_instance(), &batch)?;
    }

    let ret = db.get(ReadOptions::default_instance(), b"key1");
    assert!(ret.is_err() && ret.unwrap_err().is_not_found());

    let value = db.get(ReadOptions::default_instance(), b"key2")?;
    assert_eq!(value, b"value");

    Ok(())
}
