use std::{
    cmp::{max, min},
    collections::HashMap,
    ffi::OsString,
    fs,
    fs::File,
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use fuser::{FileAttr, FileType};
use lexiclean::Lexiclean;
use serde::{Deserialize, Serialize};

use crate::{
    error::ParcelError,
    inode::{FileReference, Inode, InodeAttr, InodeContent, InodeKind},
    PARCEL_VERSION, ROOT_ATTRS,
};

/// Temporarily holds a file we want to add to the parcel
#[derive(Debug)]
pub enum FileAdd {
    /// We want to add a file by its literal contents
    Bytes(Vec<u8>),
    /// We want to add a file by its path on disk
    Name(OsString),
}

/// A parcel is an archive format holding a directory structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Parcel {
    version:     u32,
    root_inode:  u64,
    inodes:      HashMap<u64, Inode>,
    content:     HashMap<u64, InodeContent>,
    #[serde(skip)]
    file_offset: Option<u64>,
    #[serde(skip)]
    next_inode:  u64,
    #[serde(skip)]
    next_offset: u64,
    #[serde(skip)]
    to_add:      HashMap<u64, FileAdd>,
}

impl Default for Parcel {
    fn default() -> Self {
        Self::new()
    }
}

impl Parcel {
    /// Create a new empty parcel
    pub fn new() -> Parcel {
        let mut parcel = Parcel {
            version:     PARCEL_VERSION,
            root_inode:  1,
            inodes:      HashMap::new(),
            content:     HashMap::new(),
            file_offset: None,
            next_inode:  1,
            next_offset: 0,
            to_add:      HashMap::new(),
        };

        parcel.inodes.insert(
            1,
            Inode {
                kind:   InodeKind::Directory,
                parent: 0,
                attrs:  ROOT_ATTRS,
                xattrs: HashMap::new(),
            },
        );

        parcel
            .content
            .insert(1, InodeContent::Directory(HashMap::new()));

        parcel
    }

    /// Load a parcel from disk
    pub fn load<R: BufRead + Seek>(input: &mut R) -> Result<Parcel, ParcelError> {
        let mut res: Parcel;

        let mut magic: [u8; 4] = [0; 4];

        input.read_exact(&mut magic)?;

        match &magic {
            b"413\n" => {
                let mut buf: Vec<u8> = Vec::new();
                let mut buf_size = 0;
                loop {
                    input.read_until(0xA, &mut buf)?;
                    if buf.len() == buf_size {
                        panic!();
                    }
                    buf_size = buf.len();
                    if buf.ends_with(&[0xA, 0x2E, 0x2E, 0x2E, 0xA]) {
                        break;
                    }
                }
                buf.truncate(buf.len() - 5);
                res = serde_yaml::from_slice(&buf)?;
                res.file_offset = Some(input.stream_position()?);
            }
            _ => panic!("Unknown magic: {:?}", magic),
        }
        if res.version != PARCEL_VERSION {
            return Err(ParcelError::VersionMismatch);
        }
        Ok(res)
    }

    /// Write a parcel out to disk
    pub fn store<W: Write + Seek>(&self, mut output: W) -> Result<(), ParcelError> {
        output.write_all(b"413\n")?;
        serde_yaml::to_writer(&mut output, self)?;
        output.write_all(b"\n...\n")?;
        let file_offset = output.stream_position()?;
        for (ino, val) in self.to_add.iter() {
            match &self.content[ino] {
                InodeContent::RegularFile(file) => {
                    output.seek(SeekFrom::Start(file_offset + file.offset))?;
                    assert_eq!(
                        match val {
                            FileAdd::Bytes(content) => output.write(content)? as u64,
                            FileAdd::Name(name) => io::copy(&mut File::open(name)?, &mut output)?,
                        },
                        file.size
                    );
                }
                _ => panic!(),
            }
        }
        Ok(())
    }

    /// Add a file to the parcel
    pub fn add_file(
        &mut self,
        from: FileAdd,
        attrs: InodeAttr,
        xattrs: HashMap<OsString, Vec<u8>>,
    ) -> Result<u64, ParcelError> {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert(
            self.next_inode,
            Inode {
                kind: InodeKind::RegularFile,
                parent: 0,
                attrs,
                xattrs,
            },
        );

        let filesize = match &from {
            FileAdd::Bytes(i) => i.len() as u64,
            FileAdd::Name(name) => fs::metadata(name)?.len(),
        };

        self.to_add.insert(self.next_inode, from);
        self.content.insert(
            self.next_inode,
            InodeContent::RegularFile(FileReference {
                offset: self.next_offset,
                size:   filesize,
            }),
        );
        self.next_offset += filesize;

        self.next_inode += 1;
        Ok(self.next_inode - 1)
    }

    /// Add a directory to the parcel
    pub fn add_directory(&mut self, attrs: InodeAttr, xattrs: HashMap<OsString, Vec<u8>>) -> u64 {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert(
            self.next_inode,
            Inode {
                kind: InodeKind::Directory,
                parent: 0,
                attrs,
                xattrs,
            },
        );
        self.content
            .insert(self.next_inode, InodeContent::Directory(HashMap::new()));

        self.next_inode += 1;
        self.next_inode - 1
    }

    /// Add a symlink to the parcel
    pub fn add_symlink(
        &mut self,
        target: OsString,
        attrs: InodeAttr,
        xattrs: HashMap<OsString, Vec<u8>>,
    ) -> Result<u64, ParcelError> {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert(
            self.next_inode,
            Inode {
                kind: InodeKind::Symlink,
                parent: 0,
                attrs,
                xattrs,
            },
        );
        self.content.insert(
            self.next_inode,
            InodeContent::Symlink(
                target
                    .into_string()
                    .or(Err(ParcelError::StringConversion))?,
            ),
        );

        self.next_inode += 1;
        Ok(self.next_inode - 1)
    }

    /// Add a hard link to an existing path in the parcel
    pub fn add_hardlink(&mut self, target: OsString) -> Result<u64, ParcelError> {
        self.select(PathBuf::from(target))
            .ok_or(ParcelError::Enoent)
    }

    /// Add a character device to the parcel
    pub fn add_char(&mut self, attrs: InodeAttr, xattrs: HashMap<OsString, Vec<u8>>) -> u64 {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert(
            self.next_inode,
            Inode {
                kind: InodeKind::Char,
                parent: 0,
                attrs,
                xattrs,
            },
        );
        self.content
            .insert(self.next_inode, InodeContent::Char(attrs.rdev));

        self.next_inode += 1;
        self.next_inode - 1
    }

    /// Insert an entry to a directory mapping a filename to an inode
    pub fn insert_dirent(
        &mut self,
        parent: u64,
        name: OsString,
        child: u64,
    ) -> Result<(), ParcelError> {
        match self.content.get_mut(&parent).unwrap() {
            InodeContent::Directory(dir) => dir.insert(
                name.into_string().or(Err(ParcelError::StringConversion))?,
                child,
            ),
            _ => panic!(),
        };

        self.inodes.get_mut(&child).unwrap().parent = parent;
        Ok(())
    }

    /// Get the inode number for a path
    pub fn select(&self, path: PathBuf) -> Option<u64> {
        let path = match path.has_root() {
            true => path,
            false => Path::new("/").join(path),
        };
        let mut ino: Option<u64> = None;
        for ent in path.lexiclean().iter() {
            if ent == "/" {
                ino = Some(self.root_inode);
            } else {
                ino = Some(match self.content.get(&ino?)? {
                    InodeContent::Directory(d) => *d.get(ent.to_str()?)?,
                    _ => return None,
                });
            }
        }
        ino
    }

    /// Read the contents of a file
    pub fn read<R: Read + Seek>(
        &self,
        reader: &mut R,
        ino: u64,
        offset: u64,
        size: Option<u64>,
    ) -> Result<Vec<u8>, ParcelError> {
        let file = match self.content.get(&ino).ok_or(ParcelError::Enoent)? {
            InodeContent::RegularFile(f) => f,
            _ => return Err(ParcelError::NotFile),
        };
        reader.seek(SeekFrom::Start(
            self.file_offset
                .expect("Parcel not properly loaded- no offset stored to data section")
                + file.offset
                + offset,
        ))?;
        let size = match size {
            Some(s) => max(min(s + offset, file.size) - offset, 0),
            None => file.size,
        };
        let mut buf = vec![0u8; size as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Get the attributes of an inode
    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let inode = self.inodes.get(&ino)?;
        let attrs = inode.attrs;
        let content = self.content.get(&ino)?;
        let size = match content {
            InodeContent::RegularFile(f) => f.size,
            InodeContent::Directory(_) => 0,
            InodeContent::Symlink(s) => s.len() as u64,
            InodeContent::Char(_) => 0,
        };
        let kind = match content {
            InodeContent::RegularFile(_) => FileType::RegularFile,
            InodeContent::Directory(_) => FileType::Directory,
            InodeContent::Symlink(_) => FileType::Symlink,
            InodeContent::Char(_) => FileType::CharDevice,
        };
        Some(FileAttr {
            atime: attrs.atime,
            ctime: attrs.ctime,
            mtime: attrs.mtime,
            blocks: (size + 8191) / 8192,
            blksize: 8192,
            gid: attrs.gid,
            uid: attrs.uid,
            ino,
            nlink: attrs.nlink,
            perm: attrs.perm as u16,
            rdev: attrs.rdev as u32,
            size,
            kind,
            flags: 0,
            crtime: attrs.ctime,
        })
    }

    /// Read the contents of a directory
    pub fn readdir(&self, ino: u64) -> Option<Vec<(u64, InodeKind, String)>> {
        let mut res: Vec<(u64, InodeKind, String)> = Vec::new();

        let content = match self.content.get(&ino)? {
            InodeContent::Directory(d) => d,
            _ => return None,
        };
        for (k, v) in content.iter() {
            let kind = match self.content.get(v)? {
                InodeContent::RegularFile(_) => InodeKind::RegularFile,
                InodeContent::Directory(_) => InodeKind::Directory,
                InodeContent::Symlink(_) => InodeKind::Symlink,
                InodeContent::Char(_) => InodeKind::Char,
            };
            res.push((*v, kind, k.to_string()))
        }
        Some(res)
    }

    /// Get the inode number of an object by name within a directory
    pub fn lookup(&self, parent: u64, name: String) -> Option<u64> {
        let content = match self.content.get(&parent)? {
            InodeContent::Directory(d) => d,
            _ => return None,
        };
        for (k, v) in content.iter() {
            if *k == name {
                return Some(*v);
            }
        }
        None
    }

    /// Get the target of a symlink
    pub fn readlink(&self, ino: u64) -> Option<Vec<u8>> {
        let content = self.content.get(&ino)?;
        match content {
            InodeContent::Symlink(s) => Some(s.as_bytes().to_vec()),
            _ => panic!(),
        }
    }

    /// Get the extended attributes of an inode
    pub fn getxattrs(&self, ino: u64) -> Option<HashMap<OsString, Vec<u8>>> {
        Some(self.inodes.get(&ino)?.xattrs.clone())
    }
}
