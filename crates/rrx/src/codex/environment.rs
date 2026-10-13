//! Private raw values; only bounded name projections reach Store authority.
use super::{ownership::state_error, protocol::failure};
use crate::{
    adapter::{AdapterResult, ErrorKind, LaunchRequest, SharedStore},
    domain::ProjectId,
    project::{
        environment_name_forbidden, environment_name_valid, validate_environment_references,
    },
    state::EnvironmentAdmission,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
};

const UNAVAILABLE: &str = "native environment authority unavailable";
fn unavailable() -> crate::adapter::AdapterError {
    failure(ErrorKind::InvalidConfiguration, UNAVAILABLE)
}
fn baseline_key(name: &str) -> bool {
    matches!(
        name,
        "HOME"
            | "PATH"
            | "SHELL"
            | "LANG"
            | "TERM"
            | "TMPDIR"
            | "TEMP"
            | "TMP"
            | "NODE_OPTIONS"
            | "NODE_PATH"
            | "SSL_CERT_FILE"
            | "SSL_CERT_DIR"
            | "NODE_EXTRA_CA_CERTS"
            | "OPENAI_API_KEY"
            | "OPENAI_BASE_URL"
            | "OPENAI_API_BASE"
            | "XAI_API_KEY"
            | "ANTHROPIC_API_KEY"
            | "SSH_AUTH_SOCK"
            | "SSH_ASKPASS"
            | "EDITOR"
            | "VISUAL"
    ) || name.starts_with("LC_")
        || name.starts_with("XDG_")
        || name.starts_with("CODEX_")
        || name.to_ascii_uppercase().ends_with("_PROXY")
}
fn control(name: &str) -> bool {
    baseline_key(name) && (!environment_name_valid(name) || environment_name_forbidden(name))
}
fn ordinary(name: &str) -> bool {
    matches!(
        name,
        "LANG" | "LC_ALL" | "LC_CTYPE" | "TERM" | "COLORTERM" | "TZ"
    )
}
// Deliberately no formatting/serialization implementations on value holders or iterators.
pub(super) struct FrozenEnvironment {
    values: BTreeMap<String, OsString>,
}
pub(super) struct Selection {
    values: BTreeMap<String, OsString>,
    pub admission: EnvironmentAdmission,
}
pub(super) struct Values<'a>(std::collections::btree_map::Iter<'a, String, OsString>);
impl<'a> Iterator for Values<'a> {
    type Item = (&'a str, &'a OsStr);
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|(name, value)| (name.as_str(), value.as_os_str()))
    }
}
impl FrozenEnvironment {
    pub fn new(pairs: impl IntoIterator<Item = (OsString, OsString)>) -> AdapterResult<Self> {
        use std::os::unix::ffi::OsStrExt;
        let mut values = BTreeMap::new();
        for (name, value) in pairs {
            if name.as_bytes().contains(&0)
                || name.as_bytes().contains(&b'=')
                || value.as_bytes().contains(&0)
            {
                return Err(unavailable());
            }
            let Some(name) = name.to_str() else { continue };
            if name.starts_with("GIT_") || !baseline_key(name) {
                continue;
            }
            values.insert(name.to_owned(), value);
        }
        let frozen = Self { values };
        frozen.admission(std::iter::empty())?;
        Ok(frozen)
    }
    #[cfg(test)]
    pub fn value_matches(&self, name: &str, expected: &OsStr) -> bool {
        self.values
            .get(name)
            .is_some_and(|value| value.as_os_str() == expected)
    }
    fn admission(
        &self,
        caller: impl IntoIterator<Item = String>,
    ) -> AdapterResult<EnvironmentAdmission> {
        EnvironmentAdmission::new(
            self.values.keys().filter(|key| !control(key)).cloned(),
            caller,
        )
        .map_err(|_| unavailable())
    }
    pub fn candidates(
        &self,
        store: &SharedStore,
        project: ProjectId,
    ) -> AdapterResult<BTreeSet<String>> {
        let admission = self.admission(std::iter::empty())?;
        store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
            .environment_candidates(project, &admission)
            .map_err(state_error)
    }
    pub fn select(&self, store: &SharedStore, request: &LaunchRequest) -> AdapterResult<Selection> {
        validate_environment_references(&request.project.environment_refs)
            .map_err(|_| unavailable())?;
        let mut values = self.values.clone();
        for (key, value) in &request.environment {
            if key.starts_with("GIT_")
                || key.is_empty()
                || key.contains(['=', '\0'])
                || value.contains('\0')
            {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "invalid/leaking Git environment",
                ));
            }
            if key.starts_with("RRX_")
                || !environment_name_valid(key)
                || environment_name_forbidden(key)
                || (!ordinary(key)
                    && (!baseline_key(key)
                        || self.values.get(key).map(OsString::as_os_str)
                            != Some(OsStr::new(value))))
            {
                return Err(unavailable());
            }
            values.insert(key.clone(), value.into());
        }
        let admission = self.admission(request.environment.keys().cloned())?;
        let locked = store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        if let Err(error) = locked.check_environment_admission(request.scope.project_id, &admission)
        {
            // A separate own reread preserves stale currency on an initial refusal.
            // These autocommit reads do not constitute a single initial snapshot.
            let current = locked.project(request.project.id).map_err(state_error)?;
            if current
                .as_ref()
                .map(serde_json::to_value)
                .transpose()
                .map_err(|_| failure(ErrorKind::StateFailure, "Project serialization failed"))?
                != Some(serde_json::to_value(&request.project).map_err(|_| {
                    failure(ErrorKind::StateFailure, "Project serialization failed")
                })?)
            {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native Project snapshot changed before environment admission",
                ));
            }
            return Err(state_error(error));
        }
        Ok(Selection { values, admission })
    }
}
impl Selection {
    #[cfg(test)]
    pub fn fixture_empty() -> Self {
        Self {
            values: BTreeMap::new(),
            admission: EnvironmentAdmission::new(std::iter::empty(), std::iter::empty())
                .expect("empty synthetic name projection"),
        }
    }
    pub fn values(&self) -> Values<'_> {
        Values(self.values.iter())
    }
    pub fn verify_references(&self, config: &Value) -> AdapterResult<()> {
        fn parse() -> crate::adapter::AdapterError {
            failure(
                ErrorKind::ParseFailure,
                "invalid native provider environment references",
            )
        }
        fn unsupported() -> crate::adapter::AdapterError {
            failure(
                ErrorKind::UnsupportedCapability,
                "unsupported native provider environment reference",
            )
        }
        let config = config.as_object().ok_or_else(parse)?;
        let Some(selected) = config.get("model_provider").filter(|v| !v.is_null()) else {
            return Ok(());
        };
        let selected = selected.as_str().ok_or_else(parse)?;
        if selected.is_empty() || selected.len() > 128 {
            return Err(unsupported());
        }
        let Some(providers) = config.get("model_providers").filter(|v| !v.is_null()) else {
            return Ok(());
        };
        let providers = providers.as_object().ok_or_else(parse)?;
        let Some(settings) = providers.get(selected).filter(|v| !v.is_null()) else {
            return Ok(());
        };
        let settings = settings.as_object().ok_or_else(parse)?;
        let key = settings
            .get("env_key")
            .filter(|v| !v.is_null())
            .map(|v| v.as_str().ok_or_else(parse))
            .transpose()?;
        let headers = settings
            .get("env_http_headers")
            .filter(|v| !v.is_null())
            .map(|v| v.as_object().ok_or_else(parse))
            .transpose()?;
        if usize::from(key.is_some()) + headers.map_or(0, |h| h.len()) > 128 {
            return Err(unsupported());
        }
        let mut headers = headers
            .into_iter()
            .flat_map(|h| h.iter())
            .collect::<Vec<_>>();
        headers.sort_unstable_by(|a, b| a.0.cmp(b.0));
        let mut references = Vec::with_capacity(usize::from(key.is_some()) + headers.len());
        if let Some(key) = key {
            references.push(key)
        }
        for (_, value) in headers {
            references.push(value.as_str().ok_or_else(parse)?)
        }
        for reference in references {
            if reference.is_empty()
                || reference.len() > 128
                || !environment_name_valid(reference)
                || environment_name_forbidden(reference)
                || !self.values.contains_key(reference)
            {
                return Err(unsupported());
            }
        }
        Ok(())
    }
}
// Plain Git retains its existing separate ambient policy. This wrapper cannot be formatted.
pub(super) enum ExecEnvironment<'a> {
    Ambient(Vec<(OsString, OsString)>),
    Selected(&'a Selection),
}
impl ExecEnvironment<'_> {
    pub fn selected(&self) -> bool {
        matches!(self, Self::Selected(_))
    }
    pub fn require_boundary(&self, present: bool) -> AdapterResult<()> {
        if self.selected() && !present {
            return Err(failure(
                ErrorKind::StateConflict,
                "selected native exec admission unavailable",
            ));
        }
        Ok(())
    }
    pub fn apply(&self, command: &mut tokio::process::Command) {
        match self {
            Self::Ambient(pairs) => {
                command.envs(pairs.iter().map(|(k, v)| (k.as_os_str(), v.as_os_str())));
            }
            Self::Selected(selection) => {
                command.envs(selection.values());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::{InputKind, LaunchMode, PreparedInput},
        domain::{Project, Scope, SessionRole},
        state::Store,
    };
    use std::{
        os::unix::ffi::OsStrExt,
        sync::{Arc, Mutex},
    };
    fn pairs<'a>(
        pairs: &'a [(&'a str, &'a str)],
    ) -> impl Iterator<Item = (OsString, OsString)> + 'a {
        pairs.iter().map(|(k, v)| ((*k).into(), (*v).into()))
    }
    fn request(store: &SharedStore, refs: &[&str]) -> LaunchRequest {
        let mut project = Project::new(
            "synthetic".into(),
            "/tmp/synthetic".into(),
            "synthetic-identity".into(),
            "main".into(),
        );
        project.environment_refs = refs.iter().map(|v| (*v).into()).collect();
        store.lock().unwrap().put_project(&mut project).unwrap();
        let scope = Scope::project(project.id);
        LaunchRequest {
            project,
            scope: scope.clone(),
            worktree: "/tmp/synthetic".into(),
            role: SessionRole::Consultant,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope,
                kind: InputKind::ContextPack,
                revision: "a".repeat(40),
                version: 1,
                source_versions: BTreeMap::new(),
                payload: "synthetic input".into(),
            },
            environment: BTreeMap::new(),
            model: None,
            effort: None,
        }
    }
    fn store() -> (tempfile::TempDir, SharedStore) {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(
            Store::open(&directory.path().join("state.sqlite3")).unwrap(),
        ));
        (directory, store)
    }
    fn kind<T>(result: AdapterResult<T>) -> ErrorKind {
        match result {
            Err(error) => error.kind,
            Ok(_) => panic!("expected opaque refusal"),
        }
    }
    #[test]
    fn exact_membership_control_classification_has_no_additional_controls() {
        let controls = [
            "HOME",
            "PATH",
            "SHELL",
            "TMPDIR",
            "TEMP",
            "TMP",
            "NODE_OPTIONS",
            "NODE_PATH",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "NODE_EXTRA_CA_CERTS",
            "OPENAI_BASE_URL",
            "OPENAI_API_BASE",
            "SSH_AUTH_SOCK",
            "SSH_ASKPASS",
            "EDITOR",
            "VISUAL",
            "XDG_CONFIG_HOME",
            "XDG_STATE_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_RUNTIME_DIR",
            "CODEX_HOME",
            "CODEX_SYNTHETIC",
            "HTTP_PROXY",
            "https_proxy",
            "LC_BAD-NAME",
        ];
        let non_controls = [
            "LANG",
            "TERM",
            "OPENAI_API_KEY",
            "XAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "LC_ALL",
            "LC_CTYPE",
            "LC_SYNTHETIC",
            "XDG_CONFIG_DIRS",
            "XDG_DATA_DIRS",
            "XDG_SYNTHETIC",
        ];
        for name in controls {
            assert!(baseline_key(name) && control(name));
        }
        for name in non_controls {
            assert!(baseline_key(name) && !control(name));
        }
        for name in controls.into_iter().chain(non_controls).chain([
            "USER",
            "LOGNAME",
            "COLORTERM",
            "TZ",
            "GIT_HTTP_PROXY",
            "SYNTHETIC",
        ]) {
            // With EMPTY additions, membership is the only extra condition;
            // no registry-valid, ownable name becomes a global control.
            assert!(
                control(name)
                    == (baseline_key(name)
                        && (!environment_name_valid(name) || environment_name_forbidden(name)))
            );
            assert!(
                !control(name) || !environment_name_valid(name) || environment_name_forbidden(name)
            );
        }
        assert!(!baseline_key("USER") && !baseline_key("LOGNAME"));
    }
    #[test]
    fn common_constructor_preserves_raw_values_controls_and_exact_membership() {
        let mut input = pairs(&[
            ("HOME", "synthetic"),
            ("GIT_HTTP_PROXY", "excluded"),
            ("USER", "excluded"),
            ("LOGNAME", "excluded"),
            ("LC_BAD-NAME", "control"),
            ("XDG_CONFIG_DIRS", "first"),
            ("XDG_CONFIG_DIRS", "last"),
        ])
        .collect::<Vec<_>>();
        input.push((OsStr::from_bytes(b"LC_\xff").to_owned(), "excluded".into()));
        input.push((
            "OPENAI_API_KEY".into(),
            OsStr::from_bytes(b"synthetic-\xff").to_owned(),
        ));
        let baseline = FrozenEnvironment::new(input).unwrap();
        assert!(baseline.values.len() == 4);
        assert!(
            baseline
                .values
                .get("OPENAI_API_KEY")
                .is_some_and(|v| v.as_os_str().as_bytes() == b"synthetic-\xff")
        );
        assert!(
            baseline
                .values
                .get("XDG_CONFIG_DIRS")
                .is_some_and(|v| v == "last")
        );
        let names = baseline
            .admission(std::iter::empty())
            .unwrap()
            .candidates(&[
                "HOME".into(),
                "LC_BAD-NAME".into(),
                "XDG_CONFIG_DIRS".into(),
                "OPENAI_API_KEY".into(),
            ]);
        assert!(names == BTreeSet::from(["XDG_CONFIG_DIRS".into(), "OPENAI_API_KEY".into()]));
        for (k, v) in [
            (b"BAD=NAME".as_slice(), b"v".as_slice()),
            (b"BAD\0NAME", b"v"),
            (b"OPENAI_API_KEY", b"v\0"),
        ] {
            assert!(
                kind(FrozenEnvironment::new([(
                    OsStr::from_bytes(k).to_owned(),
                    OsStr::from_bytes(v).to_owned()
                )])) == ErrorKind::InvalidConfiguration
            );
        }
        let many = (0..513).map(|i| (format!("LC_{i}").into(), "synthetic".into()));
        assert!(kind(FrozenEnvironment::new(many)) == ErrorKind::InvalidConfiguration);
        assert!(
            kind(FrozenEnvironment::new([(
                format!("LC_{}", "a".repeat(254)).into(),
                "v".into()
            )])) == ErrorKind::InvalidConfiguration
        );
        let bytes = (0..257).map(|i| (format!("LC_{i:03}{}", "x".repeat(250)).into(), "v".into()));
        assert!(kind(FrozenEnvironment::new(bytes)) == ErrorKind::InvalidConfiguration);
    }
    #[test]
    fn live_names_admit_identical_native_values_and_reject_foreign_conflicts() {
        let (_directory, store) = store();
        let request = request(&store, &["OPENAI_API_KEY", "LANG", "TZ"]);
        let baseline = FrozenEnvironment::new(pairs(&[
            ("HOME", "synthetic"),
            ("OPENAI_API_KEY", "synthetic-key"),
            ("LANG", "baseline"),
        ]))
        .unwrap();
        let mut foreign = Project::new(
            "foreign".into(),
            "/tmp/foreign".into(),
            "foreign".into(),
            "main".into(),
        );
        foreign.environment_refs = vec!["OPENAI_API_KEY".into()];
        store.lock().unwrap().put_project(&mut foreign).unwrap();
        let mut request = request;
        request
            .environment
            .insert("OPENAI_API_KEY".into(), "synthetic-key".into());
        request
            .environment
            .insert("LANG".into(), "own-locale".into());
        request.environment.insert("TZ".into(), "UTC".into());
        let selection = baseline.select(&store, &request).unwrap();
        assert!(
            selection
                .values
                .get("LANG")
                .is_some_and(|v| v == "own-locale")
        );
        request
            .environment
            .insert("OPENAI_API_KEY".into(), "different".into());
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::InvalidConfiguration);
        request.environment.clear();
        let mut current = store
            .lock()
            .unwrap()
            .project(request.project.id)
            .unwrap()
            .unwrap();
        current.environment_refs.retain(|n| n != "OPENAI_API_KEY");
        store.lock().unwrap().put_project(&mut current).unwrap();
        // A rejecting own change is stale currency, not a foreign inventory diagnostic.
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::StateConflict);
        request.project = current;
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::InvalidConfiguration);
        assert!(
            baseline.candidates(&store, request.project.id).unwrap()
                == BTreeSet::from(["LANG".into()])
        );
    }
    #[test]
    fn ordered_caller_shape_precedes_current_declaration_and_dto_count() {
        let (_directory, store) = store();
        let mut request = request(&store, &[]);
        let baseline = FrozenEnvironment::new(std::iter::empty()).unwrap();
        request.environment = BTreeMap::from([
            ("COLORTERM".into(), "synthetic".into()),
            ("GIT_DIR".into(), "bad".into()),
        ]);
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::InvalidInput);
        request
            .environment
            .insert("A_UNSUPPORTED".into(), "synthetic".into());
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::InvalidConfiguration);
        let mut malformed = request.clone();
        malformed.project.environment_refs = vec!["HOME".into()];
        malformed.environment = BTreeMap::from([("GIT_DIR".into(), "synthetic".into())]);
        assert!(kind(baseline.select(&store, &malformed)) == ErrorKind::InvalidConfiguration);
        let input = (0..129)
            .map(|i| (format!("LC_{i:03}"), "synthetic".to_owned()))
            .collect::<BTreeMap<_, _>>();
        let baseline =
            FrozenEnvironment::new(input.iter().map(|(k, v)| (k.into(), v.into()))).unwrap();
        request.project.environment_refs = input.keys().cloned().collect();
        store
            .lock()
            .unwrap()
            .put_project(&mut request.project)
            .unwrap();
        request.environment = input;
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::InvalidConfiguration);
        request
            .environment
            .insert("LC_zzz".into(), "synthetic\0".into());
        assert!(kind(baseline.select(&store, &request)) == ErrorKind::InvalidInput);
    }
    #[test]
    fn provider_reference_order_rejects_without_value_restoration() {
        let mut selection = Selection::fixture_empty();
        selection
            .values
            .insert("OPENAI_API_KEY".into(), "synthetic".into());
        use serde_json::json;
        for config in [
            json!({}),
            json!({"model_provider":null,"model_providers":7}),
            json!({"model_provider":"x"}),
            json!({"model_provider":"x","model_providers":null}),
            json!({"model_provider":"x","model_providers":{}}),
            json!({"model_provider":"x","model_providers":{"x":null}}),
            json!({"model_provider":"x","model_providers":{"x":{"env_key":"OPENAI_API_KEY","env_http_headers":{"same":"OPENAI_API_KEY"}}}}),
        ] {
            assert!(selection.verify_references(&config).is_ok());
        }
        for config in [
            json!(null),
            json!({"model_provider":3}),
            json!({"model_provider":"x","model_providers":3}),
            json!({"model_provider":"x","model_providers":{"x":3}}),
            json!({"model_provider":"x","model_providers":{"x":{"env_key":"bad-name","env_http_headers":3}}}),
            json!({"model_provider":"x","model_providers":{"x":{"env_key":"bad-name","env_http_headers":{"A":3}}}}),
        ] {
            assert!(kind(selection.verify_references(&config)) == ErrorKind::ParseFailure);
        }
        for name in ["HOME", "GIT_DIR", "MISSING", "bad-name", ""] {
            let config = json!({"model_provider":"x","model_providers":{"x":{"env_key":name}}});
            assert!(kind(selection.verify_references(&config)) == ErrorKind::UnsupportedCapability);
        }
        let headers = (0..129)
            .map(|i| (format!("{i:03}"), json!(3)))
            .collect::<serde_json::Map<_, _>>();
        let config = json!({"model_provider":"x","model_providers":{"x":{"env_key":"bad-name","env_http_headers":headers}}});
        assert!(kind(selection.verify_references(&config)) == ErrorKind::UnsupportedCapability);
        assert!(
            kind(selection.verify_references(&json!({"model_provider":"","model_providers":3})))
                == ErrorKind::UnsupportedCapability
        );
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum ExecSite {
    #[cfg(test)]
    Initial,
    Version,
    Discovery,
    Main,
}
/// Synchronous bounded admission at the already configured command's selected exec.
pub(super) struct SpawnBoundary<'a> {
    admit: &'a mut (dyn FnMut() -> AdapterResult<()> + Send),
    #[cfg(test)]
    trace: Option<(std::sync::Arc<TestHooks>, usize, ExecSite)>,
}
impl<'a> SpawnBoundary<'a> {
    pub fn new(admit: &'a mut (dyn FnMut() -> AdapterResult<()> + Send)) -> Self {
        Self {
            admit,
            #[cfg(test)]
            trace: None,
        }
    }
    pub fn admit(&mut self) -> AdapterResult<()> {
        (self.admit)()
    }
    #[cfg(test)]
    pub fn traced(
        mut self,
        hooks: std::sync::Arc<TestHooks>,
        attempt: usize,
        site: ExecSite,
    ) -> Self {
        self.trace = Some((hooks, attempt, site));
        self
    }
    pub fn spawn_attempt(&self) {
        #[cfg(test)]
        if let Some((hooks, attempt, site)) = &self.trace {
            hooks.count(*attempt, *site, false)
        }
    }
    pub fn spawned(&self) {
        #[cfg(test)]
        if let Some((hooks, attempt, site)) = &self.trace {
            hooks.count(*attempt, *site, true)
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum HookPoint {
    Initial,
    PreCas,
    PostCas,
}
#[cfg(test)]
type Hook = Box<dyn FnOnce(usize) + Send>;
#[cfg(test)]
#[derive(Default)]
pub(super) struct TestHooks {
    callbacks: std::sync::Mutex<BTreeMap<(ExecSite, HookPoint), Hook>>,
    counts: std::sync::Mutex<BTreeMap<(usize, ExecSite), (usize, usize)>>,
}
#[cfg(test)]
impl TestHooks {
    pub fn install(
        &self,
        site: ExecSite,
        point: HookPoint,
        callback: impl FnOnce(usize) + Send + 'static,
    ) {
        assert!(
            self.callbacks
                .lock()
                .unwrap()
                .insert((site, point), Box::new(callback))
                .is_none()
        );
    }
    pub fn call(&self, attempt: usize, site: ExecSite, point: HookPoint) {
        // Release the hook registry before invoking a caller-owned std rendezvous.
        let callback = self.callbacks.lock().unwrap().remove(&(site, point));
        if let Some(callback) = callback {
            callback(attempt)
        }
    }
    fn count(&self, attempt: usize, site: ExecSite, success: bool) {
        let mut counts = self.counts.lock().unwrap();
        let count = counts.entry((attempt, site)).or_default();
        if success { count.1 += 1 } else { count.0 += 1 }
    }
    pub fn counts(&self, attempt: usize, site: ExecSite) -> (usize, usize) {
        self.counts
            .lock()
            .unwrap()
            .get(&(attempt, site))
            .copied()
            .unwrap_or_default()
    }
    pub fn total(&self, site: ExecSite) -> (usize, usize) {
        self.counts
            .lock()
            .unwrap()
            .iter()
            .filter(|((_, s), _)| *s == site)
            .fold((0, 0), |a, (_, b)| (a.0 + b.0, a.1 + b.1))
    }
}
