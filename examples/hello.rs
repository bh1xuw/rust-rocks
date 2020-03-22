use rocks::prelude::*;

fn main() {
    let opt = Options::default().map_db_options(|db_opt| db_opt.create_if_missing(true));

    let db = DB::open(opt, "./data").unwrap();

    assert!(db.put(WriteOptions::default_instance(), b"hello", b"world").is_ok());
    match db.get(ReadOptions::default_instance(), b"hello") {
        Ok(ref value) => println!("hello: {:?}", value),
        Err(e) => eprintln!("error: {}", e),
    }
    let _ = db.delete(&WriteOptions::default(), b"hello").unwrap();
}
