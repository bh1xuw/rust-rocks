use rocks::prelude::*;
use std::str;

fn main() {
    let opt = Options::default().map_db_options(|db_opt| db_opt.create_if_missing(true));
    let db = DB::open(opt, "./data").unwrap();

    let mut wb = WriteBatch::new();

    for i in 0..1000 {
        wb.put(format!("{:3}-key", i).as_bytes(), format!("value-{:03}", i).as_bytes());
    }

    println!("wb => {:?}", wb);

    let _ = db.write(WriteOptions::default_instance(), wb).unwrap();

    println!(
        "db[042-key] => {:?}",
        db.get(ReadOptions::default_instance(), b"042-key")
    );

    // pin_data pins iterator key
    let mut it = db.new_iterator(&ReadOptions::default().pin_data(true));

    // this requires pin_data, since it saves key and use it after next()
    let items = (&mut it).take(10).collect::<Vec<_>>();
    for (key, value) in items {
        unsafe {
            println!(
                "{:?} => {:?}",
                str::from_utf8_unchecked(key),
                str::from_utf8_unchecked(value)
            );
        }
    }

    for (key, value) in (&mut it).take(10) {
        unsafe {
            println!(
                "{:?} => {:?}",
                str::from_utf8_unchecked(key),
                str::from_utf8_unchecked(value)
            );
        }
    }

    for key in it.keys().take(10) {
        unsafe {
            println!(
                "key = {:?}",
                str::from_utf8_unchecked(key),
            );
        }
    }

    println!("done!");
}
