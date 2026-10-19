use super::*;
use std::{collections::VecDeque, path::PathBuf};

enum Op {
    List(Vec<i32>),
    Id(u32, Option<Observation>),
    Env(u32, Option<Environment>),
    Path(u32, Option<&'static str>),
    Query(u32, Box<Query>),
}
struct Fake {
    ops: VecDeque<Op>,
    boot: Option<String>,
    wall: (u64, u64),
    calls: Vec<String>,
    expire: bool,
}
impl Fake {
    fn new(ops: Vec<Op>) -> Self {
        Self {
            ops: ops.into(),
            boot: Some("A".into()),
            wall: (2000, 0),
            calls: vec![],
            expire: false,
        }
    }
    fn run(&mut self, launch: Option<&Launch>) -> Discovery {
        let result = match launch {
            Some(launch) => scan_launched(&cookie(), bounds(), 42, self, Some(launch)),
            None => scan(&cookie(), bounds(), 42, self),
        }
        .unwrap();
        assert!(
            self.ops.is_empty(),
            "{} remaining operations",
            self.ops.len()
        );
        result
    }
}
impl Source for Fake {
    fn list(&mut self, _: u32, pids: &mut [i32]) -> i32 {
        self.calls.push("list".into());
        let Some(Op::List(values)) = self.ops.pop_front() else {
            panic!("unexpected list")
        };
        let len = values.len();
        pids[..len].copy_from_slice(&values);
        (len * 4) as i32
    }
    fn identity(&mut self, pid: u32) -> io::Result<Observation> {
        self.calls.push(format!("id:{pid}"));
        let Some(Op::Id(expected, value)) = self.ops.pop_front() else {
            panic!("unexpected id {pid}")
        };
        assert_eq!(pid, expected);
        value.ok_or_else(|| io::Error::other("id unavailable"))
    }
    fn environment(&mut self, pid: u32, _: &Cookie, _: usize) -> io::Result<Environment> {
        self.calls.push(format!("env:{pid}"));
        let Some(Op::Env(expected, value)) = self.ops.pop_front() else {
            panic!("unexpected env {pid}")
        };
        assert_eq!(pid, expected);
        value.ok_or_else(|| io::Error::other("env unavailable"))
    }
    fn path(&mut self, pid: u32) -> io::Result<PathBuf> {
        self.calls.push(format!("path:{pid}"));
        let Some(Op::Path(expected, value)) = self.ops.pop_front() else {
            panic!("unexpected path {pid}")
        };
        assert_eq!(pid, expected);
        value
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::other("path unavailable"))
    }
    fn query(&mut self, pid: u32) -> Query {
        self.calls.push(format!("query:{pid}"));
        let Some(Op::Query(expected, value)) = self.ops.pop_front() else {
            panic!("unexpected query {pid}")
        };
        assert_eq!(pid, expected);
        *value
    }
    fn boot(&mut self) -> Option<String> {
        self.calls.push("boot".into());
        self.boot.clone()
    }
    fn wall(&mut self) -> io::Result<(u64, u64)> {
        self.calls.push("wall".into());
        Ok(self.wall)
    }
    fn elapsed(&mut self) -> Duration {
        if self.expire && self.calls.last().is_some_and(|c| c.starts_with("query:")) {
            bounds().time
        } else {
            Duration::ZERO
        }
    }
}
fn bounds() -> Limits {
    Limits {
        processes: 64,
        ..Limits::default()
    }
}
fn cookie() -> Cookie {
    Cookie::new("d20-fake-cookie-1234").unwrap()
}
fn launch() -> Launch {
    Launch {
        wall: (1000, 0),
        boot: Some("A".into()),
        leader_birth: None,
    }
}
fn obs(pid: u32, birth: (u64, u64), ppid: u32) -> Observation {
    Observation {
        identity: Identity {
            pid,
            uid: 42,
            birth,
        },
        ppid,
        flags: 0,
    }
}
fn x(ppid: u32) -> Observation {
    obs(10, (1000, 0), ppid)
}
fn id(o: Observation) -> Op {
    Op::Id(o.pid, Some(o))
}
fn list(pids: &[i32]) -> Op {
    Op::List(pids.into())
}
fn env(pid: u32, e: Environment) -> Op {
    Op::Env(pid, Some(e))
}
fn q(pid: u32, birth: u64, ppid: u32) -> Query {
    // SAFETY: KinfoProc consists exclusively of integer and byte fields.
    let mut info: KinfoProc = unsafe { std::mem::zeroed() };
    info.pid = pid as i32;
    info.ppid = ppid as i32;
    info.stat = SSLEEP;
    info.start.tv_sec = birth as libc::time_t;
    Query {
        status: 0,
        len: size_of::<KinfoProc>(),
        info,
    }
}
fn query(pid: u32, value: Query) -> Op {
    Op::Query(pid, Box::new(value))
}
fn absent(o: Observation) -> Vec<Op> {
    located(
        o,
        Some("/usr/bin/perl"),
        Some(Environment::Absent),
        Some("/usr/bin/perl"),
    )
}
fn located(
    o: Observation,
    l1: Option<&'static str>,
    env2: Option<Environment>,
    l2: Option<&'static str>,
) -> Vec<Op> {
    vec![
        id(o),
        env(o.pid, Environment::Absent),
        Op::Path(o.pid, l1),
        Op::Env(o.pid, env2),
        Op::Path(o.pid, l2),
        id(o),
    ]
}
fn simple(o: Observation, e: Environment) -> Vec<Op> {
    vec![id(o), env(o.pid, e), id(o)]
}
fn run(ops: Vec<Op>) -> Discovery {
    Fake::new(ops).run(Some(&launch()))
}
fn uncertain(d: &Discovery, count: usize) {
    assert_eq!(d.coverage.ancestry_unconfirmed, count);
    assert!(d.coverage.limited());
    assert_eq!(d.coverage.relistings, 0);
    assert_eq!(d.unconfirmed.len(), count.min(16));
}

/// Pre-boundary absent observations are reusable without rereading the environment.
#[test]
fn ea01_old_record() {
    let old = obs(10, (939, 999999), 1);
    let mut ops = vec![list(&[10, 11])];
    ops.extend(simple(old, Environment::Absent));
    ops.extend([
        Op::Id(11, None),
        query(
            11,
            Query {
                len: 0,
                ..q(11, 0, 0)
            },
        ),
        list(&[10]),
        id(old),
    ]);
    let d = run(ops);
    assert_eq!(d.coverage.environment_absent, 1);
    assert!(!d.coverage.limited());
}
/// A recent orphan with absent environment refuses death proof without relisting.
#[test]
fn ea02_orphan() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(1)));
    let d = run(ops);
    uncertain(&d, 1);
    assert_eq!(d.unconfirmed[0], x(1).identity);
}
/// A live pre-boundary parent excludes an unrelated recent process.
#[test]
fn ea03_old_parent() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.extend([query(20, q(20, 900, 1)), id(x(20))]);
    let d = run(ops);
    assert!(!d.coverage.limited());
    assert_eq!(d.coverage.ancestors_read, 1);
}
fn two_parents() -> Vec<Op> {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.extend([
        query(20, q(20, 1000, 30)),
        id(x(20)),
        query(30, q(30, 900, 1)),
        query(20, q(20, 1000, 30)),
    ]);
    ops
}
/// The walk can exclude through multiple verified living ancestors.
#[test]
fn ea04_old_grandparent() {
    let d = run(two_parents());
    assert!(!d.coverage.limited());
    assert_eq!(d.coverage.ancestors_read, 2);
}
/// A chain of recent ancestors ending at launchd stays unconfirmed.
#[test]
fn ea05_recent_chain() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.extend([query(20, q(20, 1000, 1)), id(x(20))]);
    uncertain(&run(ops), 1);
}
/// The daemon and already visited PIDs never qualify as old ancestors.
#[test]
fn ea06_self() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(std::process::id())));
    let d = run(ops);
    uncertain(&d, 1);
    assert_eq!(d.coverage.ancestors_read, 0);
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.extend([query(20, q(20, 1000, std::process::id())), id(x(20))]);
    uncertain(&run(ops), 1);
}
/// Malformed, nonliving or traced ancestor observations cannot exclude a process.
#[test]
fn ea07_invalid_parent() {
    for kind in 0..11 {
        let mut parent = q(20, 900, 1);
        match kind {
            0 => parent.status = -1,
            1 => parent.len = 0,
            2 => parent.len -= 1,
            3 => parent.len += 1,
            4 => parent.info.pid = 21,
            5 => parent.info.stat = SZOMB,
            6 => parent.info.stat = 1,
            7 => parent.info.stat = 99,
            8 => parent.info.start.tv_sec = 0,
            9 => parent.info.flag = P_TRACED,
            10 => parent.info.stat = 0,
            _ => unreachable!(),
        }
        let mut ops = vec![list(&[10])];
        ops.extend(absent(x(20)));
        ops.push(query(20, parent));
        uncertain(&run(ops), 1);
    }
}
/// Each parent read must be followed by a stable, live and untraced child read.
#[test]
fn ea08_child_reread() {
    for kind in 0..4 {
        let mut child = x(20);
        match kind {
            0 => child.ppid = 1,
            1 => child.identity.birth.0 += 1,
            2 => {}
            3 => child.flags = PROC_FLAG_TRACED,
            _ => unreachable!(),
        }
        let mut ops = vec![list(&[10])];
        ops.extend(absent(x(20)));
        ops.extend([
            query(20, q(20, 1000, 30)),
            Op::Id(10, (kind != 2).then_some(child)),
        ]);
        let d = run(ops);
        uncertain(&d, 1);
        assert_eq!(d.coverage.ancestors_read, 1);
    }
    for kind in 0..5 {
        let mut child = q(20, 1000, 30);
        match kind {
            0 => child.info.ppid = 1,
            1 => child.info.start.tv_sec += 1,
            2 => child.len = 0,
            3 => child.info.stat = SZOMB,
            4 => child.info.flag = P_TRACED,
            _ => unreachable!(),
        }
        let mut ops = two_parents();
        ops.pop();
        ops.push(query(20, child));
        uncertain(&run(ops), 1);
    }
}
/// Cycles and overlong chains refuse while an old ancestor at the limit excludes.
#[test]
fn ea09_depth_and_cycle() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.extend([query(20, q(20, 1000, 10)), id(x(20))]);
    uncertain(&run(ops), 1);
    for old in [false, true] {
        let mut ops = vec![list(&[10])];
        ops.extend(absent(x(20)));
        for depth in 0..32 {
            let pid = 20 + depth;
            ops.push(query(
                pid,
                q(pid, if old && depth == 31 { 900 } else { 1000 }, pid + 1),
            ));
            ops.push(if depth == 0 {
                id(x(20))
            } else {
                query(pid - 1, q(pid - 1, 1000, pid))
            });
        }
        let d = run(ops);
        assert_eq!(d.coverage.ancestors_read, 32);
        assert_eq!(d.coverage.limited(), !old);
    }
}
/// The sole boundary uses the earlier launch or leader time minus sixty seconds.
#[test]
fn ea10_boundary() {
    for leader in [None, Some((970, 0))] {
        let mut launch = launch();
        launch.leader_birth = leader;
        let f = if leader.is_some() { 910 } else { 940 };
        for old in [false, true] {
            let o = obs(10, if old { (f - 1, 999999) } else { (f, 0) }, 1);
            let mut ops = vec![list(&[10])];
            ops.extend(if old {
                simple(o, Environment::Absent)
            } else {
                absent(o)
            });
            let d = Fake::new(ops).run(Some(&launch));
            assert_eq!(d.coverage.limited(), !old);
        }
    }
}
/// Only two readable unequal boots bypass location and lineage checks.
#[test]
fn ea11_boot() {
    for kind in 0..3 {
        let mut launch = launch();
        let mut ops = vec![list(&[10])];
        ops.extend(if kind == 0 {
            simple(x(1), Environment::Absent)
        } else {
            absent(x(1))
        });
        let mut f = Fake::new(ops);
        if kind == 0 {
            f.boot = Some("B".into());
        } else if kind == 1 {
            f.boot = None;
        } else {
            launch.boot = None;
        }
        let d = f.run(Some(&launch));
        assert_eq!(d.coverage.other_boot, kind == 0);
        assert_eq!(d.coverage.limited(), kind != 0);
        if kind == 0 {
            assert!(!f.calls.contains(&"wall".into()));
        }
    }
}
/// Boot comparison precedes the clock check even when no processes are listed.
#[test]
fn ea12_clock() {
    for kind in 0..4 {
        let mut launch = launch();
        let mut f = Fake::new(vec![list(&[0])]);
        f.wall = (900, 0);
        match kind {
            1 => f.boot = None,
            2 => launch.boot = None,
            3 => f.boot = Some("B".into()),
            _ => {}
        }
        let d = f.run(Some(&launch));
        assert_eq!(d.coverage.clock_behind, kind != 3);
        assert_eq!(d.coverage.limited(), kind != 3);
        assert_eq!(f.calls[0], "boot");
    }
}
/// Legacy discovery counts absent environments without launch-specific reads.
#[test]
fn ea13_legacy() {
    let mut ops = vec![list(&[10])];
    ops.extend(simple(x(1), Environment::Absent));
    let d = Fake::new(ops).run(None);
    assert_eq!(d.coverage.environment_absent, 1);
    assert!(!d.coverage.limited());
    assert_eq!(d.coverage.ancestors_read, 0);
}
/// An observable nonmatching environment needs no location or lineage checks.
#[test]
fn ea14_nonmatch() {
    let mut ops = vec![list(&[10])];
    ops.extend(simple(x(1), Environment::NonMatch));
    assert!(!run(ops).coverage.limited());
}
/// Ancestry reads consume the same scan deadline.
#[test]
fn ea15_time() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.push(query(20, q(20, 900, 1)));
    let mut f = Fake::new(ops);
    f.expire = true;
    let d = f.run(Some(&launch()));
    assert!(d.coverage.time_exceeded && d.coverage.limited());
}
/// Only the birth exclusion is reusable across listings.
#[test]
fn ea16_records() {
    let old = obs(10, (900, 0), 1);
    let lineage = obs(11, (1000, 0), 20);
    let elsewhere = obs(12, (1000, 0), 1);
    let mut ops = vec![list(&[10, 11, 12, 13])];
    ops.extend(simple(old, Environment::Absent));
    for round in 0..2 {
        if round == 1 {
            ops.extend([list(&[10, 11, 12]), id(old)]);
        }
        ops.extend(absent(lineage));
        ops.extend([query(20, q(20, 900, 1)), id(lineage)]);
        ops.extend(located(
            elsewhere,
            Some("/System/Library/x"),
            Some(Environment::Absent),
            Some("/System/Library/x"),
        ));
        if round == 0 {
            ops.extend([
                Op::Id(13, None),
                query(
                    13,
                    Query {
                        len: 0,
                        ..q(13, 0, 0)
                    },
                ),
            ]);
        }
    }
    let d = run(ops);
    assert!(!d.coverage.limited());
    assert_eq!(d.coverage.ancestors_read, 2);
    assert_eq!(d.coverage.environment_absent_elsewhere, 2);
    assert_eq!(d.coverage.environment_absent, 5);
}
/// Only ancestry uncertainty and clock rollback add new limited conditions.
#[test]
fn ea17_coverage() {
    for c in [
        Coverage {
            environment_absent: 1,
            ..Default::default()
        },
        Coverage {
            environment_absent_elsewhere: 1,
            ..Default::default()
        },
        Coverage {
            ancestors_read: 1,
            ..Default::default()
        },
        Coverage {
            other_boot: true,
            ..Default::default()
        },
    ] {
        assert!(!c.limited());
    }
    for c in [
        Coverage {
            ancestry_unconfirmed: 1,
            ..Default::default()
        },
        Coverage {
            clock_behind: true,
            ..Default::default()
        },
    ] {
        assert!(c.limited());
    }
}
/// Empty or NUL-only argument tails are absent rather than observable nonmatches.
#[test]
fn ea18_bytes() {
    for (tail, expected) in [
        (b"".as_slice(), Environment::Absent),
        (b"\0", Environment::Absent),
        (b"A=1\0", Environment::NonMatch),
        (
            b"RRX_PROCESS_COOKIE=d20-fake-cookie-1234\0",
            Environment::Match,
        ),
    ] {
        let mut bytes = 1_i32.to_ne_bytes().to_vec();
        bytes.extend_from_slice(b"/bin/x\0\0x\0");
        bytes.extend_from_slice(tail);
        assert_eq!(
            environment_result(&bytes[environment_offset(&bytes).unwrap()..], &cookie()),
            expected
        );
    }
}
/// Unconfirmed metadata is bounded independently of the uncertainty count.
#[test]
fn ea19_cap() {
    let mut ops = vec![list(&(10..30).collect::<Vec<_>>())];
    for pid in 10..30 {
        ops.extend(absent(obs(pid, (1000, 0), 1)));
    }
    uncertain(&run(ops), 20);
}
/// Traced target observations cannot be excluded through their parent.
#[test]
fn ea20_traced() {
    let mut o = x(20);
    o.flags = PROC_FLAG_TRACED;
    let mut ops = vec![list(&[10])];
    ops.extend(absent(o));
    uncertain(&run(ops), 1);
}
/// Stable paths outside the four tool directories exclude without ancestry reads.
#[test]
fn ea21_elsewhere() {
    for path in ["/System/Library/x", "/usr/libexec/x"] {
        let mut ops = vec![list(&[10])];
        ops.extend(located(
            x(1),
            Some(path),
            Some(Environment::Absent),
            Some(path),
        ));
        let d = run(ops);
        assert!(!d.coverage.limited());
        assert_eq!(d.coverage.environment_absent_elsewhere, 1);
    }
}
/// All four direct tool directories and failed path reads retain uncertainty.
#[test]
fn ea22_locations() {
    for (a, b) in [
        (Some("/bin/x"), Some("/bin/x")),
        (Some("/usr/bin/x"), Some("/usr/bin/x")),
        (Some("/sbin/x"), Some("/sbin/x")),
        (Some("/usr/sbin/x"), Some("/usr/sbin/x")),
        (None, Some("/System/Library/x")),
        (Some("/System/Library/x"), None),
    ] {
        let mut ops = vec![list(&[10])];
        ops.extend(located(x(1), a, Some(Environment::Absent), b));
        uncertain(&run(ops), 1);
    }
}
/// A reused parent PID is refused by the child's parent recheck despite plausible clocks.
#[test]
fn ea23_reused_parent() {
    let o = obs(10, (2000, 0), 20);
    let mut parent = q(20, 1999, 30);
    parent.info.start.tv_usec = 500000;
    let mut ops = vec![list(&[10])];
    ops.extend(absent(o));
    ops.extend([query(20, parent), id(obs(10, (2000, 0), 1))]);
    let d = run(ops);
    uncertain(&d, 1);
    assert_eq!(d.coverage.ancestors_read, 1);
}
/// Environment and location changes inside the read window cannot exclude.
#[test]
fn ea24_exec_window() {
    for (a, e, b) in [
        (
            "/usr/bin/perl",
            Some(Environment::Absent),
            "/opt/homebrew/bin/python3",
        ),
        (
            "/opt/homebrew/bin/python3",
            Some(Environment::Absent),
            "/usr/bin/perl",
        ),
        (
            "/opt/homebrew/bin/python3",
            Some(Environment::Match),
            "/opt/homebrew/bin/python3",
        ),
        (
            "/opt/homebrew/bin/python3",
            Some(Environment::NonMatch),
            "/opt/homebrew/bin/python3",
        ),
        (
            "/opt/homebrew/bin/python3",
            None,
            "/opt/homebrew/bin/python3",
        ),
    ] {
        let mut ops = vec![list(&[10])];
        ops.extend(located(x(1), Some(a), e, Some(b)));
        uncertain(&run(ops), 1);
    }
}
/// An exec between rounds is found because location exclusions leave no record.
#[test]
fn ea25_exec_between_rounds() {
    let mut ops = vec![list(&[10, 11])];
    ops.extend(located(
        x(1),
        Some("/System/Library/x"),
        Some(Environment::Absent),
        Some("/System/Library/x"),
    ));
    ops.extend([
        Op::Id(11, None),
        query(
            11,
            Query {
                len: 0,
                ..q(11, 0, 0)
            },
        ),
        list(&[10]),
    ]);
    ops.extend(simple(x(1), Environment::Match));
    let d = run(ops);
    assert_eq!(d.processes.len(), 1);
    assert_eq!(d.processes[0].identity().pid, 10);
}
/// Locations are read before identity2 and never after the ancestry recheck.
#[test]
fn ea26_post_identity_reuse() {
    let mut ops = vec![list(&[10])];
    ops.extend(absent(x(20)));
    ops.extend([query(20, q(20, 900, 1)), id(obs(10, (2001, 0), 30))]);
    uncertain(&run(ops), 1);
}
/// Clock rollback uses F including an earlier recorded leader birth.
#[test]
fn ea27_clock_boundary() {
    let mut launch = launch();
    launch.leader_birth = Some((970, 0));
    for wall in [(930, 0), (910, 0), (909, 999999)] {
        let mut f = Fake::new(vec![list(&[0])]);
        f.wall = wall;
        let d = f.run(Some(&launch));
        assert_eq!(d.coverage.limited(), wall < (910, 0));
    }
}
/// Directory comparisons are exact parents rather than prefixes or ancestors.
#[test]
fn ea28_exact_parent() {
    for path in [
        "/usr/bin/x/y",
        "/usr/binx/y",
        "/binx/y",
        "/usr/sbin2/y",
        "/usr/local/bin/y",
    ] {
        let mut ops = vec![list(&[10])];
        ops.extend(located(
            x(1),
            Some(path),
            Some(Environment::Absent),
            Some(path),
        ));
        let d = run(ops);
        assert!(!d.coverage.limited());
        assert_eq!(d.coverage.environment_absent_elsewhere, 1);
    }
}

/// Leader registration accepts matches, absent environments and zombies but refuses nonmatches.
#[test]
fn leader_cookie() {
    for state in [
        Environment::Match,
        Environment::Absent,
        Environment::NonMatch,
    ] {
        let mut f = Fake::new(vec![query(10, q(10, 1000, 1)), env(10, state)]);
        assert_eq!(
            read_owned_leader(10, &cookie(), &mut f).is_ok(),
            state != Environment::NonMatch
        );
        assert!(f.ops.is_empty());
    }
    let mut zombie = q(10, 1000, 1);
    zombie.info.stat = SZOMB;
    let mut f = Fake::new(vec![query(10, zombie)]);
    assert!(read_owned_leader(10, &cookie(), &mut f).unwrap().zombie);
    assert!(f.ops.is_empty());
    for kind in 0..4 {
        let mut after = q(10, 1000, 1);
        if kind != 0 {
            after.info.stat = SZOMB;
        }
        if kind == 2 {
            after.info.start.tv_sec += 1;
        }
        if kind == 3 {
            after.status = -1;
        }
        let mut f = Fake::new(vec![
            query(10, q(10, 1000, 1)),
            Op::Env(10, None),
            query(10, after),
        ]);
        assert_eq!(read_owned_leader(10, &cookie(), &mut f).is_ok(), kind == 1);
        assert!(f.ops.is_empty());
    }
}
