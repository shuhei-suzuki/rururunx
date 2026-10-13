#[cfg(unix)]
mod controls;
#[cfg(unix)]
mod owner;

#[test]
fn listfds_return_bytes_are_fail_closed() {
    use super::platform::classify_listfds;
    for (ret, bytes, errno, expected) in [
        (-1, 32, libc::EPERM, Err(libc::EPERM)),
        (0, 32, 0, Err(libc::EIO)),
        (9, 32, 0, Err(libc::EIO)),
        (40, 32, 0, Err(libc::EIO)),
        (32, 32, 0, Err(libc::ENOBUFS)),
        (16, 32, 0, Ok(2)),
    ] {
        assert_eq!(classify_listfds(ret, bytes, errno), expected);
    }
}
