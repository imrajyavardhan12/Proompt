# Prompt Quality Evaluations

This directory contains Proompt's versioned, non-sensitive prompt-quality corpus and review rubric.

Evaluation order:

1. Coding-agent prompts
2. General text prompts
3. Image prompts

Generated outputs belong under `evals/results/` and are ignored by Git. Do not add private prompts, repository content, API keys, or generated result files to the corpus.

## Validate the corpus

```bash
cargo run -p proompt-evals -- validate
```

Validation is offline and makes no provider calls.

## Capture a baseline

Capture uses the provider/model in the normal Proompt configuration. It calls the configured provider once per selected case, may incur API cost, and intentionally requires explicit confirmation:

```bash
cargo run -p proompt-evals -- capture \
  --output evals/results/v0.3.4-coding-baseline.json \
  --confirm-cost
```

Capture does not write normal Proompt history and does not request SuperMemory context.

Use a subset while developing:

```bash
cargo run -p proompt-evals -- capture \
  --case simple-typo-fix \
  --case ambiguous-upload-failure \
  --output evals/results/smoke.json \
  --confirm-cost
```

## Generate a blind comparison

Capture candidate outputs with the same provider/model, then generate a review sheet and separate answer key:

```bash
cargo run -p proompt-evals -- compare \
  --baseline evals/results/v0.3.4-coding-baseline.json \
  --candidate evals/results/coding-candidate-v1.json \
  --output evals/results/coding-candidate-v1-review.md
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
