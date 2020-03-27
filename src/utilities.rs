use std::ffi::{CStr, CString};
use std::ptr;

use crate::db::ColumnFamilyDescriptor;
use crate::options::{ColumnFamilyOptions, DBOptions};
use crate::to_raw::{FromRaw, ToRaw};
use crate::{Error, Result};

use rocks_sys as ll;

pub fn load_latest_options(path: &str) -> Result<(DBOptions, Vec<ColumnFamilyDescriptor>)> {
    let cpath = CString::new(path).unwrap();
    let db_opt = DBOptions::default();
    let mut cf_descs_len = 0_usize;
    let mut status = ptr::null_mut();
    let mut cf_descs: Vec<ColumnFamilyDescriptor> = Vec::new();

    let c_cf_descs =
        unsafe { ll::rocks_load_latest_options(cpath.as_ptr(), db_opt.raw(), &mut cf_descs_len, &mut status) };
    if let Err(error) = Error::from_ll(status) {
        return Err(error);
    }
    for i in 0..cf_descs_len {
        let c_cf_desc = unsafe { *c_cf_descs.offset(i as _) };
        let name = unsafe { CStr::from_ptr(ll::rocks_column_family_descriptor_get_name(c_cf_desc)) };
        let cfopt =
            unsafe { ColumnFamilyOptions::from_ll(ll::rocks_column_family_descriptor_get_cfoptions(c_cf_desc)) };
        cf_descs.push(ColumnFamilyDescriptor::new(
            name.to_str().expect("non-utf8 cf name"),
            cfopt,
        ));
    }
    unsafe { ll::rocks_load_options_destroy_cf_descs(c_cf_descs, cf_descs_len) };

    Ok((db_opt, cf_descs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn load_options() {
        let (dbopt, cf_descs) = load_latest_options("./data").unwrap();
        println!("db opt => {:?}", dbopt);
        for cf_desc in cf_descs {
            println!("name => {:?}", cf_desc.name());
            println!("opt =>\n{:?}", cf_desc.options());
        }
    }
}
