//! Finite managed command forms. Absolute/raw invocations remain uncovered.
use super::{
    resources::{ResourceProfile, resolve_program},
    *,
};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(crate) struct ToolPlan {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub serialized_git: bool,
    pub docker_name: Option<String>,
    pub kind: &'static str,
}
pub(crate) fn plan(
    profile: &ResourceProfile,
    unit: &ExecutionUnit,
    tool: &str,
    args: &[String],
    operation: OperationId,
) -> Result<ToolPlan> {
    ensure!(
        args.len() <= 512
            && args.iter().all(|a| a.len() <= 8192 && !a.contains('\0'))
            && args.iter().map(String::len).sum::<usize>() <= 65536,
        "tool input exceeds bound"
    );
    let mut result = ToolPlan {
        program: profile
            .real_tools
            .get(tool)
            .context("tool is absent from resource profile")?
            .clone(),
        args: vec![],
        environment: BTreeMap::new(),
        serialized_git: false,
        docker_name: None,
        kind: "local_tool",
    };
    let deny = |forbidden: &[&str]| -> Result<()> {
        ensure!(
            !args.iter().any(|a| forbidden
                .iter()
                .any(|f| a == f || a.starts_with(&format!("{f}=")))),
            "tool override conflicts with resource profile"
        );
        Ok(())
    };
    ensure!(
        unit.kind != UnitKind::Reviewer || tool == "git",
        "native reviewer managed tool profile is read-only Git"
    );
    match tool {
        "git" => {
            ensure!(
                !args.iter().any(|a| a == "-c"
                    || a.starts_with("--config-env")
                    || a.starts_with("--git-dir")
                    || a.starts_with("--work-tree")
                    || a == "-C"),
                "Git namespace/config override is unsupported"
            );
            let sub = args.first().context("Git subcommand required")?;
            let read = matches!(
                sub.as_str(),
                "status"
                    | "log"
                    | "show"
                    | "diff"
                    | "rev-parse"
                    | "cat-file"
                    | "ls-files"
                    | "ls-tree"
                    | "check-ignore"
                    | "check-attr"
                    | "merge-base"
                    | "rev-list"
                    | "describe"
                    | "--version"
            );
            let read_config = sub == "config"
                && matches!(
                    args.get(1).map(String::as_str),
                    Some("--get" | "--get-all" | "--get-regexp")
                )
                && args.len() == 3
                && !args[2].starts_with('-');
            ensure!(
                !args.iter().any(|a| a.starts_with("--out")
                    || a.starts_with("--textconv")
                    || a.starts_with("--ext-diff")
                    || a == "--no-index"
                    || a == "--paginate"),
                "Git output/external-diff override is unsupported"
            );
            let write = matches!(
                sub.as_str(),
                "add"
                    | "restore"
                    | "checkout"
                    | "switch"
                    | "commit"
                    | "update-ref"
                    | "branch"
                    | "reset"
                    | "rm"
                    | "fetch"
                    | "cherry-pick"
                    | "merge"
            );
            ensure!(
                read || read_config || (write && unit.kind == UnitKind::Executor),
                "Git command is unsupported by this unit profile"
            );
            ensure!(
                !args.iter().any(|a| a == "--shared"
                    || a.starts_with("--reference")
                    || a == "--system"
                    || a == "--global"),
                "shared/global Git operation is unsupported"
            );
            ensure!(
                !args.iter().any(|a| a.starts_with("--auto")
                    || a.starts_with("--recurse")
                    || a == "--stdin"
                    || a == "--filters"),
                "shared/bulk Git effect is unsupported"
            );
            if sub == "update-ref" {
                let own = format!(
                    "refs/heads/{}",
                    unit.branch.as_deref().context("managed branch missing")?
                );
                ensure!(
                    (matches!(args.len(), 3 | 4)
                        && args[1] == own
                        && valid_oid(&args[2])
                        && args.get(3).is_none_or(|s| valid_oid(s)))
                        || (args.len() == 3 && args[1] == "-d" && args[2] == own),
                    "update-ref requires one exact owned ref"
                );
            }
            if matches!(sub.as_str(), "branch" | "switch" | "checkout") {
                owned_branch_form(unit, sub, &args[1..])?;
            }
            result.args.extend([
                "-c".into(),
                "gc.auto=0".into(),
                "-c".into(),
                "maintenance.auto=false".into(),
                "-c".into(),
                "core.fsmonitor=false".into(),
                "-c".into(),
                "merge.autoStash=false".into(),
                "-c".into(),
                "rebase.autoStash=false".into(),
            ]);
            if matches!(sub.as_str(), "diff" | "log" | "show") {
                result
                    .args
                    .extend([sub.clone(), "--no-ext-diff".into(), "--no-textconv".into()]);
                result.args.extend(args.iter().skip(1).cloned());
                return Ok(result);
            }
            if sub == "fetch" {
                ensure!(
                    args.len() == 3 && !args[1].starts_with('-') && valid_oid(&args[2]),
                    "managed fetch requires one source and exact SHA"
                );
                result.args.extend([
                    "fetch".into(),
                    "--no-tags".into(),
                    "--no-write-fetch-head".into(),
                    "--no-recurse-submodules".into(),
                    "--".into(),
                    args[1].clone(),
                    args[2].clone(),
                ]);
                result.serialized_git = true;
                result.kind = "common_git";
                return Ok(result);
            }
            result.serialized_git = write;
            result.kind = if write { "common_git" } else { "local_tool" };
        }
        "gradle" => {
            deny(&["--daemon", "--stop", "-g", "--gradle-user-home"])?;
            ensure!(
                !args
                    .iter()
                    .any(|a| a.starts_with("-Dorg.gradle.daemon")
                        || a.starts_with("-Dgradle.user.home")),
                "Gradle daemon override is unsupported"
            );
            result.args.extend([
                "--no-daemon".into(),
                "--gradle-user-home".into(),
                profile.cache.join("gradle").to_string_lossy().into(),
            ]);
        }
        "bazel" | "bazelisk" => {
            deny(&[
                "--output_base",
                "--output_user_root",
                "--nobatch",
                "shutdown",
            ])?;
            result.args.extend([
                "--batch".into(),
                format!("--output_base={}", profile.cache.join("bazel").display()),
                format!(
                    "--output_user_root={}",
                    profile.cache.join("bazel-root").display()
                ),
            ]);
        }
        "sccache" => {
            let compiler = args
                .first()
                .filter(|c| !c.starts_with('-'))
                .context("only the qualified compile-wrapper bypass is supported")?;
            result.program = if compiler.contains('/') {
                let p = PathBuf::from(compiler);
                ensure!(
                    p.is_absolute(),
                    "compiler must be absolute or PATH-resolved"
                );
                p.canonicalize()?
            } else {
                resolve_program(compiler)?
            };
            result.args.extend(args.iter().skip(1).cloned());
            return Ok(result);
        }
        "tmux" => {
            ensure!(
                !args
                    .iter()
                    .any(|a| a.starts_with("-L") || a.starts_with("-S")),
                "tmux socket override is unsupported"
            );
            result.args.extend([
                "-S".into(),
                profile.root.join("tmux.sock").to_string_lossy().into(),
            ]);
        }
        "ssh" => {
            ensure!(
                !args.iter().any(|a| a.starts_with("-O")
                    || a.starts_with("-M")
                    || a.starts_with("-S")
                    || a.to_ascii_lowercase().contains("controlmaster")
                    || a.to_ascii_lowercase().contains("controlpath")
                    || a.to_ascii_lowercase().contains("controlpersist")),
                "SSH shared control operation is unsupported"
            );
            result.args.extend([
                "-o".into(),
                "ControlMaster=no".into(),
                "-o".into(),
                "ControlPath=none".into(),
                "-o".into(),
                "ControlPersist=no".into(),
            ]);
        }
        "docker" => {
            ensure!(
                unit.kind == UnitKind::Executor,
                "Docker execution requires executor profile"
            );
            let sub = args.first().context("Docker subcommand required")?;
            match sub.as_str() {
                "run" | "create" => {
                    let name = format!(
                        "{}-{}",
                        profile.docker_project,
                        operation.to_string().replace('-', "")
                    );
                    let rest = docker_create_args(profile, &args[1..])?;
                    result.args.push(sub.clone());
                    result.args.extend(["--name".into(), name.clone()]);
                    for (k, v) in &profile.docker_labels {
                        result.args.extend(["--label".into(), format!("{k}={v}")]);
                    }
                    result.args.extend(rest);
                    result.docker_name = Some(name);
                    result.kind = "docker_create";
                    return Ok(result);
                }
                "ps" => {
                    ensure!(
                        args.iter().skip(1).all(|a| matches!(
                            a.as_str(),
                            "-a" | "--all" | "-q" | "--quiet" | "--no-trunc"
                        )),
                        "Docker inventory override is unsupported"
                    );
                    result.args = args.to_vec();
                    for (k, v) in &profile.docker_labels {
                        result
                            .args
                            .extend(["--filter".into(), format!("label={k}={v}")]);
                    }
                    return Ok(result);
                }
                "compose" => bail!(
                    "Compose requires normalized owned configuration; raw compose entry is unsupported"
                ),
                _ => bail!("Docker operation requires a recorded, label-verified resource target"),
            }
        }
        _ => bail!("tool lacks a managed invocation policy"),
    }
    result.args.extend_from_slice(args);
    Ok(result)
}
fn owned_branch_form(unit: &ExecutionUnit, command: &str, args: &[String]) -> Result<()> {
    let branch = unit.branch.as_deref().context("managed branch missing")?;
    let own = |name: &String| name == branch;
    let start = |value: &String| valid_oid(value) || value == "HEAD";
    let allowed = match (command, args) {
        ("branch", []) => true,
        ("branch", [flag]) if matches!(flag.as_str(), "--show-current" | "--list") => true,
        ("branch", [name]) => own(name),
        ("branch", [flag, name]) if matches!(flag.as_str(), "-d" | "-D" | "--delete") => own(name),
        ("branch", [name, oid]) => own(name) && start(oid),
        ("branch", [flag, name, oid]) if matches!(flag.as_str(), "-f" | "--force") => {
            own(name) && start(oid)
        }
        ("switch" | "checkout", [name]) => own(name),
        ("switch", [flag, name]) if matches!(flag.as_str(), "-c" | "-C") => own(name),
        ("switch", [flag, name, oid]) if matches!(flag.as_str(), "-c" | "-C") => {
            own(name) && start(oid)
        }
        ("checkout", [flag, name]) if matches!(flag.as_str(), "-b" | "-B") => own(name),
        ("checkout", [flag, name, oid]) if matches!(flag.as_str(), "-b" | "-B") => {
            own(name) && start(oid)
        }
        ("checkout", [separator, paths @ ..]) if separator == "--" => {
            !paths.is_empty()
                && paths.iter().all(|p| {
                    !Path::new(p).is_absolute()
                        && !Path::new(p)
                            .components()
                            .any(|c| matches!(c, std::path::Component::ParentDir))
                })
        }
        _ => false,
    };
    ensure!(allowed, "unsupported or foreign mutable Git ref form");
    Ok(())
}
fn docker_create_args(profile: &ResourceProfile, args: &[String]) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut index = 0;
    let mut image = false;
    while index < args.len() {
        let a = &args[index];
        if image {
            out.push(a.clone());
            index += 1;
            continue;
        }
        if !a.starts_with('-') {
            ensure!(!a.is_empty(), "Docker image missing");
            image = true;
            out.push(a.clone());
            index += 1;
            continue;
        }
        let (flag, inline) = a
            .split_once('=')
            .map_or((a.as_str(), None), |(a, b)| (a, Some(b)));
        match flag {
            "-d" | "--detach" | "--rm" | "-i" | "--interactive" | "-t" | "--tty" | "--init"
            | "--read-only" => {
                ensure!(inline.is_none(), "unsupported Docker flag form");
                out.push(a.clone());
            }
            "--env" | "-e" | "--entrypoint" | "--user" | "--workdir" | "-w" | "--cpus"
            | "--memory" => {
                let v = match inline {
                    Some(v) => v,
                    None => {
                        index += 1;
                        args.get(index).context("Docker option value missing")?
                    }
                };
                out.extend([flag.into(), v.into()]);
            }
            "--publish" | "-p" => {
                let v = match inline {
                    Some(v) => v,
                    None => {
                        index += 1;
                        args.get(index).context("Docker port missing")?
                    }
                };
                let parts: Vec<_> = v.split(':').collect();
                ensure!(
                    parts.len() == 2,
                    "only exact HOST_PORT:CONTAINER_PORT is supported"
                );
                let host: u16 = parts[0].parse()?;
                let guest: u16 = parts[1].parse()?;
                ensure!(
                    (profile.port_start..=profile.port_end).contains(&host) && guest > 0,
                    "Docker published port outside unit lease"
                );
                out.extend(["--publish".into(), format!("127.0.0.1:{host}:{guest}")]);
            }
            "--volume" | "-v" => {
                let v = match inline {
                    Some(v) => v,
                    None => {
                        index += 1;
                        args.get(index).context("Docker mount missing")?
                    }
                };
                let parts: Vec<_> = v.split(':').collect();
                ensure!(
                    matches!(parts.len(), 2 | 3)
                        && parts.get(2).is_none_or(|v| matches!(*v, "ro" | "rw")),
                    "unsupported Docker mount form"
                );
                let host = Path::new(parts[0]);
                ensure!(
                    host.is_absolute() && Path::new(parts[1]).is_absolute(),
                    "Docker mount must be absolute"
                );
                let canonical = host.canonicalize()?;
                ensure!(
                    canonical == host
                        && (canonical.starts_with(&profile.worktree)
                            || canonical.starts_with(&profile.temp)
                            || canonical.starts_with(&profile.output)
                            || canonical.starts_with(&profile.cache)),
                    "shared Docker mount is unsupported"
                );
                ensure!(
                    !canonical.to_string_lossy().contains("docker.sock"),
                    "raw Docker socket is unsupported"
                );
                out.extend(["--volume".into(), v.into()]);
            }
            _ => bail!("unsupported Docker option in managed run/create"),
        }
        index += 1;
    }
    ensure!(image, "Docker image missing");
    Ok(out)
}
