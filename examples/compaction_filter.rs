extern crate rocks;

use lazy_static::lazy_static;
use rocks::compaction_filter::{CompactionFilter, Decision, ValueType};
use rocks::merge_operator::{MergeOperationInput, MergeOperationOutput, MergeOperator};
use rocks::prelude::*;

pub struct MyMerge;

impl MergeOperator for MyMerge {
    fn full_merge(&self, merge_in: &MergeOperationInput, merge_out: &mut MergeOperationOutput) -> bool {
        if let Some(value) = merge_in.existing_value() {
            merge_out.assign(value);
        }
        for m in merge_in.operands() {
            eprintln!("Merge({:?})", String::from_utf8_lossy(m));
            // the compaction filter filters out bad values
            assert!(m != b"bad");
            merge_out.assign(m);
        }
        true
    }

    fn name(&self) -> &str {
        "MyMerge\0"
    }
}

#[derive(Debug, Default)]
pub struct MyFilter {
    count: usize,
    merge_count: usize,
}

impl CompactionFilter for MyFilter {
    // rust-rocks only impls the `FilterV2` API.
    fn filter(&mut self, _level: i32, key: &[u8], value_type: ValueType, existing_value: &[u8]) -> Decision {
        match value_type {
            ValueType::Value => {
                eprintln!("Filter({:?})", String::from_utf8_lossy(key));
                self.count += 1;
                Decision::Keep
            },
            ValueType::MergeOperand => {
                eprintln!("FilterMerge({:?})", String::from_utf8_lossy(key));
                self.merge_count += 1;
                if existing_value == b"bad" {
                    Decision::Remove
                } else {
                    Decision::Keep
                }
            },
        }
    }

    fn name(&self) -> &str {
        "MyFilterV2\0"
    }
}

lazy_static! {
    static ref MY_FILTER: MyFilter = MyFilter::default();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const DB_PATH: &str = "/tmp/rocksmergetest";

    let options = Options::default()
        .map_db_options(|opt| opt.create_if_missing(true))
        .map_cf_options(|opt| opt.merge_operator(Box::new(MyMerge)).compaction_filter(&*MY_FILTER));

    let db = DB::open(&options, DB_PATH)?;

    let wopts = WriteOptions::default_instance();
    db.merge(wopts, b"0", b"bad")?;
    db.merge(wopts, b"1", b"data1")?;
    db.merge(wopts, b"1", b"bad")?;
    db.merge(wopts, b"1", b"data2")?;
    db.merge(wopts, b"1", b"bad")?;
    db.merge(wopts, b"3", b"data3")?;

    db.compact_range(&CompactRangeOptions::default(), ..)?;

    println!("{:?}", &*MY_FILTER);

    Ok(())
}
