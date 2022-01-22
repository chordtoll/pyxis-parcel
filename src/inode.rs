use std::{
    collections::BTreeMap,
    ffi::OsString,
    os::linux::fs::MetadataExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[allow(missing_docs)]
pub struct FileAttr {
    pub atime:   SystemTime,
    pub mtime:   SystemTime,
    pub ctime:   SystemTime,
    pub crtime:  SystemTime,
    pub blocks:  u64,
    pub blksize: u32,
    pub gid:     u32,
    pub uid:     u32,
    pub ino:     u64,
    pub nlink:   u32,
    pub perm:    u16,
    pub rdev:    u32,
    pub size:    u64,
    pub kind:    InodeKind,
    pub flags:   u32,
}

use serde::{Deserialize, Serialize};

/// Contains the attributes of the inode
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct InodeAttr {
    /// Access time
    pub atime: SystemTime,
    /// Modification time
    pub mtime: SystemTime,
    /// Creation time
    pub ctime: SystemTime,
    /// Permissions
    pub perm:  u32,
    /// Number of hard links
    pub nlink: u32,
    /// Owner user ID
    pub uid:   u32,
    /// Owner group ID
    pub gid:   u32,
    /// Special file device ID
    pub rdev:  u64,
}

impl InodeAttr {
    /// Create an InodeAttr from Linux metadata
    pub fn from_meta(meta: &dyn MetadataExt) -> InodeAttr {
        InodeAttr {
            atime: UNIX_EPOCH + Duration::new(meta.st_atime() as u64, meta.st_atime_nsec() as u32),
            ctime: UNIX_EPOCH + Duration::new(meta.st_ctime() as u64, meta.st_ctime_nsec() as u32),
            mtime: UNIX_EPOCH + Duration::new(meta.st_mtime() as u64, meta.st_mtime_nsec() as u32),
            uid:   meta.st_uid(),
            gid:   meta.st_gid(),
            nlink: 1,
            perm:  meta.st_mode(),
            rdev:  meta.st_rdev(),
        }
    }
}

impl Default for InodeAttr {
    fn default() -> Self {
        Self {
            atime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            uid:   0,
            gid:   0,
            nlink: 1,
            perm:  0,
            rdev:  0,
        }
    }
}

/// The type of the inode
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Copy, Clone)]
pub enum InodeKind {
    /// A directory
    Directory,
    /// A regular file
    RegularFile,
    /// A symlink
    Symlink,
    /// A character device
    CharDevice,
    /// A deleted inode
    Whiteout,
}

/// Holds the data for one object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Inode {
    /// The type of the inode
    pub kind:   InodeKind,
    /// The inode ID of the file's parent directory
    pub parent: u64,
    /// The inode's attributes
    pub attrs:  InodeAttr,
    /// The inode's extended attributes
    pub xattrs: BTreeMap<OsString, Vec<u8>>,
}

/// Describes how to find the contents of the file in the data section
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileReference {
    /// The offset past the start of the data section at which the file starts
    pub offset:   u64,
    /// The length of the file
    pub size:     u64,
    /// The amount of space reserved for the file
    pub capacity: u64,
}

/// Describes the contents of the object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InodeContent {
    /// A directory is a map of names to inode IDs
    Directory(BTreeMap<String, (u64, InodeKind)>),
    /// A file is a pointer to an offset+length
    RegularFile(FileReference),
    /// A symlink is a string describing the link target
    Symlink(String),
    /// A character device is a device ID
    Char(u64),
    /// A whiteout inode is a placeholder
    Whiteout,
}

impl From<FileAttr> for InodeAttr {
    fn from(v: FileAttr) -> Self {
        Self {
            atime: v.atime,
            mtime: v.mtime,
            ctime: v.ctime,
            perm:  v.perm as u32,
            nlink: v.nlink,
            uid:   v.uid,
            gid:   v.gid,
            rdev:  v.rdev as u64,
        }
    }
}
