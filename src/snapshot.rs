//! Abstract handle to particular state of a DB.

use std::fmt;
use std::marker::PhantomData;

use rocks_sys as ll;

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

impl<'a> fmt::Debug for Snapshot<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Snapshot({:?})", self.raw)
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

impl<'a> Snapshot<'a> {
    pub fn get_sequence_number(&self) -> SequenceNumber {
        unimplemented!()
    }
}



/// Simple RAII wrapper class for Snapshot.
/// Constructing this object will create a snapshot.  Destructing will
/// release the snapshot.
pub struct ManagedSnapshot;


// Instead of creating a snapshot, take ownership of the input snapshot.
// ManagedSnapshot(DB* db, const Snapshot* _snapshot);
