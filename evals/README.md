# Prompt Quality Evaluations

This directory contains Proompt's versioned, non-sensitive prompt-quality corpus and review rubric.

Evaluation order:

1. Coding-agent prompts
2. General text prompts
3. Image prompts

Generated outputs belong under `evals/results/` and are ignored by Git. Do not add private prompts, repository content, API keys, or generated result files to the corpus.

Versioned corpora:

- `evals/coding-agent-cases.json` — coding-agent tasks
- `evals/general-text-cases.json` — rewriting, summarization, translation, education, research, decision support, roleplay, creative writing, brainstorming, and structured transformation
- `evals/image-prompt-cases.json` — image prompts for Midjourney, DALL-E, Stable Diffusion, and generic generators

The general-text category coverage was informed by the CC0 prompt collection at [prompts.chat](https://github.com/f/prompts.chat). Proompt's evaluation cases are curated, purpose-written rough inputs rather than a wholesale copy of community prompts.

## Validate a corpus

```bash
cargo run -p proompt-evals -- validate
cargo run -p proompt-evals -- validate --corpus evals/general-text-cases.json
cargo run -p proompt-evals -- validate --corpus evals/image-prompt-cases.json
```

Validation is offline and makes no provider calls.

## Capture a baseline

Capture uses the provider/model in the normal Proompt configuration. It calls the configured provider once per selected case, may incur API cost, and intentionally requires explicit confirmation:

```bash
cargo run -p proompt-evals -- capture \
  --corpus evals/general-text-cases.json \
  --output evals/results/v0.3.5-general-text-baseline.json \
  --confirm-cost
```

Capture does not write normal Proompt history and does not request SuperMemory context.

Capture the image-prompt baseline with:

```bash
cargo run -p proompt-evals -- capture \
  --corpus evals/image-prompt-cases.json \
  --output evals/results/v0.3.5-image-baseline.json \
  --confirm-cost
```

Image comparisons use the corpus-selected `evals/image-rubric.md` and image-specific scoring dimensions.

Use a subset while developing:

```bash
cargo run -p proompt-evals -- capture \
  --corpus evals/general-text-cases.json \
  --case professional-email-rewrite \
  --case ambiguous-leadership-summary \
  --output evals/results/general-text-smoke.json \
  --confirm-cost
```

## Generate a blind comparison

Capture candidate outputs with the same provider/model, then generate a review sheet and separate answer key:

```bash
cargo run -p proompt-evals -- compare \
  --corpus evals/general-text-cases.json \
  --baseline evals/results/v0.3.5-general-text-baseline.json \
  --candidate evals/results/general-text-candidate-v1.json \
  --output evals/results/general-text-candidate-v1-review.md
```

Do not open the generated `.key.json` file until every case has been scored.

## Review workflow

1. Capture the current release baseline before changing prompt builders.
2. Capture candidate outputs with the same provider/model and corpus.
3. Generate the randomized blind review sheet.
4. Score using `evals/rubric.md` without opening the answer key.
5. Record failure patterns, not only aggregate scores.
6. Ship only when the candidate clears the comparison gate without critical regressions.

LLM judging can assist later, but human blind review remains the source of truth for product-quality decisions.
