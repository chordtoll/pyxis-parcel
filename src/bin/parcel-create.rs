extern crate clap;

use clap::{Arg, App};

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

    println!("Generating output file: {}", matches.value_of("output").unwrap());
    for input in matches.values_of("input").unwrap() {
        println!("Using input path: {}", input);
    }
}