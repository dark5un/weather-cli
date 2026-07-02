# LinkedIn Post Draft

---

## Title: I Built a Local LLM Agent Pipeline That Actually Downloads and Reads Library Source Code Before Writing Code

---

I've been experimenting with Hermes Agent (from Nous Research)  -  a local LLM-powered coding agent  -  and decided to give it a non-trivial task:

**Build a weather CLI in Rust, from scratch, using only agent-driven development.**

The result? A working CLI tool that fetches weather for any city via Open-Meteo (free, no API key). But the process is what surprised me.

The agent followed a 7-phase pipeline orchestrated by a reusable skill:

**Phase 1  -  Research**
Searched for Rust HTTP clients, weather APIs, and CLI libraries. Found reqwest, clap, serde, anyhow, and Open-Meteo (the only major weather API I know of that requires zero authentication).

**Phase 2  -  Clarify**
Asked me to choose between blocking reqwest, async reqwest, or ureq. I said "you choose." The agent picked blocking reqwest  -  simplest path, no tokio overhead.

**Phase 3  -  Install**
Created the cargo project, wrote Cargo.toml, hit two compilation errors (openssl-sys → switched to rustls-tls; missing .json() → added json feature), and resolved both correctly. Clean compile on third attempt.

**Phaseia 4  -  This is the novel part**

Before writing a single line of application code, the agent activated the project in Serena (an MCP server for code intelligence), listed all symbols, and then... **read the actual source code of reqwest from cargo's registry cache**.

It navigated to `reqwest::blocking::get()`, read its function signature, inspected the `Response` struct, and looked at the `.json()` method's return type.

Think about what this means: the agent wasn't guessing the API shape from training data. It wasn't scraping docs. It was doing what a human developer does when they Ctrl+click on a dependency in their IDE  -  reading the source to understand exactly what's available.

This is LSP-for-library-consumption, not LSP-for-validation. And it's the first time I've seen an agent do this systematically.

**Phase 5  -  Generate**
With the API surface fully understood, the agent wrote all ~200 lines of Rust in one pass: geocoding, weather fetch, WMO code → emoji mapping, formatted output.

**Phase 6  -  Validate**
Zero diagnostics. Zero warnings. `cargo check` passed. Symbol-level verification confirmed every function existed at its expected call site.

**Phase 7  -  Iterate**
Not needed. Everything worked on the first complete pass.

---

**Why this matters:**

Most AI coding agents either:
(a) Guess API shapes from training data (hallucinating methods that don't exist)
(b) Rely on documentation (often stale or incomplete)

This pipeline does something different: it reads the actual library source code from the local dependency cache, using MCP-based LSP tooling, before writing a single line of application code. The result is generated code that already matches the real API  -  because it was built against the real API.

The whole thing ran on local LLMs, using Hermes Agent's skill system with phase-gated tool enforcement. The two MCP servers (agent-lsp with 69 batch tools, Serena with ~32 symbol-level tools) composed naturally through the MCP protocol.

**The stack:**
- Hermes Agent (Nous Research)  -  agent framework
- agent-lsp MCP server  -  batch ops, diagnostics, blast radius
- Serena MCP server  -  symbol search, go-to-definition, source navigation
- Open-Meteo  -  free weather API (no key required)
- Rust  -  reqwest, clap, serde, anyhow

**The repo is here:** https://github.com/dark5un/weather-cli

I'm genuinely excited about where this is heading. The "read source first, write code second" pattern eliminates an entire category of AI coding failures. And the fact that it's all composed through standard MCP protocols means it's not locked to one framework.

What's the most impressive agent-driven code generation you've seen? I'm curious what else this pattern could unlock.

---

**Tags:** #Rust #AI #LLM #DeveloperTools #AgenticAI #HermesAgent #OpenSource #LocalLLM #CodingAgents

*[1350 characters  -  good for LinkedIn engagement]*
