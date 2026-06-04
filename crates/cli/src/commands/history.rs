use anyhow::Result;
use console::Style;
use proompt_core::history;

use crate::output;

pub fn list(limit: usize, favorites_only: bool) -> Result<()> {
    let mut records = history::load_history()?;
    if favorites_only {
        records.retain(|record| record.favorite);
    }
    records.truncate(limit);

    if records.is_empty() {
        output::info(if favorites_only {
            "No favorite prompts saved yet"
        } else {
            "No prompt history saved yet"
        });
        return Ok(());
    }

    let muted = Style::new().dim();
    let accent = Style::new().cyan();
    let favorite = Style::new().yellow();

    output::section_header(if favorites_only {
        "Favorite History"
    } else {
        "Prompt History"
    });
    eprintln!();

    for record in &records {
        let star = if record.favorite {
            favorite.apply_to("★").to_string()
        } else {
            muted.apply_to("·").to_string()
        };
        eprintln!(
            "  {} {} {} {}",
            star,
            accent.apply_to(&record.id),
            muted.apply_to(record.platform.to_string()),
            muted.apply_to(format!("{} / {}", record.provider, record.model))
        );
        eprintln!(
            "    {} {}",
            muted.apply_to("original:"),
            truncate(&record.original_prompt, 96)
        );
        eprintln!(
            "    {} {}",
            muted.apply_to("enhanced:"),
            truncate(&record.enhanced_prompt, 96)
        );
        eprintln!();
    }

    output::dim("  Tip: proompt history favorite <id> to keep a prompt at hand");
    output::dim("  Tip: proompt history delete <id> to remove a prompt");
    eprintln!();

    Ok(())
}

pub fn favorite(id: &str, unset: bool) -> Result<()> {
    let record = history::set_history_favorite(id, !unset)?;
    if record.favorite {
        output::success(&format!("Marked '{}' as favorite", record.id));
    } else {
        output::success(&format!("Removed favorite from '{}'", record.id));
    }
    Ok(())
}

pub fn delete(id: &str) -> Result<()> {
    if history::delete_history_record(id)? {
        output::success(&format!("Deleted history record '{}'", id));
    } else {
        output::info(&format!("History record '{}' was already absent", id));
    }
    Ok(())
}

pub fn clear() -> Result<()> {
    let count = history::clear_history()?;
    output::success(&format!(
        "Cleared {} history record{}",
        count,
        if count == 1 { "" } else { "s" }
    ));
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((idx, _)) => format!("{}...", &s[..idx]),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_handles_unicode_boundaries() {
        assert_eq!(truncate("áéíóú", 3), "áéí...");
    }
}
