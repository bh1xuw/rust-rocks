use std::str;
use rocks::prelude::*;

fn main() {
    let opt = Options::default().map_db_options(|db_opt| db_opt.create_if_missing(true));
    let db = DB::open(opt, "./data").unwrap();

    let mut wb = WriteBatch::new();

    for i in 0..1000 {
        wb.put(format!("{:03}-key", i).as_bytes(), format!("value-{:03}", i).as_bytes());
    }

    println!("wb => {:?}", wb);

    let _ = db.write(WriteOptions::default_instance(), wb).unwrap();

    println!("got => {:?}", db.get(ReadOptions::default_instance(), b"key-042"));

    for (key, value) in db.new_iterator(ReadOptions::default_instance()) {
        unsafe {
            println!("{:?} => {:?}", str::from_utf8_unchecked(key), str::from_utf8_unchecked(value));
        }
    }
    println!("done!");
}
