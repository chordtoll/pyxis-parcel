use pyxis_parcel::{FileAdd, ParcelHandle};
use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use rand_pcg::Pcg64;

mod common;
use common::Fixture;

#[test]
fn many_file_roundtrip() {
    let f = Fixture::blank("test.parcel");
    let mut parcel = ParcelHandle::new();
    parcel.set_file(f.make_rw());

    let mut rng = Pcg64::seed_from_u64(0);
    for i in 0..100 {
        let length = rng.gen_range(0..100);
        let contents = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(length)
            .collect::<Vec<u8>>();
        let ino = parcel
            .add_file(
                FileAdd::Bytes(contents),
                Default::default(),
                Default::default(),
            )
            .unwrap();
        assert_eq!(ino, i + 2);
    }

    parcel.store().unwrap();

    let mut parcel = ParcelHandle::load(f.make_rw()).unwrap();
    let mut rng = Pcg64::seed_from_u64(0);
    for i in 0..100 {
        let length = rng.gen_range(0..100);
        let contents = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(length)
            .collect::<Vec<u8>>();
        let res = parcel.read(i + 2, 0, None).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&contents),
            String::from_utf8_lossy(&res)
        );
    }
}
