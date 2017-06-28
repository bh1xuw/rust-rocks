//! Abstract handle to particular state of a DB.

use std::fmt;
use std::ops;
use std::marker::PhantomData;

use rocks_sys as ll;

use db::DB;
use types::SequenceNumber;
use to_raw::{ToRaw, FromRaw};

/// Abstract handle to particular state of a DB.
/// A Snapshot is an immutable object and can therefore be safely
/// accessed from multiple threads without any external synchronization.
///
/// To Create a Snapshot, call `DB::GetSnapshot()`.
///
/// To Destroy a Snapshot, call `DB::ReleaseSnapshot(snapshot)`.
pub struct Snapshot<'a> {
    raw: *mut ll::rocks_snapshot_t,
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Sync for Snapshot<'a> {}
unsafe impl<'a> Send for Snapshot<'a> {}

impl<'a> fmt::Debug for Snapshot<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Snapshot({:?})", self.get_sequence_number())
    }
}

impl<'a> ToRaw<ll::rocks_snapshot_t> for Snapshot<'a> {
    fn raw(&self) -> *mut ll::rocks_snapshot_t {
        self.raw
    }
}

impl<'a> FromRaw<ll::rocks_snapshot_t> for Snapshot<'a> {
    unsafe fn from_ll(raw: *mut ll::rocks_snapshot_t) -> Snapshot<'a> {
        Snapshot {
            raw: raw,
            _marker: PhantomData,
        }
    }
}

impl<'a> AsRef<Snapshot<'a>> for Snapshot<'a> {
    fn as_ref<'b>(&'b self) -> &'b Snapshot<'a> {
        self
    }
}

impl<'a> Snapshot<'a> {
    pub fn get_sequence_number(&self) -> SequenceNumber {
        unsafe { ll::rocks_snapshot_get_sequence_number(self.raw).into() }
    }
}

/// Simple RAII wrapper class for Snapshot.
/// Constructing this object will create a snapshot.  Destructing will
/// release the snapshot.
///
/// Note: this is a pure rust implementation
pub struct ManagedSnapshot<'a, 'b: 'a> {
    snapshot: Snapshot<'a>,
    db: &'b DB<'b>,
}

impl<'a, 'b: 'a> ops::Deref for ManagedSnapshot<'a, 'b> {
    type Target = Snapshot<'a>;
    fn deref<'c>(&'c self) -> &'c Snapshot<'a> {
        &self.snapshot
    }
}

impl<'a, 'b> AsRef<Snapshot<'a>> for ManagedSnapshot<'a, 'b> {
    fn as_ref<'c>(&'c self) -> &'c Snapshot<'a> {
        &self.snapshot
    }
}

impl<'a, 'b> Drop for ManagedSnapshot<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_db_release_snapshot(self.db.raw(), self.snapshot.raw());
        }
    }
}

impl<'a, 'b> ManagedSnapshot<'a, 'b> {
    pub fn new(db: &'b DB<'b>) -> ManagedSnapshot<'a, 'b> {
        let snap = db.get_snapshot().expect("should get snapshot");
        ManagedSnapshot {
            snapshot: snap,
            db: db,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rocksdb::*;

    #[test]
    fn snapshot_read() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(db.put(&WriteOptions::default(), b"k1", b"v1").is_ok());
        assert!(db.put(&WriteOptions::default(), b"k2", b"v2").is_ok());

        assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(0));

        let snap = db.get_snapshot();
        assert!(snap.is_some());

        assert!(db.put(&WriteOptions::default(), b"k2", b"v6").is_ok());
        assert!(db.put(&WriteOptions::default(), b"k3", b"v3").is_ok());

        // live time of ropts, borrows snapshot
        {
            let ropts = ReadOptions::default().snapshot(snap.as_ref());

            assert_eq!(db.get(&ropts, b"k1").expect("db[k1]"), b"v1");
            assert_eq!(db.get(&ropts, b"k2").expect("db[k2]"), b"v2");
            assert!(db.get(&ropts, b"k3").expect_err("db[k3]").is_not_found());

            assert_eq!(db.get(&ReadOptions::default(), b"k2").expect("db[k2]"), b"v6");
            assert_eq!(db.get(&ReadOptions::default(), b"k3").expect("db[k3]"), b"v3");

            // should release
            assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(1));
        }
        db.release_snapshot(snap.unwrap());
        assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(0));
    }


    #[test]
    fn managed_snapshot_read() {
        let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
        let db = DB::open(Options::default().map_db_options(|db| db.create_if_missing(true)), &tmp_dir).unwrap();

        assert!(db.put(&WriteOptions::default(), b"k1", b"v1").is_ok());
        assert!(db.put(&WriteOptions::default(), b"k2", b"v2").is_ok());

        assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(0));

        {
            let snap = ManagedSnapshot::new(&db);

            assert!(db.put(&WriteOptions::default(), b"k2", b"v6").is_ok());
            assert!(db.put(&WriteOptions::default(), b"k3", b"v3").is_ok());

            let ropts = ReadOptions::default().snapshot(Some(&snap));

            assert_eq!(db.get(&ropts, b"k1").expect("db[k1]"), b"v1");
            assert_eq!(db.get(&ropts, b"k2").expect("db[k2]"), b"v2");
            assert!(db.get(&ropts, b"k3").expect_err("db[k3]").is_not_found());

            assert_eq!(db.get(&ReadOptions::default(), b"k2").expect("db[k2]"), b"v6");
            assert_eq!(db.get(&ReadOptions::default(), b"k3").expect("db[k3]"), b"v3");

            assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(1));
            // should release
        }

        assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(0));
    }
}
