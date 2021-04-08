extern crate clap;
extern crate walkdir;

extern crate parcel;

use std::fs::File;
use std::io;
use std::io::Write;
use std::io::BufReader;
use std::path::PathBuf;
use clap::{Arg, App};
use parcel::Parcel;

fn main() {
    let matches = App::new("Parcel-Cat")
                            .version("0.1.0")
                            .author("chordtoll <git@chordtoll.com>")
                            .about("Prints the contents of a file in a parcel")
                            .arg(Arg::with_name("parcel")
                                .value_name("PARCEL")
                                .help("The parcel to read")
                                .takes_value(true))
                            .arg(Arg::with_name("path")
                                .value_name("PATH")
                                .help("The file to print")
                                .takes_value(true))
                            .get_matches();

    let f = File::open(matches.value_of("parcel").unwrap()).unwrap();
    let mut reader = BufReader::new(f);

    let parcel : Parcel = Parcel::load(&mut reader);

    let ino = parcel.select(PathBuf::from(matches.value_of("path").unwrap())).unwrap();

    io::stdout().write(&parcel.read(&mut reader,ino,0,None).unwrap()).unwrap();

}