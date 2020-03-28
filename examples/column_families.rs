extern crate rocks;

use rocks::prelude::*;

const DB_PATH: &str = "/tmp/rocksdb_column_families_example";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open DB
    let options = Options::default().map_db_options(|db_opt| db_opt.create_if_missing(true));

    let db = DB::open(&options, DB_PATH).map_err(|err| {
        eprintln!(
            "You should delete the {:?} directory before running this example.",
            DB_PATH
        );
        err
    })?;

    // create column family
    let cf = db.create_column_family(&ColumnFamilyOptions::default(), "new_cf")?;

    // close DB
    drop(cf);
    drop(db);

    // open DB with two column families
    let column_families = &[DEFAULT_COLUMN_FAMILY_NAME, "new_cf"];

    // open the new one, too
    let (db, handles) = DB::open_with_column_families(&DBOptions::default(), DB_PATH, column_families)?;

    println!("db: {:?}", db);
    println!("cf: {:?}", handles);

    // put and get from non-default column family
    db.put_cf(WriteOptions::default_instance(), &handles[1], b"key", b"value")?;
    let _ = db.get_cf(ReadOptions::default_instance(), &handles[1], b"key")?;

    // put and get from non-default column family, rustic style
    handles[1].put(WriteOptions::default_instance(), b"key0", b"value0")?;
    let _ = handles[1].get(ReadOptions::default_instance(), b"key0")?;

    // atomic write
    let mut batch = WriteBatch::new();
    batch
        .put_cf(&handles[0], b"key2", b"value2")
        .put_cf(&handles[1], b"key3", b"value3")
        .delete_cf(&handles[0], b"key");
    db.write(WriteOptions::default_instance(), &batch)?;

    // drop column family
    db.drop_column_family(&handles[1])?;

    // close db
    Ok(())
}
