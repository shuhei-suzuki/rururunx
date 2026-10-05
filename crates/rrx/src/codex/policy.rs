//! Native controls for scoped decision roles. These are verified before inference.
use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};

use super::protocol::failure;
use crate::adapter::{AdapterResult, ErrorKind};

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
    execute: bool,
    approval: Value,
    reviewer: Option<String>,
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
        Ok(Self {
            disabled_servers,
            execute: false,
            approval: json!("never"),
            reviewer: None,
        })
    }
    /// Task execution retains native rule prompts/approval reviewer, while refusing
    /// sandbox escapes and permission-profile expansion outside the Task boundary.
    pub fn for_executor(config: &Value) -> AdapterResult<Self> {
        let mut policy = Self::from_native(config)?;
        if !matches!(
            config["sandbox_mode"].as_str(),
            Some("workspace-write" | "danger-full-access")
        ) || config.get("default_permissions").is_some_and(|profile| {
            !profile.is_null() && profile != ":workspace" && profile != ":unrestricted"
        }) {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native executor cannot widen or intersect an unknown/read-only native profile",
            ));
        }
        policy.approval = match &config["approval_policy"] {
            Value::String(value) if value == "never" => json!("never"),
            Value::String(value) if value == "on-request" => {
                json!({"granular":{"sandbox_approval":false,"request_permissions":false,"rules":true,"mcp_elicitations":false,"skill_approval":true}})
            }
            Value::Object(value)
                if value.len() == 1 && value.get("granular").is_some_and(Value::is_object) =>
            {
                let mut approval = config["approval_policy"].clone();
                let granular = approval["granular"]
                    .as_object_mut()
                    .expect("validated granular policy");
                if granular.keys().any(|key| {
                    !matches!(
                        key.as_str(),
                        "sandbox_approval"
                            | "request_permissions"
                            | "rules"
                            | "mcp_elicitations"
                            | "skill_approval"
                    )
                }) || granular.values().any(|value| !value.is_boolean())
                    || !granular.contains_key("rules")
                    || !granular.contains_key("mcp_elicitations")
                {
                    return Err(failure(
                        ErrorKind::UnsupportedCapability,
                        "unsupported native granular policy",
                    ));
                }
                granular.insert("sandbox_approval".into(), json!(false));
                granular.insert("request_permissions".into(), json!(false));
                approval
            }
            _ => {
                return Err(failure(
                    ErrorKind::UnsupportedCapability,
                    "unsupported native executor approval policy",
                ));
            }
        };
        policy.reviewer = Some(
            config["approvals_reviewer"]
                .as_str()
                .filter(|value| matches!(*value, "user" | "auto_review" | "guardian_subagent"))
                .ok_or_else(|| {
                    failure(
                        ErrorKind::ParseFailure,
                        "native approval reviewer unavailable",
                    )
                })?
                .into(),
        );
        policy.execute = true;
        Ok(policy)
    }
    fn restricted_features(&self) -> impl Iterator<Item = &&'static str> {
        DISABLED_FEATURES.iter().filter(|feature| {
            !self.execute || !matches!(**feature, "shell_tool" | "unified_exec" | "code_mode_host")
        })
    }
    fn filesystem(&self) -> Value {
        json!({":root":"deny",":minimal":"read",":slash_tmp":"deny",":tmpdir":"deny",":workspace_roots":{".":if self.execute {"write"} else {"read"}}})
    }
    pub fn profile_name(&self) -> &'static str {
        if self.execute {
            "rrx_scoped_execute"
        } else {
            PROFILE
        }
    }
    /// A runtime broker may answer the native client route only when the
    /// installed native policy already routes approval to that client. Never
    /// replace native automatic/managed review to obtain callbacks.
    pub fn permits_runtime_broker(&self) -> bool {
        self.execute && self.reviewer.as_deref() == Some("user")
    }
    /// Only restrictive overrides; no rule/hook/authentication bypass switches.
    pub fn arguments(&self) -> Vec<String> {
        let mut args = Vec::new();
        for feature in self.restricted_features() {
            args.extend(["--disable".into(), (*feature).into()]);
        }
        for name in &self.disabled_servers {
            args.extend(["-c".into(), format!("mcp_servers.{name}.enabled=false")]);
        }
        args.extend([
            "-c".into(), "web_search=\"disabled\"".into(),
            "-c".into(), format!("permissions.{}.filesystem={{\":root\"=\"deny\",\":minimal\"=\"read\",\":slash_tmp\"=\"deny\",\":tmpdir\"=\"deny\",\":workspace_roots\"={{\".\"=\"{}\"}}}}",self.profile_name(),if self.execute {"write"} else {"read"}),
            "-c".into(), format!("permissions.{}.network.enabled=false",self.profile_name()),
        ]);
        args
    }
    pub fn thread_parameters(&self, cwd: &Path) -> Value {
        // A decision session cannot request an escape from its read-only profile.
        // Managed policies remain authoritative; rejection is an explicit failure.
        let environments = if self.execute {
            json!([{"environmentId":"local","cwd":cwd,"runtimeWorkspaceRoots":[cwd]}])
        } else {
            json!([])
        };
        json!({"cwd":cwd,"permissions":self.profile_name(),"approvalPolicy":self.approval,"environments":environments,"runtimeWorkspaceRoots":[cwd],"ephemeral":false})
    }
    pub fn verify_local_environment(&self, status: &Value) -> AdapterResult<()> {
        if self.execute && status["status"] != "ready" {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native local execution environment is not ready; no remote fallback",
            ));
        }
        Ok(())
    }
    pub fn verify_configuration(&self, effective: &Value) -> AdapterResult<()> {
        if effective["web_search"] != "disabled"
            || self
                .restricted_features()
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
        let mut filesystem = effective["permissions"][self.profile_name()]["filesystem"].clone();
        // Installed config/read normalizes the optional scan-depth metadata to
        // null. It is not an extra path grant; every actual path rule still matches.
        if let Some(object) = filesystem.as_object_mut()
            && object.get("glob_scan_max_depth") == Some(&Value::Null)
        {
            object.remove("glob_scan_max_depth");
        }
        if filesystem != self.filesystem()
            || effective["permissions"][self.profile_name()]["network"]["enabled"] != false
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
        if response["activePermissionProfile"]["id"] != self.profile_name()
            || response["approvalPolicy"] != self.approval
            || self
                .reviewer
                .as_ref()
                .is_some_and(|reviewer| response["approvalsReviewer"] != reviewer.as_str())
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
    let cwd_text = cwd
        .to_str()
        .filter(|_| cwd.is_absolute())
        .ok_or_else(|| failure(ErrorKind::InvalidInput, "native CWD must be absolute UTF-8"))?;
    if response["cwd"].as_str() != Some(cwd_text)
        || response["thread"]["cwd"].as_str() != Some(cwd_text)
        || expected.is_some_and(|expected| expected != id)
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "native thread CWD/UUID does not match its runtime owner",
        ));
    }
    Ok(id.to_owned())
}

pub(super) fn verify_auth_readiness(account: &Value) -> AdapterResult<()> {
    let required = account["requiresOpenaiAuth"].as_bool().ok_or_else(|| {
        failure(
            ErrorKind::ParseFailure,
            "native authentication readiness unavailable",
        )
    })?;
    if required && account.get("account").is_none_or(Value::is_null) {
        return Err(failure(
            ErrorKind::AuthenticationUnavailable,
            "native Codex authentication unavailable; use its normal login route",
        ));
    }
    if account
        .get("account")
        .is_some_and(|value| !value.is_null() && !value.is_object())
    {
        return Err(failure(
            ErrorKind::ParseFailure,
            "invalid native account readiness",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut normalized = effective();
        normalized["permissions"][PROFILE]["filesystem"]["glob_scan_max_depth"] = Value::Null;
        policy.verify_configuration(&normalized).unwrap();
        normalized["permissions"][PROFILE]["filesystem"]["/tmp/foreign-project"] = json!("read");
        assert!(policy.verify_configuration(&normalized).is_err());
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
    fn executor_preserves_native_reviewer_and_rule_prompts_without_profile_expansion() {
        let mut config = json!({"mcp_servers":{"known":{}},"sandbox_mode":"workspace-write","approval_policy":"on-request","approvals_reviewer":"auto_review"});
        let policy = DecisionPolicy::for_executor(&config).unwrap();
        assert!(!policy.permits_runtime_broker());
        let mut client_config = config.clone();
        client_config["approvals_reviewer"] = json!("user");
        let client = DecisionPolicy::for_executor(&client_config).unwrap();
        assert!(client.permits_runtime_broker());
        assert!(
            !client
                .arguments()
                .iter()
                .any(|argument| argument.contains("approvals_reviewer"))
        );
        assert!(
            client
                .thread_parameters(Path::new("/own"))
                .get("approvalsReviewer")
                .is_none()
        );
        assert_eq!(
            policy.thread_parameters(Path::new("/own"))["environments"],
            json!([{"environmentId":"local","cwd":"/own","runtimeWorkspaceRoots":["/own"]}])
        );
        policy
            .verify_local_environment(&json!({"status":"ready"}))
            .unwrap();
        assert!(
            policy
                .verify_local_environment(&json!({"status":"unknown"}))
                .is_err()
        );
        assert!(
            !policy
                .arguments()
                .iter()
                .any(|arg| arg == "shell_tool" || arg == "unified_exec" || arg == "code_mode_host")
        );
        assert_eq!(
            policy.thread_parameters(Path::new("/own"))["approvalPolicy"]["granular"]["sandbox_approval"],
            false
        );
        assert_eq!(
            policy.thread_parameters(Path::new("/own"))["approvalPolicy"]["granular"]["request_permissions"],
            false
        );
        assert_eq!(
            policy.thread_parameters(Path::new("/own"))["approvalPolicy"]["granular"]["rules"],
            true
        );
        let mut effective = effective();
        effective["features"]["shell_tool"] = json!(true);
        effective["features"]["unified_exec"] = json!(true);
        effective["permissions"][policy.profile_name()] =
            json!({"filesystem":policy.filesystem(),"network":{"enabled":false}});
        policy.verify_configuration(&effective).unwrap();
        let mut response = json!({"cwd":"/own","thread":{"id":"01a10085-deba-7831-8b1b-f302ca6c6da8","cwd":"/own"},"activePermissionProfile":{"id":policy.profile_name()},"approvalPolicy":policy.approval,"approvalsReviewer":"auto_review"});
        policy.verify_thread(&response, Path::new("/own")).unwrap();
        response["approvalsReviewer"] = json!("user");
        assert!(policy.verify_thread(&response, Path::new("/own")).is_err());
        config["sandbox_mode"] = json!("read-only");
        assert!(DecisionPolicy::for_executor(&config).is_err());
        config["sandbox_mode"] = json!("workspace-write");
        config["default_permissions"] = json!("managed-custom");
        assert!(DecisionPolicy::for_executor(&config).is_err());
        config["default_permissions"] = Value::Null;
        config["approval_policy"] = json!({"granular":{"sandbox_approval":true,"request_permissions":true,"rules":false,"mcp_elicitations":false,"skill_approval":false}});
        let policy = DecisionPolicy::for_executor(&config).unwrap();
        assert_eq!(policy.approval["granular"]["rules"], false);
        assert_eq!(policy.approval["granular"]["skill_approval"], false);
        config["approval_policy"]["granular"]["unknown_future_grant"] = json!(true);
        assert!(DecisionPolicy::for_executor(&config).is_err());
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
    fn native_auth_readiness_preserves_custom_no_auth_and_missing_login_outcomes() {
        verify_auth_readiness(&json!({"requiresOpenaiAuth":false,"account":null})).unwrap();
        verify_auth_readiness(&json!({"requiresOpenaiAuth":true,"account":{"type":"apiKey"}}))
            .unwrap();
        for value in [
            json!({"requiresOpenaiAuth":true}),
            json!({"requiresOpenaiAuth":true,"account":null}),
        ] {
            assert_eq!(
                verify_auth_readiness(&value).unwrap_err().kind,
                ErrorKind::AuthenticationUnavailable
            );
        }
        for value in [
            json!({}),
            json!({"requiresOpenaiAuth":"unknown"}),
            json!({"requiresOpenaiAuth":false,"account":"unexpected"}),
        ] {
            assert_eq!(
                verify_auth_readiness(&value).unwrap_err().kind,
                ErrorKind::ParseFailure
            );
        }
    }
}
