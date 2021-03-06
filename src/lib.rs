#![feature(new_uninit)]
#![feature(read_buf)]
#![feature(maybe_uninit_slice)]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::new_without_default)]

//! Parcel file format for managing pyxis packages.

use std::time::UNIX_EPOCH;

pub use error::ParcelError;
pub use inode::{FileAttr, InodeAttr, InodeKind};
pub use parcel::{FileAdd, ParcelHandle};

/// Error codes
mod error;
/// Inodes and utilities for representing items within a parcel.
mod inode;
/// Parcel metadata for the package manager
mod metadata;
/// The parcel container. Classes and methods.
mod parcel;

mod reader_writer;

pub use reader_writer::ReaderWriter;

const PARCEL_VERSION: u32 = 2;

const ROOT_ATTRS: InodeAttr = InodeAttr {
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    perm:  0o755,
    nlink: 1,
    uid:   0,
    gid:   0,
    rdev:  0,
};
