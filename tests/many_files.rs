use std::{fs::File, io::BufReader, path::PathBuf};

use pyxis_parcel::{FileAdd, Parcel};
use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use rand_pcg::Pcg64;

mod common;
use common::Fixture;

#[test]
fn many_file_roundtrip() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = Parcel::new();

    let mut rng = Pcg64::seed_from_u64(0);
    for i in 0..100 {
        let length = rng.gen_range(0..100);
        let contents = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(length)
            .collect::<Vec<u8>>();
        let ino = parcel
            .add_file(
                FileAdd::Bytes(contents.to_vec()),
                Default::default(),
                Default::default(),
            )
            .unwrap();
        assert_eq!(ino, i + 2);
    }

    parcel
        .store(File::create(PathBuf::from(&f)).unwrap())
        .unwrap();

    let mut f1 = File::open(PathBuf::from(&f)).unwrap();
    let mut br1 = BufReader::new(&mut f1);
    let mut f2 = File::open(PathBuf::from(&f)).unwrap();
    let mut br2 = BufReader::new(&mut f2);

    let parcel = Parcel::load(&mut br1).unwrap();
    let mut rng = Pcg64::seed_from_u64(0);
    for i in 0..100 {
        let length = rng.gen_range(0..100);
        let contents = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(length)
            .collect::<Vec<u8>>();
        let res = parcel.read(&mut br2, i + 2, 0, None).unwrap();
        assert_eq!(contents, res);
    }
}
