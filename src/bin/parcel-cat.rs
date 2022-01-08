use std::{
    fs::File,
    io,
    io::{BufReader, Write},
    path::PathBuf,
};

use clap::{App, Arg};
use pyxis_parcel::Parcel;

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
    let mut reader = BufReader::new(f);

    let parcel: Parcel = Parcel::load(&mut reader).unwrap();

    let ino = parcel
        .select(PathBuf::from(matches.value_of("path").unwrap()))
        .unwrap();

    io::stdout()
        .write_all(&parcel.read(&mut reader, ino, 0, None).unwrap())
        .unwrap();
}
