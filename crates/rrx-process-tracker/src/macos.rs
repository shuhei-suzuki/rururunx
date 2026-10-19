//! Public SDK proc_info.h/libproc.h and KERN_PROCARGS2; no private entitlement.
//! Safety: all pointer destinations below have their exact writable allocation;
//! initialized lengths and struct return sizes are checked before interpretation.
use super::*;
use std::collections::{HashMap, HashSet};
use std::mem::{MaybeUninit, size_of};

// macOS SDK sys/proc_info.h: PROC_UID_ONLY. proc_listpids returns byte counts.
const PROC_UID_ONLY: u32 = 4;
// macOS SDK sys/proc.h: SZOMB (the only state classified as a zombie).
const SZOMB: i8 = 5;

// Public SDK sys/sysctl.h kinfo_proc and sys/proc.h extern_proc, LP64 ABI
// (both supported macOS architectures). libc does not expose kinfo_proc.
// Keep only the interpreted prefix typed; the remaining SDK fields are opaque.
// SDK sizeof=648, alignof=8; p_starttime/p_stat/p_pid offsets are 0/36/40.
#[repr(C)]
#[derive(Clone, Copy)]
struct KinfoProc {
    start: libc::timeval,
    vmspace: usize,
    sigacts: usize,
    flag: i32,
    stat: i8,
    pid: i32,
    rest: [u8; 604],
}

struct Query {
    status: i32,
    len: usize,
    info: KinfoProc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unreadable {
    Vanished,
    Zombie((u64, u64)),
    Unknown,
}

fn classify_unreadable(pid: u32, query: Query) -> Unreadable {
    if query.status != 0 {
        return Unreadable::Unknown;
    }
    if query.len == 0 {
        return Unreadable::Vanished;
    }
    if query.len == size_of::<KinfoProc>()
        && query.info.pid == pid as i32
        && query.info.stat == SZOMB
    {
        return Unreadable::Zombie((
            query.info.start.tv_sec as u64,
            query.info.start.tv_usec as u64,
        ));
    }
    Unreadable::Unknown
}

fn query_pid(pid: u32) -> Query {
    // SAFETY: all fields are integer/byte types, for which zero is valid.
    // Initialize the full object so even failed/partial sysctl outputs are safe
    // to carry to the classifier, which checks status and exact length first.
    let mut info: KinfoProc = unsafe { std::mem::zeroed() };
    let mut len = size_of::<KinfoProc>();
    let mut mib = [
        libc::CTL_KERN,
        libc::KERN_PROC,
        libc::KERN_PROC_PID,
        pid as i32,
    ];
    // SAFETY: mib has four initialized integers; info is one full writable
    // kinfo_proc, len describes its allocation, and no new value is supplied.
    let status = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            (&mut info as *mut KinfoProc).cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    Query { status, len, info }
}
#[link(name = "proc")]
unsafe extern "C" {}

pub(super) struct Handle;
impl Handle {
    pub fn signal_available(&self) -> bool {
        false
    }
    pub fn terminate(&self) -> Termination {
        Termination::Unsupported
    }
    pub fn exited(&self) -> io::Result<Option<bool>> {
        Ok(None)
    }
}
fn identity(pid: u32) -> io::Result<Identity> {
    let mut info = MaybeUninit::<libc::proc_bsdinfo>::uninit();
    // SAFETY: the destination points to one full writable proc_bsdinfo. No field
    // is read unless the kernel reports the exact struct size.
    let bytes = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size_of::<libc::proc_bsdinfo>() as i32,
        )
    };
    if bytes != size_of::<libc::proc_bsdinfo>() as i32 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the exact-size return above initialized the complete public struct.
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process identity changed",
        ));
    }
    Ok(Identity {
        pid,
        uid: info.pbi_uid,
        birth: (info.pbi_start_tvsec, info.pbi_start_tvusec),
    })
}
fn environment_offset(bytes: &[u8]) -> Option<usize> {
    let argc = i32::from_ne_bytes(bytes.get(..size_of::<i32>())?.try_into().ok()?);
    if !(0..=65536).contains(&argc) {
        return None;
    }
    let mut at = size_of::<i32>();
    // Executable path followed by padding, then exactly argc argument strings.
    at += bytes.get(at..)?.iter().position(|b| *b == 0)? + 1;
    while bytes.get(at) == Some(&0) {
        at += 1;
    }
    for _ in 0..argc {
        at += bytes.get(at..)?.iter().position(|b| *b == 0)? + 1;
    }
    Some(at)
}
fn cookie_matches(pid: u32, cookie: &Cookie, cap: usize) -> io::Result<bool> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as i32];
    let mut buffer = EnvironmentBuffer::new(vec![0_u8; cap]);
    let bytes = &mut buffer.bytes;
    let mut count = bytes.len();
    // SAFETY: mib contains three initialized integers; bytes owns cap writable
    // bytes; count points to an initialized size. No new-value buffer is supplied.
    let status = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            bytes.as_mut_ptr().cast(),
            &mut count,
            std::ptr::null_mut(),
            0,
        )
    };
    let result = if status != 0 || count > cap {
        Err(io::Error::last_os_error())
    } else {
        bytes.truncate(count);
        environment_offset(bytes)
            .map(|at| matches_environment(&bytes[at..], cookie))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process environment unavailable",
                )
            })
    };
    drop(buffer);
    result
}
// Private observation seams; only discover wires production syscalls, and only
// this module can turn a completed scan into a TrackedProcess.
trait Source {
    fn list(&mut self, uid: u32, pids: &mut [i32]) -> i32;
    fn identity(&mut self, pid: u32) -> io::Result<Identity>;
    fn environment(&mut self, pid: u32, cookie: &Cookie, cap: usize) -> io::Result<bool>;
    fn query(&mut self, pid: u32) -> Query;
    fn elapsed(&mut self) -> Duration;
}
struct System(Instant);
impl Source for System {
    fn list(&mut self, uid: u32, pids: &mut [i32]) -> i32 {
        // SAFETY: pids is a fully initialized contiguous integer buffer of the
        // specified byte length. The selector returns same-effective-user PIDs.
        unsafe {
            libc::proc_listpids(
                PROC_UID_ONLY,
                uid,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(pids) as i32,
            )
        }
    }
    fn identity(&mut self, pid: u32) -> io::Result<Identity> {
        identity(pid)
    }
    fn environment(&mut self, pid: u32, cookie: &Cookie, cap: usize) -> io::Result<bool> {
        cookie_matches(pid, cookie, cap)
    }
    fn query(&mut self, pid: u32) -> Query {
        query_pid(pid)
    }
    fn elapsed(&mut self) -> Duration {
        self.0.elapsed()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Record {
    Live((u64, u64)),
    Zombie((u64, u64)),
}
fn remember(records: &mut HashMap<u32, Record>, pid: u32, record: Record) {
    let (Record::Live(key) | Record::Zombie(key)) = record;
    if key != (0, 0) {
        // A prior record must have been reconciled or explicitly invalidated.
        records.entry(pid).or_insert(record);
    }
}
fn absence(
    classification: Unreadable,
    pid: u32,
    records: &mut HashMap<u32, Record>,
    coverage: &mut Coverage,
    identity_failed: bool,
) {
    match classification {
        Unreadable::Vanished => coverage.vanished += 1,
        Unreadable::Zombie(key) => {
            coverage.zombie += 1;
            remember(records, pid, Record::Zombie(key));
        }
        Unreadable::Unknown if identity_failed => coverage.identity_unavailable += 1,
        Unreadable::Unknown => coverage.environment_unavailable += 1,
    }
}

fn scan(
    cookie: &Cookie,
    limits: Limits,
    uid: u32,
    source: &mut impl Source,
) -> io::Result<Discovery> {
    let mut result = Discovery {
        processes: Vec::new(),
        coverage: Coverage::default(),
    };
    let mut pids = vec![0_i32; limits.processes];
    let mut records = HashMap::new();
    // A single deadline covers listing, reconciliation and all full checks.
    macro_rules! check_time {
        () => {
            if source.elapsed() >= limits.time {
                result.coverage.time_exceeded = true;
                return Ok(result);
            }
        };
    }
    loop {
        check_time!();
        let count = source.list(uid, &mut pids);
        if count <= 0 {
            return Err(io::Error::last_os_error());
        }
        let count = count as usize;
        if !count.is_multiple_of(size_of::<i32>()) || count > std::mem::size_of_val(pids.as_slice())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid PID list length",
            ));
        }
        let returned = count / size_of::<i32>();
        let mut seen = HashSet::new();
        for pid in pids.iter().take(returned).filter(|pid| **pid > 0) {
            if !seen.insert(*pid) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate listed PID",
                ));
            }
        }
        result.coverage.truncated |= returned == pids.len();
        let prior_absences =
            result.coverage.zombie + result.coverage.vanished + result.coverage.reused;
        for pid in pids.iter().copied().take(returned).filter(|pid| *pid > 0) {
            let pid = pid as u32;
            check_time!();
            let before = source.identity(pid);
            let classification = if before.is_err() {
                check_time!();
                Some(classify_unreadable(pid, source.query(pid)))
            } else {
                None
            };
            if let Some(record) = records.get(&pid) {
                let reconciled = match *record {
                    Record::Live(key) => before.as_ref().is_ok_and(|i| i.birth == key),
                    Record::Zombie(key) => classification == Some(Unreadable::Zombie(key)),
                };
                if reconciled {
                    continue;
                }
                records.remove(&pid);
                if before.is_ok() {
                    result.coverage.reused += 1;
                }
            }
            result.coverage.inspected += 1;
            let before = match before {
                Ok(i) => i,
                Err(_) => {
                    absence(
                        classification.unwrap(),
                        pid,
                        &mut records,
                        &mut result.coverage,
                        true,
                    );
                    continue;
                }
            };
            if before.uid != uid {
                continue;
            }
            result.coverage.same_user += 1;
            check_time!();
            let matched = match source.environment(pid, cookie, limits.environment_bytes) {
                Ok(m) => m,
                Err(_) => {
                    check_time!();
                    let classification = classify_unreadable(pid, source.query(pid));
                    absence(
                        classification,
                        pid,
                        &mut records,
                        &mut result.coverage,
                        false,
                    );
                    continue;
                }
            };
            check_time!();
            let stable = source.identity(pid).is_ok_and(|after| after == before);
            if !stable {
                if matched {
                    result.coverage.identity_changed += 1;
                } else {
                    result.coverage.reused += 1;
                }
                continue;
            }
            remember(&mut records, pid, Record::Live(before.birth));
            if matched {
                result.processes.push(TrackedProcess {
                    identity: before,
                    handle: Handle,
                });
            }
        }
        check_time!();
        let new_absence =
            result.coverage.zombie + result.coverage.vanished + result.coverage.reused
                > prior_absences;
        if !new_absence || result.coverage.limited() {
            return Ok(result);
        }
        result.coverage.relistings += 1;
    }
}

pub(super) fn discover(cookie: &Cookie, limits: Limits, started: Instant) -> io::Result<Discovery> {
    scan(
        cookie,
        limits,
        rustix::process::geteuid().as_raw(),
        &mut System(started),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    enum Step {
        List(Vec<i32>, i32),
        Identity(u32, Option<Identity>),
        Environment(u32, Option<bool>),
        Query(u32, Box<Query>),
    }
    struct Fake {
        steps: VecDeque<Step>,
        calls: Vec<String>,
        ticks: u64,
    }
    impl Fake {
        fn new(steps: Vec<Step>) -> Self {
            Self {
                steps: steps.into(),
                calls: Vec::new(),
                ticks: 0,
            }
        }
        fn run(&mut self) -> Discovery {
            scan(&cookie(), bounds(), 42, self).unwrap()
        }
        fn finished(&self) {
            assert!(
                self.steps.is_empty(),
                "unconsumed observations: {}",
                self.steps.len()
            );
        }
    }
    impl Source for Fake {
        fn list(&mut self, uid: u32, pids: &mut [i32]) -> i32 {
            assert_eq!(uid, 42);
            self.calls.push("list".into());
            let Some(Step::List(values, count)) = self.steps.pop_front() else {
                panic!("unexpected list");
            };
            for (out, value) in pids.iter_mut().zip(values) {
                *out = value;
            }
            count
        }
        fn identity(&mut self, pid: u32) -> io::Result<Identity> {
            self.calls.push(format!("identity:{pid}"));
            let Some(Step::Identity(expected, value)) = self.steps.pop_front() else {
                panic!("unexpected identity {pid}");
            };
            assert_eq!(pid, expected);
            value.ok_or_else(|| io::Error::other("fake identity unavailable"))
        }
        fn environment(&mut self, pid: u32, _: &Cookie, cap: usize) -> io::Result<bool> {
            assert_eq!(cap, bounds().environment_bytes);
            self.calls.push(format!("environment:{pid}"));
            let Some(Step::Environment(expected, value)) = self.steps.pop_front() else {
                panic!("unexpected environment {pid}");
            };
            assert_eq!(pid, expected);
            value.ok_or_else(|| io::Error::other("fake environment unavailable"))
        }
        fn query(&mut self, pid: u32) -> Query {
            self.calls.push(format!("query:{pid}"));
            let Some(Step::Query(expected, value)) = self.steps.pop_front() else {
                panic!("unexpected query {pid}");
            };
            assert_eq!(pid, expected);
            *value
        }
        fn elapsed(&mut self) -> Duration {
            self.ticks += 1;
            Duration::from_millis(self.ticks)
        }
    }
    fn cookie() -> Cookie {
        Cookie::new("b4-tracker-test-cookie").unwrap()
    }
    fn bounds() -> Limits {
        Limits {
            processes: 8,
            environment_bytes: 128,
            time: Duration::from_millis(100),
        }
    }
    fn info(pid: u32, stat: i8, key: (u64, u64)) -> Query {
        // SAFETY: KinfoProc contains only integer/byte fields.
        let mut info: KinfoProc = unsafe { std::mem::zeroed() };
        info.pid = pid as i32;
        info.stat = stat;
        info.start.tv_sec = key.0 as libc::time_t;
        info.start.tv_usec = key.1 as libc::suseconds_t;
        Query {
            status: 0,
            len: size_of::<KinfoProc>(),
            info,
        }
    }
    fn gone() -> Query {
        Query {
            len: 0,
            ..info(0, 0, (0, 0))
        }
    }
    fn list(pids: &[i32]) -> Step {
        Step::List(pids.to_vec(), (pids.len() * 4) as i32)
    }
    fn query(pid: u32, value: Query) -> Step {
        Step::Query(pid, Box::new(value))
    }
    fn id(pid: u32, key: u64) -> Step {
        Step::Identity(
            pid,
            Some(Identity {
                pid,
                uid: 42,
                birth: (key, 1),
            }),
        )
    }
    fn dead(pid: u32, key: u64) -> Vec<Step> {
        vec![
            Step::Identity(pid, None),
            query(pid, info(pid, SZOMB, (key, 1))),
        ]
    }
    fn vanished(pid: u32) -> Vec<Step> {
        vec![Step::Identity(pid, None), query(pid, gone())]
    }
    fn live(pid: u32, key: u64, matched: bool) -> Vec<Step> {
        vec![
            id(pid, key),
            Step::Environment(pid, Some(matched)),
            id(pid, key),
        ]
    }

    /// A successful empty query proves absence at the query time.
    #[test]
    fn b4_zb_01_successful_empty_query_vanished() {
        assert_eq!(classify_unreadable(1, gone()), Unreadable::Vanished);
    }
    /// Only an exact successful zombie record proves zombie classification.
    #[test]
    fn b4_zb_02_exact_zombie() {
        assert_eq!(
            classify_unreadable(1, info(1, SZOMB, (10, 2))),
            Unreadable::Zombie((10, 2))
        );
    }
    /// Every non-zombie state preserves uncertainty.
    #[test]
    fn b4_zb_03_other_states_unknown() {
        for state in [1, 2, 3, 4, 0, 6, i8::MAX] {
            assert_eq!(
                classify_unreadable(1, info(1, state, (10, 2))),
                Unreadable::Unknown
            );
        }
    }
    /// Failed or malformed queries cannot prove absence or zombie status.
    #[test]
    fn b4_zb_04_failed_malformed_queries_unknown() {
        for errno in [libc::ESRCH, libc::EPERM, libc::ENOMEM] {
            for len in [0, size_of::<KinfoProc>()] {
                let mut query = info(1, SZOMB, (10, 2));
                query.status = -errno;
                query.len = len;
                assert_eq!(classify_unreadable(1, query), Unreadable::Unknown);
            }
        }
        for len in [size_of::<KinfoProc>() - 1, size_of::<KinfoProc>() + 1] {
            let mut query = info(1, SZOMB, (10, 2));
            query.len = len;
            assert_eq!(classify_unreadable(1, query), Unreadable::Unknown);
        }
        assert_eq!(
            classify_unreadable(1, info(2, SZOMB, (10, 2))),
            Unreadable::Unknown
        );
    }
    /// Absence counters never hide uncertainty or themselves limit coverage.
    #[test]
    fn b4_zb_05_absence_counters_not_limited() {
        let mut coverage = Coverage {
            zombie: 1,
            vanished: 1,
            reused: 1,
            relistings: 1,
            ..Default::default()
        };
        assert!(!coverage.limited());
        coverage.identity_unavailable = 1;
        assert!(coverage.limited());
    }
    /// A recorded zombie is reconciled without environment reads or authority.
    #[test]
    fn b4_zb_06_stable_zombie_reconciles() {
        let mut steps = vec![list(&[1])];
        steps.extend(dead(1, 10));
        steps.push(list(&[1]));
        steps.extend(dead(1, 10));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(
            fake.calls,
            [
                "list",
                "identity:1",
                "query:1",
                "list",
                "identity:1",
                "query:1"
            ]
        );
        assert_eq!(found.coverage.relistings, 1);
        assert_eq!(found.coverage.zombie, 1);
        assert!(!found.coverage.limited());
        assert!(found.processes.is_empty());
    }
    /// Re-listing finds a cookie descendant after its listed parent vanishes.
    #[test]
    fn b4_zb_07_vanished_parent_discovers_child() {
        let mut steps = vec![list(&[1])];
        steps.extend(vanished(1));
        steps.push(list(&[3]));
        steps.extend(live(3, 30, true));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.processes[0].identity().pid, 3);
        assert_eq!(found.coverage.relistings, 1);
    }
    /// A changed live record receives a full cookie check.
    #[test]
    fn b4_zb_08_reused_live_record_checked() {
        let mut steps = vec![list(&[1, 2])];
        steps.extend(live(1, 10, false));
        steps.extend(vanished(2));
        steps.push(list(&[1]));
        steps.extend(live(1, 20, true));
        steps.extend([list(&[1]), id(1, 20)]);
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.reused, 1);
        assert_eq!(found.processes[0].identity().birth, (20, 1));
    }
    /// A changed zombie key is replaced and forces another listing.
    #[test]
    fn b4_zb_09_changed_zombie_key() {
        let mut steps = Vec::new();
        for key in [10, 20, 20] {
            steps.push(list(&[1]));
            steps.extend(dead(1, key));
        }
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.zombie, 2);
        assert_eq!(found.coverage.relistings, 2);
        assert!(!found.coverage.limited());
    }
    /// Continuous new absence is bounded by the original deadline.
    #[test]
    fn b4_zb_10_churn_exceeds_time() {
        let mut steps = Vec::new();
        for pid in 1..100 {
            steps.push(list(&[pid]));
            steps.extend(vanished(pid as u32));
        }
        let mut fake = Fake::new(steps);
        let found = fake.run();
        assert!(found.coverage.time_exceeded && found.coverage.limited());
        assert!(found.coverage.relistings > 1);
    }
    /// A failed environment read with a live query remains limited.
    #[test]
    fn b4_zb_11_live_environment_unavailable() {
        let mut fake = Fake::new(vec![
            list(&[1]),
            id(1, 10),
            Step::Environment(1, None),
            query(1, info(1, 2, (10, 1))),
        ]);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.environment_unavailable, 1);
        assert!(found.coverage.limited());
    }
    /// A matched cookie with failed identity2 remains an identity change.
    #[test]
    fn b4_zb_12_matched_identity2_failure() {
        let mut fake = Fake::new(vec![
            list(&[1]),
            id(1, 10),
            Step::Environment(1, Some(true)),
            Step::Identity(1, None),
        ]);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.identity_changed, 1);
        assert_eq!(found.coverage.zombie, 0);
        assert!(found.coverage.limited());
    }
    /// A replaced live record stabilizes after exactly one full recheck.
    #[test]
    fn b4_zb_13_replaced_record_stabilizes() {
        let mut steps = vec![list(&[1, 2])];
        steps.extend(live(1, 10, false));
        steps.extend(vanished(2));
        steps.push(list(&[1]));
        steps.extend(live(1, 20, false));
        steps.extend([list(&[1]), id(1, 20)]);
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(
            (
                found.coverage.reused,
                found.coverage.relistings,
                found.coverage.inspected,
                found.coverage.same_user
            ),
            (1, 2, 3, 2)
        );
        assert!(!found.coverage.limited());
    }
    /// A nonmatching environment from a changed incarnation forces re-listing.
    #[test]
    fn b4_zb_14_nonmatch_identity2_changed() {
        let mut steps = vec![
            list(&[1]),
            id(1, 10),
            Step::Environment(1, Some(false)),
            id(1, 20),
            list(&[3]),
        ];
        steps.extend(live(3, 30, true));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.reused, 1);
        assert_eq!(found.processes[0].identity().pid, 3);
    }
    /// An unknown identity failure stops that PID and prevents another round.
    #[test]
    fn b4_zb_15_unknown_stops_reads_and_relisting() {
        let mut unknown = gone();
        unknown.status = -1;
        let mut steps = vec![list(&[1, 2]), Step::Identity(1, None), query(1, unknown)];
        steps.extend(vanished(2));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.identity_unavailable, 1);
        assert_eq!(found.coverage.vanished, 1);
        assert_eq!(found.coverage.relistings, 0);
        assert!(found.coverage.limited());
    }
    /// An unknown zombie recheck preserves uncertainty instead of reconciling.
    #[test]
    fn b4_zb_16_unknown_record_recheck() {
        let mut short = info(1, SZOMB, (10, 1));
        short.len -= 1;
        let mut steps = vec![list(&[1])];
        steps.extend(dead(1, 10));
        steps.extend([list(&[1]), Step::Identity(1, None), query(1, short)]);
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(found.coverage.identity_unavailable, 1);
        assert_eq!(found.coverage.relistings, 1);
        assert!(found.coverage.limited());
    }
    /// Malformed membership refuses while a full valid list is inspected as limited.
    #[test]
    fn b4_zb_17_invalid_and_full_lists() {
        for (values, count) in [
            (vec![], 0),
            (vec![], -1),
            (vec![1], 3),
            (vec![1], 36),
            (vec![1, 1], 8),
        ] {
            let mut fake = Fake::new(vec![Step::List(values, count)]);
            assert!(scan(&cookie(), bounds(), 42, &mut fake).is_err());
            fake.finished();
        }
        let mut steps = vec![list(&[1, 2, 3, 4, 5, 6, 7, 8])];
        for pid in 1..=8 {
            steps.extend(live(pid, 10, false));
        }
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert!(found.coverage.truncated && found.coverage.limited());
        assert_eq!(found.coverage.inspected, 8);
    }
    /// A zero zombie key cannot stabilize a scan through a recorded identity.
    #[test]
    fn b4_zb_18_zero_key_never_recorded() {
        let mut steps = Vec::new();
        for _ in 0..100 {
            steps.extend([
                list(&[1]),
                Step::Identity(1, None),
                query(1, info(1, SZOMB, (0, 0))),
            ]);
        }
        let mut fake = Fake::new(steps);
        let found = fake.run();
        assert!(found.coverage.time_exceeded && found.coverage.limited());
        assert!(found.coverage.zombie > 1);
    }
    /// A stable live record uses only identity1 in later rounds.
    #[test]
    fn b4_zb_19_stable_live_avoids_full_check() {
        let mut steps = vec![list(&[1, 2])];
        steps.extend(live(1, 10, false));
        steps.extend(vanished(2));
        steps.extend([list(&[1]), id(1, 10)]);
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(
            fake.calls,
            [
                "list",
                "identity:1",
                "environment:1",
                "identity:1",
                "identity:2",
                "query:2",
                "list",
                "identity:1"
            ]
        );
        assert_eq!((found.coverage.vanished, found.coverage.relistings), (1, 1));
        assert!(!found.coverage.limited());
    }
    /// A failed nonmatch identity2 never creates a live record.
    #[test]
    fn b4_zb_20_failed_identity2_rechecked_fully() {
        let mut steps = vec![
            list(&[1]),
            id(1, 10),
            Step::Environment(1, Some(false)),
            Step::Identity(1, None),
            list(&[1]),
        ];
        steps.extend(live(1, 10, false));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(
            (
                found.coverage.inspected,
                found.coverage.same_user,
                found.coverage.reused,
                found.coverage.relistings
            ),
            (2, 2, 1, 1)
        );
        assert!(!found.coverage.limited());
        assert!(found.processes.is_empty());
    }
    /// A live-to-zombie transition is a new absence even with an equal birth key.
    #[test]
    fn b4_zb_21_record_kind_is_significant() {
        let mut steps = vec![list(&[1, 2])];
        steps.extend(live(1, 10, false));
        steps.extend(vanished(2));
        steps.push(list(&[1]));
        steps.extend(dead(1, 10));
        steps.push(list(&[1, 3]));
        steps.extend(dead(1, 10));
        steps.extend(live(3, 30, true));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!((found.coverage.zombie, found.coverage.relistings), (1, 2));
        assert_eq!(found.processes[0].identity().pid, 3);
    }
    /// Records absent from the current membership list cause no reads or absence.
    #[test]
    fn b4_zb_22_unlisted_records_untouched() {
        let mut steps = vec![list(&[1, 2, 3])];
        steps.extend(live(1, 10, false));
        steps.extend(dead(2, 20));
        steps.extend(vanished(3));
        steps.push(list(&[4]));
        steps.extend(live(4, 40, false));
        let mut fake = Fake::new(steps);
        let found = fake.run();
        fake.finished();
        assert_eq!(
            (
                found.coverage.vanished,
                found.coverage.zombie,
                found.coverage.relistings
            ),
            (1, 1, 1)
        );
        assert!(!found.coverage.limited());
    }

    /// SDK-backed kinfo_proc layout agrees with the interpreted Rust prefix.
    #[test]
    fn b4_zb_sdk_layout() {
        assert_eq!(size_of::<KinfoProc>(), 648);
        assert_eq!(std::mem::align_of::<KinfoProc>(), 8);
        assert_eq!(std::mem::offset_of!(KinfoProc, start), 0);
        assert_eq!(std::mem::offset_of!(KinfoProc, stat), 36);
        assert_eq!(std::mem::offset_of!(KinfoProc, pid), 40);
    }

    /// A real unreaped same-user zombie is excluded without limiting an otherwise complete scan.
    #[cfg(target_os = "macos")]
    #[test]
    fn b4_host_zombie_then_vanished() {
        struct Reap(std::process::Child);
        impl Drop for Reap {
            fn drop(&mut self) {
                self.0.wait().expect("reap fixture child");
            }
        }
        let cookie = Cookie::new(&format!("b4-host-cookie-{}", std::process::id())).unwrap();
        let baseline = super::discover(&cookie, Limits::default(), Instant::now()).unwrap();
        let mut command = std::process::Command::new("/usr/bin/true");
        command.env("RRX_PROCESS_COOKIE", &cookie.0);
        let child = Reap(rrx_spawn_hygiene::spawn_std(command).unwrap());
        let pid = child.0.id();
        let started = Instant::now();
        loop {
            if let Unreadable::Zombie(key) = classify_unreadable(pid, query_pid(pid)) {
                assert_ne!(key, (0, 0));
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "fixture failed to become observable zombie"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let started = Instant::now();
        let found = super::discover(&cookie, Limits::default(), started).unwrap();
        eprintln!(
            "baseline={:?}; zombie_scan={:?}; elapsed={:?}",
            baseline.coverage,
            found.coverage,
            started.elapsed()
        );
        assert!(found.coverage.zombie >= 1);
        assert!(!found.processes.iter().any(|p| p.identity().pid == pid));
        assert!(
            !baseline.coverage.limited(),
            "baseline limited: {:?}",
            baseline.coverage
        );
        assert!(
            !found.coverage.limited(),
            "zombie scan limited: {:?}",
            found.coverage
        );
        drop(child);
        assert_eq!(
            classify_unreadable(pid, query_pid(pid)),
            Unreadable::Vanished,
            "reaped PID reused or unobservable"
        );
    }

    /// A listed live PID has a nonzero sysctl start time equal to its libproc birth.
    #[cfg(target_os = "macos")]
    #[test]
    fn b4_host_listed_start_time_matches_identity() {
        let pid = std::process::id();
        let uid = rustix::process::geteuid().as_raw();
        let mut pids = vec![0; Limits::default().processes];
        let count = System(Instant::now()).list(uid, &mut pids);
        assert!(count > 0 && count as usize <= std::mem::size_of_val(pids.as_slice()));
        assert_eq!(count as usize % size_of::<i32>(), 0);
        assert!(pids[..count as usize / size_of::<i32>()].contains(&(pid as i32)));
        let before = identity(pid).unwrap();
        let query = query_pid(pid);
        assert_eq!(query.status, 0);
        assert_eq!(query.len, size_of::<KinfoProc>());
        assert_eq!(query.info.pid, pid as i32);
        let key = (
            query.info.start.tv_sec as u64,
            query.info.start.tv_usec as u64,
        );
        assert_ne!(key, (0, 0));
        assert_eq!(key, before.birth);
        assert_eq!(identity(pid).unwrap(), before);
    }

    #[test]
    fn argument_that_looks_like_cookie_is_not_an_environment_match() {
        let cookie = Cookie::new("test-cookie-123456").unwrap();
        let mut bytes = 2_i32.to_ne_bytes().to_vec();
        bytes.extend_from_slice(
            b"/fixture\0\0fixture\0RRX_PROCESS_COOKIE=test-cookie-123456\0OTHER=discarded\0",
        );
        let at = environment_offset(&bytes).unwrap();
        assert!(!matches_environment(&bytes[at..], &cookie));
        bytes.extend_from_slice(b"RRX_PROCESS_COOKIE=test-cookie-123456\0");
        assert!(matches_environment(&bytes[at..], &cookie));
    }
}
