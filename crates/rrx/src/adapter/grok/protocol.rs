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
}
impl Rpc {
    pub fn new(writer: ChildStdin, reader: ChildStdout) -> Self {
        Self {
            writer,
            reader: BufReader::new(reader),
            next: 1,
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
        let mut frame = vec![];
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
            if frame.len().saturating_add(length) > FRAME_LIMIT {
                return Err(failure(ErrorKind::ParseFailure, "ACP frame exceeds budget"));
            }
            let end = chunk[length - 1] == b'\n';
            frame.extend_from_slice(&chunk[..length]);
            self.reader.consume(length);
            if end {
                break;
            }
        }
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
    pub fn callback(&mut self, root: &Path, path: &str) {
        self.callbacks += 1;
        let path = absolute(root, path);
        for tool in self
            .tools
            .values_mut()
            .filter(|t| !t.finished && t.path == path)
        {
            tool.callbacks += 1;
        }
    }
    pub fn update(
        &mut self,
        root: &Path,
        params: &Value,
        native: &str,
        prompt: &str,
        decision: bool,
    ) -> AdapterResult<Option<String>> {
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
            _ => Ok(None),
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
