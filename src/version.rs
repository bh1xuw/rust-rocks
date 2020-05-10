use rocks_sys as ll;

/// Represents a version number conforming to the semantic versioning scheme.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl ::std::fmt::Display for Version {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// RocksDB version.
pub fn version() -> Version {
    unsafe {
        Version {
            major: ll::rocks_version_major() as _,
            minor: ll::rocks_version_minor() as _,
            patch: ll::rocks_version_patch() as _,
        }
    }
}

#[test]
fn test_version() {
    assert!(version().major >= 5);
    println!("version = {}", version());
}
