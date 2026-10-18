//! Small file-controlled fake executables, following adapter.rs's shell harness.
use super::PipelineCli;
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub struct FakeCli {
    pub directory: PathBuf,
    pub cli: PipelineCli,
}
impl FakeCli {
    pub fn new(root: &Path, codex: bool) -> Self {
        let name = if codex { "codex" } else { "claude" };
        let directory = root.join(name);
        std::fs::create_dir_all(&directory).unwrap();
        let executable = directory.join(name);
        std::fs::write(&executable, include_str!("pipeline_cli.sh")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let version = format!("rrx-fixture-{name}-v1");
        let flags = if codex {
            "--sandbox <MODE>\n-c, --config <VALUE>\n-m, --model <MODEL>\n-C, --cd <PATH>\n-o, --output-last-message <FILE>\n--json\n--color <WHEN>\n--output-schema <FILE>\n"
        } else {
            "-p, --print\n--permission-mode <MODE>\n--tools <TOOLS>\n--disallowedTools <TOOLS>\n--strict-mcp-config\n--mcp-config <FILE>\n--setting-sources <SOURCES>\n--disable-slash-commands\n--model <MODEL>\n--output-format <FORMAT>\n--json-schema <SCHEMA>\n"
        };
        for (file, content) in [
            ("version", version.as_str()),
            ("help", flags),
            ("login", "0\n"),
            ("settings-status", "0\n"),
            ("exec-status", "0\n"),
            ("exec-output", "fixture output\n"),
        ] {
            std::fs::write(directory.join(file), content).unwrap();
        }
        let home = directory.join("home");
        std::fs::create_dir(&home).unwrap();
        std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            directory,
            cli: PipelineCli {
                executable,
                version,
                model: "fixture-model".into(),
                effort: codex.then(|| "high".into()),
                codex_home: codex.then_some(home),
            },
        }
    }
    pub fn control(&self, name: &str, content: &str) {
        std::fs::write(self.directory.join(name), content).unwrap();
    }
}
