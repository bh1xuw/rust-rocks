#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod c;

pub use c::*;

pub fn version() -> String {
    unsafe {
        format!(
            "{}.{}.{}",
            rocks_version_major(),
            rocks_version_minor(),
            rocks_version_patch()
        )
    }
}

#[test]
fn test_smoke() {
    assert!(version().len() > 0);
}

#[no_mangle]
pub extern "C" fn bz_internal_error(errcode: i32) {
    assert!(errcode == 0);
}

#[doc(hidden)]
pub mod rust_export {
    use std::ptr;

    #[no_mangle]
    pub extern "C" fn rust_hello_world() {
        println!("Hello World! from rust");
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_string_assign(s: *mut String, p: *const u8, len: usize) {
        (*s).reserve(len);
        ptr::copy(p, (*s).as_mut_vec().as_mut_ptr(), len);
        (*s).as_mut_vec().set_len(len);
    }

    #[no_mangle]
    pub unsafe extern "C" fn rust_vec_u8_assign(v: *mut Vec<u8>, p: *const u8, len: usize) {
        // (*v).extend_from_slice(slice::from_raw_parts(p, len))
        (*v).reserve(len);
        ptr::copy(p, (*v).as_mut_ptr(), len);
        (*v).set_len(len);
    }
}
