use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use proompt_core::{
    config,
    enhance::{ConfiguredEnhanceRequest, enhance_with_loaded_config, prepare_enhancement},
    platform::{EnhanceType, parse_platform},
};
use serde::{Deserialize, Serialize};

const DEFAULT_CORPUS: &str = "evals/coding-agent-cases.json";

#[derive(Debug, Parser)]
#[command(
    name = "proompt-evals",
    about = "Local prompt-quality evaluation tooling for Proompt"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate an evaluation corpus without making provider calls.
    Validate {
        #[arg(long, default_value = DEFAULT_CORPUS)]
        corpus: PathBuf,
    },
    /// Capture enhanced outputs using the configured provider and model.
    Capture {
        #[arg(long, default_value = DEFAULT_CORPUS)]
        corpus: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Capture only these case IDs. Repeat for multiple cases.
        #[arg(long = "case")]
        case_ids: Vec<String>,
        /// Limit the number of selected cases after filtering.
        #[arg(long)]
        limit: Option<usize>,
        /// Delay between provider calls to reduce burst rate-limit risk.
        #[arg(long, default_value_t = 750)]
        delay_ms: u64,
        /// Required acknowledgement that capture makes paid provider calls.
        #[arg(long)]
        confirm_cost: bool,
        /// Replace an existing local result file.
        #[arg(long)]
        overwrite: bool,
    },
    /// Generate a randomized local A/B review sheet and separate answer key.
    Compare {
        #[arg(long, default_value = DEFAULT_CORPUS)]
        corpus: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        candidate: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Answer-key path. Defaults to the review path with .key.json extension.
        #[arg(long)]
        key_output: Option<PathBuf>,
        /// Stable seed for reproducible A/B assignment.
        #[arg(long, default_value_t = 304)]
        seed: u64,
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalCorpus {
    schema_version: u32,
    suite: String,
    description: String,
    cases: Vec<EvalCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalCase {
    id: String,
    category: String,
    prompt: String,
    platform: String,
    complexity: String,
    expectations: EvalExpectations,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalExpectations {
    preserve: Vec<String>,
    should_include: Vec<String>,
    must_avoid: Vec<String>,
    verbosity: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalRun {
    schema_version: u32,
    suite: String,
    corpus_description: String,
    generated_at_ms: u64,
    proompt_version: String,
    git_commit: Option<String>,
    git_dirty: Option<bool>,
    provider: String,
    model: String,
    results: Vec<EvalResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalResult {
    case_id: String,
    category: String,
    complexity: String,
    platform: String,
    input: String,
    enhanced_output: Option<String>,
    latency_ms: u64,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlindAnswerKey {
    seed: u64,
    baseline_file: String,
    candidate_file: String,
    baseline_commit: Option<String>,
    candidate_commit: Option<String>,
    /// Maps case ID to the label containing the candidate output.
    candidate_labels: BTreeMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { corpus } => {
            let corpus = load_corpus(&corpus)?;
            validate_corpus(&corpus)?;
            println!(
                "Validated {} cases in '{}' (schema v{}).",
                corpus.cases.len(),
                corpus.suite,
                corpus.schema_version
            );
        }
        Commands::Capture {
            corpus,
            output,
            case_ids,
            limit,
            delay_ms,
            confirm_cost,
            overwrite,
        } => {
            if !confirm_cost {
                anyhow::bail!(
                    "Capture makes provider API calls and may incur cost. Re-run with --confirm-cost."
                );
            }
            capture(&corpus, &output, &case_ids, limit, delay_ms, overwrite).await?;
        }
        Commands::Compare {
            corpus,
            baseline,
            candidate,
            output,
            key_output,
            seed,
            overwrite,
        } => compare(
            &corpus,
            &baseline,
            &candidate,
            &output,
            key_output.as_deref(),
            seed,
            overwrite,
        )?,
    }

    Ok(())
}

fn compare(
    corpus_path: &Path,
    baseline_path: &Path,
    candidate_path: &Path,
    output_path: &Path,
    key_output_path: Option<&Path>,
    seed: u64,
    overwrite: bool,
) -> Result<()> {
    if baseline_path == candidate_path {
        anyhow::bail!("Baseline and candidate files must be different");
    }

    let default_key_path = output_path.with_extension("key.json");
    let key_output_path = key_output_path.unwrap_or(&default_key_path);
    for path in [output_path, key_output_path] {
        if path.exists() && !overwrite {
            anyhow::bail!(
                "Output already exists at {}. Use --overwrite to replace it.",
                path.display()
            );
        }
    }

    let corpus = load_corpus(corpus_path)?;
    validate_corpus(&corpus)?;
    let baseline = load_run(baseline_path)?;
    let candidate = load_run(candidate_path)?;
    validate_comparable_runs(&corpus, &baseline, &candidate)?;

    let baseline_results = baseline
        .results
        .iter()
        .map(|result| (result.case_id.as_str(), result))
        .collect::<HashMap<_, _>>();
    let candidate_results = candidate
        .results
        .iter()
        .map(|result| (result.case_id.as_str(), result))
        .collect::<HashMap<_, _>>();

    let mut review = String::new();
    writeln!(review, "# Blind Prompt Quality Review")?;
    writeln!(review)?;
    writeln!(
        review,
        "Suite: `{}`  \nProvider/model: `{}` / `{}`  \nCases: {}  \nSeed: `{}`",
        corpus.suite,
        baseline.provider,
        baseline.model,
        corpus.cases.len(),
        seed
    )?;
    writeln!(review)?;
    writeln!(
        review,
        "Do not open the separate answer key until every case has a decision. Score with `evals/rubric.md`."
    )?;

    let mut candidate_labels = BTreeMap::new();
    for (index, case) in corpus.cases.iter().enumerate() {
        let baseline_result = baseline_results[case.id.as_str()];
        let candidate_result = candidate_results[case.id.as_str()];
        let baseline_output = completed_output(baseline_result)?;
        let candidate_output = completed_output(candidate_result)?;
        let candidate_label = blind_candidate_label(&case.id, seed);
        candidate_labels.insert(case.id.clone(), candidate_label.to_string());
        let (output_a, output_b) = if candidate_label == "A" {
            (candidate_output, baseline_output)
        } else {
            (baseline_output, candidate_output)
        };

        writeln!(review)?;
        writeln!(review, "---")?;
        writeln!(review)?;
        writeln!(review, "## {}. `{}`", index + 1, case.id)?;
        writeln!(review)?;
        writeln!(
            review,
            "**Target:** {} · **Category:** {} · **Complexity:** {} · **Expected verbosity:** {}",
            case.platform, case.category, case.complexity, case.expectations.verbosity
        )?;
        writeln!(review)?;
        writeln!(review, "**Original input**")?;
        writeln!(review)?;
        writeln!(review, "````text\n{}\n````", case.prompt)?;
        writeln!(review)?;
        writeln!(
            review,
            "**Preserve:** {}  \n**Should include:** {}  \n**Must avoid:** {}",
            case.expectations.preserve.join("; "),
            case.expectations.should_include.join("; "),
            case.expectations.must_avoid.join("; ")
        )?;
        writeln!(review)?;
        writeln!(review, "### Output A")?;
        writeln!(review)?;
        writeln!(review, "````text\n{}\n````", output_a)?;
        writeln!(review)?;
        writeln!(review, "### Output B")?;
        writeln!(review)?;
        writeln!(review, "````text\n{}\n````", output_b)?;
        writeln!(review)?;
        writeln!(review, "| Dimension | A (1–5) | B (1–5) |")?;
        writeln!(review, "| --- | ---: | ---: |")?;
        for dimension in [
            "Intent preservation",
            "Useful specificity",
            "Target-platform fit",
            "Execution readiness",
            "Scope control and safety",
            "Verbosity calibration",
            "Paste-readiness",
        ] {
            writeln!(review, "| {} |  |  |", dimension)?;
        }
        writeln!(review)?;
        writeln!(
            review,
            "**Critical failure:** A [ ] · B [ ]  \n**Decision:** A clearly better [ ] · A slightly better [ ] · Tie [ ] · B slightly better [ ] · B clearly better [ ]  \n**Notes:**"
        )?;
    }

    let key = BlindAnswerKey {
        seed,
        baseline_file: baseline_path.display().to_string(),
        candidate_file: candidate_path.display().to_string(),
        baseline_commit: baseline.git_commit,
        candidate_commit: candidate.git_commit,
        candidate_labels,
    };

    write_output(output_path, review.as_bytes())?;
    let key_content = serde_json::to_vec_pretty(&key).context("Failed to serialize answer key")?;
    write_output(key_output_path, &key_content)?;

    println!(
        "Generated blind review at {} and answer key at {}.",
        output_path.display(),
        key_output_path.display()
    );
    Ok(())
}

fn validate_comparable_runs(
    corpus: &EvalCorpus,
    baseline: &EvalRun,
    candidate: &EvalRun,
) -> Result<()> {
    if baseline.suite != corpus.suite || candidate.suite != corpus.suite {
        anyhow::bail!("Corpus and run suites must match");
    }
    if baseline.provider != candidate.provider || baseline.model != candidate.model {
        anyhow::bail!(
            "Baseline and candidate must use the same provider/model ({} / {} vs {} / {})",
            baseline.provider,
            baseline.model,
            candidate.provider,
            candidate.model
        );
    }

    let expected_ids = corpus
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<HashSet<_>>();
    for (label, run) in [("baseline", baseline), ("candidate", candidate)] {
        let result_ids = run
            .results
            .iter()
            .map(|result| result.case_id.as_str())
            .collect::<HashSet<_>>();
        if result_ids != expected_ids || run.results.len() != corpus.cases.len() {
            anyhow::bail!("{} run does not contain exactly the corpus cases", label);
        }
        for case in &corpus.cases {
            let result = run
                .results
                .iter()
                .find(|result| result.case_id == case.id)
                .expect("case ID sets were checked above");
            if result.input != case.prompt || result.platform != case.platform {
                anyhow::bail!("{} result '{}' does not match the corpus", label, case.id);
            }
            completed_output(result)?;
        }
    }
    Ok(())
}

fn completed_output(result: &EvalResult) -> Result<&str> {
    if let Some(error) = &result.error {
        anyhow::bail!("Case '{}' has provider error: {}", result.case_id, error);
    }
    result
        .enhanced_output
        .as_deref()
        .filter(|output| !output.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("Case '{}' has no enhanced output", result.case_id))
}

fn blind_candidate_label(case_id: &str, seed: u64) -> &'static str {
    let mut hash = 0xcbf29ce484222325_u64 ^ seed;
    for byte in case_id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    if hash & 1 == 0 { "A" } else { "B" }
}

fn load_run(path: &Path) -> Result<EvalRun> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read eval run from {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse eval run from {}", path.display()))
}

fn write_output(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))
}

async fn capture(
    corpus_path: &Path,
    output_path: &Path,
    case_ids: &[String],
    limit: Option<usize>,
    delay_ms: u64,
    overwrite: bool,
) -> Result<()> {
    if output_path.exists() && !overwrite {
        anyhow::bail!(
            "Output already exists at {}. Use --overwrite to replace it.",
            output_path.display()
        );
    }

    let corpus = load_corpus(corpus_path)?;
    validate_corpus(&corpus)?;
    let selected = select_cases(&corpus, case_ids, limit)?;
    let config = config::load_config()?;

    let first_input = configured_request(selected[0]);
    let prepared = prepare_enhancement(&config, &first_input)?;
    eprintln!(
        "Capturing {} '{}' cases via {} / {}...",
        selected.len(),
        corpus.suite,
        prepared.provider,
        prepared.model
    );

    let mut results = Vec::with_capacity(selected.len());
    for (index, case) in selected.iter().enumerate() {
        eprintln!("[{}/{}] {}", index + 1, selected.len(), case.id);
        let started = Instant::now();
        let response = enhance_with_loaded_config(configured_request(case), config.clone()).await;
        let latency_ms = duration_ms(started.elapsed());

        let result = match response {
            Ok(response) => EvalResult {
                case_id: case.id.clone(),
                category: case.category.clone(),
                complexity: case.complexity.clone(),
                platform: case.platform.clone(),
                input: case.prompt.clone(),
                enhanced_output: Some(response.response.enhanced_prompt),
                latency_ms,
                error: None,
            },
            Err(error) => EvalResult {
                case_id: case.id.clone(),
                category: case.category.clone(),
                complexity: case.complexity.clone(),
                platform: case.platform.clone(),
                input: case.prompt.clone(),
                enhanced_output: None,
                latency_ms,
                error: Some(error.to_string()),
            },
        };
        results.push(result);

        if index + 1 < selected.len() && delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    let run = EvalRun {
        schema_version: corpus.schema_version,
        suite: corpus.suite,
        corpus_description: corpus.description,
        generated_at_ms: now_ms(),
        proompt_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: git_output(&["rev-parse", "HEAD"]),
        git_dirty: git_dirty(),
        provider: prepared.provider,
        model: prepared.model,
        results,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(&run).context("Failed to serialize eval run")?;
    fs::write(output_path, content)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    let failures = run
        .results
        .iter()
        .filter(|result| result.error.is_some())
        .count();
    println!(
        "Captured {} cases to {} ({} provider failures).",
        run.results.len(),
        output_path.display(),
        failures
    );
    if failures > 0 {
        anyhow::bail!(
            "Evaluation capture completed with {} provider failure(s); partial results were saved for diagnosis.",
            failures
        );
    }

    Ok(())
}

fn configured_request(case: &EvalCase) -> ConfiguredEnhanceRequest {
    ConfiguredEnhanceRequest {
        prompt: case.prompt.clone(),
        platform: Some(case.platform.clone()),
        enhancement_type: Some(EnhanceType::Text),
        include_memory: false,
        style_hints: None,
        max_tokens: None,
    }
}

fn load_corpus(path: &Path) -> Result<EvalCorpus> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read corpus from {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse corpus from {}", path.display()))
}

fn validate_corpus(corpus: &EvalCorpus) -> Result<()> {
    if corpus.schema_version != 1 {
        anyhow::bail!(
            "Unsupported corpus schema version {}",
            corpus.schema_version
        );
    }
    if corpus.suite.trim().is_empty() {
        anyhow::bail!("Corpus suite cannot be empty");
    }
    if corpus.description.trim().is_empty() {
        anyhow::bail!("Corpus description cannot be empty");
    }
    if corpus.cases.is_empty() {
        anyhow::bail!("Corpus must include at least one case");
    }

    let mut ids = HashSet::new();
    for case in &corpus.cases {
        if case.id.trim().is_empty() {
            anyhow::bail!("Case ID cannot be empty");
        }
        if !ids.insert(case.id.as_str()) {
            anyhow::bail!("Duplicate case ID '{}'", case.id);
        }
        if case.category.trim().is_empty() || case.complexity.trim().is_empty() {
            anyhow::bail!("Case '{}' must define category and complexity", case.id);
        }
        if case.prompt.trim().is_empty() {
            anyhow::bail!("Case '{}' prompt cannot be empty", case.id);
        }
        let platform = parse_platform(&case.platform).ok_or_else(|| {
            anyhow::anyhow!(
                "Case '{}' has invalid platform '{}'",
                case.id,
                case.platform
            )
        })?;
        if !platform.is_text_platform() {
            anyhow::bail!("Case '{}' must use a text platform", case.id);
        }
        if case.expectations.preserve.is_empty()
            || case.expectations.should_include.is_empty()
            || case.expectations.must_avoid.is_empty()
            || case.expectations.verbosity.trim().is_empty()
        {
            anyhow::bail!("Case '{}' has incomplete expectations", case.id);
        }
    }

    Ok(())
}

fn select_cases<'a>(
    corpus: &'a EvalCorpus,
    case_ids: &[String],
    limit: Option<usize>,
) -> Result<Vec<&'a EvalCase>> {
    let requested = case_ids.iter().map(String::as_str).collect::<HashSet<_>>();
    if !requested.is_empty() {
        let available = corpus
            .cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<HashSet<_>>();
        let mut unknown = requested
            .difference(&available)
            .copied()
            .collect::<Vec<_>>();
        unknown.sort_unstable();
        if !unknown.is_empty() {
            anyhow::bail!("Unknown case ID(s): {}", unknown.join(", "));
        }
    }

    let mut selected = corpus
        .cases
        .iter()
        .filter(|case| requested.is_empty() || requested.contains(case.id.as_str()))
        .collect::<Vec<_>>();
    if let Some(limit) = limit {
        selected.truncate(limit);
    }
    if selected.is_empty() {
        anyhow::bail!("No evaluation cases selected");
    }
    Ok(selected)
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}

fn git_dirty() -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| !String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_corpus_is_valid() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(DEFAULT_CORPUS);
        let corpus = load_corpus(&path).unwrap();

        validate_corpus(&corpus).unwrap();
        assert!(corpus.cases.len() >= 10);
    }

    #[test]
    fn duplicate_case_ids_are_rejected() {
        let mut corpus = sample_corpus();
        corpus.cases.push(corpus.cases[0].clone());

        let error = validate_corpus(&corpus).unwrap_err().to_string();

        assert!(error.contains("Duplicate case ID"));
    }

    #[test]
    fn selection_rejects_unknown_case_ids() {
        let corpus = sample_corpus();

        let error = select_cases(&corpus, &["missing".to_string()], None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("Unknown case ID"));
    }

    #[test]
    fn selection_preserves_corpus_order_and_applies_limit() {
        let mut corpus = sample_corpus();
        let mut second = corpus.cases[0].clone();
        second.id = "second".to_string();
        corpus.cases.push(second);

        let selected = select_cases(&corpus, &[], Some(1)).unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "sample");
    }

    #[test]
    fn blind_assignment_is_stable_and_uses_both_labels() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(DEFAULT_CORPUS);
        let corpus = load_corpus(&path).unwrap();
        let labels = corpus
            .cases
            .iter()
            .map(|case| blind_candidate_label(&case.id, 304))
            .collect::<HashSet<_>>();

        assert_eq!(blind_candidate_label("sample", 304), "B");
        assert_eq!(labels, HashSet::from(["A", "B"]));
    }

    #[test]
    fn comparison_rejects_model_mismatch() {
        let corpus = sample_corpus();
        let baseline = sample_run("model-a");
        let candidate = sample_run("model-b");

        let error = validate_comparable_runs(&corpus, &baseline, &candidate)
            .unwrap_err()
            .to_string();

        assert!(error.contains("same provider/model"));
    }

    fn sample_run(model: &str) -> EvalRun {
        EvalRun {
            schema_version: 1,
            suite: "sample".to_string(),
            corpus_description: "Sample corpus".to_string(),
            generated_at_ms: 1,
            proompt_version: "0.0.0".to_string(),
            git_commit: None,
            git_dirty: Some(false),
            provider: "provider".to_string(),
            model: model.to_string(),
            results: vec![EvalResult {
                case_id: "sample".to_string(),
                category: "bug_fix".to_string(),
                complexity: "ambiguous".to_string(),
                platform: "claude-code".to_string(),
                input: "fix the bug".to_string(),
                enhanced_output: Some("Investigate and fix the bug.".to_string()),
                latency_ms: 1,
                error: None,
            }],
        }
    }

    fn sample_corpus() -> EvalCorpus {
        EvalCorpus {
            schema_version: 1,
            suite: "sample".to_string(),
            description: "Sample corpus".to_string(),
            cases: vec![EvalCase {
                id: "sample".to_string(),
                category: "bug_fix".to_string(),
                prompt: "fix the bug".to_string(),
                platform: "claude-code".to_string(),
                complexity: "ambiguous".to_string(),
                expectations: EvalExpectations {
                    preserve: vec!["Fix the bug".to_string()],
                    should_include: vec!["Investigate".to_string()],
                    must_avoid: vec!["Invented paths".to_string()],
                    verbosity: "moderate".to_string(),
                },
            }],
        }
    }
}
