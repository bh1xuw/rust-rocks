extern crate rocks;

use rocks::prelude::*;

const DB_PATH: &str = "/tmp/rocksdb_simple_example";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::default().map_db_options(|db_opt| db_opt.max_open_files(-1));

    let secondary_path = "/tmp/rocksdb_secondary";

    let db = DB::open_as_secondary(&options, DB_PATH, secondary_path)?;

    println!("db => {:?}", db);

    db.try_catch_up_with_primary()?;

    println!("get => {:?}", db.get(ReadOptions::default_instance(), b"key2"));

    println!(
        "write => {:?}",
        db.put(WriteOptions::default_instance(), b"key3", b"key4")
    );

    Ok(())
}
