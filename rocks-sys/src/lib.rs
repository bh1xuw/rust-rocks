#![feature(untagged_unions)]


use std::ffi::{CStr, CString};
use std::mem;
use std::iter;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod c;

pub use c::*;


#[test]
fn test_cf_opt() {
    unsafe {
        let cfopt = c::rocks_column_family_options_create();
        c::rocks_column_family_options_destroy(cfopt);
    }
}

#[test]
fn test_db_list_cf_names() {
    unsafe {
        let opt = c::rocks_options_create();
        
        let mut status = mem::uninitialized::<c::rocks_status_t>();
        let dbname = CString::new("./data.test").unwrap();

        let mut lencfs = 0_usize;
        let cnames = c::rocks_db_list_column_families(opt, dbname.as_ptr(), &mut lencfs, &mut status);
        if status.code != 0 {
            println!("status => {:?}", CStr::from_ptr(status.state));
        }
        assert!(status.code == 0);

        println!("len => {:?}", lencfs);
        let mut cfnames: Vec<String> = vec![];
        for i in 0..lencfs {
            cfnames.push(CStr::from_ptr(*cnames.offset(i as isize)).to_str().unwrap().to_owned());
        }
        println!("cf => {:?}", cfnames);
        assert_eq!(cfnames, vec!["default".to_owned()]);

        c::rocks_db_list_column_families_destroy(cnames, lencfs);
        c::rocks_options_destroy(opt);
    }
}



#[test]
fn test_smoke() {
    unsafe {
        let opt = c::rocks_options_create();
        println!("opt => {:?}", opt);
        assert!(!opt.is_null());
        c::rocks_options_optimize_for_point_lookup(opt, 512);
        assert!(!opt.is_null());

        let mut status = mem::uninitialized::<c::rocks_status_t>();
        let dbname = CString::new("./data.test").unwrap();
        c::rocks_options_set_create_if_missing(opt, 1);

        println!("going to open db");
        let db = c::rocks_db_open(opt, dbname.as_ptr(), &mut status);
        println!("db => {:?}", db);
        println!("code => {:?}", status.code);
        if status.code != 0 {
            println!("status => {:?}", CStr::from_ptr(status.state));
        }

        let wopt = c::rocks_writeoptions_create();

        for i in 0..1000 {
            let key = format!("test3-key-{}", i);
            let val = format!("rocksdb-value-{}", i*10);
            let value: String = iter::repeat(val)
                .take(10000)
                .collect::<Vec<_>>()
                .concat();
            c::rocks_db_put(db, wopt,
                            key.as_bytes().as_ptr() as _, key.len(),
                            value.as_bytes().as_ptr() as _, value.len(),
                            &mut status);
            if status.code != 0 {
                println!("status => {:?}", CStr::from_ptr(status.state));
            }
        }

        c::rocks_db_close(db);
        c::rocks_writeoptions_destroy(wopt);
        c::rocks_options_destroy(opt);
    }
}
