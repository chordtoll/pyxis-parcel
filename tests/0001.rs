use pyxis_parcel::{FileAdd, ParcelHandle};

mod common;
use common::Fixture;

#[test]
fn empty_serialize() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel.store().unwrap();
    f.compare("empty_serialize.parcel");
}

#[test]
fn add_file_string() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel
        .add_file(
            FileAdd::Bytes(b"foo".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    f.compare("add_file.parcel");
}

#[test]
fn add_file_file() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel
        .add_file(
            FileAdd::Name("tests/data/foo".into()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    f.compare("add_file.parcel");
}

#[test]
fn insert_file_dirent() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    let add = parcel
        .add_file(
            FileAdd::Bytes(b"foo".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.insert_dirent(1, "foo".into(), add).unwrap();
    parcel.store().unwrap();
    f.compare("insert_file_dirent.parcel");
}

#[test]
fn add_dir() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel.add_directory(Default::default(), Default::default());
    parcel.store().unwrap();
    f.compare("add_dir.parcel");
}

#[test]
fn insert_dir_dirent() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    let add = parcel.add_directory(Default::default(), Default::default());
    parcel.insert_dirent(1, "foo".into(), add).unwrap();
    parcel.store().unwrap();
    f.compare("insert_dir_dirent.parcel");
}

#[test]
fn add_multiple_files() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel
        .add_file(
            FileAdd::Bytes(b"foo".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel
        .add_file(
            FileAdd::Bytes(b"bar".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel
        .add_file(
            FileAdd::Bytes(b"baz".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    f.compare("add_multiple_files.parcel");
}

#[test]
#[should_panic(expected = "Parcel is not on disk, cannot read without flushing")]
fn add_read_no_flush() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel.store().unwrap();
    let ino = parcel
        .add_file(
            FileAdd::Bytes(b"foo".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.read(ino, 0, None).unwrap();
}

#[test]
fn add_read_flush() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel.store().unwrap();
    let ino = parcel
        .add_file(
            FileAdd::Bytes(b"foo".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    parcel.read(ino, 0, None).unwrap();
}
