//! Native controls for scoped decision roles. These are verified before inference.
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::Path;

use serde_json::{Value, json};

use super::protocol::failure;
use crate::adapter::{AdapterResult, ErrorKind};
use crate::domain::Project;

const PROFILE: &str = "rrx_scoped_decision";
const DISABLED_FEATURES: &[&str] = &[
    "apps",
    "shell_tool",
    "unified_exec",
    "code_mode",
    "code_mode_host",
    "code_mode_only",
    "browser_use",
    "browser_use_external",
    "browser_use_full_cdp_access",
    "computer_use",
    "image_generation",
    "multi_agent",
    "multi_agent_v2",
    "goals",
    "in_app_browser",
    "in_app_chat",
    "in_app_local_automation",
    "realtime_conversation",
];

/// Derive overrides from the installed native config, never replacing user config.
/// The native process still discovers authentication, hooks, rules and trust itself.
#[derive(Debug, Clone)]
pub struct DecisionPolicy {
    disabled_servers: BTreeSet<String>,
}
impl DecisionPolicy {
    pub fn from_native(config: &Value) -> AdapterResult<Self> {
        if !config.is_object() {
            return Err(failure(
                ErrorKind::ParseFailure,
                "native configuration unavailable",
            ));
        }
        let mut disabled_servers = BTreeSet::new();
        match config.get("mcp_servers") {
            None | Some(Value::Null) => {}
            Some(Value::Object(servers)) => {
                for name in servers.keys() {
                    // The native -c parser does not unquote dotted TOML path segments.
                    // Unknown identifier shapes cannot be safely disabled using this route.
                    if name.is_empty()
                        || name.len() > 128
                        || !name
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
                    {
                        return Err(failure(
                            ErrorKind::UnsupportedCapability,
                            "native MCP identifier cannot be safely restricted",
                        ));
                    }
                    disabled_servers.insert(name.clone());
                }
            }
            Some(_) => {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    "invalid native MCP configuration",
                ));
            }
        }
        Ok(Self { disabled_servers })
    }
    pub fn profile_name(&self) -> &'static str {
        PROFILE
    }
    /// Only restrictive overrides; no rule/hook/authentication bypass switches.
    pub fn arguments(&self) -> Vec<String> {
        let mut args = Vec::new();
        for feature in DISABLED_FEATURES {
            args.extend(["--disable".into(), (*feature).into()]);
        }
        for name in &self.disabled_servers {
            args.extend(["-c".into(), format!("mcp_servers.{name}.enabled=false")]);
        }
        args.extend([
            "-c".into(), "web_search=\"disabled\"".into(),
            "-c".into(), format!("permissions.{PROFILE}.filesystem={{\":root\"=\"deny\",\":minimal\"=\"read\",\":slash_tmp\"=\"deny\",\":tmpdir\"=\"deny\",\":workspace_roots\"={{\".\"=\"read\"}}}}"),
            "-c".into(), format!("permissions.{PROFILE}.network.enabled=false"),
        ]);
        args
    }
    pub fn thread_parameters(&self, cwd: &Path) -> Value {
        // A decision session cannot request an escape from its read-only profile.
        // Managed policies remain authoritative; rejection is an explicit failure.
        json!({"cwd":cwd,"permissions":PROFILE,"approvalPolicy":"never","environments":[],"runtimeWorkspaceRoots":[cwd],"ephemeral":false})
    }
    pub fn verify_configuration(&self, effective: &Value) -> AdapterResult<()> {
        if effective["web_search"] != "disabled"
            || DISABLED_FEATURES
                .iter()
                .any(|feature| effective["features"][*feature] != false)
            || self
                .disabled_servers
                .iter()
                .any(|name| effective["mcp_servers"][name]["enabled"] != false)
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native decision controls are not effective",
            ));
        }
        let filesystem = &effective["permissions"][PROFILE]["filesystem"];
        if filesystem
            != &json!({":root":"deny",":minimal":"read",":slash_tmp":"deny",":tmpdir":"deny",":workspace_roots":{".":"read"}})
            || effective["permissions"][PROFILE]["network"]["enabled"] != false
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native scoped decision profile changed",
            ));
        }
        Ok(())
    }
    pub fn verify_thread(&self, response: &Value, cwd: &Path) -> AdapterResult<String> {
        let native_ref = verify_thread_identity(response, cwd, None)?;
        if response["activePermissionProfile"]["id"] != PROFILE
            || response["approvalPolicy"] != "never"
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native decision thread did not retain its restrictive profile",
            ));
        }
        Ok(native_ref)
    }
    /// Verify every native inventory page, including errors and plugin servers.
    /// An empty configured table is not proof that inherited tools were removed.
    pub fn verify_inventory_page(&self, response: &Value) -> AdapterResult<Option<String>> {
        let data = response["data"]
            .as_array()
            .ok_or_else(|| failure(ErrorKind::ParseFailure, "invalid native MCP inventory"))?;
        if data.len() > 100 {
            return Err(failure(
                ErrorKind::ParseFailure,
                "native inventory page exceeds requested limit",
            ));
        }
        for server in data {
            let tools = server
                .get("tools")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    failure(ErrorKind::ParseFailure, "native tool inventory unavailable")
                })?;
            if !tools.is_empty() || server.get("toolsError").is_some_and(|v| !v.is_null()) {
                return Err(failure(
                    ErrorKind::UnsupportedCapability,
                    "native decision tool absence could not be verified",
                ));
            }
        }
        match response.get("nextCursor") {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(cursor)) if !cursor.is_empty() && cursor.len() <= 4096 => {
                Ok(Some(cursor.clone()))
            }
            _ => Err(failure(
                ErrorKind::ParseFailure,
                "invalid native inventory cursor",
            )),
        }
    }
}

pub fn verify_thread_identity(
    response: &Value,
    cwd: &Path,
    expected: Option<&str>,
) -> AdapterResult<String> {
    let id = response["thread"]["id"]
        .as_str()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .ok_or_else(|| failure(ErrorKind::ParseFailure, "invalid native thread UUID"))?;
    if response["cwd"].as_str() != cwd.to_str()
        || response["thread"]["cwd"].as_str() != cwd.to_str()
        || expected.is_some_and(|expected| expected != id)
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "native thread CWD/UUID does not match its runtime owner",
        ));
    }
    Ok(id.to_owned())
}

/// Native settings come from a trusted runtime snapshot. Project values remain
/// isolated, including credentials whose names are also native provider names.
/// No values are persisted/logged, and Project references cannot replace native
/// HOME, config, policy, proxy, executable or Git routing settings.
pub fn native_environment(
    baseline: impl IntoIterator<Item = (OsString, OsString)>,
    projects: &[Project],
    current: &Project,
    provided: &BTreeMap<String, String>,
) -> AdapterResult<Vec<(OsString, OsString)>> {
    crate::project::validate_environment(current).map_err(|_| {
        failure(
            ErrorKind::InvalidInput,
            "invalid Project environment references",
        )
    })?;
    let own: BTreeSet<_> = current
        .environment_refs
        .iter()
        .map(String::as_str)
        .collect();
    if provided.iter().any(|(key, value)| {
        !own.contains(key.as_str()) || key.contains(['=', '\0']) || value.contains('\0')
    }) {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "launch environment contains undeclared Project values",
        ));
    }
    let scoped: BTreeSet<_> = projects
        .iter()
        .flat_map(|p| p.environment_refs.iter().map(String::as_str))
        .collect();
    let mut result: BTreeMap<OsString, OsString> = baseline
        .into_iter()
        .filter(|(name, _)| {
            let Some(name) = name.to_str() else {
                return false;
            };
            if name.starts_with("GIT_") || (scoped.contains(name) && !own.contains(name)) {
                return false;
            }
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
        })
        .collect();
    for (name, value) in provided {
        result.insert(name.into(), value.into());
    }
    Ok(result.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project(name: &str) -> Project {
        Project::new(
            name.into(),
            format!("/tmp/{name}").into(),
            name.into(),
            "main".into(),
        )
    }
    fn effective() -> Value {
        let mut value = json!({"web_search":"disabled","features":{},"mcp_servers":{"known":{"enabled":false}},"permissions":{PROFILE:{"filesystem":{":root":"deny",":minimal":"read",":slash_tmp":"deny",":tmpdir":"deny",":workspace_roots":{".":"read"}},"network":{"enabled":false}}}});
        for feature in DISABLED_FEATURES {
            value["features"][*feature] = json!(false);
        }
        value
    }
    #[test]
    fn policy_restricts_real_inherited_servers_and_rejects_ambiguous_override_names() {
        let policy = DecisionPolicy::from_native(&json!({"mcp_servers":{"known":{}}})).unwrap();
        assert!(
            policy
                .arguments()
                .contains(&"mcp_servers.known.enabled=false".into())
        );
        assert!(policy.arguments().iter().all(|arg| !arg.contains("hooks")
            && !arg.contains("ignore")
            && !arg.contains("dangerously")));
        policy.verify_configuration(&effective()).unwrap();
        for feature in DISABLED_FEATURES {
            let mut wrong = effective();
            wrong["features"][*feature] = json!(true);
            assert!(policy.verify_configuration(&wrong).is_err());
        }
        let mut wrong = effective();
        wrong["mcp_servers"]["known"]["enabled"] = json!(true);
        assert!(policy.verify_configuration(&wrong).is_err());
        let mut wrong = effective();
        wrong["permissions"][PROFILE]["filesystem"][":tmpdir"] = json!("read");
        assert!(policy.verify_configuration(&wrong).is_err());
        for name in ["", "contains.dot", "quote\"", "x\n"] {
            assert!(DecisionPolicy::from_native(&json!({"mcp_servers":{name:{}}})).is_err());
        }
    }
    #[test]
    fn inventory_errors_unknown_rosters_and_surviving_tools_fail_closed() {
        let policy = DecisionPolicy::from_native(&json!({})).unwrap();
        assert_eq!(
            policy
                .verify_inventory_page(
                    &json!({"data":[{"tools":{},"toolsError":null}],"nextCursor":"opaque"})
                )
                .unwrap(),
            Some("opaque".into())
        );
        for data in [
            json!([{"tools":{"write":{}}}]),
            json!([{"tools":{},"toolsError":"discovery failure"}]),
            json!([{}]),
        ] {
            assert!(policy.verify_inventory_page(&json!({"data":data})).is_err());
        }
    }
    #[test]
    fn native_cwd_uuid_and_effective_decision_profile_are_checked() {
        let policy = DecisionPolicy::from_native(&json!({})).unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let own = json!({"thread":{"id":id,"cwd":"/tmp/own"},"cwd":"/tmp/own","activePermissionProfile":{"id":PROFILE},"approvalPolicy":"never"});
        assert_eq!(
            policy.verify_thread(&own, Path::new("/tmp/own")).unwrap(),
            id
        );
        for field in ["cwd", "approvalPolicy"] {
            let mut wrong = own.clone();
            wrong[field] = json!("foreign");
            assert!(policy.verify_thread(&wrong, Path::new("/tmp/own")).is_err());
        }
        assert!(verify_thread_identity(&own, Path::new("/tmp/own"), Some("other-uuid")).is_err());
        let mut wrong = own;
        wrong["activePermissionProfile"]["id"] = json!("builtin-workspace-write");
        assert!(policy.verify_thread(&wrong, Path::new("/tmp/own")).is_err());
    }
    #[test]
    fn native_settings_survive_but_foreign_project_credentials_do_not() {
        let mut a = project("a");
        a.environment_refs = vec!["OPENAI_API_KEY".into(), "ONLY_A".into()];
        let b = project("b");
        let baseline = [
            ("HOME", "native-home"),
            ("CODEX_HOME", "native-config"),
            ("HTTPS_PROXY", "native-proxy"),
            ("OPENAI_API_KEY", "a-secret"),
            ("ONLY_A", "a-secret"),
            ("GIT_DIR", "foreign-route"),
            ("ARBITRARY_SECRET", "unrelated"),
        ];
        let pairs = || {
            baseline
                .iter()
                .map(|(k, v)| (OsString::from(k), OsString::from(v)))
        };
        let filtered: BTreeMap<_, _> =
            native_environment(pairs(), &[a.clone(), b.clone()], &b, &BTreeMap::new())
                .unwrap()
                .into_iter()
                .collect();
        assert_eq!(
            filtered.get(&OsString::from("HOME")),
            Some(&OsString::from("native-home"))
        );
        assert!(
            filtered.contains_key(&OsString::from("CODEX_HOME"))
                && filtered.contains_key(&OsString::from("HTTPS_PROXY"))
        );
        for key in ["OPENAI_API_KEY", "ONLY_A", "GIT_DIR", "ARBITRARY_SECRET"] {
            assert!(!filtered.contains_key(&OsString::from(key)));
        }
        let provided = BTreeMap::from([("OPENAI_API_KEY".into(), "current-a-value".into())]);
        let filtered: BTreeMap<_, _> =
            native_environment(pairs(), &[a.clone(), b.clone()], &a, &provided)
                .unwrap()
                .into_iter()
                .collect();
        assert_eq!(
            filtered.get(&OsString::from("OPENAI_API_KEY")),
            Some(&OsString::from("current-a-value"))
        );
        assert!(native_environment(pairs(), &[a, b.clone()], &b, &provided).is_err());
    }
}
