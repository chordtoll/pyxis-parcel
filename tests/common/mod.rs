use std::{env, fs, path::PathBuf};

use pretty_assertions::assert_eq;
use tempfile::TempDir;
pub struct Fixture {
    path:     PathBuf,
    _source:   PathBuf,
    _tempdir: TempDir,
}

impl Fixture {
    pub fn blank(fixture_filename: &str) -> Self {
        // First, figure out the right file in `tests/fixtures/`:
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let mut source = PathBuf::from(root_dir);
        source.push("tests/fixtures");
        source.push(&fixture_filename);

        // The "real" path of the file is going to be under a temporary directory:
        let tempdir = tempfile::tempdir().unwrap();
        let mut path = PathBuf::from(&tempdir.path());
        path.push(&fixture_filename);

        Fixture {
            _tempdir: tempdir,
            _source: source,
            path,
        }
    }

    pub fn compare(&self, expected: &str) {
        let _ = fs::copy(PathBuf::from(self),"found.parcel");
        let ex_b = fs::read(String::from("tests/data/expected/") + expected).unwrap();
        let fd_b = fs::read(PathBuf::from(self)).unwrap();
        let expected = String::from_utf8(ex_b.clone())
            .unwrap()
            .split('\n')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let found = String::from_utf8(fd_b.clone())
            .unwrap()
            .split('\n')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let expected = if let Some(stop) = expected.iter().position(|x| x == "...") {
            &expected[..=stop]
        } else {
            &expected
        };
        let found = if let Some(stop) = found.iter().position(|x| x == "...") {
            &found[..=stop]
        } else {
            &found
        };

        assert_eq!(expected, found);

        let ex_d_idx = twoway::find_bytes(&ex_b,b"\n...\n").unwrap()+5;
        let fd_d_idx = twoway::find_bytes(&fd_b,b"\n...\n").unwrap()+5;

        assert_eq!(ex_b[ex_d_idx..],fd_b[fd_d_idx..]);
    }
}

impl From<&Fixture> for PathBuf {
    fn from(f: &Fixture) -> Self {
        f.path.to_owned()
    }
}
