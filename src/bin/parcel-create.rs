use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    fs::File,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
};

use clap::{App, Arg};
use pyxis_parcel::{FileAdd, InodeAttr, Parcel};
use walkdir::WalkDir;

fn main() {
    let matches = App::new("Parcel-Create")
        .version("0.1.0")
        .author("chordtoll <git@chordtoll.com>")
        .about("Creates parcel archive files for the pyxis package manager")
        .arg(
            Arg::with_name("version")
                .takes_value(true)
                .long("version")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .value_name("OUTPUT")
                .help("The output parcel to generate")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("input")
                .value_name("INPUT")
                .help("The input directory to scan")
                .multiple(true)
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mut parcel: Parcel = Parcel::new();

    parcel.metadata.version = matches.value_of("version").unwrap().to_owned();

    let mut dir_map: BTreeMap<PathBuf, u64> = BTreeMap::new();
    dir_map.insert(PathBuf::from("/"), 1);

    for input in matches.values_of("input").unwrap() {
        for entry in WalkDir::new(input).min_depth(1) {
            let entry = entry.unwrap();
            let entry_path = Path::new("/").join(entry.path().strip_prefix(input).unwrap());
            let parent_inode = *dir_map.get(entry_path.parent().unwrap()).unwrap();
            let entry_name = entry_path.file_name().unwrap();

            let meta = entry.metadata().unwrap();
            let file_type = meta.file_type();

            if file_type.is_file() {
                let attrs = InodeAttr::from_meta(&meta);
                let mut xattrs: BTreeMap<OsString, Vec<u8>> = BTreeMap::new();
                for attr in xattr::list(entry.path()).unwrap() {
                    xattrs.insert(
                        attr.clone(),
                        xattr::get(entry.path(), attr.clone()).unwrap().unwrap(),
                    );
                }
                let ino = parcel
                    .add_file(
                        FileAdd::Name(entry.path().as_os_str().to_os_string()),
                        attrs,
                        xattrs,
                    )
                    .unwrap();
                dir_map.insert(entry_path.clone(), ino);
                parcel
                    .insert_dirent(parent_inode, entry_name.to_os_string(), ino)
                    .unwrap();
            } else if file_type.is_dir() {
                let attrs = InodeAttr::from_meta(&meta);
                let mut xattrs: BTreeMap<OsString, Vec<u8>> = BTreeMap::new();
                for attr in xattr::list(entry.path()).unwrap() {
                    xattrs.insert(
                        attr.clone(),
                        xattr::get(entry.path(), attr.clone()).unwrap().unwrap(),
                    );
                }
                let ino = parcel.add_directory(attrs, xattrs);
                dir_map.insert(entry_path.clone(), ino);
                parcel
                    .insert_dirent(parent_inode, entry_name.to_os_string(), ino)
                    .unwrap();
            } else if file_type.is_symlink() {
                let attrs = InodeAttr::from_meta(&meta);
                let mut xattrs: BTreeMap<OsString, Vec<u8>> = BTreeMap::new();
                for attr in xattr::list(entry.path()).unwrap() {
                    xattrs.insert(
                        attr.clone(),
                        xattr::get(entry.path(), attr.clone()).unwrap().unwrap(),
                    );
                }
                let ino = parcel
                    .add_symlink(
                        fs::read_link(entry.path())
                            .unwrap()
                            .as_os_str()
                            .to_os_string(),
                        attrs,
                        xattrs,
                    )
                    .unwrap();
                dir_map.insert(entry_path.clone(), ino);
                parcel
                    .insert_dirent(parent_inode, entry_name.to_os_string(), ino)
                    .unwrap();
            } else if file_type.is_char_device() {
                let attrs = InodeAttr::from_meta(&meta);
                let mut xattrs: BTreeMap<OsString, Vec<u8>> = BTreeMap::new();
                for attr in xattr::list(entry.path()).unwrap() {
                    xattrs.insert(
                        attr.clone(),
                        xattr::get(entry.path(), attr.clone()).unwrap().unwrap(),
                    );
                }
                let ino = parcel.add_char(attrs, xattrs);
                dir_map.insert(entry_path.clone(), ino);
                parcel
                    .insert_dirent(parent_inode, entry_name.to_os_string(), ino)
                    .unwrap();
            } else if file_type.is_block_device() {
                unimplemented!("Block device");
            } else if file_type.is_fifo() {
                unimplemented!("FIFO");
            } else if file_type.is_socket() {
                unimplemented!("Socket");
            } else {
                panic!("Unknown file type: {:?}", file_type);
            }
        }
    }

    let outfile = File::create(matches.value_of("output").unwrap()).unwrap();
    parcel.store(outfile).unwrap();
}
