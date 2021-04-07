extern crate clap;
extern crate walkdir;

extern crate parcel;

use std::io::Cursor;

use clap::{Arg, App};
use walkdir::WalkDir;
use parcel::Parcel;

fn main() {
    let matches = App::new("My Super Program")
                          .version("1.0")
                          .author("Kevin K. <kbknapp@gmail.com>")
                          .about("Does awesome things")
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

    let ino1 = parcel.add_file(parcel::FileAdd::Bytes(b"foo".to_vec()));
    let ino2 = parcel.add_file(parcel::FileAdd::Bytes(b"bar".to_vec()));
    let ino3 = parcel.add_file(parcel::FileAdd::Name("Cargo.toml".to_string()));

    parcel.insert_dirent(1,"foo.txt".to_string()   ,ino1);
    parcel.insert_dirent(1,"bar.txt".to_string()   ,ino2);
    parcel.insert_dirent(1,"Cargo.toml".to_string(),ino3);

    let mut buf : Vec<u8> = Vec::new();
    parcel.store(Cursor::new(&mut buf));
    println!("{}",String::from_utf8_lossy(&buf));

}