use std::io::{self, IsTerminal, Read, Write};

use anyhow::{Context, Result};
use proompt_core::{
    config::{self as cfg, Mode},
    enhance::{EnhanceOptions, EnhanceRequest, enhance, enhance_stream},
    integrations::supermemory::SuperMemoryClient,
    platform::{EnhanceType, detect_platform},
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

    let platform = if let Some(p) = platform_str {
        detect_platform(p)
    } else if is_image {
        config.default_image_platform
    } else {
        config.default_platform
    };

    let enhancement_type = if is_image || platform.is_image_platform() {
        EnhanceType::Image
    } else {
        EnhanceType::Text
    };

    let style_hints = style.map(|s| s.split(',').map(|h| h.trim().to_string()).collect());

    // Fetch SuperMemory context if requested
    let supermemory_context = if include_memory && config.supermemory.enabled {
        let sm_spinner = output::spinner("Fetching context from SuperMemory...");
        match fetch_supermemory_context(prompt, config.supermemory.context_limit).await {
            Ok(ctx) => {
                if !ctx.is_empty() {
                    sm_spinner.finish_with_message(format!(
                        "Found {} relevant memories",
                        ctx.len()
                    ));
                } else {
                    sm_spinner.finish_with_message("No relevant memories found");
                }
                Some(ctx)
            }
            Err(e) => {
                sm_spinner.finish_with_message(format!("SuperMemory unavailable: {}", e));
                None
            }
        }
    } else {
        None
    };

    let api_key = match config.mode {
        Mode::Byok => cfg::get_api_key(&config.byok.provider)
            .context("API key not configured. Run: proompt config set byok.api_key YOUR_KEY")?,
        Mode::Hosted => {
            anyhow::bail!("Hosted mode not yet implemented. Use BYOK mode with your own API key.");
        }
    };

    let request = EnhanceRequest {
        prompt: prompt.to_string(),
        platform,
        enhancement_type,
        options: EnhanceOptions {
            include_supermemory: include_memory,
            style_hints,
            max_tokens: None,
        },
    };

    let type_label = match enhancement_type {
        EnhanceType::Text => "text",
        EnhanceType::Image => "image",
    };

    let is_tty = io::stdout().is_terminal();
    let use_streaming = config.byok.provider == "openai" && is_tty;

    if use_streaming {
        let spinner = output::spinner(&format!(
            "Enhancing {} prompt for {} via {}...",
            type_label, platform, config.byok.provider
        ));

        let mut header_printed = false;
        let prompt_display = prompt.to_string();

        let result = enhance_stream(
            request,
            &config.byok.provider,
            &api_key,
            Some(config.byok.model.clone()),
            supermemory_context,
            |token| {
                if !header_printed {
                    spinner.finish_and_clear();
                    header_printed = true;
                    eprintln!();
                    output::section_header("Original");
                    eprintln!(
                        "  {}",
                        console::Style::new().dim().apply_to(truncate_str(&prompt_display, 120))
                    );
                    eprintln!();
                    output::section_header("Enhanced");
                    eprintln!();
                }
                print!("{}", token);
                let _ = io::stdout().flush();
            },
        )
        .await?;

        // Newline after streamed content
        println!();
        eprintln!();
        output::section_header("Info");
        eprintln!(
            "  {} {}   {} {}",
            console::Style::new().dim().apply_to("platform:"),
            console::Style::new().cyan().apply_to(platform.to_string()),
            console::Style::new().dim().apply_to("changes:"),
            result.changes_summary
        );
        eprintln!();
    } else {
        let spinner = output::spinner(&format!(
            "Enhancing {} prompt for {} via {}...",
            type_label, platform, config.byok.provider
        ));

        let result = enhance(
            request,
            &config.byok.provider,
            &api_key,
            Some(config.byok.model),
            supermemory_context,
        )
        .await;

        spinner.finish_and_clear();
        let result = result?;

        output::enhanced_output(
            prompt,
            &result.enhanced_prompt,
            &result.changes_summary,
            &platform.to_string(),
        );
    }

    Ok(())
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

async fn fetch_supermemory_context(prompt: &str, limit: u32) -> Result<Vec<String>> {
    let api_key =
        cfg::get_api_key("supermemory").context("SuperMemory API key not configured")?;
    let client = SuperMemoryClient::new(api_key);
    let memories = client.search(prompt, limit).await?;
    Ok(memories.into_iter().map(|m| m.content).collect())
}
