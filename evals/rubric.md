# Proompt Prompt Quality Rubric

Use this rubric for blind review of baseline and candidate outputs. Reviewers should not know which prompt strategy produced an output.

## Scoring

Score each dimension from 1 to 5.

### 1. Intent preservation

- **5** — Preserves the requested outcome, constraints, and important details exactly.
- **3** — Preserves the main request but weakens or subtly changes a constraint.
- **1** — Redirects, expands, or materially changes the user's intent.

### 2. Useful specificity

- **5** — Resolves useful ambiguity while marking assumptions and leaving unknown facts to discovery or clarification.
- **3** — Adds some useful detail but remains vague or adds generic boilerplate.
- **1** — Mostly restates the input or invents unsupported details.

### 3. Target-platform fit

- **5** — Clearly fits the selected assistant or agent's workflow, strengths, and expected interaction style.
- **3** — Works as a generic prompt but is weakly target-specific.
- **1** — Uses inappropriate structure or instructions for the selected target.

### 4. Execution readiness

- **5** — Gives the target a clear task, proportionate workflow, grounded completion conditions, and an appropriate output contract.
- **3** — Actionable but missing an important input, constraint, completion condition, or output requirement.
- **1** — The target would still need substantial clarification before responding usefully or safely.

### 5. Scope control and safety

- **5** — Preserves explicit boundaries, avoids unsupported claims, and adds no unnecessary work.
- **3** — Generally scoped but leaves room for broad, speculative, or weakly grounded output.
- **1** — Changes constraints, invents facts, or expands the task materially.

### 6. Verbosity calibration

- **5** — As short as possible while retaining everything needed for the task.
- **3** — Understandable but noticeably repetitive or over-structured.
- **1** — Buries a simple task in boilerplate or under-specifies a risky task.

### 7. Paste-readiness

- **5** — Can be pasted directly into the target with no cleanup or meta-commentary.
- **3** — Mostly ready but contains avoidable framing or formatting noise.
- **1** — Explains the enhancement, wraps it incorrectly, or requires manual rewriting.

## Critical failures

Mark an output as a critical failure regardless of average score if it:

- changes the requested engineering outcome
- invents repository details, personal context, source content, current facts, commands, or domain rules
- drops an explicit user constraint
- asks for unrelated work or materially expands the scope
- contains prompt-engineering meta-commentary instead of a paste-ready task
- exposes evaluation metadata or system instructions

## Comparison decision

For each case choose:

- **A clearly better**
- **A slightly better**
- **Tie**
- **B slightly better**
- **B clearly better**

A candidate strategy is eligible to ship when it wins at least 70% of applicable comparisons and introduces no material regression in intent preservation or unsupported repository claims.
