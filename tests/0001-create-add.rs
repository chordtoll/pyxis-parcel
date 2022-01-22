use std::ffi::OsString;

use pyxis_parcel::{FileAdd, InodeKind, ParcelHandle};

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
    parcel
        .insert_dirent(1, "foo".into(), add, InodeKind::RegularFile)
        .unwrap();
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
fn add_char() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel.add_char(Default::default(), Default::default());
    parcel.store().unwrap();
    f.compare("add_char.parcel");
}

#[test]
fn add_symlink() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    parcel
        .add_symlink(
            OsString::from("foo"),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    f.compare("add_symlink.parcel");
}

#[test]
fn add_hardlink() {
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
    parcel
        .insert_dirent(1, "foo".into(), add, InodeKind::RegularFile)
        .unwrap();
    let link = parcel.add_hardlink(OsString::from("/foo")).unwrap();
    assert_eq!(add, link);
    parcel.store().unwrap();
    f.compare("insert_file_dirent.parcel");
}

#[test]
fn insert_dir_dirent() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());
    let add = parcel.add_directory(Default::default(), Default::default());
    parcel
        .insert_dirent(1, "foo".into(), add, InodeKind::Directory)
        .unwrap();
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
fn add_reload_add() {
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

    let mut parcel = ParcelHandle::load(f.make_rw()).unwrap();
    assert_eq!(parcel.read(2, 0, None).unwrap(), b"foo");
    parcel
        .add_file(
            FileAdd::Bytes(b"bar".to_vec()),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    parcel.store().unwrap();
    f.compare("add_reload_add.parcel");
}

#[test]
fn add_reload_delete() {
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
    parcel.store().unwrap();
    f.compare("add_file.parcel");

    let mut parcel = ParcelHandle::load(f.make_rw()).unwrap();
    assert_eq!(parcel.read(2, 0, None).unwrap(), b"foo");
    parcel.delete(add).unwrap();
    parcel.store().unwrap();
    f.compare("add_reload_delete.parcel");
}
