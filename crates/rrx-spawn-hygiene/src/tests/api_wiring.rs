//! Each control observes its own public entry point, under the existing worker owner.
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    os::fd::{AsRawFd, OwnedFd},
    path::Path,
    process::{Command, ExitStatus, Stdio},
};

#[derive(Debug)]
enum Observation {
    Pass,
    Inherited,
    #[cfg(target_os = "linux")]
    Unequal,
    Setup(String),
    NoResult(String),
}

struct Witness {
    fd: i32,
    device: u128,
    inode: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DescriptorState {
    Closed,
    Other,
    Inherited,
}

#[derive(Debug, PartialEq, Eq)]
struct DescriptorReport {
    group: u128,
    slots: BTreeMap<i32, DescriptorState>,
}

struct Fixture {
    // Keep every allocated slot owned and occupied throughout launch/observation.
    _fds: Vec<OwnedFd>,
    writers: Vec<Witness>,
}

fn nonnegative<T: TryInto<u128>>(value: T, field: &str) -> Result<u128, String> {
    value
        .try_into()
        .map_err(|_| format!("negative or out-of-range writer {field}"))
}

impl Fixture {
    fn new() -> Result<Self, String> {
        let mut fds = Vec::new();
        for _ in 0..3 {
            let (read, write) = rustix::pipe::pipe().map_err(|e| format!("pipe: {e}"))?;
            fds.extend([read, write]);
        }
        let duplicate = rustix::io::fcntl_dupfd_cloexec(&fds[1], 3)
            .map_err(|e| format!("safe duplicate: {e}"))?;
        fds.push(duplicate);
        let slots: BTreeSet<_> = fds.iter().map(AsRawFd::as_raw_fd).collect();
        if slots != (3..=9).collect() {
            return Err(format!("occupied fd slots prevent fixture: {slots:?}"));
        }
        for fd in &fds {
            rustix::io::fcntl_setfd(fd, rustix::io::FdFlags::CLOEXEC)
                .map_err(|e| format!("fixture CLOEXEC: {e}"))?;
        }
        let mut writers = Vec::new();
        for index in [1, 3, 5, 6] {
            let fd = &fds[index];
            rustix::io::fcntl_setfd(fd, rustix::io::FdFlags::empty())
                .map_err(|e| format!("writer flags: {e}"))?;
            if !rustix::io::fcntl_getfd(fd)
                .map_err(|e| format!("writer flag observation: {e}"))?
                .is_empty()
            {
                return Err("writer remains CLOEXEC".into());
            }
            // Darwin read and write ends have different inodes: observe the writer itself.
            let stat = rustix::fs::fstat(fd).map_err(|e| format!("writer fstat: {e}"))?;
            let witness = Witness {
                fd: fd.as_raw_fd(),
                device: nonnegative(stat.st_dev, "device")?,
                inode: nonnegative(stat.st_ino, "inode")?,
            };
            println!(
                "P4API WITNESS fd={} flags=0 device={} inode={}",
                witness.fd, witness.device, witness.inode
            );
            writers.push(witness);
        }
        let identities: BTreeSet<_> = writers[..3].iter().map(|w| (w.device, w.inode)).collect();
        if identities.len() != 3
            || writers.iter().any(|w| w.inode == 0)
            || (writers[0].device, writers[0].inode) != (writers[3].device, writers[3].inode)
        {
            return Err("writer identities are not independently observable".into());
        }
        Ok(Self { _fds: fds, writers })
    }

    fn command(&self, report: &Path) -> Command {
        let mut command = Command::new("/usr/bin/python3");
        command
            .args(["-I", "-S", "-c", include_str!("fd_probe.py")])
            .arg(report)
            .stdin(Stdio::null());
        for writer in &self.writers {
            command.args([
                writer.fd.to_string(),
                writer.device.to_string(),
                writer.inode.to_string(),
            ]);
        }
        command
    }
}

fn unsigned(value: &str) -> Option<u128> {
    (!value.is_empty() && value.len() <= 39 && value.bytes().all(|b| b.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn validated_report(bytes: &[u8], fixture: &Fixture, group: u128) -> Option<DescriptorReport> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return None;
    };
    if bytes.len() > 4096 || !text.ends_with('\n') || text.contains(['\r', '\0']) {
        return None;
    }
    let lines: Vec<_> = text.lines().collect();
    if lines.len() != 5 || lines[0] != format!("P4APIv1\t{group}") {
        return None;
    }
    let mut observed = BTreeMap::new();
    for line in &lines[1..] {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 {
            return None;
        }
        let slot = unsigned(fields[0]).and_then(|v| i32::try_from(v).ok())?;
        let error = unsigned(fields[2])?;
        if observed.contains_key(&slot) || !fixture.writers.iter().any(|w| w.fd == slot) {
            return None;
        }
        let state = match (fields[1], error) {
            ("closed", value) if value == libc::EBADF as u128 => DescriptorState::Closed,
            ("other", 0) => DescriptorState::Other,
            ("inherited", 0) => DescriptorState::Inherited,
            _ => return None,
        };
        observed.insert(slot, state);
    }
    if observed.keys().copied().collect::<BTreeSet<_>>()
        != fixture.writers.iter().map(|w| w.fd).collect()
    {
        return None;
    }
    Some(DescriptorReport {
        group,
        slots: observed,
    })
}

fn parse_report(bytes: &[u8], fixture: &Fixture, group: u128) -> Observation {
    let Some(report) = validated_report(bytes, fixture, group) else {
        return Observation::NoResult("missing, malformed or contradictory report".into());
    };
    if report
        .slots
        .values()
        .any(|state| *state == DescriptorState::Inherited)
    {
        Observation::Inherited
    } else {
        Observation::Pass
    }
}

fn read_report(status: ExitStatus, report: &Path) -> Result<Vec<u8>, Observation> {
    if !status.success() {
        return Err(Observation::NoResult(format!("probe exit: {status}")));
    }
    let mut bytes = Vec::new();
    let read = std::fs::File::open(report).and_then(|file| file.take(4097).read_to_end(&mut bytes));
    if let Err(error) = read {
        return Err(Observation::NoResult(format!("report read: {error}")));
    }
    Ok(bytes)
}

#[cfg(target_os = "macos")]
fn observe(status: ExitStatus, report: &Path, fixture: &Fixture) -> Observation {
    let bytes = match read_report(status, report) {
        Ok(bytes) => bytes,
        Err(observation) => return observation,
    };
    let group =
        u128::try_from(rustix::process::getpgrp().as_raw_nonzero().get()).unwrap_or_default();
    parse_report(&bytes, fixture, group)
}

fn launch(api: &str, fixture: &Fixture, report: &Path) -> Result<ExitStatus, String> {
    let command = fixture.command(report);
    // The six arms must each call their own public API. A shared prepare control
    // cannot detect losing prepare at just one of these entry points.
    match api {
        "SPAWN_STD" => {
            let mut child = crate::spawn_std(command).map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())
        }
        "OUTPUT_STD" => crate::output_std(command)
            .map(|o| o.status)
            .map_err(|e| e.to_string()),
        "STATUS_STD" => crate::status_std(command).map_err(|e| e.to_string()),
        "SPAWN_TOKIO" | "OUTPUT_TOKIO" | "STATUS_TOKIO" => {
            // Allocate the writer slots before Tokio opens its runtime fds.
            let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            let mut command = tokio::process::Command::from(command);
            command.kill_on_drop(true);
            runtime.block_on(async move {
                match api {
                    "SPAWN_TOKIO" => {
                        let mut child = crate::spawn_tokio(command).map_err(|e| e.to_string())?;
                        child.wait().await.map_err(|e| e.to_string())
                    }
                    "OUTPUT_TOKIO" => crate::output_tokio(command)
                        .await
                        .map(|o| o.status)
                        .map_err(|e| e.to_string()),
                    "STATUS_TOKIO" => crate::status_tokio(command)
                        .await
                        .map_err(|e| e.to_string()),
                    _ => unreachable!(),
                }
            })
        }
        _ => Err("unknown API".into()),
    }
}

#[cfg(target_os = "linux")]
#[allow(
    clippy::disallowed_methods,
    reason = "Linux-only control compares each consuming wrapper with its corresponding original operation"
)]
fn launch_original(api: &str, fixture: &Fixture, report: &Path) -> Result<ExitStatus, String> {
    let mut command = fixture.command(report);
    match api {
        "SPAWN_STD" => {
            let mut child = command.spawn().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())
        }
        "OUTPUT_STD" => command
            .output()
            .map(|o| o.status)
            .map_err(|e| e.to_string()),
        "STATUS_STD" => command.status().map_err(|e| e.to_string()),
        "SPAWN_TOKIO" | "OUTPUT_TOKIO" | "STATUS_TOKIO" => {
            let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            let mut command = tokio::process::Command::from(command);
            command.kill_on_drop(true);
            runtime.block_on(async move {
                match api {
                    "SPAWN_TOKIO" => {
                        let mut child = command.spawn().map_err(|e| e.to_string())?;
                        child.wait().await.map_err(|e| e.to_string())
                    }
                    "OUTPUT_TOKIO" => command
                        .output()
                        .await
                        .map(|o| o.status)
                        .map_err(|e| e.to_string()),
                    "STATUS_TOKIO" => command.status().await.map_err(|e| e.to_string()),
                    _ => unreachable!(),
                }
            })
        }
        _ => Err("unknown API".into()),
    }
}

#[cfg(target_os = "linux")]
fn delegation_equivalent(api: &str, fixture: &Fixture, directory: &Path) -> Observation {
    let original_path = directory.join("original-api-report.tsv");
    let wrapped_path = directory.join("api-report.tsv");
    let group =
        u128::try_from(rustix::process::getpgrp().as_raw_nonzero().get()).unwrap_or_default();
    let original = match launch_original(api, fixture, &original_path) {
        Ok(status) => read_report(status, &original_path),
        Err(error) => {
            return Observation::NoResult(format!("original launch/wait observation: {error}"));
        }
    };
    let original = match original {
        Ok(bytes) => validated_report(&bytes, fixture, group),
        Err(observation) => return observation,
    };
    let Some(original) = original else {
        return Observation::NoResult("invalid original report".into());
    };
    // All four independent cleared writer witnesses must be observable in the
    // original child; equality of two unobservable reports cannot pass.
    if original.slots.len() != 4
        || original
            .slots
            .values()
            .any(|state| *state != DescriptorState::Inherited)
    {
        return Observation::NoResult(
            "original child did not inherit all four known writers".into(),
        );
    }
    let wrapped = match launch(api, fixture, &wrapped_path) {
        Ok(status) => read_report(status, &wrapped_path),
        Err(error) => {
            return Observation::NoResult(format!("wrapped launch/wait observation: {error}"));
        }
    };
    let wrapped = match wrapped {
        Ok(bytes) => validated_report(&bytes, fixture, group),
        Err(observation) => return observation,
    };
    let Some(wrapped) = wrapped else {
        return Observation::NoResult("invalid wrapped report".into());
    };
    println!("P4API LINUX original={original:?} wrapped={wrapped:?}");
    if original == wrapped {
        Observation::Pass
    } else {
        Observation::Unequal
    }
}

pub(super) fn run_worker(api: &str, directory: &Path) {
    #[cfg(target_os = "macos")]
    println!("P4API EXPECTED pass=validated report with no inherited writer");
    #[cfg(target_os = "linux")]
    println!(
        "P4API EXPECTED pass=equal validated reports with all four original writers inherited"
    );
    let observation = match Fixture::new() {
        Err(error) => Observation::Setup(error),
        Ok(fixture) => {
            #[cfg(target_os = "macos")]
            {
                let report = directory.join("api-report.tsv");
                match launch(api, &fixture, &report) {
                    Ok(status) => observe(status, &report, &fixture),
                    Err(error) => {
                        Observation::NoResult(format!("launch/wait observation: {error}"))
                    }
                }
            }
            #[cfg(target_os = "linux")]
            {
                delegation_equivalent(api, &fixture, directory)
            }
        }
    };
    // Never send observation failures through controls' String -> RESULT fail branch.
    match observation {
        Observation::Pass => println!("P4C RESULT pass"),
        Observation::Inherited => println!("P4C RESULT fail:public API inherited writer"),
        #[cfg(target_os = "linux")]
        Observation::Unequal => {
            println!("P4C RESULT fail:Linux public API changed original descriptor report")
        }
        Observation::Setup(error) => println!("P4C SETUP fail:{error}"),
        Observation::NoResult(error) => println!("P4C NO_RESULT {error}"),
    }
}

#[test]
fn api_spawn_std() {
    super::controls::run("API_SPAWN_STD");
}
#[test]
fn api_output_std() {
    super::controls::run("API_OUTPUT_STD");
}
#[test]
fn api_status_std() {
    super::controls::run("API_STATUS_STD");
}
#[test]
fn api_spawn_tokio() {
    super::controls::run("API_SPAWN_TOKIO");
}
#[test]
fn api_output_tokio() {
    super::controls::run("API_OUTPUT_TOKIO");
}
#[test]
fn api_status_tokio() {
    super::controls::run("API_STATUS_TOKIO");
}

#[test]
fn report_observation_errors_are_no_result() {
    assert!(nonnegative(-1_i32, "device").is_err());
    assert_eq!(nonnegative(0_i32, "device"), Ok(0));
    assert_eq!(nonnegative(u64::MAX, "inode"), Ok(u128::from(u64::MAX)));
    let fixture = Fixture {
        _fds: vec![],
        writers: [4, 6, 8, 9]
            .into_iter()
            .map(|fd| Witness {
                fd,
                device: 0,
                inode: 1,
            })
            .collect(),
    };
    let normal = format!(
        "P4APIv1\t42\n4\tclosed\t{}\n6\tother\t0\n8\tclosed\t{}\n9\tother\t0\n",
        libc::EBADF,
        libc::EBADF
    );
    assert!(matches!(
        parse_report(normal.as_bytes(), &fixture, 42),
        Observation::Pass
    ));
    let inherited = normal.replace("6\tother\t0", "6\tinherited\t0");
    assert!(matches!(
        parse_report(inherited.as_bytes(), &fixture, 42),
        Observation::Inherited
    ));
    for invalid in [
        Vec::new(),
        vec![b'x'; 4097],
        vec![0xff, b'\n'],
        normal.replace("42", "43").into_bytes(),
        normal.replace("9\tother", "6\tother").into_bytes(),
        normal.replace("9\tother", "7\tother").into_bytes(),
        normal.replace("other", "unknown").into_bytes(),
        normal.replace("other\t0", "other\t1").into_bytes(),
        normal.replace("other\t0", "error\t5").into_bytes(),
        normal.replace("9\tother\t0\n", "").into_bytes(),
        format!("{normal}extra\n").into_bytes(),
        normal.trim_end().as_bytes().to_vec(),
        normal.replace('\n', "\r\n").into_bytes(),
        normal.replace("other", "other\0").into_bytes(),
    ] {
        assert!(matches!(
            parse_report(&invalid, &fixture, 42),
            Observation::NoResult(_)
        ));
    }
}
