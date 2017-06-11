

use std::mem;
use std::ptr;
use std::os::raw::c_int;

use rocks_sys as ll;

use options::CompressionType;


pub fn get_supported_compressions() -> Vec<CompressionType> {
    unsafe {
        let mut n = 0;
        let ptr = ll::rocks_get_supported_compressions(&mut n);
        let mut ret = Vec::with_capacity(n);

        for i in 0 .. n {
            ret.push(mem::transmute(*ptr.offset(i as isize)));
        }
        ll::rocks_get_supported_compressions_destroy(ptr);
        ret
    }
}



#[test]
fn test_compressions() {
    let types = get_supported_compressions();
    // [ZlibCompression, SnappyCompression, LZ4HCCompression, LZ4Compression, BZip2Compression, NoCompression]
    assert!(types.len() >= 1);
    assert!(types.contains(&CompressionType::NoCompression));
}
