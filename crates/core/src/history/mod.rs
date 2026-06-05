use std::{
    cmp::Reverse,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    platform::{EnhanceType, Platform},
    routing::{ResolutionConfidence, ResolutionSource, TargetResolution},
};

const DEFAULT_MAX_HISTORY_RECORDS: usize = 500;
static HISTORY_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptHistoryRecord {
    pub id: String,
    pub original_prompt: String,
    pub enhanced_prompt: String,
    pub enhancement_type: EnhanceType,
    pub platform: Platform,
    pub provider: String,
    pub model: String,
    pub created_at_ms: u64,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<PromptRoutingMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptRoutingMetadata {
    pub source: ResolutionSource,
    pub confidence: ResolutionConfidence,
    pub reason: String,
}

impl From<TargetResolution> for PromptRoutingMetadata {
    fn from(resolution: TargetResolution) -> Self {
        Self {
            source: resolution.source,
            confidence: resolution.confidence,
            reason: resolution.reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPromptHistoryRecord {
    pub original_prompt: String,
    pub enhanced_prompt: String,
    pub enhancement_type: EnhanceType,
    pub platform: Platform,
    pub provider: String,
    pub model: String,
    pub routing: Option<PromptRoutingMetadata>,
}

#[derive(Debug, Clone)]
pub struct HistoryStore {
    path: PathBuf,
    max_records: usize,
}

impl HistoryStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_records: DEFAULT_MAX_HISTORY_RECORDS,
        }
    }

    pub fn with_max_records(mut self, max_records: usize) -> Self {
        self.max_records = max_records.max(1);
        self
    }

    pub fn load(&self) -> Result<Vec<PromptHistoryRecord>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read history from {}", self.path.display()))?;
        if content.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut records: Vec<PromptHistoryRecord> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse history from {}", self.path.display()))?;
        records.sort_by_key(|record| Reverse(record.created_at_ms));
        Ok(records)
    }

    pub fn append(&self, input: NewPromptHistoryRecord) -> Result<PromptHistoryRecord> {
        validate_new_record(&input)?;

        let mut records = self.load()?;
        let record = PromptHistoryRecord {
            id: new_history_id(),
            original_prompt: input.original_prompt,
            enhanced_prompt: input.enhanced_prompt,
            enhancement_type: input.enhancement_type,
            platform: input.platform,
            provider: input.provider.trim().to_string(),
            model: input.model.trim().to_string(),
            created_at_ms: now_ms(),
            favorite: false,
            routing: input.routing,
        };

        records.insert(0, record.clone());
        records.truncate(self.max_records);
        self.save(&records)?;

        Ok(record)
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<PromptHistoryRecord> {
        let mut records = self.load()?;
        let record = records
            .iter_mut()
            .find(|record| record.id == id)
            .ok_or_else(|| anyhow::anyhow!("History record '{}' not found", id))?;
        record.favorite = favorite;
        let updated = record.clone();
        self.save(&records)?;
        Ok(updated)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let mut records = self.load()?;
        let original_len = records.len();
        records.retain(|record| record.id != id);
        let deleted = records.len() != original_len;
        if deleted {
            self.save(&records)?;
        }
        Ok(deleted)
    }

    pub fn clear(&self) -> Result<usize> {
        let count = self.load()?.len();
        self.save(&[])?;
        Ok(count)
    }

    fn save(&self, records: &[PromptHistoryRecord]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create history dir: {}", parent.display()))?;
        }

        let content =
            serde_json::to_string_pretty(records).context("Failed to serialize history")?;
        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write history to {}", self.path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.path, perms)?;
        }

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn history_path() -> Result<PathBuf> {
    Ok(crate::config::config_dir()?.join("history.json"))
}

pub fn default_store() -> Result<HistoryStore> {
    Ok(HistoryStore::new(history_path()?))
}

pub fn load_history() -> Result<Vec<PromptHistoryRecord>> {
    default_store()?.load()
}

pub fn append_history_record(input: NewPromptHistoryRecord) -> Result<PromptHistoryRecord> {
    default_store()?.append(input)
}

pub fn set_history_favorite(id: &str, favorite: bool) -> Result<PromptHistoryRecord> {
    default_store()?.set_favorite(id, favorite)
}

pub fn delete_history_record(id: &str) -> Result<bool> {
    default_store()?.delete(id)
}

pub fn clear_history() -> Result<usize> {
    default_store()?.clear()
}

fn validate_new_record(input: &NewPromptHistoryRecord) -> Result<()> {
    if input.original_prompt.trim().is_empty() {
        anyhow::bail!("Original prompt cannot be empty");
    }
    if input.enhanced_prompt.trim().is_empty() {
        anyhow::bail!("Enhanced prompt cannot be empty");
    }
    if input.provider.trim().is_empty() {
        anyhow::bail!("Provider cannot be empty");
    }
    if input.model.trim().is_empty() {
        anyhow::bail!("Model cannot be empty");
    }
    Ok(())
}

fn now_ms() -> u64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.min(u128::from(u64::MAX)) as u64
}

fn new_history_id() -> String {
    let seq = HISTORY_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("hist_{:x}_{:x}", now_ms(), seq)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_persists_history_newest_first() {
        let store = HistoryStore::new(temp_history_path("append"));

        let first = store
            .append(sample_record("rough one", "enhanced one"))
            .unwrap();
        let second = store
            .append(sample_record("rough two", "enhanced two"))
            .unwrap();

        let records = store.load().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, second.id);
        assert_eq!(records[0].original_prompt, "rough two");
        assert_eq!(records[1].id, first.id);
        assert_eq!(records[1].enhanced_prompt, "enhanced one");
    }

    #[test]
    fn favorite_and_delete_update_existing_history() {
        let store = HistoryStore::new(temp_history_path("favorite-delete"));
        let record = store.append(sample_record("rough", "enhanced")).unwrap();

        let favorite = store.set_favorite(&record.id, true).unwrap();
        assert!(favorite.favorite);
        assert!(store.load().unwrap()[0].favorite);

        assert!(store.delete(&record.id).unwrap());
        assert!(store.load().unwrap().is_empty());
        assert!(!store.delete(&record.id).unwrap());
    }

    #[test]
    fn max_records_trims_oldest_records() {
        let store = HistoryStore::new(temp_history_path("trim")).with_max_records(2);

        store.append(sample_record("one", "enhanced one")).unwrap();
        store.append(sample_record("two", "enhanced two")).unwrap();
        let newest = store
            .append(sample_record("three", "enhanced three"))
            .unwrap();

        let records = store.load().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, newest.id);
        assert!(records.iter().any(|record| record.original_prompt == "two"));
        assert!(!records.iter().any(|record| record.original_prompt == "one"));
    }

    #[test]
    fn append_rejects_empty_prompt_content() {
        let store = HistoryStore::new(temp_history_path("empty"));
        let result = store.append(NewPromptHistoryRecord {
            original_prompt: " ".to_string(),
            ..sample_record("rough", "enhanced")
        });

        assert!(result.is_err());
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn append_serializes_coding_agent_platform() {
        let path = temp_history_path("coding-agent-platform");
        let store = HistoryStore::new(&path);

        store
            .append(NewPromptHistoryRecord {
                platform: Platform::ClaudeCode,
                ..sample_record("fix upload bug", "enhanced task")
            })
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(r#""platform": "claude-code""#));
        assert_eq!(store.load().unwrap()[0].platform, Platform::ClaudeCode);
    }

    #[test]
    fn append_persists_quick_enhance_routing_metadata() {
        let path = temp_history_path("routing-metadata");
        let store = HistoryStore::new(&path);

        store
            .append(NewPromptHistoryRecord {
                platform: Platform::ClaudeCode,
                routing: Some(PromptRoutingMetadata {
                    source: ResolutionSource::ExplicitPrefix,
                    confidence: ResolutionConfidence::Explicit,
                    reason: "via /cc".to_string(),
                }),
                ..sample_record("fix upload bug", "enhanced task")
            })
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(r#""source": "explicit_prefix""#));
        assert!(content.contains(r#""confidence": "explicit""#));

        let loaded = store.load().unwrap();
        let routing = loaded[0].routing.as_ref().unwrap();
        assert_eq!(routing.source, ResolutionSource::ExplicitPrefix);
        assert_eq!(routing.confidence, ResolutionConfidence::Explicit);
        assert_eq!(routing.reason, "via /cc");
    }

    #[test]
    fn load_accepts_legacy_history_without_routing_metadata() {
        let path = temp_history_path("legacy-no-routing");
        fs::write(
            &path,
            r#"[
              {
                "id": "hist_legacy",
                "original_prompt": "rough",
                "enhanced_prompt": "enhanced",
                "enhancement_type": "text",
                "platform": "claude",
                "provider": "openai",
                "model": "gpt-4o",
                "created_at_ms": 1,
                "favorite": false
              }
            ]"#,
        )
        .unwrap();
        let store = HistoryStore::new(&path);

        let loaded = store.load().unwrap();

        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].routing.is_none());
    }

    fn sample_record(original: &str, enhanced: &str) -> NewPromptHistoryRecord {
        NewPromptHistoryRecord {
            original_prompt: original.to_string(),
            enhanced_prompt: enhanced.to_string(),
            enhancement_type: EnhanceType::Text,
            platform: Platform::Claude,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            routing: None,
        }
    }

    fn temp_history_path(name: &str) -> PathBuf {
        let unique = format!(
            "proompt-history-{}-{}-{}.json",
            name,
            std::process::id(),
            now_ms()
        );
        std::env::temp_dir().join(unique)
    }
}
