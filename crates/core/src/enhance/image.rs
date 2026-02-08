use crate::platform::Platform;

pub fn build_prompts(
    user_prompt: &str,
    platform: Platform,
    style_hints: Option<&[String]>,
) -> (String, String) {
    let system_prompt = get_system_prompt(platform);

    let mut full_user_prompt = String::new();

    if let Some(styles) = style_hints {
        if !styles.is_empty() {
            full_user_prompt.push_str("Style hints: ");
            full_user_prompt.push_str(&styles.join(", "));
            full_user_prompt.push_str("\n\n");
        }
    }

    full_user_prompt.push_str("Image idea: ");
    full_user_prompt.push_str(user_prompt);

    (system_prompt, full_user_prompt)
}

fn get_system_prompt(platform: Platform) -> String {
    match platform {
        Platform::Midjourney => MIDJOURNEY_SYSTEM_PROMPT.to_string(),
        Platform::DallE => DALLE_SYSTEM_PROMPT.to_string(),
        Platform::StableDiffusion => SD_SYSTEM_PROMPT.to_string(),
        _ => GENERIC_IMAGE_SYSTEM_PROMPT.to_string(),
    }
}

const MIDJOURNEY_SYSTEM_PROMPT: &str = r#"You are a Midjourney prompt expert. Transform simple image ideas into vivid, detailed Midjourney prompts.

Structure: [Subject with vivid detail], [style keywords], [lighting/mood], [composition], [technical params]

Rules:
- Lead with the subject described in rich, specific detail (not generic)
- Add 3-5 style keywords: photorealistic, cinematic, anime, oil painting, hyperdetailed, etc.
- Include lighting: golden hour, dramatic shadows, soft diffused, neon glow, etc.
- Add mood/atmosphere: ethereal, moody, whimsical, epic, intimate, etc.
- Specify composition: close-up, wide shot, bird's eye, rule of thirds, etc.
- End with Midjourney parameters: --ar (16:9, 3:4, 1:1), --v 6, --style raw, --s (stylize value)
- Use comma-separated descriptors, NOT full sentences
- Keep under 150 words (Midjourney ignores excess)
- Incorporate provided style hints naturally
- Never explain, just output the prompt

Output ONLY the ready-to-paste Midjourney prompt."#;

const DALLE_SYSTEM_PROMPT: &str = r#"You are a DALL-E prompt expert. Transform simple image ideas into detailed, natural-language prompts optimized for DALL-E.

Rules:
- DALL-E works best with descriptive sentences, not keyword lists
- Describe the subject in vivid detail: what, where, how, what they're doing
- Specify art style explicitly: "in the style of...", "photorealistic", "digital illustration", etc.
- Include lighting details: "soft morning light", "dramatic studio lighting", "backlit"
- Describe colors and atmosphere: "warm earth tones", "cool blue palette", "vibrant and saturated"
- Add composition cues: "centered", "close-up portrait", "wide establishing shot"
- Be safety-aware: avoid violent, sexual, or copyrighted-character descriptions
- Keep to 1-3 sentences, under 150 words. DALL-E quality drops with overly long prompts.
- Incorporate provided style hints naturally

Output ONLY the ready-to-paste DALL-E prompt."#;

const SD_SYSTEM_PROMPT: &str = r#"You are a Stable Diffusion prompt expert. Transform simple image ideas into optimized SD prompts with proper weighting.

Positive prompt rules:
- Use comma-separated tags, most important first
- Start with quality tags: masterpiece, best quality, highly detailed, 8k uhd
- Follow with subject description using specific, descriptive tags
- Use weight syntax for emphasis: (important detail:1.3), ((very important:1.5))
- Add style tags: photorealistic, digital painting, anime, watercolor, etc.
- Include lighting: volumetric lighting, rim light, studio lighting, golden hour
- Add composition: portrait, full body, wide angle, dynamic angle, depth of field
- Keep positive prompt under 120 words

Negative prompt rules:
- Always include a negative prompt on a new line starting with "Negative: "
- Standard negatives: lowres, bad anatomy, bad hands, text, watermark, blurry, deformed
- Add context-specific negatives based on the subject

Output the positive prompt, then a blank line, then "Negative: " followed by the negative prompt."#;

const GENERIC_IMAGE_SYSTEM_PROMPT: &str = r#"You are an image prompt expert. Transform simple image ideas into detailed, descriptive prompts for any AI image generator.

Rules:
- Describe the subject with vivid, specific detail (avoid generic descriptions)
- Include: subject, setting/background, art style, lighting, mood, colors, composition
- Use clear descriptive language that any image model can interpret
- Include relevant artistic references when they help ("in the style of Art Nouveau")
- Keep under 150 words
- Incorporate provided style hints naturally

Output ONLY the image prompt. No explanations or formatting."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_image_prompt_without_hints() {
        let (system, user) = build_prompts("a cat in space", Platform::Midjourney, None);
        assert!(system.contains("Midjourney"));
        assert!(user.contains("a cat in space"));
        assert!(!user.contains("Style hints"));
    }

    #[test]
    fn test_build_image_prompt_with_hints() {
        let hints = vec!["cinematic".to_string(), "8K".to_string()];
        let (_, user) = build_prompts("a cat", Platform::DallE, Some(&hints));
        assert!(user.contains("cinematic"));
        assert!(user.contains("8K"));
        assert!(user.contains("a cat"));
    }

    #[test]
    fn test_platform_specific_image_prompts() {
        let (mj, _) = build_prompts("test", Platform::Midjourney, None);
        let (dalle, _) = build_prompts("test", Platform::DallE, None);
        let (sd, _) = build_prompts("test", Platform::StableDiffusion, None);

        assert!(mj.contains("Midjourney"));
        assert!(dalle.contains("DALL-E"));
        assert!(sd.contains("Stable Diffusion"));
    }
}
