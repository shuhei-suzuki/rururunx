#[cfg(any(target_os = "macos", target_os = "linux"))]
mod api_wiring;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod controls;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod eof_observation;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod owner;

#[cfg(target_os = "macos")]
#[test]
fn listfds_return_bytes_are_fail_closed() {
    controls::run("LISTFDS");
}

#[cfg(target_os = "macos")]
fn check_listfds() -> Result<(), String> {
    use super::platform::classify_listfds;
    for (ret, bytes, errno, expected) in [
        (-1, 32, libc::EPERM, Err(libc::EPERM)),
        (0, 32, 0, Err(libc::EIO)),
        (9, 32, 0, Err(libc::EIO)),
        (40, 32, 0, Err(libc::EIO)),
        (32, 32, 0, Err(libc::ENOBUFS)),
        (16, 32, 0, Ok(2)),
    ] {
        let actual = classify_listfds(ret, bytes, errno);
        if actual != expected {
            return Err(format!(
                "listfds ret={ret} bytes={bytes} errno={errno}: expected {expected:?}, got {actual:?}"
            ));
        }
    }
    Ok(())
}
