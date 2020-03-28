use rocks::listener::*;
use rocks::prelude::*;
use rocks::sst_file_writer::SstFileWriter;

#[derive(Default)]
struct MyEventListener {
    flush_completed_called: usize,
    flush_begin_called: usize,
    table_file_deleted_called: usize,
    compaction_completed_called: usize,
    table_file_created_called: usize,
    table_file_creation_started_called: usize,
    on_memtable_sealed_called: usize,
    on_external_file_ingested_called: usize,
    on_column_family_handle_deletion_started_called: usize,
}

impl Drop for MyEventListener {
    fn drop(&mut self) {
        assert!(self.flush_begin_called > 0);
        assert!(self.flush_completed_called > 0);
        assert!(self.table_file_deleted_called > 0);
        assert!(self.compaction_completed_called > 0);
        assert!(self.table_file_created_called > 0);
        assert!(self.table_file_creation_started_called > 0);
        assert!(self.on_memtable_sealed_called > 0);
        assert!(self.on_external_file_ingested_called > 0);

        // FIXME: seems default cf is deleted twice
        assert!(self.on_column_family_handle_deletion_started_called > 0);

        // assert!(false);
        // FIXME: must assert drop is called
    }
}

impl EventListener for MyEventListener {
    fn on_flush_completed(&mut self, db: &DBRef, flush_job_info: &FlushJobInfo) {
        assert!(db.name().len() > 0, "DB name is accessible");
        println!(
            "! flush completed: cf={:?} path={:?}",
            flush_job_info.cf_name, flush_job_info.file_path
        );
        self.flush_completed_called += 1;
    }

    fn on_flush_begin(&mut self, _db: &DBRef, flush_job_info: &FlushJobInfo) {
        println!(
            "! flush begin: cf={:?} path={:?}",
            flush_job_info.cf_name, flush_job_info.file_path
        );
        self.flush_begin_called += 1;
    }

    fn on_table_file_deleted(&mut self, info: &TableFileDeletionInfo) {
        assert!(info.status.is_ok());
        println!("! table file deleted: path={:?}", info.file_path);
        self.table_file_deleted_called += 1;
    }

    fn on_compaction_completed(&mut self, _db: &DBRef, ci: &CompactionJobInfo) {
        assert!(ci.status().is_ok());
        assert!(ci.stats().num_input_files() > 0);
        println!(
            "! compaction completed: {:?} => {:?}",
            ci.input_files(),
            ci.output_files()
        );
        self.compaction_completed_called += 1;
    }

    fn on_table_file_created(&mut self, info: &TableFileCreationInfo) {
        // maybe: Err(ShutdownInProgress(None, "Database shutdown or Column family drop during compaction"))
        // so `db.pause_background_work()` is needed
        assert!(info.status().is_ok());
        assert!(info.file_size() > 0);
        assert!(info.table_properties().num_entries() > 0);
        assert!(info.reason() != TableFileCreationReason::Recovery);
        println!("! table file created: path={:?}", info.file_path());
        self.table_file_created_called += 1;
    }

    fn on_table_file_creation_started(&mut self, info: &TableFileCreationBriefInfo) {
        assert!(info.reason() != TableFileCreationReason::Recovery);
        println!("! table file creation started");
        self.table_file_creation_started_called += 1;
    }

    fn on_memtable_sealed(&mut self, info: &MemTableInfo) {
        assert!(info.num_entries() > 0);
        println!("! memtable sealed");
        self.on_memtable_sealed_called += 1;
    }

    fn on_column_family_handle_deletion_started(&mut self, handle: &ColumnFamilyHandle) {
        assert_eq!(handle.id(), 0); // default cf
        println!("! colum family handle deletion started, id={}", handle.id());
        self.on_column_family_handle_deletion_started_called += 1;
    }

    fn on_external_file_ingested(&mut self, _db: &DBRef, info: &ExternalFileIngestionInfo) {
        assert_eq!(info.table_properties().num_entries(), 9);
        println!(
            "! external file ingested, entries={}",
            info.table_properties().num_entries()
        );
        self.on_external_file_ingested_called += 1;
    }

    // TODO: how to test this?
    fn on_background_error(&mut self, reason: BackgroundErrorReason, bg_error: Error) -> Result<(), Error> {
        println!("! background error: reason={:?}", reason);
        Err(bg_error)
    }

    fn get_compaction_event_listener(&mut self) -> Option<&mut dyn CompactionEventListener> {
        static mut FUNC: &'static dyn Fn(CompactionEvent) = &|event| {
            assert!(event.is_new);
            // print here will suppress rust test's capture
            // since it'll be called from C++
            // println!("listen compaction event: got => {:?} {:?}", event.sn, event);
            println!("! got compaction event");
        };
        unsafe { Some(&mut FUNC) }
    }
}

fn main() {
    let tmp_dir = ::tempdir::TempDir::new_in(".", "rocks").unwrap();
    let db = DB::open(
        Options::default().map_db_options(|db| db.create_if_missing(true).add_listener(MyEventListener::default())),
        &tmp_dir,
    )
    .unwrap();

    for i in 0..100 {
        let key = format!("test2-key-{}", i);
        let val = format!("rocksdb-value-{}", i * 10);

        db.put(&WriteOptions::default(), key.as_bytes(), val.as_bytes())
            .unwrap();

        if i % 6 == 0 {
            db.flush(&FlushOptions::default().wait(true)).unwrap();
        }
        if i % 36 == 0 {
            db.compact_range(&CompactRangeOptions::default(), ..).unwrap();
        }
    }

    assert!(db.flush(&Default::default()).is_ok());

    // ingest an sst file
    let sst_dir = ::tempdir::TempDir::new_in(".", "rocks.sst").unwrap();
    let writer = SstFileWriter::builder().build();
    writer.open(sst_dir.path().join("2333.sst")).unwrap();
    for i in 0..9 {
        let key = format!("B{:05}", i);
        let value = format!("ABCDEFGH{:03}IJKLMN", i);
        writer.put(key.as_bytes(), value.as_bytes()).unwrap();
    }
    let info = writer.finish().unwrap();
    assert_eq!(info.num_entries(), 9);

    let ret = db.ingest_external_file(
        &[sst_dir.path().join("2333.sst")],
        &IngestExternalFileOptions::default(),
    );
    assert!(ret.is_ok(), "ingest external file fails: {:?}", ret);

    // must have bg threads
    assert!(Env::default_instance().get_thread_list().len() > 0);

    // safe shutdown
    assert!(db.pause_background_work().is_ok());
}
