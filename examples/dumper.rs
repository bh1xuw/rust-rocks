extern crate rocks;

use rocks::rocksdb::*;
use std::env;

pub fn escape(data: &[u8]) -> String {
    let mut escaped = Vec::with_capacity(data.len() * 4);
    for &c in data {
        match c {
            b'\n' => escaped.extend_from_slice(br"\n"),
            b'\r' => escaped.extend_from_slice(br"\r"),
            b'\t' => escaped.extend_from_slice(br"\t"),
            b'"' => escaped.extend_from_slice(b"\\\""),
            b'\\' => escaped.extend_from_slice(br"\\"),
            _ => {
                if c >= 0x20 && c < 0x7f {
                    // c is printable
                    escaped.push(c);
                } else {
                    escaped.push(b'\\');
                    escaped.push(b'0' + (c >> 6));
                    escaped.push(b'0' + ((c >> 3) & 7));
                    escaped.push(b'0' + (c & 7));
                }
            }
        }
    }
    escaped.shrink_to_fit();
    unsafe { String::from_utf8_unchecked(escaped) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::default();

    let db_path = env::args().skip(1).next().expect("usage: ./dumper XXXX");

    let cfs = DB::list_column_families(&opts, &db_path).unwrap();
    let (db, cfs) = DB::open_for_readonly_with_column_families(&DBOptions::default(), &db_path, cfs, false)?;
    println!("DB => {:?}", db);

    for cf in &cfs {
        println!("{:?}", cf);
        let meta = db.get_column_family_metadata(cf);
        println!("{:?}", meta);
        let it = cf.new_iterator(&ReadOptions::default().pin_data(true));
        for (k, val) in it {
            println!(r#"  "{}" => "{}""#, escape(k), escape(val));
        }
    }
    Ok(())
}
