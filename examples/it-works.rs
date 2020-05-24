use rocks::rocksdb;

fn main() {
    println!("RocksDB: {}", rocksdb::version());
    println!("Compression Supported:");
    let mut compressions = rocks::convenience::get_supported_compressions();
    compressions.sort();
    for compression in compressions {
        println!("  - {:?}", compression);
    }
}
