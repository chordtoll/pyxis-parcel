use pyxis_parcel::{FileAdd, ParcelHandle};

mod common;
use common::Fixture;

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

#[test]
fn write() {
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
    parcel.write(ino, 0, b"bar").unwrap();
    assert_eq!(parcel.read(ino, 0, None).unwrap(),b"bar");
}