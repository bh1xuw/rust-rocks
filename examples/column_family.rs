use rocks::prelude::*;
use std::str;

fn main() {
    let opt = Options::default().map_db_options(|db_opt| db_opt.create_if_missing(true));

    let path = "./data";
    // must hava a `default`
    let cf_names = &["default", "account", "transaction", "index"];

    if let Ok(cf_names) = DB::list_column_families(&Options::default(), path) {
        if cf_names.len() == 1 {
            let db = DB::open(&Options::default(), path).unwrap();
            for cf_name in &["account", "transaction", "index"] {
                let cf = db
                    .create_column_family(&ColumnFamilyOptions::default(), cf_name)
                    .unwrap();
                println!("creating cf => {:?}", cf);
            }
        }
    } else {
        let db = DB::open(&opt, path).unwrap();
        for cf_name in &["account", "transaction", "index"] {
            let cf = db
                .create_column_family(&ColumnFamilyOptions::default(), cf_name)
                .unwrap();
            println!("creating cf => {:?}", cf);
        }
    }

    let (db, cfs) = DB::open_with_column_families(&opt, path, cf_names).unwrap();
    println!("db => {:?}", db);
    println!("cfs => {:?}", cfs);

    let cf_account = &cfs[1];
    let cf_transaction = &cfs[2];

    cf_account
        .put(WriteOptions::default_instance(), b"hello", b"world")
        .unwrap();

    for (k, v) in cf_account.new_iterator(ReadOptions::default_instance()) {
        println!("{:?} => {:?}", str::from_utf8(k).unwrap(), str::from_utf8(v).unwrap());
    }

    db.drop_column_family(cf_transaction).unwrap();
    db.create_column_family(&ColumnFamilyOptions::default(), "transaction")
        .unwrap();
}
