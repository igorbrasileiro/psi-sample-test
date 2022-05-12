use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

pub fn read_lines<P>(filename: P) -> Lines<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).unwrap();
    let file_lines = BufReader::new(file).lines();

    return file_lines;
}

pub fn check_file_availability(filename: &str) -> &Path {
    let filename_path = Path::new(filename);

    if !filename_path.exists() {
        return filename_path;
    }

    // TODO: create a file name with "filename (x).ext"
    // let mut new_filename = &mut filename.clone().replace(".", concat!("()."));

    // while !Path::new(&new_filename.clone()).exists() {
    // new_filename = &mut new_filename.replace(".", concat!("()."));
    // }

    return filename_path;
}
