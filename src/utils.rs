use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

pub fn read_lines<P>(filename: P) -> Lines<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).unwrap();
    BufReader::new(file).lines()
}

pub fn check_file_availability(filename: &str) -> String {
    let filename_path = Path::new(filename);

    if !filename_path.exists() {
        return filename.to_string();
    }

    let filename_without_extension = filename_path.file_stem().unwrap().to_str().unwrap();
    let file_extension = filename_path.extension().unwrap().to_str().unwrap();

    let mut index = 1;
    let mut new_filename = format!(
        "./{filename} ({ind}).{ext}",
        filename = filename_without_extension,
        ind = index,
        ext = file_extension,
    );

    while Path::new(&*new_filename).exists() {
        index += 1;
        new_filename = format!(
            "./{filename} ({ind}).{ext}",
            filename = filename_without_extension,
            ind = index,
            ext = file_extension,
        );
    }

    new_filename
}

#[cfg(test)]
mod utils_tests {
    #[test]
    fn check_file_availability() {
        // remove previous files
        let _x = std::fs::remove_file("./test.txt");
        let _x = std::fs::remove_file("./test (1).txt");

        assert_eq!("./test.txt", super::check_file_availability("./test.txt"));

        // Create test.txt file
        let _x = std::fs::File::create("./test.txt");
        assert_eq!(
            "./test (1).txt",
            super::check_file_availability("./test.txt")
        );

        // Create test (1).txt file
        let _x = std::fs::File::create("./test (1).txt");
        assert_eq!(
            "./test (2).txt",
            super::check_file_availability("./test.txt")
        );

        // remove created files during tests
        let _x = std::fs::remove_file("./test.txt");
        let _x = std::fs::remove_file("./test (1).txt");
    }
}
