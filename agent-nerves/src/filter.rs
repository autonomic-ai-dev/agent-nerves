//! JSON + WASM filter registry for JetStream / NATS events.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::wasm_filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub name: String,
    #[serde(default)]
    pub subject_prefix: Option<String>,
    #[serde(default)]
    pub payload_contains: Option<String>,
    #[serde(default = "default_allow")]
    pub allow: bool,
    #[serde(default)]
    pub wasm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterDecision {
    pub allowed: bool,
    pub matched_rule: Option<String>,
    pub reason: String,
}

fn default_allow() -> bool {
    true
}

pub fn filters_dir(config_dir: Option<&Path>) -> PathBuf {
    config_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(default_filters_dir)
}

pub fn default_filters_dir() -> PathBuf {
    std::env::var("AUTONOMIC_FILTERS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| agent_body_core::autonomic_root().join("filters"))
}

pub fn load_rules(dir: &Path) -> Result<Vec<FilterRule>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut rules = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)?;
        let rule: FilterRule = serde_json::from_str(&raw)
            .with_context(|| format!("parse filter rule {}", path.display()))?;
        rules.push(rule);
    }
    rules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(rules)
}

pub fn evaluate_event(dir: &Path, subject: &str, payload: &[u8]) -> Result<FilterDecision> {
    let rules = load_rules(dir)?;
    if rules.is_empty() {
        return Ok(FilterDecision {
            allowed: true,
            matched_rule: None,
            reason: "no filter rules configured".into(),
        });
    }

    let payload_text = String::from_utf8_lossy(payload);
    for rule in &rules {
        if let Some(prefix) = &rule.subject_prefix {
            if !subject.starts_with(prefix) {
                continue;
            }
        }
        if let Some(needle) = &rule.payload_contains {
            if !payload_text.contains(needle) {
                continue;
            }
        }

        if let Some(wasm_path) = &rule.wasm {
            let path = Path::new(wasm_path);
            let allowed = wasm_filter::evaluate_wasm(path, payload)?;
            return Ok(FilterDecision {
                allowed,
                matched_rule: Some(rule.name.clone()),
                reason: format!("wasm filter `{}` -> {}", path.display(), allowed),
            });
        }

        return Ok(FilterDecision {
            allowed: rule.allow,
            matched_rule: Some(rule.name.clone()),
            reason: if rule.allow {
                "rule allows".into()
            } else {
                "rule denies".into()
            },
        });
    }

    Ok(FilterDecision {
        allowed: true,
        matched_rule: None,
        reason: "no rule matched — default allow".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn json_rule_denies_prefix() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("drop-debug.json"),
            r#"{"name":"drop-debug","subject_prefix":"autonomic.debug.","allow":false}"#,
        )
        .unwrap();
        let decision = evaluate_event(dir.path(), "autonomic.debug.test", b"{}").unwrap();
        assert!(!decision.allowed);
    }
}
