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
