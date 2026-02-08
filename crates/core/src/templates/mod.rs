pub mod builtin;
pub mod manager;

pub use manager::*;

use serde::{Deserialize, Serialize};

use crate::platform::Platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub platform: Platform,
    pub description: String,
    pub trending: bool,
    pub fields: Vec<TemplateField>,
    pub template: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    Image,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateField {
    pub name: String,
    pub label: String,
    pub placeholder: String,
    pub required: bool,
}
