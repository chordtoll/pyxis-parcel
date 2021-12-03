#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]

//! Parcel file format for managing pyxis packages.

use std::time::UNIX_EPOCH;

pub use inode::{InodeAttr, InodeKind};
pub use parcel::{FileAdd, Parcel};

/// Error codes
mod error;
/// Inodes and utilities for representing items within a parcel.
mod inode;
/// The parcel container. Classes and methods.
mod parcel;

const PARCEL_VERSION: u32 = 0;

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
