
use rocks_sys as ll;


pub use self::ll::version;



#[test]
fn test_version() {
    let v = version();
    assert!(v >= "5.2.1".into());
    println!("ver = {}", v);
}
