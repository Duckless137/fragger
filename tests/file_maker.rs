use std::io::Write;

#[test]
fn make_null() {
    let buf: [u8; 0] = [];
    let mut file = std::fs::File::create("./tests/files/null.txt").unwrap();
    file.write_all(&buf).unwrap();
}

#[test] 
fn make_small() {
    let large_buf: [u8; 1024] = [0; 1024];
    std::fs::File::create("./tests/files/small.txt").unwrap();
    let mut file = std::fs::OpenOptions::new().append(true).open("./tests/files/small.txt").unwrap();
    for _ in 0..1024 {
        file.write_all(&large_buf).unwrap();
    }
}


#[test] 
fn make_large() {
    let large_buf: [u8; 1024] = [0; 1024];
    std::fs::File::create("./tests/files/large.txt").unwrap();
    let mut file = std::fs::OpenOptions::new().append(true).open("./tests/files/large.txt").unwrap();
    for _ in 0..(1024 * 1024) {
        file.write_all(&large_buf).unwrap();
    }
}