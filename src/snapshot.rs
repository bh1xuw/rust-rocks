use types::SequenceNumber;

/// Abstract handle to particular state of a DB.
/// A Snapshot is an immutable object and can therefore be safely
/// accessed from multiple threads without any external synchronization.
///
/// To Create a Snapshot, call DB::GetSnapshot().
/// To Destroy a Snapshot, call DB::ReleaseSnapshot(snapshot).
pub struct Snapshot;

impl Snapshot {
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
