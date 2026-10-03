//! Bounded, sequential ACP JSON-RPC; no daemon adoption or arbitrary metadata forwarding.
use super::{AdapterResult, ErrorKind, failure};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};

pub(super) const FRAME_LIMIT: usize = 1_048_576;
pub(super) struct Rpc {
    writer: ChildStdin,
    reader: BufReader<ChildStdout>,
    next: u64,
    partial: Vec<u8>,
}
impl Rpc {
    pub fn new(writer: ChildStdin, reader: ChildStdout) -> Self {
        Self {
            writer,
            reader: BufReader::new(reader),
            next: 1,
            partial: vec![],
        }
    }
    pub fn frame(&mut self, method: &str, params: Value) -> AdapterResult<(u64, Vec<u8>)> {
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| failure(ErrorKind::ParseFailure, "RPC ID exhausted"))?;
        Ok((
            id,
            encode(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?,
        ))
    }
    pub async fn send(&mut self, frame: &[u8]) -> AdapterResult<()> {
        tokio::time::timeout(Duration::from_secs(2), self.writer.write_all(frame))
            .await
            .map_err(|_| failure(ErrorKind::Timeout, "ACP pipe write timed out"))?
            .map_err(|_| failure(ErrorKind::ProcessFailure, "ACP pipe write failed"))
    }
    pub async fn receive(&mut self) -> AdapterResult<Value> {
        loop {
            let chunk = self
                .reader
                .fill_buf()
                .await
                .map_err(|_| failure(ErrorKind::ProcessFailure, "ACP read failed"))?;
            if chunk.is_empty() {
                return Err(failure(ErrorKind::ProcessFailure, "ACP stream closed"));
            }
            let length = chunk
                .iter()
                .position(|b| *b == b'\n')
                .map_or(chunk.len(), |p| p + 1);
            if self.partial.len().saturating_add(length) > FRAME_LIMIT {
                return Err(failure(ErrorKind::ParseFailure, "ACP frame exceeds budget"));
            }
            let end = chunk[length - 1] == b'\n';
            self.partial.extend_from_slice(&chunk[..length]);
            self.reader.consume(length);
            if end {
                break;
            }
        }
        let frame = std::mem::take(&mut self.partial);
        let value: Value = serde_json::from_slice(&frame)
            .map_err(|_| failure(ErrorKind::ParseFailure, "malformed ACP JSON"))?;
        if value["jsonrpc"] != "2.0" || !value.is_object() {
            return Err(failure(ErrorKind::ParseFailure, "invalid ACP envelope"));
        }
        Ok(value)
    }
    pub async fn reply(&mut self, id: Value, result: AdapterResult<Value>) -> AdapterResult<()> {
        if !(id.as_u64().is_some() || id.as_str().is_some_and(|s| s.len() <= 128)) {
            return Err(failure(
                ErrorKind::ParseFailure,
                "invalid reverse request ID",
            ));
        }
        let value = match result {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err(error) => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32001,"message":format!("rrx scoped capability denied: {:?}",error.kind)}})
            }
        };
        self.send(&encode(value)?).await
    }
}
fn encode(value: Value) -> AdapterResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec(&value)
        .map_err(|_| failure(ErrorKind::ParseFailure, "ACP encoding failed"))?;
    if bytes.len() + 1 > FRAME_LIMIT {
        return Err(failure(
            ErrorKind::InvalidInput,
            "ACP request exceeds frame budget",
        ));
    }
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Default)]
struct Tool {
    path: PathBuf,
    callbacks: usize,
    finished: bool,
    write: bool,
}
#[derive(Default)]
pub(super) struct TurnEvidence {
    tools: BTreeMap<String, Tool>,
    pub callbacks: usize,
}
/// Lexical correlation only; never used to authorize filesystem access.
fn absolute(root: &Path, path: &str) -> PathBuf {
    let mut result = if Path::new(path).is_absolute() {
        PathBuf::new()
    } else {
        root.to_path_buf()
    };
    for part in Path::new(path).components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    result
}
impl TurnEvidence {
    pub fn callback(
        &mut self,
        root: &Path,
        path: &str,
        method: &str,
        succeeded: bool,
    ) -> AdapterResult<()> {
        self.callbacks += 1;
        if !succeeded {
            return Ok(());
        }
        let path = absolute(root, path);
        let write = method == "fs/write_text_file";
        let mut matching = self
            .tools
            .values_mut()
            .filter(|t| !t.finished && t.path == path && t.write == write);
        let target = matching.next();
        if matching.next().is_some() {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "ambiguous concurrent native file tool callbacks",
            ));
        }
        if let Some(tool) = target {
            tool.callbacks += 1;
        } else if !write {
            // search_replace may read its target before writing. Native ACP
            // also issues supplemental reads at already successful tool paths.
            // Neither kind of read grants new tool or write completion credit.
            let mut dependencies = self
                .tools
                .values()
                .filter(|t| !t.finished && t.path == path && t.write);
            let pending_write = dependencies.next().is_some();
            let ambiguous = dependencies.next().is_some();
            let prior_successful = self
                .tools
                .values()
                .any(|t| t.path == path && t.finished && t.callbacks > 0);
            if ambiguous || (!pending_write && !prior_successful) {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    format!(
                        "unowned native file read callback; prior_successful_tool_path={}",
                        prior_successful
                    ),
                ));
            }
        } else {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "unowned native file write callback",
            ));
        }
        Ok(())
    }
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
    pub fn update(
        &mut self,
        root: &Path,
        params: &Value,
        native: &str,
        prompt: &str,
        decision: bool,
    ) -> AdapterResult<Option<String>> {
        // Native title/summary housekeeping is Session-scoped, not a prompt
        // event. It never feeds output, tool evidence, usage or completion.
        if params["update"]["sessionUpdate"] == "session_info_update" {
            return if params["sessionId"] == native {
                Ok(None)
            } else {
                Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "foreign native Session housekeeping",
                ))
            };
        }
        if params["sessionId"] != native || params["_meta"]["promptId"] != prompt {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                format!(
                    "foreign/replayed ACP notification kind={:?} session_matches={} prompt_matches={} prompt_present={}",
                    params["update"]["sessionUpdate"].as_str(),
                    params["sessionId"] == native,
                    params["_meta"]["promptId"] == prompt,
                    params["_meta"].get("promptId").is_some()
                ),
            ));
        }
        let update = &params["update"];
        match update["sessionUpdate"].as_str() {
            Some("tool_call" | "tool_call_update") => {
                if decision {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "decision session attempted a model tool",
                    ));
                }
                let id = update["toolCallId"]
                    .as_str()
                    .filter(|s| s.len() <= 256)
                    .ok_or_else(|| failure(ErrorKind::ParseFailure, "missing tool identity"))?;
                let name = update["_meta"]["x.ai/tool"]["name"].as_str();
                if let Some(name) = name
                    && !["read_file", "search_replace"].contains(&name)
                {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "native tool outside curated contract",
                    ));
                }
                if update["sessionUpdate"] == "tool_call" {
                    let name = name.ok_or_else(|| {
                        failure(ErrorKind::ParseFailure, "native tool name evidence missing")
                    })?;
                    let field = if name == "read_file" {
                        "target_file"
                    } else {
                        "file_path"
                    };
                    let path = update["rawInput"][field].as_str().ok_or_else(|| {
                        failure(ErrorKind::ParseFailure, "native file tool path missing")
                    })?;
                    if self.tools.len() >= 128 || self.tools.contains_key(id) {
                        return Err(failure(
                            ErrorKind::ParseFailure,
                            "duplicate/over-budget native tool",
                        ));
                    }
                    self.tools.insert(
                        id.to_owned(),
                        Tool {
                            path: absolute(root, path),
                            write: name == "search_replace",
                            ..Tool::default()
                        },
                    );
                } else {
                    let tool = self
                        .tools
                        .get_mut(id)
                        .ok_or_else(|| failure(ErrorKind::ParseFailure, "unowned tool update"))?;
                    if update["status"] == "completed" || update["status"] == "failed" {
                        if update["status"] == "completed" && tool.callbacks == 0 {
                            return Err(failure(
                                ErrorKind::OwnershipMismatch,
                                "native successful file tool bypassed ACP filesystem",
                            ));
                        }
                        tool.finished = true;
                    }
                }
                Ok(None)
            }
            Some("agent_message_chunk") => {
                if update["content"]["type"] == "text" {
                    Ok(update["content"]["text"].as_str().map(str::to_owned))
                } else {
                    Ok(None)
                }
            }
            Some("agent_thought_chunk" | "plan" | "usage_update" | "available_commands_update") => {
                Ok(None)
            }
            _ => Err(failure(
                ErrorKind::OwnershipMismatch,
                "unsupported native live notification",
            )),
        }
    }
    pub fn finished(&self) -> AdapterResult<()> {
        if self.tools.values().any(|t| !t.finished) {
            Err(failure(
                ErrorKind::ParseFailure,
                "incomplete native tool evidence",
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_title_housekeeping_never_grants_prompt_or_tool_authority() {
        let mut evidence = TurnEvidence::default();
        let root = Path::new("/owned/task");
        let mut params = json!({"sessionId":"native","update":{"sessionUpdate":"session_info_update","title":"native title"}});
        assert!(
            evidence
                .update(root, &params, "native", "prompt", true)
                .unwrap()
                .is_none()
        );
        assert_eq!(evidence.callbacks, 0);
        evidence.finished().unwrap();
        params["sessionId"] = json!("foreign");
        assert!(
            evidence
                .update(root, &params, "native", "prompt", true)
                .is_err()
        );
        params["sessionId"] = json!("native");
        params["update"]["sessionUpdate"] = json!("tool_call");
        assert!(
            evidence
                .update(root, &params, "native", "prompt", false)
                .is_err()
        );
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[tokio::test]
    async fn cancelled_receive_retains_already_consumed_partial_frame() {
        let directory = tempfile::tempdir().unwrap();
        let ready = directory.path().join("ready");
        let go = directory.path().join("go");
        let mut child = tokio::process::Command::new("python3")
            .args(["-u", "-c", "import sys,time,pathlib\nsys.stdout.write('{\"jsonrpc\":\"2.0\",');sys.stdout.flush();pathlib.Path(sys.argv[1]).touch()\nwhile not pathlib.Path(sys.argv[2]).exists():time.sleep(0.001)\nprint('\"id\":1,\"result\":{}}')"])
            .arg(&ready).arg(&go).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).kill_on_drop(true).spawn().unwrap();
        let mut rpc = Rpc::new(child.stdin.take().unwrap(), child.stdout.take().unwrap());
        tokio::time::timeout(Duration::from_secs(5), async {
            while !ready.exists() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(10), rpc.receive())
                .await
                .is_err()
        );
        std::fs::write(&go, "continue").unwrap();
        let value = tokio::time::timeout(Duration::from_secs(5), rpc.receive())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(value["id"], 1);
        assert!(child.wait().await.unwrap().success());
    }
}
