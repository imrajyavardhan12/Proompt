use std::io::{self, IsTerminal, Read, Write};

use anyhow::{Context, Result};
use proompt_core::{
    config as cfg,
    enhance::{
        ConfiguredEnhanceRequest, SuperMemoryStatus, enhance_with_loaded_config,
        enhance_with_loaded_config_stream, prepare_enhancement, provider_supports_streaming,
    },
    platform::EnhanceType,
};

use crate::output;

pub fn read_stdin() -> Result<Option<String>> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Ok(None);
    }
    let mut input = String::new();
    stdin
        .lock()
        .read_to_string(&mut input)
        .context("Failed to read from stdin")?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed))
    }
}

pub async fn run(
    prompt: &str,
    platform_str: Option<&str>,
    is_image: bool,
    include_memory: bool,
    style: Option<&str>,
) -> Result<()> {
    let config = cfg::load_config()?;
    let input = ConfiguredEnhanceRequest {
        prompt: prompt.to_string(),
        platform: platform_str.map(str::to_string),
        enhancement_type: if is_image {
            Some(EnhanceType::Image)
        } else {
            None
        },
        include_memory,
        style_hints: style.map(parse_style_hints),
        max_tokens: None,
    };
    let prepared = prepare_enhancement(&config, &input)?;

    let type_label = match prepared.request.enhancement_type {
        EnhanceType::Text => "text",
        EnhanceType::Image => "image",
    };

    let is_tty = io::stdout().is_terminal();
    let use_streaming = provider_supports_streaming(&prepared.provider) && is_tty;

    if use_streaming {
        let spinner = output::spinner(&format!(
            "Enhancing {} prompt for {} via {}...",
            type_label, prepared.request.platform, prepared.provider
        ));

        let mut header_printed = false;
        let prompt_display = prompt.to_string();

        let result = enhance_with_loaded_config_stream(input, config, |token| {
            if !header_printed {
                spinner.finish_and_clear();
                header_printed = true;
                eprintln!();
                output::section_header("Original");
                eprintln!(
                    "  {}",
                    console::Style::new()
                        .dim()
                        .apply_to(truncate_str(&prompt_display, 120))
                );
                eprintln!();
                output::section_header("Enhanced");
                eprintln!();
            }
            print!("{}", token);
            let _ = io::stdout().flush();
        })
        .await;

        if !header_printed {
            spinner.finish_and_clear();
        }

        let result = result?;

        // Newline after streamed content
        println!();
        eprintln!();
        output::section_header("Info");
        print_info(
            &result.response.platform.to_string(),
            &result.response.changes_summary,
            &result.memory_status,
        );
        eprintln!();
    } else {
        let spinner = output::spinner(&format!(
            "Enhancing {} prompt for {} via {}...",
            type_label, prepared.request.platform, prepared.provider
        ));

        let result = enhance_with_loaded_config(input, config).await;

        spinner.finish_and_clear();
        let result = result?;

        output::enhanced_output(
            prompt,
            &result.response.enhanced_prompt,
            &result.response.changes_summary,
            &result.response.platform.to_string(),
        );
        print_memory_note(&result.memory_status);
    }

    Ok(())
}

fn parse_style_hints(style: &str) -> Vec<String> {
    style
        .split(',')
        .map(|hint| hint.trim().to_string())
        .collect()
}

fn print_info(platform: &str, summary: &str, memory_status: &SuperMemoryStatus) {
    eprintln!(
        "  {} {}   {} {}",
        console::Style::new().dim().apply_to("platform:"),
        console::Style::new().cyan().apply_to(platform),
        console::Style::new().dim().apply_to("changes:"),
        summary
    );
    print_memory_note(memory_status);
}

fn print_memory_note(memory_status: &SuperMemoryStatus) {
    match memory_status {
        SuperMemoryStatus::Used { count } => {
            output::dim(&format!("  SuperMemory: used {} relevant memories", count));
        }
        SuperMemoryStatus::Unavailable { message } => {
            output::warn(&format!("SuperMemory unavailable: {}", message));
        }
        _ => {}
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((idx, _)) => format!("{}...", &s[..idx]),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_str_handles_unicode_boundaries() {
        assert_eq!(truncate_str("áéíóú", 3), "áéí...");
    }

    #[test]
    fn truncate_str_leaves_short_unicode_unchanged() {
        assert_eq!(truncate_str("🙂🙂", 2), "🙂🙂");
    }
}
