extern crate fuser;

use fuser::FileType;
use std::collections::HashMap;
const PARCEL_VERSION: u32 = 0;

#[derive(Debug)]
struct Inode {
    kind: FileType,
    parent: u64,
}

#[derive(Debug)]
pub struct Parcel {
    version: u32,
    root_inode: u32,
    inodes: HashMap<u64,Inode>,
    directories: HashMap<u64,HashMap<String,u64>>,
    files: HashMap<u64,Vec<u8>>,
}

impl Parcel {
    pub fn new() -> Parcel {
        let mut parcel = Parcel {
            version: PARCEL_VERSION,
            root_inode: 1,
            inodes: HashMap::new(),
            directories: HashMap::new(),
            files: HashMap::new(),
        };
        
        parcel.inodes.insert(1,Inode { kind: FileType::Directory, parent: 0 });
        parcel.inodes.insert(2,Inode { kind: FileType::RegularFile, parent: 1 });

        parcel.directories.insert(1,HashMap::new());
        parcel.directories.get_mut(&1).unwrap().insert(".PARCEL".to_string(),2);

        parcel.files.insert(2,PARCEL_VERSION.to_string().as_bytes().to_vec());

        parcel
    }
}