mod detector;

pub use detector::*;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Claude,
    #[serde(alias = "openai")]
    OpenAI,
    Gemini,
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
            Platform::OpenAI => write!(f, "openai"),
            Platform::Gemini => write!(f, "gemini"),
            Platform::Midjourney => write!(f, "midjourney"),
            Platform::DallE => write!(f, "dalle"),
            Platform::StableDiffusion => write!(f, "stablediffusion"),
            Platform::Generic => write!(f, "generic"),
        }
    }
}

impl Platform {
    pub fn is_image_platform(&self) -> bool {
        matches!(
            self,
            Platform::Midjourney | Platform::DallE | Platform::StableDiffusion
        )
    }

    pub fn is_text_platform(&self) -> bool {
        matches!(
            self,
            Platform::Claude | Platform::OpenAI | Platform::Gemini | Platform::Generic
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnhanceType {
    Text,
    Image,
}
