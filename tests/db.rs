use rocks::prelude::*;
use std::collections::HashMap;
use tempdir::TempDir;

#[test]
fn it_works() {
    use rocks::advanced_options::CompactionPri;

    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    // staircase style config
    let opt = Options::default()
        .map_db_options(|dbopt| dbopt.create_if_missing(true))
        .map_cf_options(|cfopt| cfopt.compaction_pri(CompactionPri::MinOverlappingRatio))
        .optimize_for_small_db();
    let db = DB::open(&opt, path);
    assert!(db.is_ok(), "err => {:?}", db);
    let db = db.unwrap();
    let cfhandle = db.create_column_family(&ColumnFamilyOptions::default(), "lock");
    println!("cf => {:?}", cfhandle);

    assert!(db.name().contains("rocks"));

    // FIXME: missing on static build?
    // assert!(db.get_info_log_list().is_ok());
    // assert!(db.get_info_log_list().unwrap().contains(&"LOG".to_string()));
}

#[test]
fn test_open_for_readonly() {
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {
        let opt = Options::default().map_db_options(|opt| opt.create_if_missing(true));
        let db = DB::open(&opt, path);
        assert!(db.is_ok());
    }

    let db = DB::open_for_readonly(&Options::default(), path, false);
    assert!(db.is_ok());
}

#[test]
fn test_list_cfs() {
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {
        let opt = Options::default().map_db_options(|opt| opt.create_if_missing(true));
        let db = DB::open(&opt, path);
        assert!(db.is_ok());

        let db = db.unwrap();
        let ret = db.create_column_family(&ColumnFamilyOptions::default(), "cf1");
        assert!(ret.is_ok());

        let ret = db.create_column_family(&ColumnFamilyOptions::default(), "cf2");
        assert!(ret.is_ok());
    }

    let opt = Options::default();
    let ret = DB::list_column_families(&opt, path);
    assert!(ret.is_ok());
    assert!(ret.as_ref().unwrap().contains(&"default".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"cf1".to_owned()));
    assert!(ret.as_ref().unwrap().contains(&"cf2".to_owned()));

    let cfs = ret.unwrap();
    if let Ok((db, cf_handles)) = DB::open_with_column_families(&Options::default(), path, cfs) {
        let iters = db.new_iterators(&ReadOptions::default().pin_data(true), &cf_handles);
        println!("its => {:?}", iters);
        assert!(iters.is_ok());
    }
}

#[test]
fn test_db_get() {
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();
    let path = tmp_dir.path().to_str().unwrap();

    {
        let opt = Options::default().map_db_options(|dbopt| dbopt.create_if_missing(true));

        let db = DB::open(&opt, path);
        assert!(db.is_ok(), "err => {:?}", db.as_ref().unwrap_err());
        let db = db.unwrap();
        let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW");
    }

    let db = DB::open(Options::default(), path).unwrap();
    let val = db.get(&ReadOptions::default(), b"name");
    assert_eq!(val.unwrap().as_ref(), b"BH1XUW");
}

#[test]
fn test_open_cf() {
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let ret = DB::open_with_column_families(
        &opt,
        tmp_dir.path().to_str().unwrap(),
        vec![ColumnFamilyDescriptor::default()],
    );
    assert!(ret.is_ok(), "err => {:?}", ret);
    println!("cfs => {:?}", ret);

    if let Ok((_db, cfs)) = ret {
        let cf = &cfs[0];
        println!("cf name => {:?} id => {}", cf.name(), cf.id());
    }
}

#[test]
#[ignore]
// FIXME: lifetime leaks
fn test_cf_lifetime() {
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|db| db.create_if_missing(true));

    let mut cf_handle = None;
    {
        let ret = DB::open_with_column_families(
            &opt,
            tmp_dir.path().to_str().unwrap(),
            vec![ColumnFamilyDescriptor::default()],
        );
        assert!(ret.is_ok(), "err => {:?}", ret);
        println!("cfs => {:?}", ret);

        if let Ok((_db, mut cfs)) = ret {
            let cf = cfs.pop().unwrap();
            println!("cf name => {:?} id => {}", cf.name(), cf.id());
            cf_handle = Some(cf);
        }
    }
    println!("cf name => {:?}", cf_handle.unwrap().name());
}

#[test]
fn test_key_may_exist() {
    let tmp_dir = TempDir::new_in(".", "rocks").unwrap();

    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        tmp_dir,
    )
    .unwrap();

    db.put(&WriteOptions::default(), b"name", b"value").unwrap();

    assert!(db.key_may_exist(&ReadOptions::default(), b"name"));
    assert!(!db.key_may_exist(&ReadOptions::default(), b"name2"))
}

#[test]
fn test_ingest_sst_file() {
    use rocks::sst_file_writer::SstFileWriter;

    let sst_dir = ::tempdir::TempDir::new_in(".", "rocks.sst").unwrap();

    let writer = SstFileWriter::builder().build();
    writer.open(sst_dir.path().join("2333.sst")).unwrap();
    for i in 0..999 {
        let key = format!("B{:05}", i);
        let value = format!("ABCDEFGH{:03}IJKLMN", i);
        writer.put(key.as_bytes(), value.as_bytes()).unwrap();
    }
    let info = writer.finish().unwrap();
    assert_eq!(info.num_entries(), 999);

    let tmp_db_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_db_dir,
    )
    .unwrap();

    let ret = db.ingest_external_file(
        &[sst_dir.path().join("2333.sst")],
        &IngestExternalFileOptions::default(),
    );
    assert!(ret.is_ok(), "ingest external file: {:?}", ret);

    assert!(db.get(&ReadOptions::default(), b"B00000").is_ok());
    assert_eq!(
        db.get(&ReadOptions::default(), b"B00000").unwrap(),
        b"ABCDEFGH000IJKLMN"
    );
    assert_eq!(
        db.get(&ReadOptions::default(), b"B00998").unwrap(),
        b"ABCDEFGH998IJKLMN"
    );
    assert!(db.get(&ReadOptions::default(), b"B00999").is_err());

    drop(sst_dir);
    drop(tmp_db_dir);
}

#[test]
fn compact_range() {
    let s = b"123123123";
    let e = b"asdfasfasfasf";

    let _: ::std::ops::Range<&[u8]> = s.as_ref()..e.as_ref();

    let tmp_db_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|dbopt| dbopt.create_if_missing(true));

    let db = DB::open(opt, &tmp_db_dir).unwrap();

    let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW").unwrap();
    for i in 0..100 {
        let key = format!("test2-key-{}", i);
        let val = format!("rocksdb-value-{}", i * 10);

        db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
            .unwrap();

        db.flush(&Default::default()).unwrap()
    }

    // will be shown in LOG file
    let ret = db.compact_range(
        &CompactRangeOptions::default(),
        b"test2-key-5".as_ref()..b"test2-key-9".as_ref(),
    );
    assert!(ret.is_ok());

    let ret = db.compact_range(&CompactRangeOptions::default(), ..);
    assert!(ret.is_ok());

    drop(tmp_db_dir);
}

#[test]
fn multi_get() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();

    assert!(db.put(&Default::default(), b"a", b"1").is_ok());
    assert!(db.put(&Default::default(), b"b", b"2").is_ok());
    assert!(db.put(&Default::default(), b"c", b"3").is_ok());
    assert!(db.put(&Default::default(), b"long-key", b"long-value").is_ok());
    assert!(db.put(&Default::default(), b"e", b"5").is_ok());
    assert!(db.put(&Default::default(), b"f", b"6").is_ok());

    assert!(db.compact_range(&Default::default(), ..).is_ok());

    let ret = db.multi_get(
        &ReadOptions::default(),
        &[b"a", b"b", b"c", b"f", b"long-key", b"non-exist"],
    );

    assert_eq!(ret[0].as_ref().unwrap(), b"1".as_ref());
    assert_eq!(ret[1].as_ref().unwrap(), b"2".as_ref());
    assert_eq!(ret[2].as_ref().unwrap(), b"3".as_ref());
    assert_eq!(ret[3].as_ref().unwrap(), b"6".as_ref());
    assert_eq!(ret[4].as_ref().unwrap(), b"long-value".as_ref());
    assert!(ret[5].as_ref().unwrap_err().is_not_found());
}

#[test]
fn multi_get_cf() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();

    let def = db.default_column_family();
    let cf1 = db.create_column_family(&Default::default(), "db1").unwrap();
    let cf2 = db.create_column_family(&Default::default(), "db2").unwrap();
    let cf3 = db.create_column_family(&Default::default(), "db3").unwrap();
    let cf4 = db.create_column_family(&Default::default(), "db4").unwrap();

    // via DB api
    assert!(db.put_cf(&WriteOptions::default(), &def, b"AA", b"aa").is_ok());
    assert!(db.put_cf(&WriteOptions::default(), &cf1, b"BB", b"bb").is_ok());
    assert!(db.put_cf(&WriteOptions::default(), &cf2, b"CC", b"cc").is_ok());
    assert!(db.put_cf(&WriteOptions::default(), &cf3, b"DD", b"dd").is_ok());
    assert!(db.put_cf(&WriteOptions::default(), &cf4, b"EE", b"ee").is_ok());

    // via CF api
    assert!(def.put(&WriteOptions::default(), b"AA", b"aa").is_ok());
    assert!(cf1.put(&WriteOptions::default(), b"BB", b"bb").is_ok());
    assert!(cf2.put(&WriteOptions::default(), b"CC", b"cc").is_ok());
    assert!(cf3.put(&WriteOptions::default(), b"DD", b"dd").is_ok());
    assert!(cf4.put(&WriteOptions::default(), b"EE", b"ee").is_ok());

    assert!(def.compact_range(&Default::default(), ..).is_ok());

    assert!(db.compact_range(&Default::default(), ..).is_ok());

    let ret = db.multi_get_cf(
        &ReadOptions::default(),
        &[&def, &cf1, &cf2, &cf3, &cf4, &def],
        &[b"AA", b"BB", b"CC", b"DD", b"EE", b"233"],
    );

    assert_eq!(ret[0].as_ref().unwrap(), b"aa".as_ref());
    assert_eq!(ret[2].as_ref().unwrap(), b"cc".as_ref());
    assert_eq!(ret[4].as_ref().unwrap(), b"ee".as_ref());
    assert!(ret[5].as_ref().unwrap_err().is_not_found());

    // mem::forget(def);
}

#[test]
fn db_paths() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let dir1 = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let dir2 = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let wal_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();

    let opt = Options::default().map_db_options(|dbopt| {
        dbopt
            .create_if_missing(true)
            .db_paths(vec![&dir1.path(), &dir2.path()]) /* only has sst file */
            .wal_dir(&wal_dir)
    });

    let db = DB::open(opt, &tmp_dir);
    if db.is_err() {
        println!("db error");
        println!("err => {:?}", db);
        return;
    }
    let db = db.unwrap();
    let _ = db.put(&WriteOptions::default(), b"name", b"BH1XUW").unwrap();
    for i in 0..10 {
        let key = format!("k{}", i);
        let value = format!("v{:03}", i);
        if i == 5 {
            let s = db.get_snapshot();
            assert!(s.is_some());
            db.release_snapshot(s.unwrap());
        }

        db.put(&WriteOptions::default(), key.as_bytes(), value.as_bytes())
            .unwrap();

        assert!(db.flush(&FlushOptions::default()).is_ok());
    }

    let d1 = dir1.path().read_dir().expect("should have a sst dir");
    let d2 = dir2.path().read_dir().expect("should have a sst dir");
    assert!(d1.count() > 0 || d2.count() > 0);

    let w = wal_dir.path().read_dir().expect("should have a wal dir");
    assert!(w.count() >= 1); // at least 1 log file

    let d = tmp_dir.path().read_dir().expect("should have a data dir");
    assert!(d.count() >= 2); // OPTIONS, MANIFEST, etc.
}

#[test]
fn key_may_exist() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();

    assert!(db.put(&Default::default(), b"long-key", b"long-value").is_ok());
    assert!(db.compact_range(&Default::default(), ..).is_ok());

    assert!(db.key_may_exist(&ReadOptions::default(), b"long-key"));
    assert!(!db.key_may_exist(&ReadOptions::default(), b"long-key-not-exist"));

    let (found, maybe_val) = db.key_may_get(&ReadOptions::default(), b"long-key");
    assert!(found);
    // it depends, Some/None are all OK
    let _ = maybe_val;

    let (found, maybe_val) = db.key_may_get(&ReadOptions::default(), b"not-exist");
    assert!(!found);
    assert!(!maybe_val.is_some());
}

#[test]
fn get_prop() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();

    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());

    let cf1 = db.create_column_family(&Default::default(), "db1").unwrap();

    assert!(db.compact_range(&Default::default(), ..).is_ok());

    let snap = db.get_snapshot();
    assert_eq!(db.get_property("rocksdb.num-snapshots"), Some("1".to_string()));

    // dump status
    println!("stats => {}", db.get_property("rocksdb.stats").unwrap());
    assert_eq!(db.get_int_property("rocksdb.num-snapshots"), Some(1));

    assert!(db
        .put(&Default::default(), b"long-key2", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());

    assert!(cf1
        .put(&Default::default(), b"long-key2", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());

    assert!(db.get_int_property("rocksdb.size-all-mem-tables").unwrap() < 2 * 1024 * 1024);

    assert!(db.get_aggregated_int_property("rocksdb.size-all-mem-tables").unwrap() > 2 * 1024 * 1024);

    db.release_snapshot(snap.unwrap());
}

#[test]
fn misc_functions() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.disable_auto_compactions(true)),
        &tmp_dir,
    )
    .unwrap();

    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.put(&Default::default(), b"a", b"1").is_ok());
    assert!(db.put(&Default::default(), b"b", b"2").is_ok());
    assert!(db.put(&Default::default(), b"c", b"3").is_ok());

    assert!(db.compact_range(&Default::default(), ..).is_ok());

    assert!(db.pause_background_work().is_ok());
    assert!(db.continue_background_work().is_ok());

    assert!(db.enable_auto_compaction(&[&db.default_column_family()]).is_ok());

    assert_eq!(db.number_levels(), 7); // default
    assert_eq!(db.max_mem_compaction_level(), 0); // TODO: wtf
    assert_eq!(db.level0_stop_write_trigger(), 36); // default

    assert!(db.get_db_identity().is_ok());
    println!("id => {:?}", db.get_db_identity());
}

#[test]
fn flush() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.disable_auto_compactions(true)),
        &tmp_dir,
    )
    .unwrap();

    assert_eq!(*db.get_latest_sequence_number(), 0);

    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.put(&Default::default(), b"a", b"1").is_ok());
    assert!(db.put(&Default::default(), b"b", b"2").is_ok());
    assert!(db.put(&Default::default(), b"c", b"3").is_ok());

    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db.sync_wal().is_ok());

    // 5th transaction
    assert_eq!(*db.get_latest_sequence_number(), 4);
}

#[test]
fn livemetadata() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();

    assert!(db.disable_file_deletions().is_ok());
    let meta = db.get_live_files_metadata();
    assert_eq!(meta.len(), 0);

    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    let meta = db.get_live_files_metadata();
    assert_eq!(meta.len(), 1);
    assert_eq!(meta[0].level, 0);

    assert!(db.put(&Default::default(), b"a", b"1").is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db.put(&Default::default(), b"b", b"2").is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db.put(&Default::default(), b"c", b"3").is_ok());
    assert!(db.put(&Default::default(), b"d", b"3").is_ok());
    assert!(db.put(&Default::default(), b"asdlfkjasl", b"askdfjkl3").is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    let meta = db.get_live_files_metadata();
    assert_eq!(meta.len(), 4);
    assert!(db.compact_range(&CompactRangeOptions::default(), ..).is_ok());

    let meta = db.get_live_files_metadata();
    assert!(meta.len() < 4);
    assert_eq!(meta[0].level, 1);
}

#[test]
fn column_family_meta() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();

    for i in 0..10 {
        let key = format!("k{}", i);
        let val = format!("v{}", i * 10);

        db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
            .unwrap();

        // 2 keys into a sst
        if i % 2 == 0 {
            assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        }

        // leave 6-9 uncompacted
        if i == 5 {
            assert!(db.compact_range(&CompactRangeOptions::default(), ..).is_ok());
        }
    }

    let meta = db.get_column_family_metadata(&db.default_column_family());
    println!("Meta => {:?}", meta);
    assert_eq!(meta.levels.len(), 7, "default level num");
    assert!(meta.levels[0].files.len() + meta.levels[1].files.len() > 1);
    assert!(meta.levels[4].files.len() == 0);
}

#[test]
fn list_live_files() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        &tmp_dir,
    )
    .unwrap();
    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db
        .put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
        .is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());

    if let Ok((_size, files)) = db.get_live_files(false) {
        assert!(files.contains(&"/CURRENT".to_string()));
    } else {
        assert!(false, "get_live_files fails");
    }
}

#[test]
fn get_sorted_wal_files() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| {
            db.create_if_missing(true)
                .db_write_buffer_size(2 << 20) // 2MB per wal log
                .wal_ttl_seconds(1000)
        }),
        &tmp_dir,
    )
    .unwrap();
    for i in 0..10 {
        assert!(db
            .put(
                &Default::default(),
                format!("key{}", i).as_bytes(),
                format!("val{:01000000}", i).as_bytes()
            ) // 1MB value
            .is_ok());
    }
    let files = db.get_sorted_wal_files();
    assert!(files.is_ok());
    assert!(files.unwrap().len() > 2);
}

#[test]
fn change_options() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.disable_auto_compactions(true)), // disable
        &tmp_dir,
    )
    .unwrap();
    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db
        .put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
        .is_ok());

    let new_opt: HashMap<&str, &str> = [("base_background_compactions", "6"), ("stats_dump_period_sec", "10")] // dump every 10s
        .iter()
        .cloned()
        .collect();
    let ret = db.set_db_options(&new_opt);
    assert!(ret.is_ok());

    let new_opt: HashMap<&str, &str> = [
        ("write_buffer_size", "10000000"),
        ("level0_file_num_compaction_trigger", "2"),
    ]
    .iter()
    .cloned()
    .collect();
    assert!(db.set_options(&new_opt).is_ok());

    let new_opt: HashMap<&str, &str> = [("non-exist-write_buffer_size", "10000000")].iter().cloned().collect();
    let ret = db.set_options(&new_opt);
    assert!(ret.is_err());
    assert!(format!("{:?}", ret).contains("Unrecognized option"));
}

#[test]
fn approximate_sizes() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.disable_auto_compactions(true)), // disable
        &tmp_dir,
    )
    .unwrap();
    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db
        .put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
        .is_ok());

    let sizes = db.get_approximate_sizes(&[b"long-key".as_ref()..&b"long-key-".as_ref()]);
    assert_eq!(sizes.len(), 1);
    assert!(sizes[0] > 0);

    for i in 0..100 {
        let key = format!("k{}", i);
        let val = format!("v{}", i * 10);

        db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
            .unwrap();
    }
    let (count, size) = db.get_approximate_memtable_stats(b"a".as_ref()..&b"z".as_ref());
    assert!(count > 0 && count < 200);
    assert!(size > 0);
}

#[test]
fn compact_files() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.disable_auto_compactions(true)), // disable
        &tmp_dir,
    )
    .unwrap();
    assert!(db
        .put(&Default::default(), b"long-key", vec![b'A'; 1024 * 1024].as_ref())
        .is_ok());
    assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    assert!(db
        .put(&Default::default(), b"long-key-2", vec![b'A'; 2 * 1024].as_ref())
        .is_ok());

    for i in 0..10 {
        let key = format!("k{}", i);
        let val = format!("v{}", i * 10);

        db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
            .unwrap();

        if i % 2 == 0 {
            assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        }
    }
    let v = db.get_live_files(true);

    let sst_files = v
        .as_ref()
        .unwrap()
        .1
        .iter()
        .filter(|name| name.ends_with(".sst"))
        .map(|name| name.as_ref())
        .collect::<Vec<&str>>();
    assert!(sst_files.len() > 2); // many sst files

    let _st = db.compact_files(
        &CompactionOptions::default().compression(CompressionType::BZip2Compression),
        &sst_files,
        4,
    ); // output to level 4

    let result = db.get_live_files_metadata();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].level, 4); // compacted to 4
}

#[test]
fn get_properties_of_all_tables() {
    let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
    let db = DB::open(
        Options::default()
            .map_db_options(|db| db.create_if_missing(true))
            .map_cf_options(|cf| cf.disable_auto_compactions(true)),
        &tmp_dir,
    )
    .unwrap();

    for i in 0..10 {
        let key = format!("k{}", i);
        let val = format!("v{}", i * i);

        db.put(WriteOptions::default_instance(), key.as_bytes(), val.as_bytes())
            .unwrap();

        if i % 2 == 0 {
            assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
        }
    }

    let props = db.get_properties_of_all_tables_cf(&db.default_column_family());
    assert!(props.is_ok());
    let props = props.unwrap();
    assert!(props.len() > 4, "should be more than 4 sst files");

    for (k, prop) in props.iter() {
        println!("key => {:?}", k);
        println!("    => {:?}", prop);
        println!("data size ={}", prop.data_size());
        assert_eq!(prop.column_family_name(), Some("default"));

        println!("filter policy name = {:?}", prop.filter_policy_name());
        println!("comparator name = {:?}", prop.comparator_name());

        assert_eq!(prop.property_collectors_names(), "[]");

        let user_prop = prop.user_collected_properties();
        println!("len => {:?}", user_prop.len());
        for (k, v) in user_prop.iter() {
            println!("    {}=>{:?}", k, v);
        }
        let readable_prop = prop.readable_properties();
        println!("readable => {:?}", readable_prop);
    }

    let vals = props.iter().map(|(k, _)| k).collect::<Vec<_>>();
    assert!(vals.len() > 4);
}

#[test]
fn delete_files_in_range() {
    let tmp_dir = ::tempdir::TempDir::new_in("", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true)),
        // NOTE: delete_files_in_range() requires auto compaction
        // .map_cf_options(|cf| cf.disable_auto_compactions(true)),
        &tmp_dir,
    )
    .unwrap();

    // will have 10 sst file
    for i in 0..10 {
        let key = format!("k{}", i);
        let val = format!("v{}", i * i);

        db.put(WriteOptions::default_instance(), key.as_bytes(), val.as_bytes())
            .unwrap();

        assert!(db.flush(&FlushOptions::default().wait(true)).is_ok());
    }

    // NOTE: size is manifest_file_size, not total size
    let (_old_size, old_files) = db.get_live_files(false).expect("should get live files");

    assert!(db
        .delete_files_in_range(&db.default_column_family(), b"k2", b"k8")
        .is_ok());

    let (_new_size, new_files) = db.get_live_files(false).expect("should get live files");

    assert!(new_files.len() < old_files.len());
    for f in &new_files {
        assert!(old_files.contains(f));
    }
}
