extern crate clap;
extern crate walkdir;

extern crate parcel;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path,PathBuf};
use clap::{Arg, App};
use walkdir::WalkDir;
use parcel::{Parcel,InodeAttr};

fn main() {
    let matches = App::new("Parcel-Create")
                            .version("0.1.0")
                            .author("chordtoll <git@chordtoll.com>")
                            .about("Creates parcel archive files for the pyxis package manager")
                            .arg(Arg::with_name("output")
                                .value_name("OUTPUT")
                                .help("The output parcel to generate")
                                .takes_value(true))
                            .arg(Arg::with_name("input")
                                .value_name("INPUT")
                                .help("The input directory to scan")
                                .multiple(true)
                                .takes_value(true))
                            .get_matches();

    let mut parcel : Parcel = Parcel::new();

    let mut dir_map: HashMap<PathBuf,u64> = HashMap::new();
    dir_map.insert(PathBuf::from("/"),1);

    for input in matches.value_of("input") {
        for entry in WalkDir::new(input).min_depth(1) {
            let entry = entry.unwrap();
            let entry_path = Path::new("/").join(entry.path().strip_prefix(input).unwrap());
            let parent_inode = dir_map.get(entry_path.parent().unwrap()).unwrap().clone();
            let entry_name = entry_path.file_name().unwrap();

            let meta = entry.metadata().unwrap();
            let file_type = meta.file_type();

            if file_type.is_file() {
                let attrs = InodeAttr::from_meta(&meta);
                let ino = parcel.add_file(parcel::FileAdd::Name(entry.path().as_os_str().to_os_string()),attrs);
                dir_map.insert(entry_path.clone(),ino);
                parcel.insert_dirent(parent_inode,entry_name.to_os_string(),ino);
            }
            else if file_type.is_dir() {
                let attrs = InodeAttr::from_meta(&meta);
                let ino = parcel.add_directory(attrs);
                dir_map.insert(entry_path.clone(),ino);
                parcel.insert_dirent(parent_inode,entry_name.to_os_string(),ino);
            }
            else if file_type.is_symlink() {
                let attrs = InodeAttr::from_meta(&meta);
                let ino = parcel.add_symlink(fs::read_link(entry.path()).unwrap().as_os_str().to_os_string(),attrs);
                dir_map.insert(entry_path.clone(),ino);
                parcel.insert_dirent(parent_inode,entry_name.to_os_string(),ino);
            }
            else {
                panic!("Unknown file type: {:?}",file_type);
            }
        }
    }

    let outfile = File::create(matches.value_of("output").unwrap()).unwrap();
    parcel.store(outfile);
}