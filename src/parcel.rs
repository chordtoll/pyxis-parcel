use std::{
    cmp::{max, min},
    collections::BTreeMap,
    ffi::OsString,
    fmt::Debug,
    fs,
    fs::File,
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use lexiclean::Lexiclean;
use serde::{Deserialize, Serialize};

use crate::{
    error::ParcelError,
    inode::{FileAttr, FileReference, Inode, InodeAttr, InodeContent, InodeKind},
    metadata::ParcelMetadata,
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

pub trait FileBacking: BufRead + Write + Seek {}

impl Debug for dyn FileBacking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("File Backing")
    }
}

/// A parcel is an archive format holding a directory structure.
/// We use a handle to hold both the parcel's data and the backing file.
pub struct ParcelHandle {
    parcel:  Parcel,
    backing: Option<Box<dyn FileBacking>>,
}

impl ParcelHandle {
    /// Create a new empty parcel
    pub fn new() -> Self {
        Self {
            parcel:  Parcel::new(),
            backing: None,
        }
    }
    /// Set the handle's backing file
    pub fn set_file(&mut self, f: Box<dyn FileBacking>) {
        self.backing = Some(f)
    }

    /// Load a parcel from disk
    pub fn load(mut f: Box<dyn FileBacking>) -> Result<Self> {
        Ok(Self {
            parcel:  Parcel::load(&mut f)?,
            backing: Some(f),
        })
    }
    /// Write a parcel out to disk
    pub fn store(&mut self) -> Result<()> {
        self.parcel.store(
            self.backing
                .as_mut()
                .expect("Writing parcel with no backing file"),
        )
    }
    /// Add a file to the parcel
    pub fn add_file(
        &mut self,
        from: FileAdd,
        attrs: InodeAttr,
        xattrs: BTreeMap<OsString, Vec<u8>>,
    ) -> Result<u64> {
        self.parcel.add_file(from, attrs, xattrs)
    }
    /// Add a directory to the parcel
    pub fn add_directory(&mut self, attrs: InodeAttr, xattrs: BTreeMap<OsString, Vec<u8>>) -> u64 {
        self.parcel.add_directory(attrs, xattrs)
    }
    /// Add a symlink to the parcel
    pub fn add_symlink(
        &mut self,
        target: OsString,
        attrs: InodeAttr,
        xattrs: BTreeMap<OsString, Vec<u8>>,
    ) -> Result<u64> {
        self.parcel.add_symlink(target, attrs, xattrs)
    }
    /// Add a hard link to an existing path in the parcel
    pub fn add_hardlink(&mut self, target: OsString) -> Result<u64> {
        self.parcel.add_hardlink(target)
    }
    /// Get the inode number for a path
    pub fn select(&self, path: PathBuf) -> Option<u64> {
        self.parcel.select(path)
    }
    /// Read the contents of a file
    pub fn read(&mut self, ino: u64, offset: u64, size: Option<u64>) -> Result<Vec<u8>> {
        self.parcel.read(
            self.backing
                .as_mut()
                .expect("Reading from parcel with no backing file"),
            ino,
            offset,
            size,
        )
    }
    /// Write to a file
    pub fn write(&mut self, ino: u64, offset: u64, buf: &[u8]) -> Result<u64> {
        self.parcel.write(
            self.backing
                .as_mut()
                .expect("Writing to parcel with no backing file"),
            ino,
            offset,
            buf,
        )
    }
    /// Add a character device to the parcel
    pub fn add_char(&mut self, attrs: InodeAttr, xattrs: BTreeMap<OsString, Vec<u8>>) -> u64 {
        self.parcel.add_char(attrs, xattrs)
    }
    /// Insert an entry to a directory mapping a filename to an inode
    pub fn insert_dirent(&mut self, parent: u64, name: OsString, child: u64) -> Result<()> {
        self.parcel.insert_dirent(parent, name, child)
    }
    /// Get the attributes of an inode
    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        self.parcel.getattr(ino)
    }
    /// Read the contents of a directory
    pub fn readdir(&self, ino: u64) -> Option<Vec<(u64, InodeKind, String)>> {
        self.parcel.readdir(ino)
    }
    /// Get the inode number of an object by name within a directory
    pub fn lookup(&self, parent: u64, name: String) -> Option<u64> {
        self.parcel.lookup(parent, name)
    }
    /// Get the target of a symlink
    pub fn readlink(&self, ino: u64) -> Option<Vec<u8>> {
        self.parcel.readlink(ino)
    }
    /// Get the extended attributes of an inode
    pub fn getxattrs(&self, ino: u64) -> Option<BTreeMap<OsString, Vec<u8>>> {
        self.parcel.getxattrs(ino)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Parcel {
    version:      u32,
    root_inode:   u64,
    /// The parcel's package metadata
    pub metadata: ParcelMetadata,
    inodes:       BTreeMap<u64, Inode>,
    content:      BTreeMap<u64, InodeContent>,
    #[serde(skip)]
    file_offset:  Option<u64>,
    #[serde(skip)]
    next_inode:   u64,
    #[serde(skip)]
    next_offset:  u64,
    #[serde(skip)]
    to_add:       BTreeMap<u64, FileAdd>,
    #[serde(skip)]
    on_disk:      bool,
}

fn get_parcel_version(buf: &[u8]) -> Result<u32> {
    let contents: serde_yaml::Mapping = serde_yaml::from_slice(buf)?;
    let version = contents
        .get(&serde_yaml::Value::String("version".to_string()))
        .ok_or(ParcelError::NoVersion)?;
    if let serde_yaml::Value::Number(ver) = version {
        Ok(ver.as_u64().ok_or(ParcelError::VersionType)? as u32)
    } else {
        Err(ParcelError::VersionType.into())
    }
}

impl Parcel {
    fn new() -> Parcel {
        let mut parcel = Parcel {
            version:     PARCEL_VERSION,
            root_inode:  1,
            metadata:    ParcelMetadata::new(),
            inodes:      BTreeMap::new(),
            content:     BTreeMap::new(),
            file_offset: None,
            next_inode:  1,
            next_offset: 0,
            to_add:      BTreeMap::new(),
            on_disk:     false,
        };

        parcel.inodes.insert(
            1,
            Inode {
                kind:   InodeKind::Directory,
                parent: 0,
                attrs:  ROOT_ATTRS,
                xattrs: BTreeMap::new(),
            },
        );

        parcel
            .content
            .insert(1, InodeContent::Directory(BTreeMap::new()));

        parcel
    }

    fn load<R: BufRead + Seek>(input: &mut R) -> Result<Parcel> {
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

                // We must first check the version, as the full deserialization will fail if fields have changed.
                let ver = get_parcel_version(&buf)?;
                if ver != PARCEL_VERSION {
                    return Err(ParcelError::VersionMismatch {
                        expected: PARCEL_VERSION,
                        found:    ver,
                    }
                    .into());
                }

                res = serde_yaml::from_slice(&buf)?;
                res.file_offset = Some(input.stream_position()?);
            }
            _ => panic!("Unknown magic: {:?}", magic),
        }
        res.on_disk = true;
        Ok(res)
    }

    fn store<W: Write + Seek>(&mut self, mut output: W) -> Result<()> {
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
        self.on_disk = true;
        self.file_offset = Some(file_offset);
        Ok(())
    }

    fn add_file(
        &mut self,
        from: FileAdd,
        attrs: InodeAttr,
        xattrs: BTreeMap<OsString, Vec<u8>>,
    ) -> Result<u64> {
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
        self.on_disk = false;
        Ok(self.next_inode - 1)
    }

    fn add_directory(&mut self, attrs: InodeAttr, xattrs: BTreeMap<OsString, Vec<u8>>) -> u64 {
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
            .insert(self.next_inode, InodeContent::Directory(BTreeMap::new()));

        self.next_inode += 1;
        self.next_inode - 1
    }

    fn add_symlink(
        &mut self,
        target: OsString,
        attrs: InodeAttr,
        xattrs: BTreeMap<OsString, Vec<u8>>,
    ) -> Result<u64> {
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

    fn add_hardlink(&mut self, target: OsString) -> Result<u64> {
        self.select(PathBuf::from(target))
            .ok_or_else(|| ParcelError::Enoent.into())
    }

    fn add_char(&mut self, attrs: InodeAttr, xattrs: BTreeMap<OsString, Vec<u8>>) -> u64 {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert(
            self.next_inode,
            Inode {
                kind: InodeKind::CharDevice,
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

    fn insert_dirent(&mut self, parent: u64, name: OsString, child: u64) -> Result<()> {
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

    fn select(&self, path: PathBuf) -> Option<u64> {
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

    fn read<R: Read + Seek>(
        &self,
        reader: &mut R,
        ino: u64,
        offset: u64,
        size: Option<u64>,
    ) -> Result<Vec<u8>> {
        assert!(
            self.on_disk,
            "Parcel is not on disk, cannot read without flushing"
        );
        let file = match self.content.get(&ino).ok_or(ParcelError::Enoent)? {
            InodeContent::RegularFile(f) => f,
            _ => return Err(ParcelError::NotFile.into()),
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

    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        ino: u64,
        offset: u64,
        buf: &[u8],
    ) -> Result<u64> {
        assert!(
            self.on_disk,
            "Parcel is not on disk, cannot write without flushing"
        );
        let file = match self.content.get(&ino).ok_or(ParcelError::Enoent)? {
            InodeContent::RegularFile(f) => f,
            _ => return Err(ParcelError::NotFile.into()),
        };
        writer.seek(SeekFrom::Start(
            self.file_offset
                .expect("Parcel not properly loaded- no offset stored to data section")
                + file.offset
                + offset,
        ))?;
        let size = buf.len().try_into()?;
        if size + offset > file.size {
            Err(ParcelError::NeedExpansion.into())
        } else {
            writer.write_all(buf)?;
            Ok(size)
        }
    }

    fn getattr(&self, ino: u64) -> Option<FileAttr> {
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
            InodeContent::RegularFile(_) => InodeKind::RegularFile,
            InodeContent::Directory(_) => InodeKind::Directory,
            InodeContent::Symlink(_) => InodeKind::Symlink,
            InodeContent::Char(_) => InodeKind::CharDevice,
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

    fn readdir(&self, ino: u64) -> Option<Vec<(u64, InodeKind, String)>> {
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
                InodeContent::Char(_) => InodeKind::CharDevice,
            };
            res.push((*v, kind, k.to_string()))
        }
        Some(res)
    }

    fn lookup(&self, parent: u64, name: String) -> Option<u64> {
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

    fn readlink(&self, ino: u64) -> Option<Vec<u8>> {
        let content = self.content.get(&ino)?;
        match content {
            InodeContent::Symlink(s) => Some(s.as_bytes().to_vec()),
            _ => panic!(),
        }
    }

    fn getxattrs(&self, ino: u64) -> Option<BTreeMap<OsString, Vec<u8>>> {
        Some(self.inodes.get(&ino)?.xattrs.clone())
    }
}
