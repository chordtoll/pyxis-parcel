use std::{
    collections::HashMap,
    ffi::OsString,
    os::linux::fs::MetadataExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use fuser::FileType;
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
            ctime: UNIX_EPOCH + Duration::new(meta.st_atime() as u64, meta.st_atime_nsec() as u32),
            mtime: UNIX_EPOCH + Duration::new(meta.st_atime() as u64, meta.st_atime_nsec() as u32),
            uid:   meta.st_uid(),
            gid:   meta.st_gid(),
            nlink: 1,
            perm:  meta.st_mode(),
            rdev:  meta.st_rdev(),
        }
    }
}

/// The type of the inode
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum InodeKind {
    /// A directory
    Directory,
    /// A regular file
    RegularFile,
    /// A symlink
    Symlink,
    /// A character device
    Char,
}

impl From<InodeKind> for FileType {
    fn from(item: InodeKind) -> Self {
        match item {
            InodeKind::Directory => FileType::Directory,
            InodeKind::RegularFile => FileType::RegularFile,
            InodeKind::Symlink => FileType::Symlink,
            InodeKind::Char => FileType::CharDevice,
        }
    }
}

/// Holds the data for one object
#[derive(Debug, Serialize, Deserialize)]
pub struct Inode {
    /// The type of the inode
    pub kind:   InodeKind,
    /// The inode ID of the file's parent directory
    pub parent: u64,
    /// The inode's attributes
    pub attrs:  InodeAttr,
    /// The inode's extended attributes
    pub xattrs: HashMap<OsString, Vec<u8>>,
}

/// Describes how to find the contents of the file in the data section
#[derive(Debug, Serialize, Deserialize)]
pub struct FileReference {
    /// The offset past the start of the data section at which the file starts
    pub offset: u64,
    /// The length of the file
    pub size:   u64,
}

/// Describes the contents of the object
#[derive(Debug, Serialize, Deserialize)]
pub enum InodeContent {
    /// A directory is a map of names to inode IDs
    Directory(HashMap<String, u64>),
    /// A file is a pointer to an offset+length
    RegularFile(FileReference),
    /// A symlink is a string describing the link target
    Symlink(String),
    /// A character device is a device ID
    Char(u64),
}
