# PromptEnhancer - Product Requirements Document

## Section 1: Document Info & Executive Summary

---

### Document Info

| Field | Value |
|-------|-------|
| **Product Name** | PromptEnhancer |
| **Version** | 1.0.0-MVP |
| **Author** | Architecture Agent |
| **Last Updated** | 2025-01-13 |
| **Status** | Ready for Implementation |

---

### Table of Contents

1. Executive Summary ← **(This section)**
2. Problem Statement & Goals
3. User Personas
4. Product Overview & Features
5. Architecture
6. System Tray App Design
7. CLI Design
8. Core Enhancement Engine
9. SuperMemory Integration
10. Image Prompt Enhancement
11. Viral Templates System
12. Data Design & Storage
13. API Specification
14. Security & Privacy
15. Monetization & Billing
16. Observability & Analytics
17. Testing Strategy
18. Work Breakdown & Delivery Plan

---

### Executive Summary

#### What We're Building

**PromptEnhancer** is a universal prompt enhancement tool delivered as a **System Tray application** with an accompanying **CLI**. It helps users transform rough, unstructured prompts into well-crafted, platform-optimized prompts that yield significantly better AI responses.

The tool works across all AI platforms (Claude, ChatGPT, Gemini, Midjourney, etc.) and supports three enhancement modes:
1. **Text LLM Enhancement** - Optimize prompts for conversational AI
2. **Image Generation Enhancement** - Craft detailed prompts for image AI
3. **Viral Templates** - One-click access to trending prompt formats

#### Key Differentiators

| Differentiator | Description |
|----------------|-------------|
| **Universal Access** | System tray + CLI - works everywhere, one install |
| **Platform-Aware** | Tailors prompts specifically for Claude, GPT, Gemini, Midjourney, etc. |
| **SuperMemory Integration** | Enriches prompts with user's personal context and knowledge |
| **Open Source + BYOK** | Users can bring their own API keys - free forever |
| **Hosted Option** | Convenience tier for users who don't want to manage keys |
| **Image + Viral Support** | Not just text - covers image generation and trending formats |

#### Business Model

| Tier | Price | What's Included |
|------|-------|-----------------|
| **BYOK (Free)** | $0 forever | Full features, user pays LLM provider directly |
| **Hosted Pro** | $8/month | Full features, we provide API access, no key needed |
| **Trial** | $0 | 5 enhancements/day, no signup required |

#### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Core Engine** | Rust | Performance, single binary, shared between tray & CLI |
| **System Tray App** | Tauri (Rust + Web UI) | Small bundle (~20MB), native feel, cross-platform |
| **UI Framework** | SvelteKit or React | Fast, lightweight, good Tauri integration |
| **Backend (Hosted)** | Supabase | Auth, DB, Edge Functions - minimal ops for solo dev |
| **Payments** | Stripe | Industry standard, easy subscription management |
| **LLM Provider** | Anthropic Claude (primary) | Best quality for prompt rewriting |

#### Success Metrics (MVP)

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Installation Success** | >95% | Users complete setup without errors |
| **Enhancement Acceptance** | >80% | User uses enhanced prompt vs discards |
| **Daily Active Users** | 500+ | Within 2 months of launch |
| **BYOK vs Hosted Split** | 70/30 | Healthy mix indicates both tiers valuable |
| **P95 Latency** | <3 seconds | Time from trigger to enhanced prompt |

---

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              User Interfaces                                 │
│                                                                              │
│         ┌─────────────────────┐         ┌─────────────────────┐             │
│         │   System Tray App   │         │        CLI          │             │
│         │      (Tauri)        │         │      (Rust)         │             │
│         │                     │         │                     │             │
│         │  • GUI Windows      │         │  • Terminal usage   │             │
│         │  • Global Hotkeys   │         │  • Scripting        │             │
│         │  • Settings UI      │         │  • Piping           │             │
│         │  • Notifications    │         │  • Automation       │             │
│         └──────────┬──────────┘         └──────────┬──────────┘             │
│                    │                               │                         │
│                    └───────────────┬───────────────┘                         │
│                                    │                                         │
│                                    ▼                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                      Core Engine (Rust Library)                        │  │
│  │                                                                        │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────────────────┐ │  │
│  │  │ Enhance    │ │ Platform   │ │ Template   │ │ SuperMemory         │ │  │
│  │  │ Engine     │ │ Detector   │ │ Manager    │ │ Client              │ │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └─────────────────────┘ │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────────────────┐ │  │
│  │  │ Image      │ │ Config     │ │ API Client │ │ Usage               │ │  │
│  │  │ Enhancer   │ │ Manager    │ │ (LLM)      │ │ Tracker             │ │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └─────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│                    ┌───────────────┴───────────────┐                        │
│                    ▼                               ▼                        │
│           ┌──────────────────┐           ┌──────────────────┐               │
│           │    BYOK Mode     │           │   Hosted Mode    │               │
│           │                  │           │                  │               │
│           │  User's API Key  │           │  Our Backend     │               │
│           │  Direct to LLM   │           │  (Supabase)      │               │
│           └────────┬─────────┘           └────────┬─────────┘               │
│                    │                              │                          │
│                    ▼                              ▼                          │
│           ┌──────────────────┐           ┌──────────────────┐               │
│           │  LLM Providers   │           │  Our LLM Keys    │               │
│           │  (Claude/GPT)    │           │  (via Backend)   │               │
│           └──────────────────┘           └──────────────────┘               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

External Integrations:
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│   SuperMemory    │  │     Stripe       │  │  GitHub (for     │
│   (Context)      │  │   (Billing)      │  │  templates repo) │
└──────────────────┘  └──────────────────┘  └──────────────────┘
```

---

### MVP Scope Summary

#### In Scope (v1.0)

| Feature | Priority | Notes |
|---------|----------|-------|
| System Tray App (macOS, Windows, Linux) | P0 | Core delivery mechanism |
| CLI | P0 | Power user interface |
| Text LLM Enhancement | P0 | Claude, GPT, Gemini, Generic |
| Image Prompt Enhancement | P0 | Midjourney, DALL-E, Stable Diffusion |
| Viral Templates | P0 | Curated list, community submissions |
| BYOK Mode | P0 | Anthropic, OpenAI keys |
| Hosted Mode | P0 | Stripe billing, usage tracking |
| SuperMemory Integration | P0 | Optional context enrichment |
| Global Hotkeys | P0 | Quick enhance, open window |
| Settings UI | P0 | API keys, preferences, integrations |

#### Out of Scope (Future)

| Feature | Version | Notes |
|---------|---------|-------|
| Browser Extension | v1.5 | DOM injection for seamless experience |
| Team/Enterprise Features | v2.0 | Shared templates, admin controls |
| Custom Enhancement Rules | v2.0 | User-defined enhancement logic |
| Prompt History & Analytics | v1.5 | Track what worked well |
| Mobile App | v2.0+ | iOS/Android |

---

## Section 2: Problem Statement & Goals

---

### The Problem

Users interact with AI assistants daily across multiple platforms—ChatGPT, Claude, Gemini, Midjourney, coding agents like Claude Code and Cursor. However, most users write suboptimal prompts that lead to poor AI responses.

#### Common Prompt Problems

| Problem | Example | Result |
|---------|---------|--------|
| **Too vague** | "help me with code" | AI asks clarifying questions, wastes time |
| **Missing context** | "fix the bug" | AI doesn't know what bug, what codebase, what language |
| **Unstructured** | Long paragraph with multiple asks buried inside | AI misses requirements, partial response |
| **No output format** | "explain kubernetes" | AI gives generic essay instead of actionable info |
| **Platform-unaware** | Same prompt for Claude and Midjourney | Doesn't leverage platform-specific features |
| **No constraints** | "write a function" | AI makes wrong assumptions about language, style, edge cases |

#### The Cost of Bad Prompts

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Bad Prompt Cycle                                     │
│                                                                              │
│    ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐         │
│    │  Vague   │────►│  Poor    │────►│  Follow  │────►│  Still   │──┐      │
│    │  Prompt  │     │ Response │     │   Up     │     │  Wrong   │  │      │
│    └──────────┘     └──────────┘     └──────────┘     └──────────┘  │      │
│         ▲                                                           │      │
│         └───────────────────────────────────────────────────────────┘      │
│                                                                              │
│    Result: 5-10 back-and-forth messages instead of 1                        │
│    Wasted: Time, tokens, money, patience                                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Quantified Impact:**

| Metric | Bad Prompt | Good Prompt | Improvement |
|--------|------------|-------------|-------------|
| Messages to complete task | 5-10 | 1-2 | 5x fewer |
| Tokens consumed | 3000+ | 800 | 4x cheaper |
| Time spent | 10+ minutes | 2 minutes | 5x faster |
| User satisfaction | Frustrated | Delighted | Priceless |

---

### Why Existing Solutions Fall Short

| Solution | What It Does | Why It's Not Enough |
|----------|--------------|---------------------|
| **Prompt libraries** (PromptBase, etc.) | Pre-written prompts for common tasks | Static, don't adapt to user's specific need |
| **"Improve my prompt" in ChatGPT** | Ask AI to improve the prompt | Extra step, breaks flow, not available everywhere |
| **Browser extensions** | Inject UI into AI chat pages | Only work in browser, not terminal/IDE, break when DOM changes |
| **Prompt engineering courses** | Teach techniques | Requires expertise, time to apply manually each time |
| **Copy-paste templates** | Markdown templates | Manual, tedious, easy to forget |

#### The Gap We're Filling

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   Current State                        Desired State                        │
│   ─────────────                        ─────────────                        │
│                                                                              │
│   User thinks of task                  User thinks of task                  │
│         │                                    │                              │
│         ▼                                    ▼                              │
│   User writes rough prompt             User writes rough prompt             │
│         │                                    │                              │
│         ▼                                    ▼                              │
│   User manually tries to               User presses Cmd+Shift+E             │
│   improve it (or doesn't)                    │                              │
│         │                                    ▼                              │
│         ▼                              PromptEnhancer automatically         │
│   Submits suboptimal prompt            enhances with:                       │
│         │                              • Structure                          │
│         ▼                              • Context                            │
│   Gets mediocre response               • Platform optimization              │
│         │                              • SuperMemory context                │
│         ▼                                    │                              │
│   Multiple follow-ups                        ▼                              │
│         │                              User submits great prompt            │
│         ▼                                    │                              │
│   Eventually gets answer                     ▼                              │
│   (frustrated)                         Gets excellent response              │
│                                        (first try, delighted)               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Our Solution

**PromptEnhancer** solves this by:

1. **Being Universal** - Works everywhere via system tray + CLI (not just browser)
2. **Being Instant** - One hotkey, 2-3 seconds, enhanced prompt ready
3. **Being Smart** - LLM-powered understanding of intent, not just templates
4. **Being Platform-Aware** - Optimizes for Claude vs GPT vs Midjourney differently
5. **Being Context-Aware** - SuperMemory integration brings user's knowledge
6. **Being Accessible** - BYOK free tier means anyone can use it

---

### Goals & Success Metrics

#### Primary Goals (MVP)

| Goal | Description | How We'll Achieve It |
|------|-------------|---------------------|
| **G1: Universal Access** | Work in any context - terminal, browser, IDE | System tray app + CLI, global hotkeys |
| **G2: Quality Enhancement** | Measurably better prompts | LLM-powered engine with platform-specific optimization |
| **G3: Zero Friction** | Enhancement in <3 seconds, minimal steps | Hotkey trigger, clipboard integration, fast API |
| **G4: Sustainable Business** | Revenue to cover costs + growth | BYOK free tier + Hosted paid tier |
| **G5: Open & Trustworthy** | Users trust us with their prompts | Open source core, no prompt logging, BYOK option |

#### Success Metrics

**North Star Metric:**
> **Weekly Enhancement Rate per Active User**
> Target: >15 enhancements/week for active users

**Supporting Metrics:**

| Category | Metric | Target | Measurement Method |
|----------|--------|--------|-------------------|
| **Adoption** | Total Installs | 5,000 in 3 months | Download counts |
| **Adoption** | Weekly Active Users | 1,000 in 3 months | Telemetry (opt-in) |
| **Engagement** | Enhancements per User per Week | >15 | Usage tracking |
| **Quality** | Enhancement Acceptance Rate | >80% | User copies/uses result vs discards |
| **Quality** | Enhancement Latency (P95) | <3 seconds | Performance monitoring |
| **Revenue** | Hosted Pro Subscribers | 200 in 3 months | Stripe dashboard |
| **Revenue** | Monthly Recurring Revenue | $1,600 in 3 months | Stripe dashboard |
| **Satisfaction** | App Store Rating | >4.5 stars | Store reviews |
| **Reliability** | Error Rate | <1% | Error tracking |
| **Reliability** | Uptime (Hosted) | 99.5% | Monitoring |

#### Goals by Phase

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Roadmap Phases                                  │
│                                                                              │
│  Phase 1: MVP (Month 1-2)                                                   │
│  ────────────────────────                                                   │
│  • System Tray App (macOS first, then Windows/Linux)                        │
│  • CLI                                                                       │
│  • Text LLM Enhancement (4 platforms)                                       │
│  • Image Prompt Enhancement (3 platforms)                                   │
│  • Viral Templates (10 curated)                                             │
│  • BYOK Mode                                                                │
│  • SuperMemory Integration                                                  │
│  • Basic Hosted Mode (Stripe billing)                                       │
│                                                                              │
│  Success: 1,000 installs, 100 WAU, 50 paid subscribers                     │
│                                                                              │
│  Phase 2: Growth (Month 3-4)                                                │
│  ───────────────────────────                                                │
│  • Community templates (submission + voting)                                │
│  • Prompt history & favorites                                               │
│  • Browser extension (Chrome)                                               │
│  • More viral templates (trending detection)                                │
│  • Referral program                                                         │
│                                                                              │
│  Success: 5,000 installs, 500 WAU, 200 paid subscribers                    │
│                                                                              │
│  Phase 3: Scale (Month 5-6)                                                 │
│  ──────────────────────────                                                 │
│  • Team features                                                            │
│  • Custom enhancement rules                                                 │
│  • API for integrations                                                     │
│  • Analytics dashboard                                                      │
│                                                                              │
│  Success: 15,000 installs, 2,000 WAU, 500 paid subscribers                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Non-Goals (Explicit Exclusions)

| Non-Goal | Rationale |
|----------|-----------|
| **Building our own LLM** | Use existing APIs (Claude, GPT), focus on UX |
| **Prompt marketplace** | Adds complexity, moderation burden; maybe v3 |
| **Real-time collaboration** | Solo/individual use case first |
| **Mobile app** | Desktop-first where AI work happens; mobile later |
| **Replacing AI platforms** | We enhance prompts, not replace ChatGPT/Claude |
| **Enterprise SSO/compliance** | Solo dev, consumer-first; enterprise later |

---

### Constraints

| Constraint | Impact | Mitigation |
|------------|--------|------------|
| **Solo developer** | Limited bandwidth, can't do everything | AI coding agents, managed services, ruthless prioritization |
| **Bootstrap budget** | No VC money, must be sustainable fast | BYOK reduces our costs, Hosted tier for revenue |
| **Cross-platform** | macOS, Windows, Linux all needed | Tauri handles this well, but testing burden |
| **LLM costs** | Enhancement uses tokens = money | BYOK shifts cost to user, Hosted tier priced to cover |
| **Privacy sensitivity** | Users wary of sending prompts to servers | Open source, BYOK option, no logging, clear privacy policy |

---

### Assumptions

| Assumption | Risk if Wrong | Validation Plan |
|------------|---------------|-----------------|
| Users will pay $8/mo for convenience | Low revenue | Validate with beta users, adjust pricing |
| 80% will prefer BYOK | Hosted tier underused | Track split, adjust marketing |
| SuperMemory integration adds value | Wasted dev time | Beta feedback, usage metrics |
| Viral templates drive adoption | Feature unused | Track template usage, social shares |
| 3 second latency is acceptable | Users abandon | A/B test, optimize if needed |
| Users trust open source more | Doesn't affect adoption | Survey users on trust factors |

---

### Dependencies

| Dependency | Type | Risk | Mitigation |
|------------|------|------|------------|
| **Anthropic API** | External | API changes, rate limits, pricing | Abstract provider, support multiple LLMs |
| **SuperMemory API** | External | API changes, service availability | Graceful degradation, optional feature |
| **Stripe** | External | Low risk, stable | Standard integration |
| **Supabase** | External | Service issues | Can self-host if needed |
| **Tauri** | Framework | Breaking changes | Pin versions, follow releases |
| **App Store approvals** | Process | Rejection, delays | Follow guidelines, plan buffer time |

---

## Section 3: User Personas

---

### Persona Overview

| Persona | Role | Primary Interface | Payment Model | Key Need |
|---------|------|-------------------|---------------|----------|
| **Terminal Dev** | Senior Engineer | CLI | BYOK | Speed, no GUI |
| **Visual Dev** | Full-Stack Dev | System Tray + CLI | Either | Visual feedback, learning |
| **Creative Pro** | Content Creator | System Tray only | Hosted | Viral templates, image prompts |
| **Privacy Advocate** | Security-conscious Dev | CLI | BYOK only | Open source, no logging |

---

### Persona 1: Terminal Dev ("Alex")

**Profile:** Senior engineer, lives in terminal, uses Claude Code/Cursor daily

**Pain:** Writes terse prompts → mediocre AI responses, hates context-switching

**Needs:**
- CLI that works instantly (`pe "prompt"`)
- Pipe support (`echo "prompt" | pe`)
- < 2 second latency

**Quote:** *"I want to type `pe` and have it just work. No GUI."*

---

### Persona 2: Visual Dev ("Jordan")

**Profile:** Full-stack dev, uses VS Code + browser-based AI tools

**Pain:** Prompts are unstructured, doesn't know platform-specific tricks

**Needs:**
- System Tray app with visual UI
- Before/after diff to learn
- Image prompt support for side projects

**Quote:** *"Show me what makes a good prompt."*

---

### Persona 3: Creative Pro ("Sam")

**Profile:** Content creator/marketer, heavy Midjourney + ChatGPT user

**Pain:** Can't craft image prompts, misses viral trends

**Needs:**
- Viral templates (one-click trending formats)
- Image prompt enhancement
- Simple UI, no technical setup

**Quote:** *"I want to make those Ghibli portraits everyone's posting."*

---

### Persona 4: Privacy Advocate ("Riley")

**Profile:** Security-conscious dev, self-hosts everything, reads source code

**Pain:** Won't use tools that log prompts or are closed-source

**Needs:**
- Open source (auditable)
- BYOK only (no data to our servers)
- No telemetry or opt-in only

**Quote:** *"If I can't see the code, I'm not using it."*

---

### Design Implications

| Decision | Rationale |
|----------|-----------|
| Build both CLI and System Tray | Serves Terminal Dev + Visual Dev + Creative Pro |
| BYOK as first-class citizen | Serves Privacy Advocate, reduces our costs |
| Viral templates in MVP | Key draw for Creative Pro |
| Open source core | Builds trust, serves Privacy Advocate |
| Visual diff in UI | Helps Visual Dev learn |

---

## Section 4: Product Overview & Features

---

### Product Summary

PromptEnhancer is a **universal prompt enhancement tool** delivered as:
1. **System Tray Application** (Tauri) - GUI for all users
2. **Command Line Interface** (Rust) - For terminal power users

Both interfaces share a common core engine and configuration.

---

### Feature Categories

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PromptEnhancer Features                              │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │  ENHANCEMENT    │  │  INTEGRATIONS   │  │  PLATFORM & DISTRIBUTION    │  │
│  │  ─────────────  │  │  ────────────   │  │  ──────────────────────────  │  │
│  │                 │  │                 │  │                             │  │
│  │  • Text LLM     │  │  • SuperMemory  │  │  • System Tray App          │  │
│  │  • Image Gen    │  │  • LLM APIs     │  │  • CLI                      │  │
│  │  • Viral Temps  │  │  • Stripe       │  │  • Global Hotkeys           │  │
│  │                 │  │                 │  │  • Cross-platform           │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘  │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐                                   │
│  │  MODES          │  │  USER MGMT      │                                   │
│  │  ─────          │  │  ─────────      │                                   │
│  │                 │  │                 │                                   │
│  │  • BYOK (Free)  │  │  • Settings UI  │                                   │
│  │  • Hosted (Paid)│  │  • Usage Stats  │                                   │
│  │  • Trial        │  │  • Billing      │                                   │
│  │                 │  │                 │                                   │
│  └─────────────────┘  └─────────────────┘                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Feature 1: Text LLM Enhancement

**Purpose:** Transform rough prompts into well-structured, platform-optimized prompts for conversational AI.

#### Supported Platforms

| Platform | Optimization Techniques |
|----------|------------------------|
| **Claude** | XML tags, structured thinking, artifact hints |
| **OpenAI (GPT)** | System prompt patterns, markdown structure |
| **Gemini** | Google-style formatting, safety considerations |
| **Generic** | Universal best practices |

#### Enhancement Process

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Text Enhancement Pipeline                             │
│                                                                              │
│  Input                    Processing                      Output            │
│  ─────                    ──────────                      ──────            │
│                                                                              │
│  "make a func that       ┌─────────────────────┐         "Create a          │
│   sorts users by age"    │ 1. Intent Analysis  │         TypeScript         │
│                          │ 2. Context Fetch    │         function with:     │
│         │                │    (SuperMemory)    │                            │
│         │                │ 3. Structure Apply  │         1. Input: User[]   │
│         ▼                │ 4. Platform Optimize│         2. Sort by age     │
│                          └─────────────────────┘         3. Return sorted   │
│  Platform: Claude                 │                      4. Add types       │
│  Context: TypeScript proj         │                      5. Handle edge     │
│                                   ▼                         cases"          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Example Enhancement

**Before (User Input):**
```
make a function that sorts users by age and removes inactive ones
```

**After (Enhanced for Claude):**
```
Create a TypeScript function with the following requirements:

<context>
- Working in a Node.js backend environment
- Users are stored as objects with properties
</context>

<requirements>
1. Input: Array of User objects
   - Each user has: id (string), name (string), age (number), isActive (boolean)

2. Processing:
   - Filter out users where isActive === false
   - Sort remaining users by age in ascending order

3. Output: Return the filtered and sorted array

4. Constraints:
   - Use proper TypeScript types
   - Handle empty array input
   - Do not mutate the original array
</requirements>

Please provide the implementation with:
- Type definitions
- JSDoc comments
- A brief explanation of the approach
```

#### UI for Text Enhancement

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ PromptEnhancer                                                    [−][□][×] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  [💬 Text LLM ●]    [🎨 Image Gen]    [🔥 Viral Templates]                  │
│                                                                              │
│  Platform:  [Claude ▼]   [GPT]   [Gemini]   [Generic]                       │
│                                                                              │
│  ┌─ Your Prompt ────────────────────────────────────────────────────────┐   │
│  │                                                                       │   │
│  │  make a function that sorts users by age and removes inactive ones  │   │
│  │                                                                       │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ☑️ Include SuperMemory context   [🧠 Found: "TypeScript project, Node.js"] │
│                                                                              │
│                                              [✨ Enhance]                    │
│                                                                              │
│  ┌─ Enhanced Prompt ────────────────────────────────────────────────────┐   │
│  │                                                                       │   │
│  │  Create a TypeScript function with the following requirements:       │   │
│  │                                                                       │   │
│  │  <context>                                                            │   │
│  │  - Working in a Node.js backend environment                          │   │
│  │  ...                                                                  │   │
│  │                                                                       │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│                              [📋 Copy]   [✅ Copy & Close]                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### CLI Usage

```bash
# Basic enhancement (uses default platform from config)
$ pe "make a function that sorts users"

# Specify platform
$ pe --platform claude "make a function that sorts users"
$ pe -p openai "explain kubernetes"

# With SuperMemory context
$ pe --memory "fix the auth bug in login flow"

# Pipe mode
$ echo "explain docker" | pe -p gemini

# Interactive mode (multi-line input)
$ pe -i
```

---

### Feature 2: Image Prompt Enhancement

**Purpose:** Transform simple image ideas into detailed, platform-optimized prompts for AI image generators.

#### Supported Platforms

| Platform | Key Optimizations |
|----------|------------------|
| **Midjourney** | Style keywords, `--ar`, `--v`, `--style` parameters |
| **DALL-E** | Descriptive natural language, safety-aware |
| **Stable Diffusion** | Weighted tokens, negative prompts, technical params |
| **Generic** | Universal descriptive approach |

#### Image Prompt Anatomy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Anatomy of a Good Image Prompt                           │
│                                                                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │   Subject   │ │    Style    │ │   Details   │ │   Technical Params      ││
│  │             │ │             │ │             │ │                         ││
│  │ What is in  │ │ Art style,  │ │ Lighting,   │ │ Aspect ratio,           ││
│  │ the image   │ │ artist ref  │ │ mood, color │ │ version, quality        ││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────────┘│
│                                                                              │
│  Example:                                                                    │
│  "A cat"  →  "A fluffy orange tabby cat sitting on a windowsill,            │
│               Studio Ghibli style, soft morning light, warm colors,          │
│               detailed fur, cozy atmosphere --ar 16:9 --v 6 --style raw"    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Example Enhancement

**Before (User Input):**
```
a cat in space
```

**After (Enhanced for Midjourney):**
```
A fluffy orange tabby cat floating weightlessly inside a space station, 
Earth visible through a large circular window in the background, 
the cat's fur drifting slightly in zero gravity, 
soft blue ambient lighting from control panels, 
photorealistic, highly detailed fur with visible whiskers, 
NASA-style interior, 8K quality, cinematic composition 
--ar 16:9 --v 6 --style raw
```

**After (Enhanced for DALL-E):**
```
A photorealistic image of a fluffy orange tabby cat floating in zero 
gravity inside the International Space Station. The cat is in the 
center of the frame with its paws stretched out and fur slightly 
floating. Through a large circular window behind the cat, Earth is 
visible with blue oceans and white clouds. The space station interior 
has white walls with technical equipment and soft ambient lighting. 
The image should be highly detailed with individual whiskers visible 
and a sense of wonder and playfulness.
```

#### UI for Image Enhancement

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ PromptEnhancer                                                    [−][□][×] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  [💬 Text LLM]    [🎨 Image Gen ●]    [🔥 Viral Templates]                  │
│                                                                              │
│  Platform:  [Midjourney ▼]   [DALL-E]   [Stable Diffusion]   [Generic]      │
│                                                                              │
│  ┌─ Your Idea ──────────────────────────────────────────────────────────┐   │
│  │                                                                       │   │
│  │  a cat in space                                                      │   │
│  │                                                                       │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  Style Hints (optional):                                                    │
│  [ ] Photorealistic   [x] Cinematic   [ ] Anime   [ ] Oil Painting         │
│  [ ] 3D Render        [ ] Watercolor  [ ] Sketch  [ ] Custom: [        ]   │
│                                                                              │
│                                              [✨ Enhance]                    │
│                                                                              │
│  ┌─ Enhanced Prompt ────────────────────────────────────────────────────┐   │
│  │                                                                       │   │
│  │  A fluffy orange tabby cat floating weightlessly inside a space      │   │
│  │  station, Earth visible through a large circular window...           │   │
│  │  --ar 16:9 --v 6 --style raw                                         │   │
│  │                                                                       │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│                              [📋 Copy]   [✅ Copy & Close]                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### CLI Usage

```bash
# Image prompt enhancement
$ pe --image "a cat in space"
$ pe -i --platform midjourney "sunset over mountains"
$ pe -i --platform dalle "robot playing chess"

# With style hint
$ pe --image --style "ghibli" "my childhood home"
```

---

You're right, let me complete Section 4 more concisely.

---

## Section 4: Product Overview & Features (Continued)

---

Feature 3: Viral Templates
Purpose: One-click access to trending prompt formats that are popular on social media.
How It Works
plaintext

Copy
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Viral Templates System                               │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      Template Repository                             │    │
│  │                      (GitHub-hosted JSON)                            │    │
│  │                                                                      │    │
│  │  • Curated by us (quality controlled)                               │    │
│  │  • Community submissions via PR                                      │    │
│  │  • Updated weekly                                                    │    │
│  │  • Synced to app on launch                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      User Selects Template                           │    │
│  │                                                                      │    │
│  │  "Ghibli Style Portrait"                                            │    │
│  │  ─────────────────────────                                          │    │
│  │                                                                      │    │
│  │  Subject: [my cat sitting on the couch        ]                     │    │
│  │                                                                      │    │
│  │  [Generate Prompt]                                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      Output                                          │    │
│  │                                                                      │    │
│  │  "my cat sitting on the couch, Studio Ghibli anime style,           │    │
│  │   hand-drawn animation aesthetic, soft pastel colors,                │    │
│  │   detailed background, Hayao Miyazaki inspired, whimsical           │    │
│  │   atmosphere, cel shading, nostalgic mood --ar 3:4 --v 6 --niji 6"  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │


### Feature 3: Viral Templates (continued)

#### MVP Templates (Curated)

| Template Name | Category | Platform | Description |
|---------------|----------|----------|-------------|
| Ghibli Style Portrait | Image | Midjourney | Studio Ghibli anime aesthetic |
| Action Figure Box | Image | Midjourney/DALL-E | Product in toy packaging |
| Pixar Character | Image | Midjourney | 3D Pixar-style character |
| 90s Yearbook Photo | Image | Midjourney | Retro school photo style |
| Oil Painting Portrait | Image | All | Classical oil painting |
| Cyberpunk Scene | Image | All | Neon-lit futuristic |
| Code Review Expert | Text | Claude/GPT | Thorough code review prompt |
| Technical Explainer | Text | All | ELI5 technical concepts |
| Blog Post Writer | Text | All | SEO-optimized blog content |
| Debug Assistant | Text | Claude/GPT | Systematic debugging help |

#### Template Data Structure

```json
{
  "id": "ghibli-style",
  "name": "Ghibli Style Portrait",
  "category": "image",
  "platform": "midjourney",
  "description": "Transform any subject into Studio Ghibli anime style",
  "fields": [
    {
      "name": "subject",
      "label": "Your Subject",
      "placeholder": "my cat sitting on the couch",
      "required": true
    }
  ],
  "template": "{{subject}}, Studio Ghibli anime style, hand-drawn animation aesthetic, soft pastel colors, detailed background, Hayao Miyazaki inspired, whimsical atmosphere, cel shading, nostalgic mood --ar 3:4 --v 6 --niji 6",
  "trending": true,
  "uses_count": 15420
}
```

#### UI for Viral Templates

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ PromptEnhancer                                                    [−][□][×] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  [💬 Text LLM]    [🎨 Image Gen]    [🔥 Viral Templates ●]                  │
│                                                                              │
│  ┌─ 🔥 Trending ────────────────────────────────────────────────────────┐   │
│  │                                                                       │   │
│  │  [Ghibli Style]  [Action Figure]  [Pixar Character]  [90s Yearbook]  │   │
│  │                                                                       │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌─ All Templates ──────────────────────────────────────────────────────┐   │
│  │                                                                       │   │
│  │  Filter: [All ▼]  [Image]  [Text]     Search: [____________] 🔍     │   │
│  │                                                                       │   │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐   │   │
│  │  │ 🎨 Ghibli Style  │  │ 🎨 Action Figure │  │ 🎨 Pixar Style   │   │   │
│  │  │    15.4k uses    │  │    12.1k uses    │  │    9.8k uses     │   │   │
│  │  └──────────────────┘  └──────────────────┘  └──────────────────┘   │   │
│  │                                                                       │   │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐   │   │
│  │  │ 💬 Code Review   │  │ 💬 Debug Helper  │  │ 🎨 Cyberpunk     │   │   │
│  │  │    8.2k uses     │  │    7.5k uses     │  │    6.9k uses     │   │   │
│  │  └──────────────────┘  └──────────────────┘  └──────────────────┘   │   │
│  │                                                                       │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Selected: Ghibli Style Portrait                                            │
│                                                                              │
│  Subject: [my cat sitting on the couch                    ]                 │
│                                                                              │
│                              [✨ Generate]   [📋 Copy]                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### CLI Usage

```bash
# List available templates
$ pe templates list
$ pe templates list --trending

# Use a template
$ pe --template ghibli "my cat on the couch"
$ pe -t action-figure "portrait of me"
$ pe -t code-review  # Opens interactive mode for code input
```

---

### Feature 4: SuperMemory Integration

**Purpose:** Enrich prompts with user's personal context and knowledge stored in SuperMemory.

#### How It Works

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      SuperMemory Integration Flow                            │
│                                                                              │
│  1. User prompt: "fix the auth bug"                                         │
│                       │                                                      │
│                       ▼                                                      │
│  2. Query SuperMemory: "auth bug context"                                   │
│                       │                                                      │
│                       ▼                                                      │
│  3. Retrieved memories:                                                      │
│     • "Project uses NextAuth with Prisma adapter"                           │
│     • "Had JWT expiry issues last week"                                     │
│     • "Auth middleware in /src/middleware.ts"                               │
│                       │                                                      │
│                       ▼                                                      │
│  4. Enhanced prompt includes context:                                        │
│     "Fix auth bug in my Next.js app using NextAuth + Prisma.                │
│      Previously had JWT expiry issues. Check middleware.ts..."              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Configuration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Settings → Integrations → SuperMemory                                      │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Status: [🟢 Connected]                                                      │
│                                                                              │
│  API Key: [sm_************************] [👁️] [Test Connection]              │
│                                                                              │
│  Options:                                                                    │
│  ☑️  Auto-fetch context for enhancements                                    │
│  ☐  Auto-save enhanced prompts to memory                                    │
│                                                                              │
│  Context limit: [5] memories per enhancement                                │
│                                                                              │
│  [Disconnect]                                          [Save]               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### CLI Usage

```bash
# Enable SuperMemory for this prompt
$ pe --memory "fix the auth bug"
$ pe -m "why is the API slow"

# Configure SuperMemory
$ pe config set supermemory.api_key "sm_xxxxx"
$ pe config set supermemory.auto_context true
```

---

### Feature 5: BYOK & Hosted Modes

**Purpose:** Support both privacy-conscious users (BYOK) and convenience-seekers (Hosted).

#### Mode Comparison

| Aspect | BYOK Mode | Hosted Mode |
|--------|-----------|-------------|
| **Price** | Free forever | $8/month |
| **API Key** | User provides | We provide |
| **Data Flow** | Direct to LLM provider | Via our backend |
| **Privacy** | Maximum (we see nothing) | Prompts processed by our server |
| **Setup** | Requires API key | Just login |
| **Usage Limits** | Provider's limits | Fair use (500/day) |

#### BYOK Setup Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Settings → API Keys                                                        │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Mode: (●) BYOK (Bring Your Own Key)    ( ) Hosted Pro                      │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Primary Provider (for enhancement):                                        │
│  [Anthropic (Claude) ▼]                                                     │
│                                                                              │
│  API Key: [sk-ant-api03-*******************] [👁️]                           │
│                                                                              │
│  Model: [claude-3-5-sonnet-20241022 ▼]                                      │
│                                                                              │
│  [Test Connection]  ✅ Connected successfully                               │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Additional Keys (optional):                                                │
│                                                                              │
│  OpenAI:  [sk-*************************] [👁️]  ✅                           │
│  Google:  [Not configured]                      [Add Key]                   │
│                                                                              │
│                                                         [Save]              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Hosted Mode Setup

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Settings → Subscription                                                    │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Mode: ( ) BYOK (Bring Your Own Key)    (●) Hosted Pro                      │
│                                                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Plan: Pro ($8/month)                                                       │
│  Status: Active                                                              │
│  Next billing: February 13, 2025                                            │
│                                                                              │
│  Usage this month:                                                          │
│  ████████████░░░░░░░░░░░░░░░░░░  247 / 500 daily limit                     │
│                                                                              │
│  [Manage Subscription]  [View Invoices]                                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Feature 6: Global Hotkeys

**Purpose:** Instant access to enhancement from anywhere on the system.

#### Default Hotkeys

| Hotkey | Action |
|--------|--------|
| `Cmd/Ctrl + Shift + E` | Quick enhance (reads clipboard, enhances, writes back) |
| `Cmd/Ctrl + Shift + P` | Open main window |
| `Cmd/Ctrl + Shift + T` | Open viral templates |

#### Quick Enhance Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Quick Enhance (Cmd+Shift+E)                          │
│                                                                              │
│  1. User copies prompt text                                                  │
│  2. User presses Cmd+Shift+E                                                │
│  3. Floating notification appears:                                          │
│                                                                              │
│     ┌────────────────────────────────────────┐                              │
│     │  🔮 Enhancing...                       │                              │
│     │  ████████░░░░░░░░░░                    │                              │
│     └────────────────────────────────────────┘                              │
│                                                                              │
│  4. After 1-2 seconds:                                                       │
│                                                                              │
│     ┌────────────────────────────────────────┐                              │
│     │  ✅ Enhanced! Copied to clipboard.     │                              │
│     │  Press Cmd+V to paste                  │                              │
│     └────────────────────────────────────────┘                              │
│                                                                              │
│  5. User pastes enhanced prompt                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Feature 7: Settings & Configuration

#### Settings Categories

| Category | Options |
|----------|---------|
| **General** | Default platform, theme, launch at login |
| **API Keys** | BYOK configuration, provider selection |
| **Subscription** | Hosted mode billing, usage stats |
| **Integrations** | SuperMemory connection |
| **Hotkeys** | Customize keyboard shortcuts |
| **Privacy** | Telemetry opt-in/out |

---

### Feature Summary Table

| Feature | CLI | Tray App | BYOK | Hosted | MVP |
|---------|-----|----------|------|--------|-----|
| Text LLM Enhancement | ✅ | ✅ | ✅ | ✅ | ✅ |
| Image Prompt Enhancement | ✅ | ✅ | ✅ | ✅ | ✅ |
| Viral Templates | ✅ | ✅ | ✅ | ✅ | ✅ |
| SuperMemory Integration | ✅ | ✅ | ✅ | ✅ | ✅ |
| Global Hotkeys | - | ✅ | ✅ | ✅ | ✅ |
| Settings UI | Config file | ✅ | ✅ | ✅ | ✅ |
| Usage Stats | - | ✅ | - | ✅ | ✅ |
| Billing Management | - | ✅ | - | ✅ | ✅ |

---

## Section 5: Architecture

---

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USER INTERFACES                                 │
│                                                                              │
│         ┌─────────────────────────┐       ┌─────────────────────────┐       │
│         │    System Tray App      │       │          CLI            │       │
│         │        (Tauri)          │       │        (Rust)           │       │
│         │                         │       │                         │       │
│         │  • Main Window          │       │  • pe "prompt"          │       │
│         │  • Quick Enhance Popup  │       │  • pe --image "idea"    │       │
│         │  • Settings UI          │       │  • pe --template xyz    │       │
│         │  • Tray Menu            │       │  • Pipe support         │       │
│         │  • Global Hotkeys       │       │                         │       │
│         └───────────┬─────────────┘       └───────────┬─────────────┘       │
│                     │                                 │                      │
│                     └─────────────┬───────────────────┘                      │
│                                   │                                          │
│                                   ▼                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                              CORE ENGINE (Rust Library)                      │
│                                                                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │  Enhance    │ │  Platform   │ │  Template   │ │    SuperMemory      │    │
│  │  Service    │ │  Detector   │ │  Manager    │ │      Client         │    │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────┘    │
│                                                                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │   Image     │ │   Config    │ │ LLM Client  │ │      Usage          │    │
│  │  Enhancer   │ │  Manager    │ │  (API)      │ │     Tracker         │    │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────┘    │
│                                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                              EXTERNAL SERVICES                               │
│                                                                              │
│    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐    │
│    │   LLM APIs      │  │   SuperMemory   │  │   Backend (Hosted)      │    │
│    │                 │  │                 │  │                         │    │
│    │  • Anthropic    │  │  • Context      │  │  • Supabase Auth        │    │
│    │  • OpenAI       │  │    retrieval    │  │  • Usage tracking       │    │
│    │  • Google       │  │  • Memory       │  │  • Stripe billing       │    │
│    │                 │  │    storage      │  │  • LLM proxy            │    │
│    └─────────────────┘  └─────────────────┘  └─────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Component Architecture

#### Core Engine (Rust Library)

Shared library used by both System Tray App and CLI.

```
promptenhancer-core/
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── enhance/
│   │   ├── mod.rs
│   │   ├── text.rs            # Text LLM enhancement logic
│   │   ├── image.rs           # Image prompt enhancement logic
│   │   └── prompts/           # System prompts for enhancement
│   │       ├── claude.txt
│   │       ├── openai.txt
│   │       ├── gemini.txt
│   │       └── image.txt
│   ├── platform/
│   │   ├── mod.rs
│   │   └── detector.rs        # Platform detection & optimization
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── manager.rs         # Template loading & management
│   │   └── builtin.rs         # Built-in viral templates
│   ├── integrations/
│   │   ├── mod.rs
│   │   ├── supermemory.rs     # SuperMemory API client
│   │   └── llm/
│   │       ├── mod.rs
│   │       ├── anthropic.rs   # Claude API client
│   │       ├── openai.rs      # OpenAI API client
│   │       └── google.rs      # Gemini API client
│   ├── config/
│   │   ├── mod.rs
│   │   └── manager.rs         # Config file management
│   └── usage/
│       ├── mod.rs
│       └── tracker.rs         # Usage tracking (hosted mode)
└── Cargo.toml
```

#### System Tray App (Tauri)

```
promptenhancer-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs            # Tauri entry point
│   │   ├── commands.rs        # Tauri commands (IPC)
│   │   ├── tray.rs            # System tray setup
│   │   ├── hotkeys.rs         # Global hotkey registration
│   │   └── clipboard.rs       # Clipboard operations
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # Frontend (Svelte/React)
│   ├── App.svelte
│   ├── lib/
│   │   ├── components/
│   │   │   ├── EnhancePanel.svelte
│   │   │   ├── ImagePanel.svelte
│   │   │   ├── TemplatesPanel.svelte
│   │   │   ├── SettingsPanel.svelte
│   │   │   └── QuickEnhance.svelte
│   │   └── stores/
│   │       ├── config.ts
│   │       └── usage.ts
│   └── routes/
│       ├── +page.svelte       # Main window
│       └── settings/
│           └── +page.svelte   # Settings window
├── package.json
└── vite.config.ts
```

#### CLI

```
promptenhancer-cli/
├── src/
│   ├── main.rs                # CLI entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── enhance.rs         # pe "prompt"
│   │   ├── image.rs           # pe --image "idea"
│   │   ├── template.rs        # pe --template xyz
│   │   └── config.rs          # pe config ...
│   └── output.rs              # Terminal output formatting
└── Cargo.toml
```

---

### Data Flow Diagrams

#### BYOK Mode Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              BYOK Mode                                       │
│                                                                              │
│  User                App/CLI              Core Engine           LLM API     │
│   │                    │                      │                    │        │
│   │  "enhance this"    │                      │                    │        │
│   │───────────────────►│                      │                    │        │
│   │                    │                      │                    │        │
│   │                    │  enhance(prompt,     │                    │        │
│   │                    │    platform, opts)   │                    │        │
│   │                    │─────────────────────►│                    │        │
│   │                    │                      │                    │        │
│   │                    │                      │  Load user's API   │        │
│   │                    │                      │  key from config   │        │
│   │                    │                      │                    │        │
│   │                    │                      │  POST /messages    │        │
│   │                    │                      │  (with user's key) │        │
│   │                    │                      │───────────────────►│        │
│   │                    │                      │                    │        │
│   │                    │                      │◄───────────────────│        │
│   │                    │                      │  Enhanced prompt   │        │
│   │                    │                      │                    │        │
│   │                    │◄─────────────────────│                    │        │
│   │                    │  EnhanceResult       │                    │        │
│   │                    │                      │                    │        │
│   │◄───────────────────│                      │                    │        │
│   │  Display result    │                      │                    │        │
│   │                    │                      │                    │        │
│                                                                              │
│  Note: No data goes to our servers. Direct user → LLM provider.            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Hosted Mode Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Hosted Mode                                     │
│                                                                              │
│  User       App/CLI        Core Engine        Our Backend        LLM API   │
│   │            │                │                  │                │       │
│   │  "enhance" │                │                  │                │       │
│   │───────────►│                │                  │                │       │
│   │            │                │                  │                │       │
│   │            │  enhance()     │                  │                │       │
│   │            │───────────────►│                  │                │       │
│   │            │                │                  │                │       │
│   │            │                │  POST /enhance   │                │       │
│   │            │                │  (with JWT)      │                │       │
│   │            │                │─────────────────►│                │       │
│   │            │                │                  │                │       │
│   │            │                │                  │  Validate JWT  │       │
│   │            │                │                  │  Check usage   │       │
│   │            │                │                  │  limits        │       │
│   │            │                │                  │                │       │
│   │            │                │                  │  POST /messages│       │
│   │            │                │                  │  (our API key) │       │
│   │            │                │                  │───────────────►│       │
│   │            │                │                  │                │       │
│   │            │                │                  │◄───────────────│       │
│   │            │                │                  │                │       │
│   │            │                │                  │  Increment     │       │
│   │            │                │                  │  usage count   │       │
│   │            │                │                  │                │       │
│   │            │                │◄─────────────────│                │       │
│   │            │                │  Enhanced prompt │                │       │
│   │            │                │                  │                │       │
│   │            │◄───────────────│                  │                │       │
│   │◄───────────│                │                  │                │       │
│   │  Result    │                │                  │                │       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### SuperMemory Integration Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SuperMemory Enhancement Flow                         │
│                                                                              │
│  Core Engine                    SuperMemory API                              │
│       │                              │                                       │
│       │  1. Extract keywords         │                                       │
│       │     from user prompt         │                                       │
│       │                              │                                       │
│       │  2. POST /search             │                                       │
│       │     {query: "auth bug"}      │                                       │
│       │─────────────────────────────►│                                       │
│       │                              │                                       │
│       │◄─────────────────────────────│                                       │
│       │  3. Relevant memories:       │                                       │
│       │     - "NextAuth + Prisma"    │                                       │
│       │     - "JWT issues"           │                                       │
│       │                              │                                       │
│       │  4. Inject context into      │                                       │
│       │     enhancement prompt       │                                       │
│       │                              │                                       │
│       │  5. Call LLM with enriched   │                                       │
│       │     context                  │                                       │
│       │                              │                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Core Engine** | Rust | Performance, safety, shared across interfaces |
| **System Tray App** | Tauri 2.0 | Small bundle (~20MB), native feel, Rust backend |
| **App Frontend** | SvelteKit | Lightweight, fast, good Tauri integration |
| **CLI** | Rust (clap) | Fast startup, single binary |
| **Backend** | Supabase | Auth + DB + Edge Functions, minimal ops |
| **Database** | PostgreSQL (Supabase) | Reliable, SQL, built into Supabase |
| **Auth** | Supabase Auth | OAuth + email, JWT tokens |
| **Payments** | Stripe | Industry standard, subscription support |
| **LLM** | Anthropic Claude | Best quality for prompt enhancement |

---


---

## Section 5: Architecture (Continued)

---

### Backend Architecture (Hosted Mode)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Supabase Backend                                   │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Edge Functions                               │    │
│  │                                                                      │    │
│  │  POST /enhance          GET /usage           POST /stripe-webhook   │    │
│  │  • Validate JWT         • Return stats       • Handle Stripe events │    │
│  │  • Check limits         • Daily/monthly      • Update subscription  │    │
│  │  • Call Claude API                                                   │    │
│  │  • Track usage                                                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         PostgreSQL Tables                            │    │
│  │                                                                      │    │
│  │  users              subscriptions           usage_daily             │    │
│  │  • id               • user_id               • user_id               │    │
│  │  • email            • stripe_customer_id    • date                  │    │
│  │  • tier             • status                • count                 │    │
│  │  • created_at       • current_period_end    • created_at            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Configuration & Storage

#### Config File Location

| OS | Path |
|----|------|
| macOS | `~/.config/promptenhancer/config.toml` |
| Linux | `~/.config/promptenhancer/config.toml` |
| Windows | `%APPDATA%\promptenhancer\config.toml` |

#### Config File Structure

```toml
# Mode: "byok" or "hosted"
mode = "byok"

# Default platform for enhancement
default_platform = "claude"

# BYOK Settings
[byok]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-3-5-sonnet-20241022"

[byok.openai]
api_key = "sk-..."

# Hosted Settings
[hosted]
access_token = "eyJ..."
refresh_token = "..."

# SuperMemory Integration
[supermemory]
enabled = true
api_key = "sm_..."
auto_context = true
context_limit = 5

# Hotkeys
[hotkeys]
quick_enhance = "CmdOrCtrl+Shift+E"
open_window = "CmdOrCtrl+Shift+P"

# Preferences
[preferences]
theme = "system"
launch_at_login = true
telemetry = false
```

---

### Shared Data Directory

```
~/.config/promptenhancer/
├── config.toml              # User configuration
├── templates/               # Custom user templates
│   └── my-template.json
├── cache/
│   ├── templates.json       # Cached viral templates
│   └── supermemory/         # Cached context
└── logs/
    └── app.log              # Debug logs (opt-in)
```

---

### Cross-Platform Considerations

| Concern | Solution |
|---------|----------|
| **Global Hotkeys** | Tauri's globalShortcut API (works on all platforms) |
| **System Tray** | Tauri's tray API with platform-specific icons |
| **Clipboard** | Tauri's clipboard API |
| **Auto-launch** | Tauri plugin + platform-specific registration |
| **Notifications** | Tauri's notification API |
| **File Paths** | Rust's `dirs` crate for platform-specific paths |

---

### Security Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Security Layers                                    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Client Security                              │    │
│  │                                                                      │    │
│  │  • API keys stored in OS keychain (not plain text config)          │    │
│  │  • Config file permissions: 600 (user read/write only)             │    │
│  │  • No prompt content logged                                         │    │
│  │  • Telemetry opt-in only                                            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Backend Security (Hosted)                    │    │
│  │                                                                      │    │
│  │  • JWT validation on all requests                                   │    │
│  │  • Rate limiting per user                                           │    │
│  │  • No prompt content stored (processed in memory only)              │    │
│  │  • Stripe webhook signature verification                            │    │
│  │  • Row Level Security (RLS) on all tables                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Deployment                                         │
│                                                                              │
│  App Distribution                    Backend                                │
│  ────────────────                    ───────                                │
│                                                                              │
│  macOS:                              Supabase (managed):                    │
│  • .dmg via GitHub Releases          • Edge Functions (Deno)               │
│  • Homebrew Cask                      • PostgreSQL                          │
│  • Notarized & signed                 • Auth                                │
│                                                                              │
│  Windows:                            Stripe (managed):                      │
│  • .exe installer via Releases        • Payment processing                  │
│  • Winget                             • Subscription management             │
│  • Code signed                                                              │
│                                                                              │
│  Linux:                              GitHub:                                │
│  • .AppImage                          • Template repository                 │
│  • .deb package                       • Release hosting                     │
│  • Flatpak (future)                                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Repository Structure

```
promptenhancer/
├── README.md
├── LICENSE                    # MIT
├── .github/
│   └── workflows/
│       ├── ci.yml             # Test & lint
│       ├── release.yml        # Build & publish
│       └── templates.yml      # Sync templates
│
├── crates/
│   ├── core/                  # promptenhancer-core (shared library)
│   │   ├── Cargo.toml
│   │   └── src/
│   └── cli/                   # promptenhancer-cli
│       ├── Cargo.toml
│       └── src/
│
├── app/                       # Tauri app
│   ├── src-tauri/
│   ├── src/                   # Svelte frontend
│   ├── package.json
│   └── tauri.conf.json
│
├── backend/                   # Supabase
│   ├── supabase/
│   │   ├── functions/         # Edge functions
│   │   └── migrations/        # SQL migrations
│   └── seed.sql
│
├── templates/                 # Viral templates (synced to CDN)
│   ├── image/
│   │   ├── ghibli.json
│   │   └── action-figure.json
│   └── text/
│       ├── code-review.json
│       └── debug-helper.json
│
└── docs/
    ├── PRD.md
    ├── CONTRIBUTING.md
    └── ARCHITECTURE.md
```

---

### Key Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Monorepo** | Yes | Easier to maintain, shared types |
| **Core as library** | Rust crate | Shared between CLI and Tauri |
| **Frontend framework** | SvelteKit | Lightweight, fast, Tauri-friendly |
| **Backend** | Supabase | All-in-one, minimal ops for solo dev |
| **LLM provider** | Claude primary | Best quality for rewriting tasks |
| **Config format** | TOML | Human-readable, Rust-native support |
| **API key storage** | OS keychain | More secure than plain text |

---

## Section 6: Data Design & Storage

---

### Local Storage

**Config Location:** `~/.config/promptenhancer/`

```
~/.config/promptenhancer/
├── config.toml                 # User settings
├── cache/
│   └── templates.json          # Cached viral templates
└── custom-templates/           # User-created templates
```

**Config File (`config.toml`):**
```toml
mode = "byok"                          # "byok" | "hosted"
default_platform = "claude"
default_image_platform = "midjourney"

[byok]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"

[supermemory]
enabled = true
auto_context = true
context_limit = 5

[hotkeys]
quick_enhance = "CmdOrCtrl+Shift+E"
open_window = "CmdOrCtrl+Shift+P"

[preferences]
theme = "system"
launch_at_login = true
telemetry = false
```

**Keychain Storage (Sensitive Data):**

| Key | Description |
|-----|-------------|
| `promptenhancer.anthropic_key` | Anthropic API key |
| `promptenhancer.openai_key` | OpenAI API key |
| `promptenhancer.supermemory_key` | SuperMemory API key |
| `promptenhancer.hosted_tokens` | Hosted mode JWT tokens |

---

### Backend Database (Supabase - Hosted Mode Only)

**Tables:**

```sql
-- Users
CREATE TABLE public.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    tier TEXT NOT NULL DEFAULT 'free' CHECK (tier IN ('free', 'pro', 'trial')),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Subscriptions
CREATE TABLE public.subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    stripe_customer_id TEXT NOT NULL,
    stripe_subscription_id TEXT UNIQUE NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'canceled', 'past_due')),
    current_period_end TIMESTAMPTZ NOT NULL
);

-- Daily Usage
CREATE TABLE public.usage_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(user_id, date)
);
```

---

### Usage Limits

| Tier | Daily Limit |
|------|-------------|
| **Trial** | 10 |
| **Free** | 5 |
| **Pro** | 500 |

---

### Template Structure

```json
{
  "id": "ghibli-style",
  "name": "Ghibli Style Portrait",
  "category": "image",
  "platform": "midjourney",
  "trending": true,
  "fields": [
    {
      "name": "subject",
      "label": "Your Subject",
      "placeholder": "a cat on a windowsill",
      "required": true
    }
  ],
  "template": "{{subject}}, Studio Ghibli anime style, soft pastel colors, Hayao Miyazaki inspired --ar 3:4 --v 6 --niji 6"
}
```

---

### Data Privacy

| Data Type | Stored? | Location |
|-----------|---------|----------|
| User prompts | ❌ No | Never stored |
| Enhanced prompts | ❌ No | Never stored |
| API keys | ✅ Local | OS Keychain |
| Usage counts | ✅ Backend | Supabase (hosted only) |

---

## Section 7: API Specification

---

### Overview

Two API contexts:
1. **Core Engine APIs** - Internal Rust library interfaces
2. **Backend APIs** - Supabase Edge Functions (hosted mode only)

---

### Core Engine APIs (Rust)

#### Enhancement Service

```rust
// Main enhancement function
pub async fn enhance(request: EnhanceRequest) -> Result<EnhanceResponse, EnhanceError>;

pub struct EnhanceRequest {
    pub prompt: String,
    pub platform: Platform,           // Claude, OpenAI, Gemini, Generic
    pub enhancement_type: EnhanceType, // Text, Image
    pub options: EnhanceOptions,
}

pub struct EnhanceOptions {
    pub include_supermemory: bool,
    pub style_hints: Option<Vec<String>>,  // For image prompts
    pub max_tokens: Option<u32>,
}

pub struct EnhanceResponse {
    pub enhanced_prompt: String,
    pub changes_summary: String,
    pub context_used: Option<Vec<String>>,  // SuperMemory context if used
    pub platform: Platform,
}

pub enum Platform {
    Claude,
    OpenAI,
    Gemini,
    Midjourney,
    DallE,
    StableDiffusion,
    Generic,
}

pub enum EnhanceType {
    Text,
    Image,
}

pub enum EnhanceError {
    ApiKeyMissing,
    ApiKeyInvalid,
    RateLimited { retry_after: u64 },
    ProviderError { message: String },
    NetworkError { message: String },
    InvalidInput { message: String },
}
```

#### Template Service

```rust
pub async fn list_templates(filter: TemplateFilter) -> Result<Vec<Template>, TemplateError>;
pub async fn get_template(id: &str) -> Result<Template, TemplateError>;
pub async fn apply_template(id: &str, fields: HashMap<String, String>) -> Result<String, TemplateError>;
pub async fn sync_templates() -> Result<SyncResult, TemplateError>;

pub struct TemplateFilter {
    pub category: Option<TemplateCategory>,  // Image, Text
    pub platform: Option<Platform>,
    pub trending_only: bool,
}

pub struct Template {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub platform: Platform,
    pub trending: bool,
    pub fields: Vec<TemplateField>,
    pub template: String,
}
```

#### Config Service

```rust
pub fn get_config() -> Result<Config, ConfigError>;
pub fn set_config(config: Config) -> Result<(), ConfigError>;
pub fn get_api_key(provider: Provider) -> Result<String, KeychainError>;
pub fn set_api_key(provider: Provider, key: &str) -> Result<(), KeychainError>;
```

#### SuperMemory Service

```rust
pub async fn search_context(query: &str, limit: u32) -> Result<Vec<Memory>, SuperMemoryError>;
pub async fn test_connection() -> Result<bool, SuperMemoryError>;

pub struct Memory {
    pub content: String,
    pub relevance_score: f32,
}
```

---

### Backend APIs (Supabase Edge Functions)

Base URL: `https://<project>.supabase.co/functions/v1`

#### POST /enhance

Enhance a prompt (hosted mode).

**Request:**
```json
{
  "prompt": "make a function that sorts users",
  "platform": "claude",
  "type": "text",
  "include_supermemory": true
}
```

**Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Response 200:**
```json
{
  "enhanced_prompt": "Create a TypeScript function...",
  "changes_summary": "Added structure, types, and constraints",
  "usage": {
    "used_today": 15,
    "daily_limit": 500,
    "tier": "pro"
  }
}
```

**Response 429 (Rate Limited):**
```json
{
  "error": "rate_limited",
  "message": "Daily limit reached",
  "resets_at": "2025-01-14T00:00:00Z"
}
```

---

#### GET /usage

Get current usage stats.

**Response 200:**
```json
{
  "tier": "pro",
  "used_today": 15,
  "daily_limit": 500,
  "used_this_month": 342
}
```

---

#### POST /auth/login

Login with email (sends magic link).

**Request:**
```json
{
  "email": "user@example.com"
}
```

**Response 200:**
```json
{
  "message": "Magic link sent to email"
}
```

---

#### POST /auth/verify

Verify magic link token.

**Request:**
```json
{
  "token": "abc123..."
}
```

**Response 200:**
```json
{
  "access_token": "eyJ...",
  "refresh_token": "...",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "tier": "free"
  }
}
```

---

#### POST /billing/create-checkout

Create Stripe checkout session.

**Request:**
```json
{
  "success_url": "promptenhancer://billing/success",
  "cancel_url": "promptenhancer://billing/cancel"
}
```

**Response 200:**
```json
{
  "checkout_url": "https://checkout.stripe.com/..."
}
```

---

#### POST /billing/portal

Get Stripe customer portal URL.

**Response 200:**
```json
{
  "portal_url": "https://billing.stripe.com/..."
}
```

---

#### POST /webhooks/stripe

Handle Stripe webhook events.

**Events Handled:**
- `checkout.session.completed` → Create subscription, upgrade tier
- `customer.subscription.updated` → Update status
- `customer.subscription.deleted` → Downgrade to free
- `invoice.payment_failed` → Mark past_due

---

### LLM Provider APIs (External)

#### Anthropic Claude

```
POST https://api.anthropic.com/v1/messages

Headers:
  x-api-key: <api_key>
  anthropic-version: 2023-06-01

Body:
{
  "model": "claude-3-5-sonnet-20241022",
  "max_tokens": 2048,
  "messages": [
    {
      "role": "user",
      "content": "<enhancement_prompt>"
    }
  ]
}
```

#### OpenAI

```
POST https://api.openai.com/v1/chat/completions

Headers:
  Authorization: Bearer <api_key>

Body:
{
  "model": "gpt-4-turbo",
  "messages": [
    {
      "role": "system",
      "content": "<system_prompt>"
    },
    {
      "role": "user", 
      "content": "<user_prompt>"
    }
  ]
}
```

#### SuperMemory

```
POST https://api.supermemory.ai/v1/search

Headers:
  Authorization: Bearer <api_key>

Body:
{
  "query": "auth bug nextjs",
  "limit": 5
}

Response:
{
  "memories": [
    {
      "content": "Project uses NextAuth with Prisma adapter",
      "score": 0.92
    }
  ]
}
```

---

### Error Codes

| Code | HTTP | Description |
|------|------|-------------|
| `invalid_input` | 400 | Bad request data |
| `unauthorized` | 401 | Missing/invalid auth |
| `forbidden` | 403 | Not allowed |
| `not_found` | 404 | Resource not found |
| `rate_limited` | 429 | Usage limit exceeded |
| `provider_error` | 502 | LLM API failed |
| `internal_error` | 500 | Server error |

---

### CLI Commands Reference

```bash
# Enhancement
pe "prompt"                      # Enhance text prompt
pe --platform claude "prompt"    # Specify platform
pe --image "idea"                # Enhance image prompt
pe --template ghibli "subject"   # Use template
pe --memory "prompt"             # Include SuperMemory context

# Configuration
pe config show                   # Show current config
pe config set mode byok          # Set mode
pe config set default_platform claude
pe login                         # Login (hosted mode)
pe logout                        # Logout

# Templates
pe templates list                # List all templates
pe templates list --trending     # List trending only
pe templates sync                # Force sync templates

# Utilities
pe --version                     # Show version
pe --help                        # Show help
```

---

## Section 8: Security & Privacy

---

### Security Principles

1. **Privacy First** - Never store user prompts
2. **Local by Default** - BYOK mode keeps data on user's machine
3. **Minimal Data** - Collect only what's necessary
4. **Transparent** - Open source, auditable code

---

### Data Flow Security

#### BYOK Mode (Maximum Privacy)

```
┌─────────────────────────────────────────────────────────────────┐
│                    BYOK Data Flow                                │
│                                                                  │
│  User's Machine                         External                 │
│  ──────────────                         ────────                 │
│                                                                  │
│  ┌──────────────┐                      ┌──────────────┐         │
│  │   Prompt     │─────────────────────►│  LLM API     │         │
│  │   (memory)   │◄─────────────────────│  (Claude)    │         │
│  └──────────────┘                      └──────────────┘         │
│                                                                  │
│  ┌──────────────┐                      ┌──────────────┐         │
│  │   API Key    │                      │  SuperMemory │         │
│  │  (keychain)  │                      │  (optional)  │         │
│  └──────────────┘                      └──────────────┘         │
│                                                                  │
│  ❌ Nothing goes to our servers                                 │
│  ✅ Direct connection to LLM provider                           │
│  ✅ Prompts never stored                                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Hosted Mode

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hosted Data Flow                              │
│                                                                  │
│  User          Our Backend              LLM API                  │
│  ────          ───────────              ───────                  │
│                                                                  │
│  Prompt ──────► Process in ───────────► Claude                  │
│                 memory only                                      │
│         ◄────── Enhanced    ◄───────────                        │
│                 prompt                                           │
│                                                                  │
│  ✅ Prompts processed in memory only                            │
│  ✅ No prompt logging                                           │
│  ✅ Only usage count stored                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Sensitive Data Handling

| Data | Storage Method | Protection |
|------|----------------|------------|
| **API Keys** | OS Keychain | Encrypted by OS, never in plain text |
| **Auth Tokens** | OS Keychain | Short-lived JWT, auto-refresh |
| **Config File** | Local file | File permissions 600 (user only) |
| **Prompts** | Memory only | Never written to disk or logs |

#### Keychain Implementation

```rust
// Using keyring crate for cross-platform keychain access
use keyring::Entry;

pub fn store_api_key(provider: &str, key: &str) -> Result<()> {
    let entry = Entry::new("promptenhancer", provider)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn get_api_key(provider: &str) -> Result<String> {
    let entry = Entry::new("promptenhancer", provider)?;
    entry.get_password()
}

pub fn delete_api_key(provider: &str) -> Result<()> {
    let entry = Entry::new("promptenhancer", provider)?;
    entry.delete_password()?;
    Ok(())
}
```

---

### Authentication (Hosted Mode)

#### Auth Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Magic Link Auth Flow                          │
│                                                                  │
│  1. User enters email                                           │
│  2. Backend sends magic link via email                          │
│  3. User clicks link → opens app with token                     │
│  4. App exchanges token for JWT                                 │
│  5. JWT stored in keychain                                      │
│  6. JWT auto-refreshes before expiry                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Token Security

| Token | Lifetime | Storage |
|-------|----------|---------|
| Access Token | 1 hour | OS Keychain |
| Refresh Token | 30 days | OS Keychain |
| Magic Link | 15 minutes | Not stored |

---

### Backend Security (Supabase)

#### Row Level Security

```sql
-- Users can only access their own data
CREATE POLICY "Users own data" ON public.users
    FOR ALL USING (auth.uid() = id);

CREATE POLICY "Users own usage" ON public.usage_daily
    FOR ALL USING (auth.uid() = user_id);

CREATE POLICY "Users own subscription" ON public.subscriptions
    FOR ALL USING (auth.uid() = user_id);
```

#### Edge Function Security

```typescript
// Every edge function validates JWT
import { createClient } from '@supabase/supabase-js';

export async function handler(req: Request) {
  const authHeader = req.headers.get('Authorization');
  if (!authHeader?.startsWith('Bearer ')) {
    return new Response(JSON.stringify({ error: 'unauthorized' }), { status: 401 });
  }

  const token = authHeader.split(' ')[1];
  const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY);
  
  const { data: { user }, error } = await supabase.auth.getUser(token);
  if (error || !user) {
    return new Response(JSON.stringify({ error: 'unauthorized' }), { status: 401 });
  }

  // Proceed with authenticated user
}
```

#### Stripe Webhook Verification

```typescript
import Stripe from 'stripe';

export async function handleWebhook(req: Request) {
  const signature = req.headers.get('stripe-signature');
  const body = await req.text();

  try {
    const event = stripe.webhooks.constructEvent(
      body,
      signature,
      STRIPE_WEBHOOK_SECRET
    );
    // Process verified event
  } catch (err) {
    return new Response('Invalid signature', { status: 400 });
  }
}
```

---

### Privacy Policy Summary

| What We Collect | Why | Hosted Mode | BYOK Mode |
|-----------------|-----|-------------|-----------|
| Email | Account identification | ✅ | ❌ |
| Usage counts | Enforce limits, analytics | ✅ | ❌ |
| Payment info | Billing (via Stripe) | ✅ | ❌ |
| Prompts | - | ❌ Never | ❌ Never |
| Enhanced output | - | ❌ Never | ❌ Never |
| Telemetry | Improve product | Opt-in | Opt-in |

---

### Telemetry (Opt-in Only)

**What we collect (if opted in):**
- App version
- OS type
- Feature usage counts (not content)
- Error types (not messages)

**What we never collect:**
- Prompt content
- Enhanced prompt content
- API keys
- Personal information

```toml
# config.toml
[preferences]
telemetry = false  # Default off, user must opt-in
```

---

### Security Checklist

| Area | Measure | Status |
|------|---------|--------|
| **API Keys** | Stored in OS keychain | ✅ |
| **Config File** | Permissions 600 | ✅ |
| **Prompts** | Never logged or stored | ✅ |
| **Auth Tokens** | Short-lived, auto-refresh | ✅ |
| **Backend** | Row Level Security | ✅ |
| **Webhooks** | Signature verification | ✅ |
| **HTTPS** | All API calls | ✅ |
| **Open Source** | Auditable code | ✅ |
| **Telemetry** | Opt-in only | ✅ |

---

### Threat Mitigation

| Threat | Mitigation |
|--------|------------|
| API key theft | OS keychain encryption, never in plain text |
| Man-in-the-middle | HTTPS only, certificate pinning |
| Unauthorized access | JWT auth, RLS policies |
| Prompt leakage | Never stored, processed in memory |
| Malicious templates | Curated templates, community review |
| Rate abuse | Per-user limits, backend enforcement |

---

## Section 9: Monetization & Billing

---

### Pricing Model

| Tier | Price | Limits | Features |
|------|-------|--------|----------|
| **BYOK (Free)** | $0 forever | Provider limits | All features, user's API keys |
| **Trial** | $0 (7 days) | 10/day | All features, no signup needed |
| **Free** | $0 | 5/day | All features, signup required |
| **Pro** | $8/month | 500/day | All features, our API keys |

---

### Revenue Model

```
┌─────────────────────────────────────────────────────────────────┐
│                    Revenue Breakdown                             │
│                                                                  │
│  BYOK Users (70%)              Pro Subscribers (30%)            │
│  ─────────────────             ─────────────────────            │
│  • $0 revenue                  • $8/month each                  │
│  • No cost to us               • LLM costs ~$0.50/user/month    │
│  • Builds community            • Net ~$7.50/user/month          │
│  • Converts to Pro             • Main revenue source            │
│                                                                  │
│  Target: 1000 users            Target: 300 Pro subscribers      │
│  BYOK: 700                     MRR: $2,400                      │
│  Pro: 300                      Annual: $28,800                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Cost Analysis

#### Per-User Costs (Pro Tier)

| Cost Item | Estimate | Notes |
|-----------|----------|-------|
| Claude API | ~$0.40/user/month | ~50 enhancements × $0.008 avg |
| Supabase | ~$0.05/user/month | Free tier covers most |
| Stripe fees | ~$0.50/user/month | 2.9% + $0.30 per $8 |
| **Total Cost** | ~$0.95/user/month | |
| **Net Revenue** | ~$7.05/user/month | 88% margin |

#### Fixed Costs

| Item | Monthly Cost |
|------|--------------|
| Supabase Pro (if needed) | $25 |
| Domain | ~$1 |
| Code signing certificates | ~$8 |
| **Total Fixed** | ~$34/month |

#### Break-even

```
Break-even = Fixed Costs / Net Revenue per User
           = $34 / $7.05
           = 5 Pro subscribers
```

---

### Stripe Integration

#### Products & Prices

```javascript
// Stripe product setup
const product = {
  name: "PromptEnhancer Pro",
  description: "Unlimited prompt enhancement with our API"
};

const price = {
  unit_amount: 800,  // $8.00
  currency: "usd",
  recurring: {
    interval: "month"
  }
};
```

#### Checkout Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Subscription Flow                             │
│                                                                  │
│  1. User clicks "Upgrade to Pro" in app                         │
│  2. App calls POST /billing/create-checkout                     │
│  3. Backend creates Stripe Checkout Session                     │
│  4. App opens Stripe Checkout URL in browser                    │
│  5. User completes payment                                      │
│  6. Stripe sends webhook to /webhooks/stripe                    │
│  7. Backend updates user tier to "pro"                          │
│  8. App refreshes user status                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Webhook Handling

```typescript
// Edge function: /webhooks/stripe
async function handleStripeWebhook(event: Stripe.Event) {
  switch (event.type) {
    case 'checkout.session.completed':
      // Create subscription record, upgrade user to pro
      const session = event.data.object;
      await createSubscription(session.customer, session.subscription);
      await updateUserTier(session.client_reference_id, 'pro');
      break;

    case 'customer.subscription.updated':
      // Update subscription status
      const sub = event.data.object;
      await updateSubscriptionStatus(sub.id, sub.status);
      break;

    case 'customer.subscription.deleted':
      // Downgrade to free
      const deletedSub = event.data.object;
      await downgradeUser(deletedSub.customer);
      break;

    case 'invoice.payment_failed':
      // Mark as past_due, notify user
      await handlePaymentFailed(event.data.object);
      break;
  }
}
```

---

### Billing UI

#### Upgrade Prompt (Free User)

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│  ⚡ You've used 5/5 enhancements today                          │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │   Upgrade to Pro - $8/month                             │    │
│  │                                                          │    │
│  │   ✓ 500 enhancements per day                            │    │
│  │   ✓ All platforms (Claude, GPT, Gemini, Image)          │    │
│  │   ✓ Viral templates                                     │    │
│  │   ✓ SuperMemory integration                             │    │
│  │   ✓ Priority support                                    │    │
│  │                                                          │    │
│  │   [Upgrade Now]                                         │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Or use BYOK mode with your own API key → [Setup BYOK]          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Subscription Management (Pro User)

```
┌─────────────────────────────────────────────────────────────────┐
│  Settings → Subscription                                        │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  Plan: Pro ($8/month)                                           │
│  Status: ✅ Active                                              │
│  Next billing: February 13, 2025                                │
│                                                                  │
│  Usage this month:                                              │
│  ████████░░░░░░░░░░░░░░░░░░░░░░  127 / 500 daily                │
│                                                                  │
│  [Manage on Stripe]    [Cancel Subscription]                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Conversion Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                    User Journey                                  │
│                                                                  │
│  Download App                                                    │
│       │                                                          │
│       ▼                                                          │
│  Trial (7 days, 10/day) ──────────────────┐                     │
│       │                                    │                     │
│       ▼                                    ▼                     │
│  Trial Expires                        Setup BYOK                 │
│       │                                    │                     │
│       ├──► Upgrade to Pro ($8/mo)         │                     │
│       │                                    │                     │
│       └──► Free (5/day) ──► Upgrade ◄─────┘                     │
│                                                                  │
│  Conversion triggers:                                            │
│  • Hit daily limit                                               │
│  • Trial expiring reminder                                       │
│  • Feature discovery (SuperMemory, templates)                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Key Metrics to Track

| Metric | Target | Purpose |
|--------|--------|---------|
| Trial → Pro conversion | 15% | Revenue |
| Trial → BYOK conversion | 40% | Retention |
| Pro churn rate | <5%/month | Revenue retention |
| BYOK → Pro upgrade | 5% | Upsell |
| MRR | $2,400 (month 3) | Business health |

---


## Section 10: Testing Strategy

---

### Testing Pyramid

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│                         E2E Tests                                │
│                        (Few, Slow)                               │
│                      ┌─────────────┐                             │
│                      │  10 tests   │                             │
│                      └─────────────┘                             │
│                                                                  │
│                    Integration Tests                             │
│                      (Some, Medium)                              │
│                 ┌─────────────────────┐                          │
│                 │      30 tests       │                          │
│                 └─────────────────────┘                          │
│                                                                  │
│                      Unit Tests                                  │
│                    (Many, Fast)                                  │
│            ┌─────────────────────────────┐                       │
│            │         100+ tests          │                       │
│            └─────────────────────────────┘                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Unit Tests

#### Core Engine (Rust)

| Module | Test Cases |
|--------|------------|
| **enhance/text.rs** | Platform detection, prompt structuring, context injection |
| **enhance/image.rs** | Style application, parameter formatting, platform-specific output |
| **templates/manager.rs** | Template parsing, field validation, placeholder replacement |
| **config/manager.rs** | Config loading, saving, defaults, migration |
| **integrations/llm** | Request building, response parsing, error handling |

**Example Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        assert_eq!(detect_platform("claude"), Platform::Claude);
        assert_eq!(detect_platform("gpt"), Platform::OpenAI);
        assert_eq!(detect_platform("unknown"), Platform::Generic);
    }

    #[test]
    fn test_template_placeholder_replacement() {
        let template = "{{subject}}, Ghibli style";
        let mut fields = HashMap::new();
        fields.insert("subject".to_string(), "a cat".to_string());
        
        let result = apply_template(template, &fields).unwrap();
        assert_eq!(result, "a cat, Ghibli style");
    }

    #[test]
    fn test_template_missing_required_field() {
        let template = Template {
            fields: vec![TemplateField { name: "subject".into(), required: true, .. }],
            ..
        };
        let fields = HashMap::new();
        
        let result = apply_template(&template, &fields);
        assert!(matches!(result, Err(TemplateError::MissingField(_))));
    }

    #[test]
    fn test_image_prompt_midjourney_params() {
        let enhanced = enhance_image("a cat", Platform::Midjourney, None).unwrap();
        assert!(enhanced.contains("--ar"));
        assert!(enhanced.contains("--v 6"));
    }
}
```

---

### Integration Tests

#### API Integration

| Test | Description |
|------|-------------|
| **LLM API calls** | Mock API responses, verify request format |
| **SuperMemory integration** | Mock context retrieval, verify injection |
| **Stripe webhooks** | Simulate events, verify DB updates |
| **Auth flow** | Token exchange, refresh, expiry |

**Example:**

```rust
#[tokio::test]
async fn test_enhance_with_supermemory() {
    let mock_supermemory = MockSuperMemory::new()
        .with_response(vec![
            Memory { content: "Uses TypeScript".into(), score: 0.9 }
        ]);
    
    let mock_llm = MockLLM::new()
        .expect_context_contains("TypeScript")
        .with_response("Enhanced prompt...");

    let engine = EnhanceEngine::new(mock_llm, Some(mock_supermemory));
    
    let result = engine.enhance(EnhanceRequest {
        prompt: "fix the bug".into(),
        include_supermemory: true,
        ..
    }).await.unwrap();

    assert!(result.context_used.is_some());
}
```

#### Backend Integration

```typescript
// Supabase edge function tests
describe('/enhance endpoint', () => {
  it('should reject unauthorized requests', async () => {
    const res = await fetch('/enhance', { method: 'POST' });
    expect(res.status).toBe(401);
  });

  it('should enforce rate limits', async () => {
    // Use up daily limit
    for (let i = 0; i < 5; i++) {
      await enhanceWithAuth(freeUserToken, 'test prompt');
    }
    
    const res = await enhanceWithAuth(freeUserToken, 'one more');
    expect(res.status).toBe(429);
  });

  it('should track usage', async () => {
    const before = await getUsage(userId);
    await enhanceWithAuth(token, 'test prompt');
    const after = await getUsage(userId);
    
    expect(after.count).toBe(before.count + 1);
  });
});
```

---

### E2E Tests

#### Critical User Flows

| Flow | Steps |
|------|-------|
| **First launch setup** | Open app → Select mode → Configure API key → Test connection |
| **Quick enhance** | Copy text → Press hotkey → Verify clipboard updated |
| **Template usage** | Open templates → Select → Fill fields → Copy result |
| **Subscription** | Click upgrade → Complete Stripe → Verify Pro access |

**Example (using Playwright + Tauri):**

```typescript
import { test, expect } from '@playwright/test';

test('quick enhance flow', async ({ page }) => {
  // Setup: Copy text to clipboard
  await page.evaluate(() => navigator.clipboard.writeText('make a function'));
  
  // Trigger quick enhance (simulate hotkey)
  await page.keyboard.press('Control+Shift+E');
  
  // Wait for notification
  await expect(page.locator('.notification')).toContainText('Enhanced');
  
  // Verify clipboard changed
  const clipboard = await page.evaluate(() => navigator.clipboard.readText());
  expect(clipboard).toContain('Create a');
  expect(clipboard.length).toBeGreaterThan(50);
});

test('template flow', async ({ page }) => {
  await page.click('[data-tab="templates"]');
  await page.click('[data-template="ghibli-style"]');
  await page.fill('[name="subject"]', 'my cat sleeping');
  await page.click('button:has-text("Generate")');
  
  await expect(page.locator('.output')).toContainText('Ghibli');
  await expect(page.locator('.output')).toContainText('my cat sleeping');
});
```

---

### CLI Tests

```rust
#[test]
fn test_cli_basic_enhance() {
    let output = Command::new("pe")
        .arg("make a function")
        .output()
        .expect("Failed to execute");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Create"));
}

#[test]
fn test_cli_pipe_mode() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo 'test prompt' | pe")
        .output()
        .expect("Failed to execute");
    
    assert!(output.status.success());
}

#[test]
fn test_cli_missing_api_key() {
    let output = Command::new("pe")
        .env_remove("ANTHROPIC_API_KEY")
        .arg("--config")
        .arg("/tmp/empty-config.toml")
        .arg("test")
        .output()
        .expect("Failed to execute");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("API key"));
}
```

---

### Test Coverage Targets

| Component | Target | Critical Paths |
|-----------|--------|----------------|
| Core Engine | 80% | Enhancement logic, template parsing |
| CLI | 70% | All commands, error handling |
| Backend | 75% | Auth, billing, usage tracking |
| UI | 60% | Critical flows only |

---

### CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test-core:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --package promptenhancer-core

  test-cli:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --package promptenhancer-cli

  test-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: supabase/setup-cli@v1
      - run: supabase start
      - run: supabase test db

  test-app:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm test
      - run: pnpm build
```

---

### Manual Testing Checklist (Pre-Release)

| Area | Test |
|------|------|
| **Install** | Fresh install on macOS, Windows, Linux |
| **BYOK Setup** | Add Anthropic key, test connection |
| **Hosted Setup** | Login, verify subscription |
| **Quick Enhance** | Hotkey works from different apps |
| **Text Enhancement** | All 4 platforms produce valid output |
| **Image Enhancement** | All 3 platforms produce valid output |
| **Templates** | Trending templates work, custom fields |
| **SuperMemory** | Context fetched and included |
| **Billing** | Upgrade flow, cancel flow |
| **Offline** | Graceful error when no network |

---
## Section 11: Deployment & Distribution

---

### Distribution Channels

| Platform | Method | Auto-Update |
|----------|--------|-------------|
| **macOS** | DMG, Homebrew Cask | ✅ Tauri updater |
| **Windows** | EXE installer, Winget | ✅ Tauri updater |
| **Linux** | AppImage, .deb | ✅ Tauri updater |
| **CLI** | Homebrew, Cargo, curl script | Manual |

---

### Build & Release Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    Release Pipeline                              │
│                                                                  │
│  Push tag (v1.0.0)                                              │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  GitHub Actions                                          │    │
│  │                                                          │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                  │    │
│  │  │  macOS  │  │ Windows │  │  Linux  │                  │    │
│  │  │  Build  │  │  Build  │  │  Build  │                  │    │
│  │  └────┬────┘  └────┬────┘  └────┬────┘                  │    │
│  │       │            │            │                        │    │
│  │       ▼            ▼            ▼                        │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                  │    │
│  │  │  Sign   │  │  Sign   │  │ Package │                  │    │
│  │  │Notarize │  │  EXE    │  │ AppImage│                  │    │
│  │  └────┬────┘  └────┬────┘  └────┬────┘                  │    │
│  │       │            │            │                        │    │
│  │       └────────────┴────────────┘                        │    │
│  │                    │                                      │    │
│  │                    ▼                                      │    │
│  │            GitHub Release                                 │    │
│  │            (artifacts + changelog)                        │    │
│  │                    │                                      │    │
│  │                    ▼                                      │    │
│  │            Update Homebrew/Winget                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Release Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'PromptEnhancer ${{ github.ref_name }}'
          releaseBody: 'See CHANGELOG.md for details.'
          releaseDraft: true

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies
        run: sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev
      
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0

  build-cli:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build CLI
        run: cargo build --release --package promptenhancer-cli --target ${{ matrix.target }}
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: cli-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/pe*
```

---

### Code Signing

#### macOS

| Requirement | Purpose |
|-------------|---------|
| Apple Developer ID | Sign app for distribution |
| Notarization | Apple security verification |
| Hardened Runtime | Required for notarization |

```toml
# tauri.conf.json (macOS signing)
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name",
      "entitlements": "./entitlements.plist"
    }
  }
}
```

#### Windows

| Requirement | Purpose |
|-------------|---------|
| Code Signing Certificate | Sign EXE to avoid SmartScreen warnings |

---

### Auto-Update Configuration

```json
// tauri.conf.json
{
  "plugins": {
    "updater": {
      "active": true,
      "dialog": true,
      "endpoints": [
        "https://releases.promptenhancer.dev/{{target}}/{{current_version}}"
      ],
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

**Update Server Response:**
```json
{
  "version": "1.1.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2025-01-20T00:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "url": "https://github.com/.../PromptEnhancer_1.1.0_x64.dmg.tar.gz",
      "signature": "..."
    },
    "darwin-aarch64": {
      "url": "https://github.com/.../PromptEnhancer_1.1.0_aarch64.dmg.tar.gz",
      "signature": "..."
    },
    "windows-x86_64": {
      "url": "https://github.com/.../PromptEnhancer_1.1.0_x64-setup.nsis.zip",
      "signature": "..."
    }
  }
}
```

---

### Installation Methods

#### macOS

```bash
# Homebrew (recommended)
brew install --cask promptenhancer

# Direct download
curl -L https://github.com/user/promptenhancer/releases/latest/download/PromptEnhancer.dmg -o PromptEnhancer.dmg
open PromptEnhancer.dmg

# CLI only
brew install promptenhancer/tap/pe
```

#### Windows

```powershell
# Winget (recommended)
winget install promptenhancer

# Direct download
# Download .exe from GitHub releases and run installer
```

#### Linux

```bash
# AppImage
curl -L https://github.com/.../PromptEnhancer.AppImage -o PromptEnhancer.AppImage
chmod +x PromptEnhancer.AppImage
./PromptEnhancer.AppImage

# Debian/Ubuntu
curl -L https://github.com/.../promptenhancer.deb -o promptenhancer.deb
sudo dpkg -i promptenhancer.deb

# CLI only
curl -fsSL https://get.promptenhancer.dev | sh
```

---

### Backend Deployment (Supabase)

```bash
# Deploy edge functions
supabase functions deploy enhance
supabase functions deploy usage
supabase functions deploy stripe-webhook

# Run migrations
supabase db push

# Set secrets
supabase secrets set ANTHROPIC_API_KEY=sk-ant-...
supabase secrets set STRIPE_SECRET_KEY=sk_live_...
supabase secrets set STRIPE_WEBHOOK_SECRET=whsec_...
```

---

### Template Distribution

```
┌─────────────────────────────────────────────────────────────────┐
│                    Template Sync                                 │
│                                                                  │
│  GitHub Repo                    CDN                    App      │
│  ───────────                    ───                    ───      │
│                                                                  │
│  templates/          Push       jsDelivr/              Fetch    │
│  ├── image/      ──────────►   GitHub Raw   ◄──────────────     │
│  │   └── *.json                                                 │
│  └── text/                                                      │
│      └── *.json                                                 │
│                                                                  │
│  URL: https://cdn.jsdelivr.net/gh/user/promptenhancer/          │
│       templates/index.json                                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Environment Configuration

| Environment | Backend URL | Stripe | Purpose |
|-------------|-------------|--------|---------|
| **Development** | localhost:54321 | Test keys | Local dev |
| **Staging** | staging.supabase.co | Test keys | Pre-release testing |
| **Production** | prod.supabase.co | Live keys | Users |

---

### Rollback Strategy

| Component | Rollback Method |
|-----------|-----------------|
| **App** | Users can download previous version from GitHub |
| **Backend** | Supabase dashboard → restore previous function version |
| **Database** | Point-in-time recovery (Supabase Pro) |
| **Templates** | Git revert, CDN cache purge |

---

## Section 12: Work Breakdown & Milestones

---

### Project Timeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    MVP Timeline (8 Weeks)                        │
│                                                                  │
│  Week 1-2        Week 3-4        Week 5-6        Week 7-8       │
│  ────────        ────────        ────────        ────────       │
│                                                                  │
│  Foundation      Core Features   Integrations    Polish         │
│  ───────────     ─────────────   ────────────    ──────         │
│  • Project setup • Text enhance  • SuperMemory   • Testing      │
│  • Core engine   • Image enhance • Stripe        • Bug fixes    │
│  • CLI basic     • Templates     • Auth flow     • Docs         │
│  • Tauri shell   • Settings UI   • Usage track   • Release      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Milestone 1: Foundation (Week 1-2)

#### Goals
- Project structure setup
- Core engine with basic enhancement
- CLI working with BYOK
- Tauri app shell running

#### Tasks

| Task | Priority | Estimate | Description |
|------|----------|----------|-------------|
| **M1.1** Project setup | P0 | 2h | Monorepo, Cargo workspace, Tauri init |
| **M1.2** Config manager | P0 | 4h | TOML config, keychain integration |
| **M1.3** LLM client (Anthropic) | P0 | 4h | Claude API integration |
| **M1.4** Basic text enhancer | P0 | 6h | Enhancement logic, system prompts |
| **M1.5** CLI commands | P0 | 4h | `pe "prompt"`, `pe config` |
| **M1.6** Tauri app shell | P0 | 4h | Window, tray icon, basic UI |
| **M1.7** Global hotkeys | P0 | 3h | Quick enhance hotkey |
| **M1.8** Clipboard integration | P0 | 2h | Read/write clipboard |

**Deliverable:** CLI and app can enhance a prompt using user's API key.

---

### Milestone 2: Core Features (Week 3-4)

#### Goals
- All enhancement types working
- Templates system
- Full settings UI
- Platform-specific optimization

#### Tasks

| Task | Priority | Estimate | Description |
|------|----------|----------|-------------|
| **M2.1** Platform detector | P0 | 3h | Claude/GPT/Gemini/Generic detection |
| **M2.2** Platform-specific prompts | P0 | 4h | Optimization per platform |
| **M2.3** Image enhancer | P0 | 6h | Midjourney/DALL-E/SD support |
| **M2.4** Template manager | P0 | 4h | Load, parse, apply templates |
| **M2.5** Built-in templates | P0 | 4h | 10 curated viral templates |
| **M2.6** Template UI | P0 | 6h | Browse, select, fill fields |
| **M2.7** Settings UI | P0 | 6h | API keys, preferences, hotkeys |
| **M2.8** Enhancement UI | P0 | 6h | Input, output, copy buttons |
| **M2.9** OpenAI client | P1 | 3h | GPT API integration |
| **M2.10** Google client | P1 | 3h | Gemini API integration |

**Deliverable:** Full enhancement experience for text and images with templates.

---

### Milestone 3: Integrations (Week 5-6)

#### Goals
- SuperMemory working
- Hosted mode with auth
- Stripe billing
- Usage tracking

#### Tasks

| Task | Priority | Estimate | Description |
|------|----------|----------|-------------|
| **M3.1** SuperMemory client | P0 | 4h | API integration |
| **M3.2** Context injection | P0 | 3h | Inject memories into enhancement |
| **M3.3** SuperMemory UI | P0 | 3h | Connect, settings, status |
| **M3.4** Supabase setup | P0 | 2h | Project, tables, RLS |
| **M3.5** Auth edge functions | P0 | 4h | Login, verify, refresh |
| **M3.6** Auth UI | P0 | 4h | Login flow in app |
| **M3.7** Enhance edge function | P0 | 4h | Hosted enhancement endpoint |
| **M3.8** Usage tracking | P0 | 3h | Count, limits, display |
| **M3.9** Stripe integration | P0 | 6h | Checkout, webhooks, portal |
| **M3.10** Billing UI | P0 | 4h | Upgrade, manage subscription |

**Deliverable:** Complete hosted mode with billing.

---

### Milestone 4: Polish & Release (Week 7-8)

#### Goals
- Testing complete
- Bugs fixed
- Documentation ready
- Release published

#### Tasks

| Task | Priority | Estimate | Description |
|------|----------|----------|-------------|
| **M4.1** Unit tests | P0 | 6h | Core engine tests |
| **M4.2** Integration tests | P0 | 4h | API, auth, billing tests |
| **M4.3** E2E tests | P1 | 4h | Critical flows |
| **M4.4** Bug fixes | P0 | 8h | Buffer for issues |
| **M4.5** Performance optimization | P1 | 4h | Startup time, memory |
| **M4.6** Code signing setup | P0 | 4h | macOS, Windows certs |
| **M4.7** Release pipeline | P0 | 4h | GitHub Actions workflow |
| **M4.8** README & docs | P0 | 4h | Installation, usage guide |
| **M4.9** Landing page | P1 | 4h | Simple marketing page |
| **M4.10** Beta release | P0 | 2h | Publish v0.9.0 |
| **M4.11** Beta feedback | P0 | 4h | Collect, triage, fix |
| **M4.12** v1.0.0 release | P0 | 2h | Final release |

**Deliverable:** Production-ready v1.0.0 release.

---

### Task Summary

| Milestone | Tasks | Total Hours | Weeks |
|-----------|-------|-------------|-------|
| M1: Foundation | 8 | 29h | 1-2 |
| M2: Core Features | 10 | 45h | 3-4 |
| M3: Integrations | 10 | 37h | 5-6 |
| M4: Polish | 12 | 50h | 7-8 |
| **Total** | **40** | **161h** | **8** |

---

### Sprint Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                    Weekly Sprint                                 │
│                                                                  │
│  Monday        Tuesday-Thursday      Friday                     │
│  ──────        ────────────────      ──────                     │
│                                                                  │
│  • Plan week   • Build features      • Test & review            │
│  • Review PRD  • Daily commits       • Fix issues               │
│  • Setup tasks • AI agent coding     • Update docs              │
│                                                                  │
│  Hours: ~20h/week (side project pace)                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

### Risk Buffer

| Risk | Mitigation | Buffer |
|------|------------|--------|
| API integration issues | Mock early, test often | +4h |
| Code signing problems | Start early, follow guides | +4h |
| Tauri/platform bugs | Check GitHub issues | +4h |
| Scope creep | Strict MVP, defer features | +8h |

**Total buffer:** 20h (built into Week 7-8)

---

### Definition of Done (MVP)

| Requirement | Status |
|-------------|--------|
| ☐ CLI works on macOS, Linux, Windows |  |
| ☐ App works on macOS, Linux, Windows |  |
| ☐ Text enhancement for 4 platforms |  |
| ☐ Image enhancement for 3 platforms |  |
| ☐ 10 viral templates included |  |
| ☐ BYOK mode fully functional |  |
| ☐ Hosted mode with Stripe billing |  |
| ☐ SuperMemory integration working |  |
| ☐ Global hotkeys working |  |
| ☐ Auto-update working |  |
| ☐ Documentation complete |  |
| ☐ <1% error rate |  |
| ☐ <3s enhancement latency (P95) |  |

---

### Post-MVP Roadmap

| Version | Timeline | Features |
|---------|----------|----------|
| **v1.1** | +2 weeks | Prompt history, favorites |
| **v1.2** | +4 weeks | Browser extension (Chrome) |
| **v1.5** | +8 weeks | Community templates, voting |
| **v2.0** | +12 weeks | Team features, custom rules |

---

