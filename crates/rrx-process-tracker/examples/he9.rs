#[cfg(target_os = "macos")]
fn main() -> std::io::Result<()> {
    use rrx_process_tracker::{Cookie, Limits, discover_launched, launch_mark};
    use std::time::Instant;
    let cookie = Cookie::new(&format!("he9-measurement-{}", std::process::id()))?;
    for seconds in [10, 300, 1800, 3600] {
        let mut launch = launch_mark()?;
        // F = launch.wall - 60 seconds; measure the requested age of F.
        if seconds >= 60 {
            launch.wall.0 -= seconds - 60;
        } else {
            launch.wall.0 += 60 - seconds;
        }
        let mut limited = 0;
        let mut unconfirmed = 0;
        let mut ancestors = 0;
        let start = Instant::now();
        for _ in 0..20 {
            let d = discover_launched(&cookie, Limits::default(), &launch)?;
            limited += usize::from(d.coverage.limited());
            unconfirmed += d.coverage.ancestry_unconfirmed;
            ancestors += d.coverage.ancestors_read;
        }
        println!(
            "F_age_seconds={seconds} scans=20 limited={limited} ancestry_unconfirmed={unconfirmed} ancestors_read={ancestors} elapsed_us={}",
            start.elapsed().as_micros()
        );
    }
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn main() {
    println!("HE9 measures macOS discovery; run this example on the owner's Mac.");
}
