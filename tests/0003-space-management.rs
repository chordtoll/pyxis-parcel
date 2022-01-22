use pyxis_parcel::{FileAdd, ParcelHandle};

mod common;
use common::Fixture;

#[test]
fn realloc_last() {
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
    parcel.realloc_reserved(ino, 6).unwrap();
    parcel.store().unwrap();
    f.compare("realloc_last.parcel");
}

#[test]
fn realloc_first() {
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
    parcel
        .add_file(
            FileAdd::Bytes(b"bar".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    parcel.realloc_reserved(ino, 6).unwrap();
    parcel.store().unwrap();
    f.compare("realloc_first.parcel");
}

#[test]
fn realloc_middle() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel.store().unwrap();
    parcel
        .add_file(
            FileAdd::Bytes(b"foo".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    let ino = parcel
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
    parcel.realloc_reserved(ino, 6).unwrap();
    parcel.store().unwrap();
    f.compare("realloc_middle.parcel");
}

#[test]
fn realloc_create() {
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
    parcel.realloc_reserved(ino, 6).unwrap();
    parcel
        .add_file(
            FileAdd::Bytes(b"bar".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    f.compare("realloc_create.parcel");
}
