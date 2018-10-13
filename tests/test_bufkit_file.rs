extern crate sounding_bufkit;

use sounding_bufkit::BufkitFile;
use std::path::Path;

const EXAMPLE_DIR: &str = "example_data";

#[test]
fn test_bufkit_file() {
    let example_dir = Path::new(EXAMPLE_DIR);
    assert!(example_dir.is_dir(), "Example data directory not found.");

    let files: Vec<_> = example_dir
        .read_dir()
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.to_str().unwrap().ends_with(".buf") && path.is_file())
        .collect();

    for file in files {
        println!("Testing: {:?}", file);
        let example_file = BufkitFile::load(&file).expect("Error loading data.");
        let data = example_file.data().unwrap();
        if file.to_str().unwrap().contains("2017040100Z_gfs3_kmso") {
            // This file has been seeded with invalid data.
            continue;
        }
        if file.to_str().unwrap().contains("nam") {
            assert_eq!(85, data.into_iter().count());
        } else if file.to_str().unwrap().contains("gfs") {
            assert_eq!(61, data.into_iter().count());
        }
    }
}

#[test]
fn test_bufkit_file_validation() {
    let example_dir = Path::new(EXAMPLE_DIR);
    assert!(example_dir.is_dir(), "Example data directory not found.");

    validate_dir(example_dir);
}

fn validate_dir(dir: &Path) {
    assert!(dir.is_dir(), "Example data directory not found.");
    let files: Vec<_> = dir
        .read_dir()
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.is_file() && path.to_str().unwrap().ends_with(".buf"))
        .collect();

    let sub_dirs: Vec<_> = dir
        .read_dir()
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();

    for file in files {
        let example_file = BufkitFile::load(&file).expect("Error loading data.");
        if file.to_str().unwrap().contains("2017040100Z_gfs_kmso") {
            assert!(
                example_file.validate_file_format().is_err(),
                format!("Erroneously passed validation: {:?}", file)
            );
        } else {
            assert!(
                example_file.validate_file_format().is_ok(),
                format!("Failded validation: {:?}", file)
            );
        }
    }

    for dir in sub_dirs {
        validate_dir(&dir);
    }
}
