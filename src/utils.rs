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

pub fn check_file_availability(filename: &str) -> String {
    let filename_path = Path::new(filename);

    if !filename_path.exists() {
        return filename.to_string();
    }

    // TODO: create a file name with "filename (x).ext"
    let filename_without_extension = filename_path.file_stem().unwrap().to_str().unwrap();
    let file_extension = filename_path.extension().unwrap().to_str().unwrap();

    let mut index = 1;
    let mut new_filename = format!(
        "{filename} ({ind}).{ext}",
        filename = filename_without_extension,
        ind = index,
        ext = file_extension,
    );

    while Path::new(&*new_filename).exists() {
        index = index + 1;
        new_filename = format!(
            "{filename} ({ind}).{ext}",
            filename = filename_without_extension,
            ind = index,
            ext = file_extension,
        );
    }

    return new_filename;
}
