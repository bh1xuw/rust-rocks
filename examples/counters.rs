extern crate rocks;

use rocks::prelude::*;

const DB_PATH: &str = "./data.merge_op";

pub struct UInt64AddOperator;

fn deserialize(value: &[u8]) -> u64 {
    value
        .iter()
        .enumerate()
        .fold(0, |acc, (i, &v)| acc + ((v as u64) << ((7 - i) * 8)))
}

fn serialize(value: u64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

impl AssociativeMergeOperator for UInt64AddOperator {
    fn merge(&self, key: &[u8], existing_value: Option<&[u8]>, value: &[u8], _logger: &Logger) -> Option<Vec<u8>> {
        println!(
            "merge: key = {:?} existing_value = {:?} value = {:?}",
            key, existing_value, value
        );
        // assuming 0 if no existing value
        let existing = existing_value.map(|raw| deserialize(raw)).unwrap_or_default();
        let oper = deserialize(value);

        let new = existing + oper;
        return Some(serialize(new));
    }
}

pub struct MergeBasedCounters {
    db: DB,
}

impl MergeBasedCounters {
    pub fn new(db: DB) -> Self {
        MergeBasedCounters { db }
    }

    pub fn add(&self, key: &str, value: u64) {
        let serialized = serialize(value);
        let _ = self
            .db
            .merge(WriteOptions::default_instance(), key.as_bytes(), &serialized);
    }

    pub fn get(&self, key: &str) -> Option<u64> {
        self.db
            .get(ReadOptions::default_instance(), key.as_bytes())
            .map(|raw| deserialize(&*raw))
            .ok()
    }

    /// mapped to a RocksDB Delete
    pub fn remove(&self, key: &str) {
        self.db
            .delete(WriteOptions::default_instance(), key.as_bytes())
            .unwrap();
    }

    /// mapped to a RocksDB Put
    pub fn set(&self, key: &str, value: u64) {
        let serialized = serialize(value);
        let _ = self
            .db
            .put(WriteOptions::default_instance(), key.as_bytes(), &serialized);
    }
}

fn main() {
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.associative_merge_operator(Box::new(UInt64AddOperator))),
        DB_PATH,
    )
    .unwrap();

    let counters = MergeBasedCounters::new(db);
    // counters.remove("a");
    counters.add("a", 5);
    println!("val => {:?}", counters.get("a"));
    // counters.set("a", 100);
    // println!("val => {:?}", counters.get("a"));
}
