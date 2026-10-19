//! Fixed conservative patterns; receipts never contain the matching bytes.
use anyhow::Result;
#[derive(Debug)]
pub(crate) struct SecretHit {
    pub pattern: &'static str,
    pub offset_sha256: String,
    offset: usize,
}
impl std::fmt::Display for SecretHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("secret_detected")
    }
}
impl std::error::Error for SecretHit {}
pub struct SecretScanner;
impl SecretScanner {
    pub fn check(bytes: &[u8]) -> Result<()> {
        let text = String::from_utf8_lossy(bytes);
        let found = [
            "PRIVATE KEY-----",
            "AKIA",
            "ASIA",
            "ghp_",
            "gho_",
            "ghu_",
            "ghs_",
            "ghr_",
            "github_pat_",
            "xoxb-",
            "xoxp-",
        ]
        .iter()
        .find_map(|p| text.find(p).map(|offset| ("fixed_secret", offset)))
        .or_else(|| {
            text.split(|c: char| c.is_whitespace() || matches!(c, '\'' | '"' | ';'))
                .find_map(|word| {
                    word.split_once('=')
                        .is_some_and(|(name, _)| {
                            let name = name.to_ascii_uppercase();
                            name.ends_with("_SECRET") || name.ends_with("_TOKEN")
                        })
                        .then(|| ("secret_assignment", text.find(word).unwrap_or(0)))
                })
        });
        if let Some((pattern, offset)) = found {
            return Err(SecretHit {
                pattern,
                offset_sha256: super::launcher::hash(&offset.to_le_bytes()),
                offset,
            }
            .into());
        }
        Ok(())
    }
    pub(super) fn additions(diff: &[u8]) -> Result<()> {
        let mut offset = 0;
        for line in diff.split(|b| *b == b'\n') {
            if line.starts_with(b"+")
                && !line.starts_with(b"+++")
                && let Err(error) = Self::check(&line[1..])
            {
                let Some(hit) = error.downcast_ref::<SecretHit>() else {
                    return Err(error);
                };
                let absolute = offset + 1 + hit.offset;
                return Err(SecretHit {
                    pattern: hit.pattern,
                    offset_sha256: super::launcher::hash(&absolute.to_le_bytes()),
                    offset: absolute,
                }
                .into());
            }
            offset += line.len() + 1;
        }
        Ok(())
    }
}
