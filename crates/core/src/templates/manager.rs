use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};

use super::{Template, TemplateCategory, builtin};
use crate::platform::Platform;

pub struct TemplateManager {
    templates: Vec<Template>,
}

#[derive(Default)]
pub struct TemplateFilter {
    pub category: Option<TemplateCategory>,
    pub platform: Option<Platform>,
    pub trending_only: bool,
}

impl TemplateManager {
    pub fn new() -> Self {
        let mut templates = builtin::get_builtin_templates();

        // Load cached remote templates if available
        if let Ok(cached) = load_cached_templates() {
            for remote in cached {
                if !templates.iter().any(|t| t.id == remote.id) {
                    templates.push(remote);
                }
            }
        }

        Self { templates }
    }

    pub fn list(&self, filter: &TemplateFilter) -> Vec<&Template> {
        self.templates
            .iter()
            .filter(|t| {
                if let Some(cat) = filter.category
                    && t.category != cat
                {
                    return false;
                }
                if let Some(plat) = filter.platform
                    && t.platform != plat
                {
                    return false;
                }
                if filter.trending_only && !t.trending {
                    return false;
                }
                true
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn apply(&self, id: &str, fields: &HashMap<String, String>) -> Result<String> {
        let template = self
            .get(id)
            .context(format!("Template '{}' not found", id))?;

        for field in &template.fields {
            if field.required && !fields.contains_key(&field.name) {
                anyhow::bail!("Missing required field: {}", field.name);
            }
        }

        let mut result = template.template.clone();
        for (key, value) in fields {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        Ok(result)
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

fn cache_path() -> Result<std::path::PathBuf> {
    let dir = crate::config::config_dir()?;
    Ok(dir.join("cache").join("templates.json"))
}

fn load_cached_templates() -> Result<Vec<Template>> {
    let path = cache_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path)?;
    let templates: Vec<Template> = serde_json::from_str(&content)?;
    Ok(templates)
}

fn save_cached_templates(templates: &[Template]) -> Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(templates)?;
    fs::write(&path, content)?;
    Ok(())
}

pub async fn sync_templates_from_remote(url: &str) -> Result<Vec<Template>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "Proompt/0.1.0")
        .send()
        .await
        .context("Failed to fetch remote templates")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch templates: HTTP {}", response.status());
    }

    let templates: Vec<Template> = response
        .json()
        .await
        .context("Failed to parse remote templates")?;

    save_cached_templates(&templates)?;

    Ok(templates)
}

pub const DEFAULT_TEMPLATES_URL: &str =
    "https://raw.githubusercontent.com/proompt/templates/main/index.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_all_templates() {
        let manager = TemplateManager::new();
        let all = manager.list(&TemplateFilter::default());
        assert!(!all.is_empty());
    }

    #[test]
    fn test_filter_by_category() {
        let manager = TemplateManager::new();
        let image_templates = manager.list(&TemplateFilter {
            category: Some(TemplateCategory::Image),
            ..Default::default()
        });
        assert!(
            image_templates
                .iter()
                .all(|t| t.category == TemplateCategory::Image)
        );
    }

    #[test]
    fn test_apply_template() {
        let manager = TemplateManager::new();
        let templates = manager.list(&TemplateFilter::default());
        if let Some(t) = templates.first() {
            let mut fields = HashMap::new();
            for field in &t.fields {
                fields.insert(field.name.clone(), "test subject".to_string());
            }
            let result = manager.apply(&t.id, &fields).unwrap();
            assert!(result.contains("test subject"));
        }
    }

    #[test]
    fn test_apply_missing_required_field() {
        let manager = TemplateManager::new();
        let templates = manager.list(&TemplateFilter::default());
        if let Some(t) = templates.first() {
            let fields = HashMap::new();
            let result = manager.apply(&t.id, &fields);
            assert!(result.is_err());
        }
    }
}
