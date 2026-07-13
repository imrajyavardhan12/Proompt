use crate::platform::Platform;

pub fn build_prompts(
    user_prompt: &str,
    platform: Platform,
    style_hints: Option<&[String]>,
) -> (String, String) {
    let system_prompt = get_system_prompt(platform);

    let mut full_user_prompt = String::new();

    if let Some(styles) = style_hints
        && !styles.is_empty()
    {
        full_user_prompt.push_str("Style hints: ");
        full_user_prompt.push_str(&styles.join(", "));
        full_user_prompt.push_str("\n\n");
    }

    full_user_prompt.push_str("Image idea: ");
    full_user_prompt.push_str(user_prompt);

    (system_prompt, full_user_prompt)
}

pub(super) fn sanitize_output(user_prompt: &str, platform: Platform, output: &str) -> String {
    let sanitized = if platform == Platform::StableDiffusion
        && requests_intentional_unusual_anatomy(user_prompt)
    {
        strip_conflicting_anatomy_negatives(output)
    } else {
        output.to_string()
    };

    restore_without_clause(user_prompt, &sanitized)
}

fn strip_conflicting_anatomy_negatives(output: &str) -> String {
    let lowercase = output.to_lowercase();
    let Some(index) = lowercase.find("\nnegative:") else {
        return output.to_string();
    };
    let positive = output[..index].trim_end();
    let negative = &output[index + "\nnegative:".len()..];
    let retained = negative
        .split([',', ';'])
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .filter(|term| {
            let lowercase = term.to_lowercase();
            !ANATOMY_TERMS.iter().any(|word| lowercase.contains(word))
                && ![
                    "anatom",
                    "malformed",
                    "deformed",
                    "extra part",
                    "missing part",
                ]
                .iter()
                .any(|word| lowercase.contains(word))
        })
        .collect::<Vec<_>>();

    if retained.is_empty() {
        positive.to_string()
    } else {
        format!("{}\n\nNegative: {}", positive, retained.join(", "))
    }
}

fn restore_without_clause(user_prompt: &str, output: &str) -> String {
    let lowercase_prompt = user_prompt.to_lowercase();
    let Some(start) = lowercase_prompt.find("without ") else {
        return output.to_string();
    };
    let remainder = &user_prompt[start..];
    let clause = remainder
        .split(['.', '\n'])
        .next()
        .unwrap_or(remainder)
        .trim();
    let lowercase_output = output.to_lowercase();
    let has_exclusion_marker = lowercase_output
        .split(|character: char| !character.is_alphanumeric())
        .any(|word| {
            matches!(
                word,
                "no" | "without" | "exclude" | "excluding" | "avoid" | "negative"
            )
        });
    let all_terms_present = clause
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| {
            !word.is_empty()
                && !matches!(
                    *word,
                    "without" | "using" | "or" | "and" | "the" | "a" | "an"
                )
        })
        .all(|word| lowercase_output.contains(word));
    if has_exclusion_marker && all_terms_present {
        return output.to_string();
    }

    let mut restored = output.trim_end().to_string();
    if !restored.ends_with(['.', '!', '?']) {
        restored.push('.');
    }
    restored.push(' ');
    let mut characters = clause.chars();
    if let Some(first) = characters.next() {
        restored.extend(first.to_uppercase());
        restored.extend(characters);
    }
    restored.push('.');
    restored
}

const ANATOMY_TERMS: &[&str] = &[
    "finger", "hand", "arm", "leg", "foot", "toe", "eye", "ear", "head", "limb", "wing", "tail",
];

fn requests_intentional_unusual_anatomy(prompt: &str) -> bool {
    let lowercase = prompt.to_lowercase();
    let intentional = lowercase.contains("designed with exactly")
        || lowercase.contains("intentional")
        || lowercase.contains("intentionally");
    let anatomy = ANATOMY_TERMS.iter().any(|term| lowercase.contains(term));

    intentional && anatomy
}

fn get_system_prompt(platform: Platform) -> String {
    let base = match platform {
        Platform::Midjourney => MIDJOURNEY_SYSTEM_PROMPT,
        Platform::DallE => DALLE_SYSTEM_PROMPT,
        Platform::StableDiffusion => SD_SYSTEM_PROMPT,
        _ => GENERIC_IMAGE_SYSTEM_PROMPT,
    };

    format!("{}\n\n{}", base, IMAGE_QUALITY_RULES)
}

const MIDJOURNEY_SYSTEM_PROMPT: &str = r#"You are a Midjourney prompt editor. Transform image ideas into faithful, controllable Midjourney prompts.

- Lead with the requested subject and composition.
- Use concise comma-separated visual descriptors.
- Add only compatible details about medium, lighting, atmosphere, color, or framing when the user left them open.
- Treat supplied style hints as explicit requirements.
- Preserve user-supplied Midjourney parameters exactly and place them once at the end.
- NON-NEGOTIABLE: if a `--parameter` does not appear in the user's image idea, it must not appear in your output. Never choose an aspect ratio or append defaults.
- Never invent an aspect ratio, model version, style mode, stylize value, seed, or other parameter.

Output ONLY the ready-to-paste Midjourney prompt. Before output, remove every `--parameter` that was not present in the user's input."#;

const DALLE_SYSTEM_PROMPT: &str = r#"You are a DALL-E prompt editor. Transform image ideas into faithful, controllable natural-language prompts.

- Describe the requested subject, setting, medium, and composition in clear sentences.
- Add compatible lighting, color, or atmosphere only when those choices are open.
- Treat supplied style hints as explicit requirements.
- Quote requested visible text exactly and make spelling/capitalization constraints explicit.
- State exclusions directly when the user provides them.
- Do not add named artists, copyrighted characters, brands, people, objects, or narrative details.

Output ONLY the ready-to-paste DALL-E prompt."#;

const SD_SYSTEM_PROMPT: &str = r#"You are a Stable Diffusion prompt editor. Transform image ideas into faithful prompts.

- Put the requested subject, count, composition, medium, and explicit style first in a concise comma-separated positive prompt.
- Add only context-compatible lighting, color, detail, and framing.
- Treat supplied style hints as explicit requirements.
- Use weighting sparingly and only to reinforce a user-supplied feature that needs control.
- Do not prefix the positive prompt with `Positive:` or any other label.
- For portraits requesting natural or unretouched skin, target smoothing, airbrushing, plastic skin, and glamour retouching rather than inventing clothing, pose, prop, or background exclusions.
- For graphic design, negatives should repeat the user's prohibited effects, text, colors, and extra instances of the requested subject; never ban broad geometry or shapes.
- Never use generic quality or anatomy boilerplate when it conflicts with the medium or an intentional unusual feature.
- Never negate an explicitly requested trait.

Choose exactly one output format:
1. Intentional unusual count, anatomy, or construction: emphasize the exact feature in the positive prompt and output the positive prompt only. Do not create a negative prompt unless the user supplied an unrelated explicit exclusion. If the subject intentionally has exactly six fingers, do not output `Negative:` and do not mention extra fingers, missing fingers, malformed hands, or anatomy as failures.
2. Otherwise, output the positive prompt, a blank line, and the `Negative:` prompt containing explicit user exclusions and only directly relevant failure modes.

Output ONLY the selected format."#;

const GENERIC_IMAGE_SYSTEM_PROMPT: &str = r#"You are an image-prompt editor. Transform image ideas into faithful, descriptive prompts that work across image generators.

- Preserve the requested subject and visual constraints first.
- Add only coherent detail about setting, medium, lighting, color, atmosphere, and composition where the user left room.
- Treat supplied style hints as explicit requirements.
- Prefer model-neutral visual language; do not add platform parameters.
- Do not add named artists, brands, people, objects, text, or narrative events.

Output ONLY the ready-to-paste image prompt."#;

const IMAGE_QUALITY_RULES: &str = r#"Quality and constraint rules that override any conflicting guidance above:
- Preserve every explicit subject, count, spatial relationship, color, material, phrase, capitalization, style, mood, framing, aspect ratio, and platform parameter.
- Preserve every exclusion in the final prompt. Make `no` and `without` constraints explicit instead of relying on omission. Before output, compare against the input and restore every exclusion that is missing.
- Never invent people, animals, objects, props, logos, text, brands, architecture, historical facts, narrative events, named artists, copyrighted characters, or technical parameters. In particular, never output a `--parameter` absent from the user's input.
- Add aesthetic detail only when it is compatible and does not narrow a deliberate ambiguity. Details inherently implied by the subject are allowed; new scene content is not.
- For exact counts or layouts, reinforce the count and ordering once. Do not add extra scene elements.
- For visible text, reproduce the requested text exactly, including spelling and capitalization; add no other text.
- For historical images, request plausible period detail and explicitly exclude modern elements without inventing named people or events.
- For abstract images, preserve every prohibited symbol and develop only nonliteral forms consistent with the supplied palette and mood.
- For unusual intentional anatomy or construction, emphasize that it is deliberate and omit the negative prompt rather than risk contradicting it. Never output the feature, its count, anatomy terms, or extra/missing-part language as negatives.
- For graphic design, icons, and vector work, avoid photographic quality tags, depth effects, anatomy negatives, and extra colors.
- If the original is already detailed, perform light cleanup only. Do not append a default aesthetic recipe.
- Keep the result concise, paste-ready, and free of explanations, headings, and meta-commentary.

Output only the enhanced image prompt."#;

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
    fn image_prompts_share_constraint_and_grounding_rules() {
        for platform in [
            Platform::Midjourney,
            Platform::DallE,
            Platform::StableDiffusion,
            Platform::Generic,
        ] {
            let (system, _) = build_prompts("exactly two boats, no text", platform, None);
            assert!(system.contains("Preserve every explicit subject, count"));
            assert!(system.contains("Preserve every exclusion"));
            assert!(system.contains("restore every exclusion that is missing"));
            assert!(system.contains("Never invent people, animals, objects"));
            assert!(system.contains("For visible text, reproduce the requested text exactly"));
            assert!(system.contains("For unusual intentional anatomy"));
            assert!(system.contains("omit the negative prompt rather than risk contradicting it"));
            assert!(system.contains("If the original is already detailed"));
        }
    }

    #[test]
    fn platform_rules_avoid_fixed_prompt_recipes() {
        let (midjourney, _) = build_prompts("test", Platform::Midjourney, None);
        assert!(midjourney.contains("Never invent an aspect ratio"));
        assert!(!midjourney.contains("--v 6"));

        let (stable_diffusion, _) = build_prompts("test", Platform::StableDiffusion, None);
        assert!(stable_diffusion.contains("Never negate an explicitly requested trait"));
        assert!(stable_diffusion.contains(
            "If the subject intentionally has exactly six fingers, do not output `Negative:`"
        ));
        assert!(stable_diffusion.contains(
            "Otherwise, output the positive prompt, a blank line, and the `Negative:` prompt"
        ));
        assert!(!stable_diffusion.contains(
            "Output ONLY the unlabeled positive prompt, a blank line, and the `Negative:` prompt"
        ));
        assert!(!stable_diffusion.contains("masterpiece, best quality"));
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

    #[test]
    fn removes_generated_negatives_for_intentional_unusual_anatomy() {
        let input = "red glove designed with exactly six fingers on a pedestal";
        let output = "red glove with exactly six fingers, on a pedestal\n\nNegative: no extra fingers, no malformed hands";

        assert_eq!(
            sanitize_output(input, Platform::StableDiffusion, output),
            "red glove with exactly six fingers, on a pedestal"
        );
    }

    #[test]
    fn keeps_stable_diffusion_negatives_for_ordinary_subjects_and_explicit_exclusions() {
        let ordinary = "portrait with natural skin";
        let ordinary_output = "portrait, natural skin\n\nNegative: plastic skin";
        assert_eq!(
            sanitize_output(ordinary, Platform::StableDiffusion, ordinary_output),
            ordinary_output
        );

        let excluded = "glove designed with exactly six fingers, without text";
        let excluded_output = "six-finger glove\n\nNegative: no extra fingers, text";
        assert_eq!(
            sanitize_output(excluded, Platform::StableDiffusion, excluded_output),
            "six-finger glove\n\nNegative: text"
        );
    }

    #[test]
    fn restores_a_dropped_explicit_exclusion_clause() {
        let input = "abstract collaboration without using people, hands, puzzle pieces, gears, or text. calm blue and amber palette";
        let output = "abstract flowing shapes in a calm blue and amber palette";

        assert_eq!(
            sanitize_output(input, Platform::Generic, output),
            "abstract flowing shapes in a calm blue and amber palette. Without using people, hands, puzzle pieces, gears, or text."
        );
    }
}
