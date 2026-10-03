//! Restrict capabilities without replacing native authentication/settings/rules.
use super::protocol::failure;
use crate::{
    adapter::{AdapterResult, ErrorKind, LaunchRequest},
    domain::{Project, SessionRole},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
};

pub(super) fn decision(role: SessionRole) -> bool {
    role != SessionRole::Executor
}
pub(super) fn arguments(
    request: &LaunchRequest,
    native: &str,
    resume: bool,
) -> AdapterResult<Vec<String>> {
    if uuid::Uuid::parse_str(native).is_err() {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "invalid owned native UUID",
        ));
    }
    let mut args = vec![
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    args.push(format!(
        "{}={native}",
        if resume { "--resume" } else { "--session-id" }
    ));
    if decision(request.role) {
        args.extend(
            [
                "--tools",
                "",
                "--disallowedTools",
                "*",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--permission-prompts",
                "none",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    } else {
        args.extend(
            ["--permission-prompt-tool", "stdio"]
                .into_iter()
                .map(str::to_owned),
        );
    }
    if let Some(model) = &request.model {
        args.push(format!("--model={model}"));
    }
    if let Some(effort) = &request.effort {
        if !matches!(effort.as_str(), "low" | "medium" | "high" | "xhigh" | "max") {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native effort unsupported by installed Claude",
            ));
        }
        args.push(format!("--effort={effort}"));
    }
    Ok(args)
}
const OS_IDENTITY: &[&str] = &[
    "USER",
    "LOGNAME",
    "SECURITYSESSIONID",
    "__CF_USER_TEXT_ENCODING",
    "XPC_SERVICE_NAME",
    "XPC_FLAGS",
];
pub(super) fn environment(
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
    if projects.iter().chain(std::iter::once(current)).any(|p| {
        p.environment_refs
            .iter()
            .any(|name| OS_IDENTITY.contains(&name.as_str()))
    }) {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "Project reference cannot replace trusted native OS login identity",
        ));
    }
    let own: BTreeSet<_> = current
        .environment_refs
        .iter()
        .map(String::as_str)
        .collect();
    if provided
        .iter()
        .any(|(k, v)| !own.contains(k.as_str()) || k.contains(['=', '\0']) || v.contains('\0'))
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "undeclared scoped environment value",
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
            OS_IDENTITY.contains(&name)
                || matches!(
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
                        | "ANTHROPIC_API_KEY"
                        | "OPENAI_API_KEY"
                        | "SSH_AUTH_SOCK"
                        | "SSH_ASKPASS"
                        | "EDITOR"
                        | "VISUAL"
                        | "REQUESTS_CA_BUNDLE"
                        | "CURL_CA_BUNDLE"
                        | "GOOGLE_APPLICATION_CREDENTIALS"
                        | "GOOGLE_CLOUD_PROJECT"
                        | "GCLOUD_PROJECT"
                        | "CLOUD_ML_REGION"
                        | "GH_TOKEN"
                        | "GITHUB_TOKEN"
                        | "DISABLE_PROMPT_CACHING"
                        | "DISABLE_PROMPT_CACHING_HAIKU"
                        | "DISABLE_PROMPT_CACHING_SONNET"
                        | "DISABLE_PROMPT_CACHING_OPUS"
                        | "ENABLE_PROMPT_CACHING_1H"
                        | "DISABLE_NON_ESSENTIAL_MODEL_CALLS"
                )
                || name.starts_with("CLAUDE_")
                || name.starts_with("ANTHROPIC_")
                || name.starts_with("AWS_")
                || name.starts_with("AZURE_")
                || name.starts_with("CLOUDSDK_")
                || name.starts_with("VERTEX_REGION_")
                || name.starts_with("LC_")
                || name.starts_with("XDG_")
                || name.to_ascii_uppercase().ends_with("_PROXY")
        })
        .collect();
    for (k, v) in provided {
        result.insert(k.into(), v.into());
    }
    // Official SDK's additive request for authoritative native run-state events.
    // A user's explicit existing value remains native authority.
    result
        .entry("CLAUDE_CODE_SDK_READS_SESSION_STATE".into())
        .or_insert_with(|| "1".into());
    Ok(result.into_iter().collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn other_project_api_key_is_not_a_native_baseline_escape() {
        let mut a = Project::new("a".into(), "/a".into(), "identity".into(), "main".into());
        a.environment_refs = vec!["ANTHROPIC_API_KEY".into()];
        let b = Project::new("b".into(), "/b".into(), "identity".into(), "main".into());
        let baseline = || {
            [
                "HOME",
                "CLAUDE_CONFIG_DIR",
                "ANTHROPIC_AUTH_TOKEN",
                "HTTPS_PROXY",
                "ANTHROPIC_API_KEY",
                "GIT_DIR",
                "OTHER_SECRET",
            ]
            .into_iter()
            .map(|k| (OsString::from(k), OsString::from("private-value")))
        };
        let filtered: BTreeMap<_, _> =
            environment(baseline(), &[a.clone(), b.clone()], &b, &BTreeMap::new())
                .unwrap()
                .into_iter()
                .collect();
        for name in ["ANTHROPIC_API_KEY", "GIT_DIR", "OTHER_SECRET"] {
            assert!(!filtered.contains_key(&OsString::from(name)));
        }
        for name in [
            "HOME",
            "CLAUDE_CONFIG_DIR",
            "ANTHROPIC_AUTH_TOKEN",
            "HTTPS_PROXY",
        ] {
            assert!(filtered.contains_key(&OsString::from(name)));
        }
        let provided = BTreeMap::from([("ANTHROPIC_API_KEY".into(), "own-value".into())]);
        let filtered: BTreeMap<_, _> =
            environment(baseline(), &[a.clone(), b.clone()], &a, &provided)
                .unwrap()
                .into_iter()
                .collect();
        assert_eq!(
            filtered.get(&OsString::from("ANTHROPIC_API_KEY")),
            Some(&OsString::from("own-value"))
        );
        assert!(environment(baseline(), &[a, b.clone()], &b, &provided).is_err());
    }
    #[test]
    fn project_cannot_override_trusted_os_login_context() {
        let mut own = Project::new(
            "own".into(),
            "/own".into(),
            "identity".into(),
            "main".into(),
        );
        for name in OS_IDENTITY {
            own.environment_refs = vec![(*name).into()];
            let provided = BTreeMap::from([((*name).into(), "foreign-identity".into())]);
            assert_eq!(
                environment(
                    [(OsString::from(name), OsString::from("trusted-identity"))],
                    std::slice::from_ref(&own),
                    &own,
                    &provided
                )
                .unwrap_err()
                .kind,
                ErrorKind::OwnershipMismatch
            );
        }
    }
}
