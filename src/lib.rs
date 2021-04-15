extern crate fuser;
extern crate serde;
extern crate serde_yaml;
extern crate indoc;
extern crate lexiclean;

use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs;
use std::fs::File;
use std::os::linux::fs::MetadataExt;
use std::ffi::OsString;
use std::cmp::{max,min};
use std::collections::HashMap;
use std::path::{Path,PathBuf};
use std::time::{SystemTime,UNIX_EPOCH,Duration};
use lexiclean::Lexiclean;
use fuser::{FileAttr,FileType};

use serde::{Serialize, Deserialize};



const PARCEL_VERSION: u32 = 0;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum InodeKind {
    Directory,
    RegularFile,
    Symlink,
}

impl From<InodeKind> for FileType {
    fn from(item: InodeKind) -> Self {
        match item {
            InodeKind::Directory => FileType::Directory,
            InodeKind::RegularFile => FileType::RegularFile,
            InodeKind::Symlink => FileType::Symlink,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct InodeAttr {
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub perm:  u32,
    pub nlink: u32,
    pub uid:   u32,
    pub gid:   u32,
    pub rdev:  u64,
}

impl InodeAttr {
    pub fn from_meta(meta: &dyn MetadataExt) -> InodeAttr {
        InodeAttr {
            atime: UNIX_EPOCH+Duration::new(meta.st_atime() as u64,meta.st_atime_nsec() as u32),
            ctime: UNIX_EPOCH+Duration::new(meta.st_atime() as u64,meta.st_atime_nsec() as u32),
            mtime: UNIX_EPOCH+Duration::new(meta.st_atime() as u64,meta.st_atime_nsec() as u32),
            uid:   meta.st_uid(),
            gid:   meta.st_gid(),
            nlink: 1,
            perm:  meta.st_mode(),
            rdev:  meta.st_rdev(),
        }
    }
}

const ROOT_ATTRS: InodeAttr = InodeAttr {
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    perm: 0o755,
    nlink: 1,
    uid: 0,
    gid: 0,
    rdev: 0,
};

#[derive(Debug, Serialize, Deserialize)]
enum InodeContent {
    Directory(HashMap<String,u64>),
    RegularFile(FileReference),
    Symlink(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct FileReference {
    offset: u64,
    size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Inode {
    kind: InodeKind,
    parent: u64,
    attrs: InodeAttr,
    xattrs: HashMap<OsString,Vec<u8>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parcel {
    version: u32,
    root_inode: u64,
    inodes: HashMap<u64,Inode>,
    content: HashMap<u64,InodeContent>,
    #[serde(skip)]
    file_offset: Option<u64>,
    #[serde(skip)]
    next_inode: u64,
    #[serde(skip)]
    next_offset: u64,
    #[serde(skip)]
    to_add: HashMap<u64,FileAdd>,
}

#[derive(Debug)]
pub enum FileAdd {
    Bytes(Vec<u8>),
    Name(OsString),
}

impl Parcel {
    pub fn new() -> Parcel {
        let mut parcel = Parcel {
            version: PARCEL_VERSION,
            root_inode: 1,
            inodes: HashMap::new(),
            content: HashMap::new(),
            file_offset: None,
            next_inode: 1,
            next_offset: 0,
            to_add: HashMap::new(),
        };
        
        parcel.inodes.insert(1,Inode { kind: InodeKind::Directory, parent: 0, attrs: ROOT_ATTRS, xattrs: HashMap::new() });

        parcel.content.insert(1,InodeContent::Directory(HashMap::new()));

        parcel
    }

    pub fn load<R: BufRead + Seek>(input: &mut R) -> Parcel {
        let mut res : Parcel;

        let mut magic : [u8;4] = [0;4];

        input.read_exact(&mut magic).unwrap();

        match &magic {
            b"413\n" => {

                let mut buf : Vec<u8> = Vec::new();
                let mut buf_size=0;
                loop {
                    input.read_until(0xA,&mut buf).unwrap();
                    if buf.len()==buf_size {
                        panic!();
                    }
                    buf_size = buf.len();
                    if buf.ends_with(&[0xA,0x2E,0x2E,0x2E,0xA]) {
                        break;
                    }
                }
                buf.truncate(buf.len()-5);
                res = serde_yaml::from_slice(&buf).unwrap();
                res.file_offset = Some(input.stream_position().unwrap());
            },
            _ => panic!("Unknown magic: {:?}",magic),
        }
        res
    }

    pub fn store<W: Write + Seek>(&self, mut output: W) {
        output.write(b"413\n").unwrap();
        serde_yaml::to_writer(&mut output,self).unwrap();
        output.write(b"\n...\n").unwrap();
        let file_offset = output.stream_position().unwrap();
        for (ino,val) in self.to_add.iter() {
            match &self.content[ino] {
                InodeContent::RegularFile(file) => {
                    output.seek(SeekFrom::Start(file_offset + file.offset)).unwrap();
                    assert_eq!(match val {
                        FileAdd::Bytes(content) => output.write(content).unwrap() as u64,
                        FileAdd::Name(name) => io::copy(&mut File::open(name).unwrap(),&mut output).unwrap(),
                    },file.size);
                },
                _ => panic!(),
            }
        }
    }

    pub fn add_file(&mut self, from: FileAdd, attrs: InodeAttr, xattrs: HashMap<OsString,Vec<u8>>) -> u64 {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert(self.next_inode, Inode {kind: InodeKind::RegularFile, parent: 0, attrs: attrs, xattrs: xattrs});

        let filesize = match &from {
            FileAdd::Bytes(i) => i.len() as u64,
            FileAdd::Name(name) => fs::metadata(name).unwrap().len(),
        };

        self.to_add.insert(self.next_inode, from);
        self.content.insert(self.next_inode, InodeContent::RegularFile(FileReference { offset: self.next_offset, size: filesize}));
        self.next_offset += filesize;

        self.next_inode+=1;
        self.next_inode-1
    }

    pub fn add_directory(&mut self, attrs: InodeAttr, xattrs: HashMap<OsString,Vec<u8>>) -> u64 {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert( self.next_inode, Inode {kind: InodeKind::Directory, parent: 0, attrs: attrs, xattrs: xattrs});
        self.content.insert(self.next_inode,InodeContent::Directory(HashMap::new()));

        self.next_inode+=1;
        self.next_inode-1
    }

    pub fn add_symlink(&mut self, target: OsString, attrs: InodeAttr, xattrs: HashMap<OsString,Vec<u8>>) -> u64 {
        while self.inodes.contains_key(&self.next_inode) {
            self.next_inode += 1;
        }

        self.inodes.insert( self.next_inode, Inode {kind: InodeKind::Symlink, parent: 0, attrs: attrs, xattrs: xattrs});
        self.content.insert(self.next_inode,InodeContent::Symlink(target.into_string().unwrap()));

        self.next_inode+=1;
        self.next_inode-1
    }

    pub fn insert_dirent(&mut self, parent: u64, name: OsString, child: u64) {
        match self.content.get_mut(&parent).unwrap() {
            InodeContent::Directory(dir) => dir.insert(name.into_string().unwrap(),child),
            _ => panic!(),
        };

        self.inodes.get_mut(&child).unwrap().parent = parent;
    }

    pub fn select(&self, path: PathBuf) -> Option<u64> {
        let path = match path.has_root() {
            true => path,
            false => Path::new("/").join(path),
        };
        let mut ino : Option<u64>= None;
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

    pub fn read<R: Read + Seek>(&self, reader: &mut R, ino: u64, offset: u64, size:Option<u64>) -> Option<Vec<u8>> {
        let file = match self.content.get(&ino)? {
            InodeContent::RegularFile(f) => f,
            _ => return None,
        };
        reader.seek(SeekFrom::Start(self.file_offset? + file.offset + offset)).unwrap();
        let size = match size {
            Some(s) => max(min(s+offset,file.size)-offset,0),
            None => file.size,
        };
        let mut buf = vec![0u8; size as usize];
        reader.read_exact(&mut buf).ok()?;
        Some(buf)
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let inode = self.inodes.get(&ino)?;
        let attrs = inode.attrs;
        let content = self.content.get(&ino)?;
        let size = match content {
            InodeContent::RegularFile(f) => f.size,
            InodeContent::Directory(_) => 0,
            InodeContent::Symlink(s) => s.len() as u64,
        };
        let kind = match content {
            InodeContent::RegularFile(_) => FileType::RegularFile,
            InodeContent::Directory(_) => FileType::Directory,
            InodeContent::Symlink(_) => FileType::Symlink,
        };
        Some(
            FileAttr {
                 atime:   attrs.atime
                ,ctime:   attrs.ctime
                ,mtime:   attrs.mtime
                ,blksize: 8192
                ,blocks:  (size+8191)/8192
                ,gid:     attrs.gid
                ,uid:     attrs.uid
                ,ino:     ino
                ,nlink:   attrs.nlink
                ,padding: 0
                ,perm:    attrs.perm as u16
                ,rdev:    attrs.rdev as u32
                ,size:    size
                ,kind:    kind
                ,flags:   0
                ,crtime:  attrs.ctime
            }
        )
    }

    pub fn readdir(&self,ino: u64) -> Option<Vec<(u64,InodeKind,String)>> {
        let mut res : Vec<(u64,InodeKind,String)> = Vec::new();

        let content = match self.content.get(&ino)? {
            InodeContent::Directory(d) => d,
            _ => return None,
        };
        for (k,v) in content.iter() {
            let kind = match self.content.get(&v)? {
                InodeContent::RegularFile(_) => InodeKind::RegularFile,
                InodeContent::Directory(_) => InodeKind::Directory,
                InodeContent::Symlink(_) => InodeKind::Symlink,
            };
            res.push((*v,kind,k.to_string()))
        }
        Some(res)
    }

    pub fn lookup(&self,parent: u64, name: String) -> Option<u64> {

        let content = match self.content.get(&parent)? {
            InodeContent::Directory(d) => d,
            _ => return None,
        };
        for (k,v) in content.iter() {
            if *k==name {
                return Some(*v);
            }
        }
        None
    }

    pub fn readlink(&self, ino: u64) -> Option<Vec<u8>> {
        let content = self.content.get(&ino)?;
        match content {
            InodeContent::Symlink(s) => Some(s.as_bytes().to_vec()),
            _ => panic!(),
        }
    }

    pub fn getxattrs(&self, ino: u64) -> Option<HashMap<OsString,Vec<u8>>> {
        Some(self.inodes.get(&ino)?.xattrs.clone())
    }
}

#[cfg(test)]
use std::io::Cursor;
#[cfg(test)]
use indoc::indoc;

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        Parcel::new();
    }

    #[test]
    fn serialize() {
        let buf : Vec<u8> = Vec::new();
        Parcel::new().store(Cursor::new(buf));
    }

    #[test]
    fn deserialize() {
        let mut instr = Cursor::new(indoc! {br#"
            413
            ---
            version: 0
            root_inode: 1
            inodes:
              1:
                kind: Directory
                parent: 0
                attrs:
                  atime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  mtime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  ctime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  perm: 493
                  nlink: 1
                  uid: 0
                  gid: 0
                  rdev: 0
                xattrs: {}
              2:
                kind: RegularFile
                parent: 1
                attrs:
                  atime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  mtime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  ctime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  perm: 0
                  nlink: 1
                  uid: 0
                  gid: 0
                  rdev: 0
                xattrs: {}
              4:
                kind: RegularFile
                parent: 1
                attrs:
                  atime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  mtime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  ctime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  perm: 0
                  nlink: 1
                  uid: 0
                  gid: 0
                  rdev: 0
                xattrs: {}
              3:
                kind: RegularFile
                parent: 1
                attrs:
                  atime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  mtime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  ctime:
                    secs_since_epoch: 0
                    nanos_since_epoch: 0
                  perm: 0
                  nlink: 1
                  uid: 0
                  gid: 0
                  rdev: 0
                xattrs: {}
            content:
              1:
                Directory:
                  foo.txt: 2
                  Cargo.toml: 4
                  bar.txt: 3
              2:
                RegularFile:
                  offset: 0
                  size: 3
              4:
                RegularFile:
                  offset: 6
                  size: 360
              3:
                RegularFile:
                  offset: 3
                  size: 3
            ...
            foobar[package]
            name = "parcel"
            version = "0.1.0"
            authors = ["chordtoll <git@chordtoll.com>"]
            edition = "2018"

            # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

            [dependencies]
            clap = "2.33.3"
            walkdir = "2.3.2"
            fuser = "0.7.0"
            serde_yaml = "0.8.17"
            serde =  { version = "1.0.125", features = ["derive"] }
            indoc = "1.0"
            "#
        });
        Parcel::load(&mut instr);
    }
}*/