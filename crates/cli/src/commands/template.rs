use std::collections::HashMap;

use anyhow::Result;
use console::Style;
use proompt_core::templates::{
    self, TemplateCategory, TemplateFilter, TemplateManager,
};

use crate::output;

pub fn list(trending_only: bool) -> Result<()> {
    let manager = TemplateManager::new();
    let filter = TemplateFilter {
        trending_only,
        ..Default::default()
    };
    let templates = manager.list(&filter);

    if templates.is_empty() {
        output::info("No templates found");
        return Ok(());
    }

    let accent = Style::new().cyan();
    let muted = Style::new().dim();
    let bold = Style::new().bold();
    let trending_style = Style::new().magenta();

    output::section_header("Templates");
    eprintln!();

    for t in &templates {
        let badge = match t.category {
            TemplateCategory::Image => Style::new().blue().apply_to(" IMG "),
            TemplateCategory::Text => Style::new().green().apply_to(" TXT "),
        };
        let trending_badge = if t.trending {
            format!(" {}", trending_style.apply_to("★ trending"))
        } else {
            String::new()
        };

        eprintln!(
            "  {} {} {}{}",
            badge,
            bold.apply_to(format!("{:<22}", t.id)),
            muted.apply_to(&t.description),
            trending_badge
        );
    }

    eprintln!();
    eprintln!(
        "  {} templates available",
        accent.apply_to(templates.len())
    );
    eprintln!();
    output::dim("  Usage: proompt --template <id> \"your subject\"");
    eprintln!();

    Ok(())
}

pub async fn sync() -> Result<()> {
    let spinner = output::spinner("Syncing templates from remote...");
    match templates::sync_templates_from_remote(templates::DEFAULT_TEMPLATES_URL).await {
        Ok(fetched) => {
            spinner.finish_and_clear();
            output::success(&format!("Synced {} templates from remote", fetched.len()));
        }
        Err(e) => {
            spinner.finish_and_clear();
            output::warn(&format!(
                "Could not sync remote templates: {}. Using built-in templates.",
                e
            ));
        }
    }
    Ok(())
}

pub fn apply_from_cli(template_id: &str, subject: Option<&str>) -> Result<()> {
    let manager = TemplateManager::new();

    let template = manager
        .get(template_id)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

    let subject = subject.ok_or_else(|| {
        anyhow::anyhow!(
            "Template '{}' requires a subject. Usage: proompt --template {} \"your subject\"",
            template_id,
            template_id
        )
    })?;

    let mut fields = HashMap::new();
    let field_name = template
        .fields
        .first()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "subject".to_string());
    fields.insert(field_name, subject.to_string());

    let result = manager.apply(template_id, &fields)?;

    let muted = Style::new().dim();

    eprintln!();
    output::section_header(&format!("Template: {}", template_id));
    eprintln!();
    println!("{}", result);
    eprintln!();
    eprintln!("  {} {}", muted.apply_to("platform:"), template.platform);
    eprintln!();

    Ok(())
}
