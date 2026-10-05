//! Validate compact artifact encoding without retaining an encoded buffer.
use anyhow::Result;
use serde::Serialize;
use serde_json::ser::Formatter;
use std::{
    cell::Cell,
    fmt,
    io::{self, Write},
};

const MAX_DEPTH: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    Bytes,
    Depth,
    Raw,
}
impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Bytes => "mandatory pack/checkpoint exceeds 1 MiB; narrow explicitly",
            Self::Depth => "artifact JSON nesting exceeds 120 containers",
            Self::Raw => "raw artifact JSON encoding is unsupported",
        })
    }
}
impl std::error::Error for Refusal {}

#[derive(Default)]
struct State {
    bytes: Cell<usize>,
    depth: Cell<usize>,
    refusal: Cell<Option<Refusal>>,
}
impl State {
    fn reject(&self, reason: Refusal) -> io::Error {
        let first = self.refusal.get().unwrap_or(reason);
        self.refusal.set(Some(first));
        io::Error::other(first)
    }
    fn active(&self) -> io::Result<()> {
        match self.refusal.get() {
            Some(reason) => Err(self.reject(reason)),
            None => Ok(()),
        }
    }
    fn open(&self) -> io::Result<()> {
        self.active()?;
        let next = self
            .depth
            .get()
            .checked_add(1)
            .filter(|depth| *depth <= MAX_DEPTH)
            .ok_or_else(|| self.reject(Refusal::Depth))?;
        self.depth.set(next);
        Ok(())
    }
    fn close(&self) -> io::Result<()> {
        self.active()?;
        self.depth.set(
            self.depth
                .get()
                .checked_sub(1)
                .ok_or_else(|| self.reject(Refusal::Depth))?,
        );
        Ok(())
    }
    fn finish(&self, result: serde_json::Result<()>) -> Result<usize> {
        if let Some(reason) = self.refusal.get() {
            return Err(reason.into());
        }
        result?;
        Ok(self.bytes.get())
    }
}

struct Counter<'a>(&'a State);
impl Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        self.0.active()?;
        let next = self
            .0
            .bytes
            .get()
            .checked_add(bytes.len())
            .filter(|bytes| *bytes <= super::MAX_BYTES)
            .ok_or_else(|| self.0.reject(Refusal::Bytes))?;
        self.0.bytes.set(next);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.active()
    }
}

struct ArtifactFormatter<'a>(&'a State);
// Keep composite trait defaults (notably write_byte_array): they dispatch through
// these overridden container methods. No inner CompactFormatter forwarding.
impl Formatter for ArtifactFormatter<'_> {
    fn begin_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.open()?;
        writer.write_all(b"[")
    }
    fn end_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"]")?;
        self.0.close()
    }
    fn begin_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.open()?;
        writer.write_all(b"{")
    }
    fn end_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"}")?;
        self.0.close()
    }
    fn write_raw_fragment<W: ?Sized + Write>(
        &mut self,
        _writer: &mut W,
        _fragment: &str,
    ) -> io::Result<()> {
        Err(self.0.reject(Refusal::Raw))
    }
}

pub(super) fn encoded_len(value: &impl Serialize) -> Result<usize> {
    let state = State::default();
    let mut serializer =
        serde_json::Serializer::with_formatter(Counter(&state), ArtifactFormatter(&state));
    state.finish(value.serialize(&mut serializer))
}

#[cfg(test)]
pub(super) const CONTEXT_DIGEST: u8 = 1;
#[cfg(test)]
pub(super) const CHECKPOINT_DIGEST: u8 = 2;
#[cfg(test)]
pub(super) const TYPED_DECODE: u8 = 4;
#[cfg(test)]
pub(super) const CHECKPOINT_APPEND: u8 = 8;
#[cfg(test)]
thread_local! {
    static READ_STAGES: Cell<u8> = const { Cell::new(0) };
}
#[cfg(test)]
pub(super) fn read_stage(stage: u8) {
    READ_STAGES.with(|seen| seen.set(seen.get() | stage));
}
#[cfg(test)]
pub(super) fn take_read_stages() -> u8 {
    READ_STAGES.with(|seen| seen.replace(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::ser::{Error, SerializeSeq};
    use serde_json::{Value, json};

    fn refusal(value: &impl Serialize, reason: Refusal) {
        let err = super::super::bounded(value).unwrap_err();
        assert_eq!(err.downcast_ref::<Refusal>(), Some(&reason));
    }
    fn nested(depth: usize, object: bool) -> Value {
        (0..depth).fold(Value::Null, |value, _| {
            if object {
                json!({"x":value})
            } else {
                json!([value])
            }
        })
    }

    #[test]
    fn retained_event_accounting_matches_its_actual_complete_encoding() {
        let event = super::super::RetainedEvent {
            session: crate::domain::SessionId::new(),
            event: super::super::HistoryEvent {
                sequence: 1,
                kind: super::super::EventKind::Transient,
                text: "東京\\\"\n".repeat(100),
            },
        };
        let expected = serde_json::to_vec(&event).unwrap().len();
        assert_eq!(encoded_len(&event).unwrap(), expected);
        assert_eq!(
            super::super::retained_bytes(&[event.clone(), event]).unwrap(),
            expected * 2
        );
    }

    #[test]
    fn exact_encoded_bytes_match_compact_json_and_bound_escaped_utf8() {
        for v in [
            json!(null),
            json!(true),
            json!(-9),
            json!(0.25),
            json!(["é", "東京", "\"\\\n\t\u{0001}"]),
            json!({"a":[], "b":{}}),
        ] {
            assert_eq!(
                encoded_len(&v).unwrap(),
                serde_json::to_vec(&v).unwrap().len()
            );
        }
        let fitting = "a".repeat(super::super::MAX_BYTES - 2);
        assert_eq!(encoded_len(&fitting).unwrap(), super::super::MAX_BYTES);
        super::super::bounded(&fitting).unwrap();
        refusal(&(fitting.clone() + "a"), Refusal::Bytes);
        refusal(&(fitting.clone() + "é"), Refusal::Bytes);
        refusal(&(fitting + "\""), Refusal::Bytes);
    }

    #[test]
    fn array_and_object_depth_include_root() {
        for object in [false, true] {
            super::super::bounded(&nested(120, object)).unwrap();
            refusal(&nested(121, object), Refusal::Depth);
        }
    }

    #[test]
    fn sibling_empty_and_nonempty_containers_do_not_accumulate_depth() {
        for child in [json!([]), json!({}), json!([0]), json!({"x":0})] {
            let value = Value::Array(vec![child; 10_000]);
            super::super::bounded(&value).unwrap();
            assert_eq!(
                encoded_len(&value).unwrap(),
                serde_json::to_vec(&value).unwrap().len()
            );
        }
    }

    struct ByteArrays(usize);
    impl Serialize for ByteArrays {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> std::result::Result<S::Ok, S::Error> {
            if self.0 == 0 {
                return serializer.serialize_bytes(&[1]);
            }
            let mut seq = serializer.serialize_seq(Some(1))?;
            seq.serialize_element(&ByteArrays(self.0 - 1))?;
            seq.end()
        }
    }
    #[test]
    fn byte_array_default_routes_through_the_depth_guard() {
        super::super::bounded(&ByteArrays(119)).unwrap();
        refusal(&ByteArrays(120), Refusal::Depth);
        assert_eq!(
            encoded_len(&ByteArrays(119)).unwrap(),
            serde_json::to_vec(&ByteArrays(119)).unwrap().len()
        );
    }

    struct Sentinel<'a>(&'a Cell<bool>);
    impl Serialize for Sentinel<'_> {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> std::result::Result<S::Ok, S::Error> {
            self.0.set(true);
            serializer.serialize_u8(0)
        }
    }
    struct LargeFirst<'a> {
        first: &'a str,
        visited: &'a Cell<bool>,
    }
    impl Serialize for LargeFirst<'_> {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> std::result::Result<S::Ok, S::Error> {
            let mut seq = serializer.serialize_seq(Some(2))?;
            seq.serialize_element(self.first)?;
            seq.serialize_element(&Sentinel(self.visited))?;
            seq.end()
        }
    }
    #[test]
    fn existing_bounded_consumer_stops_before_later_children() {
        let first = "a".repeat(8 * super::super::MAX_BYTES);
        let visited = Cell::new(false);
        refusal(
            &LargeFirst {
                first: &first,
                visited: &visited,
            },
            Refusal::Bytes,
        );
        assert!(
            !visited.get(),
            "oversize first child must stop later serialization"
        );
    }

    struct ReplaceError(Value);
    impl Serialize for ReplaceError {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> std::result::Result<S::Ok, S::Error> {
            let mut seq = serializer.serialize_seq(Some(2))?;
            let _ = seq.serialize_element(&self.0);
            let _ = seq.serialize_element(&0);
            Err(S::Error::custom("replacement Serialize error"))
        }
    }
    #[test]
    fn first_refusal_wins_over_swallowed_and_replaced_serialize_errors() {
        refusal(
            &ReplaceError(json!("a".repeat(2 * super::super::MAX_BYTES))),
            Refusal::Bytes,
        );
        refusal(&ReplaceError(nested(120, false)), Refusal::Depth);
        let err = encoded_len(&ReplaceError(json!(0))).unwrap_err();
        assert_eq!(err.root_cause().to_string(), "replacement Serialize error");
    }

    #[test]
    fn sink_refusal_is_sticky_noninterrupted_even_when_final_result_is_ok() {
        let state = State::default();
        let mut sink = Counter(&state);
        let oversized = vec![b'a'; super::super::MAX_BYTES + 1];
        let err = sink.write(&oversized).unwrap_err();
        assert_ne!(err.kind(), io::ErrorKind::Interrupted);
        assert_eq!(sink.write(&[]).unwrap(), 0);
        assert!(sink.write(b"a").is_err());
        assert_eq!(
            state.finish(Ok(())).unwrap_err().downcast_ref::<Refusal>(),
            Some(&Refusal::Bytes)
        );
        assert_eq!(state.bytes.get(), 0);
        for reason in [Refusal::Depth, Refusal::Raw] {
            let state = State::default();
            let err = state.reject(reason);
            assert_ne!(err.kind(), io::ErrorKind::Interrupted);
            assert!(Counter(&state).write(b"a").is_err());
            assert_eq!(
                state.finish(Ok(())).unwrap_err().downcast_ref::<Refusal>(),
                Some(&reason)
            );
        }
    }

    #[test]
    fn raw_formatter_fragment_refuses_without_encoding_bytes() {
        let state = State::default();
        let mut formatter = ArtifactFormatter(&state);
        assert!(
            formatter
                .write_raw_fragment(&mut Counter(&state), "[[1]]")
                .is_err()
        );
        assert_eq!(
            state.finish(Ok(())).unwrap_err().downcast_ref::<Refusal>(),
            Some(&Refusal::Raw)
        );
        assert_eq!(state.bytes.get(), 0);
    }
}
