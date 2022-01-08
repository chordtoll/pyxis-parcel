use std::{fs::File, io, io::Write, path::PathBuf};

use clap::{App, Arg};
use pyxis_parcel::{ParcelHandle, ReaderWriter};

fn main() {
    let matches = App::new("Parcel-Cat")
        .version("0.1.0")
        .author("chordtoll <git@chordtoll.com>")
        .about("Prints the contents of a file in a parcel")
        .arg(
            Arg::new("parcel")
                .value_name("PARCEL")
                .help("The parcel to read")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .help("The file to print")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let f = File::open(matches.value_of("parcel").unwrap()).unwrap();
    let readerwriter = ReaderWriter::new(f);

    let mut parcel: ParcelHandle = ParcelHandle::load(Box::new(readerwriter)).unwrap();

    let ino = parcel
        .select(PathBuf::from(matches.value_of("path").unwrap()))
        .unwrap();

    io::stdout()
        .write_all(&parcel.read(ino, 0, None).unwrap())
        .unwrap();
}
