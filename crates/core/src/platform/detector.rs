use super::Platform;

pub fn parse_platform(input: &str) -> Option<Platform> {
    let lower = input.trim().to_lowercase();
    Some(match lower.as_str() {
        "claude" | "anthropic" => Platform::Claude,
        "openai" | "gpt" | "chatgpt" => Platform::OpenAI,
        "gemini" | "google" => Platform::Gemini,
        "midjourney" | "mj" => Platform::Midjourney,
        "dalle" | "dall-e" | "dall_e" => Platform::DallE,
        "sd" | "stablediffusion" | "stable-diffusion" | "stable_diffusion" => {
            Platform::StableDiffusion
        }
        "generic" => Platform::Generic,
        _ => return None,
    })
}

pub fn detect_platform(input: &str) -> Platform {
    parse_platform(input).unwrap_or(Platform::Generic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_known_platforms() {
        assert_eq!(detect_platform("claude"), Platform::Claude);
        assert_eq!(detect_platform("anthropic"), Platform::Claude);
        assert_eq!(detect_platform("openai"), Platform::OpenAI);
        assert_eq!(detect_platform("gpt"), Platform::OpenAI);
        assert_eq!(detect_platform("chatgpt"), Platform::OpenAI);
        assert_eq!(detect_platform("gemini"), Platform::Gemini);
        assert_eq!(detect_platform("midjourney"), Platform::Midjourney);
        assert_eq!(detect_platform("dalle"), Platform::DallE);
        assert_eq!(detect_platform("dall-e"), Platform::DallE);
        assert_eq!(detect_platform("sd"), Platform::StableDiffusion);
    }

    #[test]
    fn test_detect_unknown_defaults_to_generic() {
        assert_eq!(detect_platform("unknown"), Platform::Generic);
        assert_eq!(detect_platform("foobar"), Platform::Generic);
    }

    #[test]
    fn test_parse_platform_rejects_unknown() {
        assert_eq!(parse_platform("generic"), Some(Platform::Generic));
        assert_eq!(parse_platform("unknown"), None);
    }

    #[test]
    fn test_detect_case_insensitive() {
        assert_eq!(detect_platform("Claude"), Platform::Claude);
        assert_eq!(detect_platform("OPENAI"), Platform::OpenAI);
        assert_eq!(detect_platform("Gemini"), Platform::Gemini);
    }
}
