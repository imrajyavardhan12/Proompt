# Proompt Image-Prompt Quality Rubric

Use this rubric for blind comparison of image-prompt enhancements. Review the generated prompt text, not an image rendered from it.

## Scoring

Score each dimension from 1 to 5.

### 1. Intent preservation

- **5** — Preserves the subject, count, relationships, setting, text, colors, exclusions, and all explicit constraints exactly.
- **3** — Preserves the main image but weakens or subtly changes one detail.
- **1** — Changes the subject, composition, requested text, style, or another material constraint.

### 2. Useful visual specificity

- **5** — Adds coherent visual detail that supports the idea without narrowing unspecified creative choices unnecessarily.
- **3** — Adds generic quality words or leaves important visual ambiguity unresolved.
- **1** — Mostly repeats the input or adds contradictory, incoherent, or unsupported detail.

### 3. Target-platform fit

- **5** — Uses syntax and prompt structure appropriate for the selected image generator.
- **3** — Usable but mostly generic or includes unnecessary platform syntax.
- **1** — Uses invalid, stale, conflicting, or wrong-platform parameters and conventions.

### 4. Composition and constraint control

- **5** — Makes counts, spatial relationships, framing, aspect ratio, typography, and negative constraints unambiguous where supplied.
- **3** — Captures the general composition but leaves an explicit relationship or exclusion weak.
- **1** — Contradicts or drops an explicit visual constraint.

### 5. Unsupported additions

- **5** — Adds only compatible aesthetic detail and does not invent objects, people, text, branding, narrative events, artists, or technical parameters.
- **3** — Includes harmless but unnecessary aesthetic defaults.
- **1** — Adds material content, named styles, parameters, or exclusions that redirect the image.

### 6. Verbosity calibration

- **5** — Uses the minimum detail needed for a strong, controllable image prompt.
- **3** — Usable but repetitive, keyword-stuffed, or under-specified.
- **1** — Buries the image concept in boilerplate or remains too vague to guide generation.

### 7. Paste-readiness

- **5** — Can be pasted directly into the target generator without cleanup.
- **3** — Mostly ready but contains avoidable labels or formatting noise.
- **1** — Contains explanation, meta-commentary, invalid structure, or requires rewriting.

## Critical failures

Mark an output as a critical failure regardless of average score if it:

- changes an explicit subject, count, relationship, color, phrase, or exclusion
- invents people, objects, text, logos, brands, narrative events, or source facts that materially alter the image
- adds a conflicting style, composition, aspect ratio, or platform parameter
- uses a named artist or copyrighted character that the user did not request
- drops a safety, privacy, or no-identification constraint
- outputs commentary instead of a ready-to-paste image prompt

## Comparison decision

For each case choose:

- **A clearly better**
- **A slightly better**
- **Tie**
- **B slightly better**
- **B clearly better**

A candidate strategy is eligible to ship when it wins at least 70% of applicable comparisons and introduces no material regression in intent preservation, explicit visual constraints, or unsupported additions.
