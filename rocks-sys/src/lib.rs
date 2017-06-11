#[cfg(test)]
use std::ffi::{CStr, CString};
#[cfg(test)]
use std::mem;
#[cfg(test)]
use std::iter;
#[cfg(test)]
use std::ptr;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod c;

pub use c::*;


pub fn version() -> String {
    unsafe {
        format!("{}.{}.{}",
                rocks_version_major(),
                rocks_version_minor(),
                rocks_version_patch())
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
        assert!(cfnames.contains(&"default".to_owned()));



        c::rocks_db_list_column_families_destroy(cnames, lencfs);
        c::rocks_options_destroy(opt);
    }
}

#[test]
fn test_smoke() {
    unsafe {
        // let opt = c::rocks_options_create();
        let cfopt = c::rocks_cfoptions_create();
        let dbopt = c::rocks_dboptions_create();

        c::rocks_cfoptions_optimize_for_point_lookup(cfopt, 512); 
        // 
        let mut status = mem::uninitialized::<c::rocks_status_t>();
        let dbname = CString::new("./data.test.default").unwrap();

        c::rocks_dboptions_set_create_if_missing(dbopt, 1);

        let opt = c::rocks_options_create_from_db_cf_options(dbopt, cfopt);
        println!("opt => {:?}", opt);
        assert!(!opt.is_null());

        assert!(!opt.is_null());


        println!("going to open db");
        let db = c::rocks_db_open(opt, dbname.as_ptr(), &mut status);
        println!("db => {:?}", db);
        println!("code => {:?}", status.code);

        assert!(status.code == 0, "status => {:?}", CStr::from_ptr(status.state));

        let wopt = c::rocks_writeoptions_create();

        for i in 0..1000 {
            let key = format!("test3-key-{}", i);
            let val = format!("rocksdb-value-{}", i*10);
            let value: String = iter::repeat(val)
                .take(100)
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

