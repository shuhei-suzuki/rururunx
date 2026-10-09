//! Finite dedicated Runtime control frames; never the native TaskTool endpoint.
use crate::execution::strict_json::{self, Limits};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{io::Write, path::PathBuf, time::Duration};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

pub const PROTOCOL_VERSION: u32 = 3;
/// Protocols whose leftover descriptor (same canonical state, same strict
/// shape) an exclusive owner may replace. Never a live-discovery range.
pub const REPLACEABLE_PROTOCOLS: [u32; 3] = [1, 2, 3];
pub const REQUEST_BYTES: usize = 1024 * 1024;
pub const RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// A service observation, not an executable Human/Driver capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceIdentity {
    pub state: PathBuf,
    pub instance: String,
    pub epoch: u64,
}
impl ServiceIdentity {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.state.is_absolute(), "control state must be absolute");
        ensure!(
            self.state.to_str().is_some_and(|s| s.len() <= 4096)
                && uuid::Uuid::parse_str(&self.instance).is_ok()
                && self.epoch > 0,
            "invalid control service identity"
        );
        Ok(())
    }
}

/// Sent from actual service composition before admitting a mutating request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    pub protocol: u32,
    pub identity: ServiceIdentity,
}
impl Hello {
    pub fn validate_for(&self, expected: &ServiceIdentity) -> Result<()> {
        self.identity.validate()?;
        expected.validate()?;
        ensure!(
            self.protocol == PROTOCOL_VERSION && self.identity == *expected,
            "control service identity changed"
        );
        Ok(())
    }
}

struct LimitedBytes {
    bytes: Vec<u8>,
    maximum: usize,
}
impl Write for LimitedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("control frame limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn limits(maximum: usize) -> Result<Limits> {
    ensure!(
        matches!(maximum, REQUEST_BYTES | RESPONSE_BYTES),
        "unsupported control frame budget"
    );
    Ok(Limits {
        frame_bytes: maximum,
        total_string_bytes: maximum,
        ..Limits::CEILINGS
    })
}

/// JSON serialization is capped while writing, before constructing a full buffer.
pub async fn send<T: Serialize>(
    stream: &mut (impl AsyncWrite + Unpin),
    value: &T,
    maximum: usize,
) -> Result<()> {
    let profile = limits(maximum)?;
    let mut encoded = LimitedBytes {
        bytes: Vec::new(),
        maximum,
    };
    serde_json::to_writer(&mut encoded, value)
        .map_err(|_| anyhow::anyhow!("control encoding or frame limit"))?;
    // Also enforce decoded budgets on outgoing typed projections. They remain data.
    strict_json::decode(&encoded.bytes, profile)?;
    tokio::time::timeout(IO_TIMEOUT, async {
        stream.write_all(&encoded.bytes).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await
    })
    .await
    .context("control write deadline")?
    .context("control write unavailable")?;
    Ok(())
}

/// Read only through a bounded buffered slice. No unbounded read_line/read_until.
pub async fn receive<T: DeserializeOwned>(
    stream: &mut (impl AsyncBufRead + Unpin),
    maximum: usize,
) -> Result<T> {
    let profile = limits(maximum)?;
    let bytes = tokio::time::timeout(IO_TIMEOUT, async {
        let mut bytes = Vec::new();
        loop {
            let available = stream.fill_buf().await?;
            ensure!(
                !available.is_empty(),
                "control frame ended before delimiter"
            );
            let delimiter = available.iter().position(|b| *b == b'\n');
            let take = delimiter.unwrap_or(available.len());
            ensure!(
                take <= maximum.saturating_sub(bytes.len()),
                "control frame byte limit"
            );
            bytes.extend_from_slice(&available[..take]);
            stream.consume(take + usize::from(delimiter.is_some()));
            if delimiter.is_some() {
                return Ok::<_, anyhow::Error>(bytes);
            }
        }
    })
    .await
    .context("control read deadline")??;
    let value = strict_json::decode(&bytes, profile)?;
    serde_json::from_value(value).map_err(|_| anyhow::anyhow!("invalid control message shape"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct Request {
        value: String,
    }

    #[tokio::test]
    async fn raw_control_decoder_rejects_duplicates_shape_and_trailing_values() {
        for raw in [
            br#"{"value":"A","value":"B"}
"#
            .as_slice(),
            br#"{"value":"A","unknown":true}
"#
            .as_slice(),
            br#"{"value":"A"} {}
"#
            .as_slice(),
            br#"{"value":"A","v\u0061lue":"B"}
"#
            .as_slice(),
        ] {
            let (mut writer, reader) = tokio::io::duplex(1024);
            writer.write_all(raw).await.unwrap();
            let mut reader = BufReader::new(reader);
            assert!(
                receive::<Request>(&mut reader, REQUEST_BYTES)
                    .await
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn frames_preserve_buffered_next_message_and_refuse_overflow_before_output() {
        let (mut writer, reader) = tokio::io::duplex(1024);
        let first = Request { value: "A".into() };
        let second = Request { value: "B".into() };
        send(&mut writer, &first, REQUEST_BYTES).await.unwrap();
        send(&mut writer, &second, REQUEST_BYTES).await.unwrap();
        let mut reader = BufReader::new(reader);
        assert_eq!(
            receive::<Request>(&mut reader, REQUEST_BYTES)
                .await
                .unwrap(),
            first
        );
        assert_eq!(
            receive::<Request>(&mut reader, REQUEST_BYTES)
                .await
                .unwrap(),
            second
        );
        let oversized = Request {
            value: "X".repeat(REQUEST_BYTES + 1),
        };
        assert!(send(&mut writer, &oversized, REQUEST_BYTES).await.is_err());
        // Outgoing overflow is discovered before any partial frame reaches the peer.
        drop(writer);
        let mut remaining = Vec::new();
        reader.read_to_end(&mut remaining).await.unwrap();
        assert!(
            remaining.is_empty(),
            "overflow emitted a partial control frame"
        );
    }

    #[test]
    fn service_hello_checks_selected_state_and_current_epoch_before_actions() {
        let identity = ServiceIdentity {
            state: "/private/tmp/selected.db".into(),
            instance: uuid::Uuid::new_v4().to_string(),
            epoch: 1,
        };
        let mut hello = Hello {
            protocol: PROTOCOL_VERSION,
            identity: identity.clone(),
        };
        hello.validate_for(&identity).unwrap();
        hello.identity.state = "/private/tmp/foreign-copy.db".into();
        assert!(hello.validate_for(&identity).is_err());
        hello.identity = identity.clone();
        hello.identity.epoch += 1;
        assert!(hello.validate_for(&identity).is_err());
        hello.identity = identity.clone();
        hello.protocol += 1;
        assert!(hello.validate_for(&identity).is_err());
    }
}
