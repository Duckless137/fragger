use fragger::*;

const CHUNK_SIZE: u32 = 100 * 1024 * 1024;

#[test]
fn weed_errors() {
    file_service::split_file(std::path::PathBuf::from("./tests/files/small.txt"), 100 * 1024);
    file_service::split_file(std::path::PathBuf::from("./tests/files/large.txt"), CHUNK_SIZE);
}

#[test] 
fn split_and_reassemble() {
    file_service::split_file(std::path::PathBuf::from("./tests/files/small.txt"), 90 * 999);
    file_service::combine_files(std::path::PathBuf::from("./tests/files/small"));

    file_service::split_file(std::path::PathBuf::from("./tests/files/large.txt"), 8 * 1024 * 1024);
    file_service::combine_files(std::path::PathBuf::from("./tests/files/large"));
}

#[test] 
fn split_large_file() {

}

#[test] 
fn reassemble_large_file() {
    file_service::combine_files(std::path::PathBuf::from("./tests/files/large"))
}
