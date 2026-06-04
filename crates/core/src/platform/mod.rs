mod detector;

pub use detector::*;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Claude,
    #[serde(rename = "claude-code", alias = "claudecode", alias = "cc")]
    ClaudeCode,
    #[serde(alias = "openai")]
    OpenAI,
    Gemini,
    Cursor,
    #[serde(alias = "openai-codex")]
    Codex,
    #[serde(rename = "coding-agent", alias = "agent", alias = "generic-agent")]
    CodingAgent,
    Midjourney,
    #[serde(alias = "dalle")]
    DallE,
    StableDiffusion,
    Generic,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Claude => write!(f, "claude"),
            Platform::ClaudeCode => write!(f, "claude-code"),
            Platform::OpenAI => write!(f, "openai"),
            Platform::Gemini => write!(f, "gemini"),
            Platform::Cursor => write!(f, "cursor"),
            Platform::Codex => write!(f, "codex"),
            Platform::CodingAgent => write!(f, "coding-agent"),
            Platform::Midjourney => write!(f, "midjourney"),
            Platform::DallE => write!(f, "dalle"),
            Platform::StableDiffusion => write!(f, "stablediffusion"),
            Platform::Generic => write!(f, "generic"),
        }
    }
}

impl Platform {
    pub fn label(&self) -> &'static str {
        match self {
            Platform::Claude => "Claude",
            Platform::ClaudeCode => "Claude Code",
            Platform::OpenAI => "GPT",
            Platform::Gemini => "Gemini",
            Platform::Cursor => "Cursor",
            Platform::Codex => "Codex",
            Platform::CodingAgent => "Coding Agent",
            Platform::Midjourney => "Midjourney",
            Platform::DallE => "DALL-E",
            Platform::StableDiffusion => "Stable Diffusion",
            Platform::Generic => "Generic",
        }
    }

    pub fn is_image_platform(&self) -> bool {
        matches!(
            self,
            Platform::Midjourney | Platform::DallE | Platform::StableDiffusion
        )
    }

    pub fn is_text_platform(&self) -> bool {
        matches!(
            self,
            Platform::Claude
                | Platform::ClaudeCode
                | Platform::OpenAI
                | Platform::Gemini
                | Platform::Cursor
                | Platform::Codex
                | Platform::CodingAgent
                | Platform::Generic
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnhanceType {
    Text,
    Image,
}
