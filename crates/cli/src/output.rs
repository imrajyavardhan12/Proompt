use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ── Styles ──────────────────────────────────────────────────

fn style_accent() -> Style {
    Style::new().cyan()
}

fn style_muted() -> Style {
    Style::new().dim()
}

fn style_success() -> Style {
    Style::new().green()
}

fn style_warn() -> Style {
    Style::new().yellow()
}

fn style_err() -> Style {
    Style::new().red()
}

fn style_bold() -> Style {
    Style::new().bold()
}

fn style_header() -> Style {
    Style::new().cyan().bold()
}

// ── Messages ────────────────────────────────────────────────

pub fn success(msg: &str) {
    eprintln!("{} {}", style_success().apply_to("✓"), msg);
}

pub fn info(msg: &str) {
    eprintln!("{} {}", style_accent().apply_to("ℹ"), msg);
}

pub fn warn(msg: &str) {
    eprintln!("{} {}", style_warn().apply_to("⚠"), msg);
}

#[allow(dead_code)]
pub fn error(msg: &str) {
    eprintln!("{} {}", style_err().apply_to("✗"), msg);
}

pub fn dim(msg: &str) {
    eprintln!("{}", style_muted().apply_to(msg));
}

// ── Spinner ─────────────────────────────────────────────────

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

// ── Banner ──────────────────────────────────────────────────

pub fn banner() {
    let accent = style_accent();
    let muted = style_muted();
    let bold = style_bold();

    eprintln!();
    eprintln!(
        "  {}",
        accent.apply_to("┌─────────────────────────────────────┐")
    );
    eprintln!(
        "  {}  {}  {}",
        accent.apply_to("│"),
        bold.apply_to("⚡ Proompt"),
        accent.apply_to("                        │")
    );
    eprintln!(
        "  {}  {}  {}",
        accent.apply_to("│"),
        muted.apply_to("Transform prompts. Get better AI."),
        accent.apply_to(" │")
    );
    eprintln!(
        "  {}",
        accent.apply_to("└─────────────────────────────────────┘")
    );
    eprintln!();
}

pub fn usage_hint() {
    let accent = style_accent();
    let muted = style_muted();
    let header = style_header();

    eprintln!("  {} Quick Start", header.apply_to("▸"));
    eprintln!();
    eprintln!(
        "    {}  proompt {}",
        accent.apply_to("enhance"),
        muted.apply_to("\"your rough prompt\"")
    );
    eprintln!(
        "    {}  proompt --image {}",
        accent.apply_to("image  "),
        muted.apply_to("\"a cat in space\"")
    );
    eprintln!(
        "    {}  proompt --template ghibli-style {}",
        accent.apply_to("template"),
        muted.apply_to("\"subject\"")
    );
    eprintln!(
        "    {}  proompt --platform openai {}",
        accent.apply_to("platform"),
        muted.apply_to("\"your prompt\"")
    );
    eprintln!();
    eprintln!("  {} Commands", header.apply_to("▸"));
    eprintln!();
    eprintln!(
        "    {}              {}",
        accent.apply_to("proompt config show"),
        muted.apply_to("Show current settings")
    );
    eprintln!(
        "    {}        {}",
        accent.apply_to("proompt config set <k> <v>"),
        muted.apply_to("Update a setting")
    );
    eprintln!(
        "    {}             {}",
        accent.apply_to("proompt history list"),
        muted.apply_to("Browse local prompt history")
    );
    eprintln!(
        "    {}           {}",
        accent.apply_to("proompt templates list"),
        muted.apply_to("Browse viral templates")
    );
    eprintln!(
        "    {}           {}",
        accent.apply_to("proompt templates sync"),
        muted.apply_to("Fetch latest templates")
    );
    eprintln!();
    eprintln!("  {}", muted.apply_to("Run proompt --help for all options"));
    eprintln!();
}

// ── Section display ─────────────────────────────────────────

pub fn section_header(title: &str) {
    let header = style_header();
    let muted = style_muted();
    let width: usize = 50;
    let dash_count = width.saturating_sub(title.len() + 3);
    eprintln!(
        "  {} {}",
        header.apply_to(format!("─ {}", title)),
        muted.apply_to("─".repeat(dash_count))
    );
}

pub fn enhanced_output(original: &str, enhanced: &str, summary: &str, platform: &str) {
    let muted = style_muted();
    let accent = style_accent();

    eprintln!();
    section_header("Original");
    eprintln!("  {}", muted.apply_to(truncate(original, 120)));
    eprintln!();
    section_header("Enhanced");
    eprintln!();

    // Print the enhanced prompt to stdout (for piping)
    println!("{}", enhanced);

    eprintln!();
    section_header("Info");
    eprintln!(
        "  {} {}   {} {}",
        muted.apply_to("platform:"),
        accent.apply_to(platform),
        muted.apply_to("changes:"),
        summary
    );
    eprintln!();
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

    #[test]
    fn truncate_leaves_short_unicode_unchanged() {
        assert_eq!(truncate("🙂🙂", 2), "🙂🙂");
    }
}
