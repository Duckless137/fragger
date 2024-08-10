use fragger::*;

#[test]
fn unit_converter() {
    let res = file_service::into_u8(50417721);
    assert_eq!(res, [57, 80, 1, 3]);

    let res = file_service::into_u8(10);
    assert_eq!(res, [10, 0, 0, 0]);

    let res = file_service::into_u32([0, 0, 0, 10]);
    assert_eq!(res, 167772160);

    let res = file_service::into_u32([255, 0, 2, 1]);
    assert_eq!(res, 16908543);

    let res = file_service::into_u32([25, 0, 0, 0]);
    assert_eq!(res, 25);
}

#[test]
fn ordering() {
    use std::cmp::Ordering;
    let cmp = "abcd".cmp("bcde");
    assert_eq!(cmp, Ordering::Less);

    let cmp = "ZDE".cmp("zde");
    assert_eq!(cmp, Ordering::Less);

    let cmp = "abcde".cmp("abd");
    assert_eq!(cmp, Ordering::Less);
}