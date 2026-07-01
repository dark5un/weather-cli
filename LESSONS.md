# Lessons Learned

A collection of insights and takeaways from building `weather-cli` with Hermes Agent.

---

## 1. LSP-for-Library-Consumption Is the Game-Changer

The single biggest insight from this project: **LSP tooling isn't just for validation — it's for library consumption.**

Most AI coding agents rely on:
- Training data recall (stale or wrong for new library versions)
- Documentation scraping (incomplete or out of sync)
- Guessing API shapes (leads to hallucinated methods)

This pipeline demonstrated a third approach: **read the actual library source code from cargo's registry cache**, the same way a human developer Ctrl+clicks into a dependency.

Specifically:
- `reqwest::blocking::get()` — read the actual function signature
- `Response` struct — inspected its methods including `.json()`
- `.json()` — confirmed it returned `Result<T>` with serde deserialization

**Key takeaway:** If your agent has file system access to downloaded dependencies (cargo registry, npm `node_modules`, Go module cache), you can resolve API shapes with **100% accuracy** by reading source. No guessing, no hallucination.

---

## 2. Dependency Selection Needs a Human-Like Decision Tree

When the agent hit two compilation errors in Phase 3, it didn't just try random fixes. It:

1. Identified the root cause (`openssl-sys` missing a system library)
2. Recognized the available alternative (`rustls-tls` — pure Rust)
3. Applied the targeted fix (switch TLS feature)

This mirrors how an experienced Rust developer troubleshoots: understand the error, know the alternatives, pick the minimal fix.

**Lesson:** Agents need to understand *why* a dependency fails to compile, not just blindly retry. The `openssl-sys` → `rustls-tls` switch is a textbook Rust pattern that the agent correctly identified.

---

## 3. The "You Choose" Pattern Is Surprisingly Effective

In Phase 2, when asked to pick between blocking reqwest, async reqwest, or ureq, the user said *"you choose"*. The agent made a reasoned decision:

- **Blocking reqwest** — simplest API, no async runtime needed, smallest binary
- No tokio, no `async`/`await`, no `futures`
- 3 fewer dependencies than the async path

**Lesson:** Agents should be able to make reasoned default choices when the user defers. The choice should be explained so the user can override if they disagree. This "explain + proceed" pattern keeps the workflow moving while remaining transparent.

---

## 4. MCP Server Composition Creates a Richer Agent

The pipeline uses two MCP servers with complementary strengths:

| Server | Strengths | Used For |
|--------|-----------|----------|
| **agent-lsp** | 69 tools, batch ops, diagnostics | Validation, error detection, blast radius |
| **Serena** | Symbol search, declarations, references | Library introspection, code navigation |

Together they provide:
- **Symbol-level code intelligence** (Serena) — "find all functions in this file"
- **Batch diagnostics** (agent-lsp) — "check the whole file for errors after edit"
- **Blast radius analysis** (agent-lsp) — "what would break if I changed this?"

**Lesson:** Don't rely on a single LSP adapter. Multiple servers with different strengths cover more of the developer workflow. The MCP protocol makes composition trivial.

---

## 5. Generated Code Was Clean on First Pass — Here's Why

The agent generated all ~200 lines of Rust in Phase 5 and got **zero diagnostics** in Phase 6. This didn't happen by luck — it happened because:

- The agent **already understood the exact API shapes** from Phase 4 source reading
- The agent used `serde::Deserialize` derive macros matching the exact JSON structure from Open-Meteo's documented response format
- The agent applied patterns consistent with `anyhow` error handling (`.with_context()`, `anyhow!()`)
- The agent built a WMO code map explicitly, avoiding guesswork

**Lesson:** The quality of generated code is bounded by the quality of API understanding achieved before generation. Skipping Phase 4 (or doing it poorly) would have produced hallucinated method calls and wrong field names.

---

## 6. The Skill Pipeline Is a Reusable Pattern

The 7-phase pipeline used here is **not hardcoded** — it's a Hermes Agent **skill** that can be applied to any project. Skills define:

- **Phase gates** — which tools are available in which phase
- **Phase transitions** — automatic advancement as tools from later phases are called
- **Enforcement modes** — `warn` (log violations) or `block` (return error with recovery guidance)

**Lesson:** Building a pipeline as a skill (rather than a script) makes it reusable across projects, maintainable, and safe. Enforcement prevents the agent from skipping critical phases like validation before writing code.

---

## 7. The "Seven Issues" Pattern

Across all phases, there were exactly 7 issues:

1. Openssl-sys missing (Phase 3)
2. `.json()` feature missing (Phase 3)
3–5. Three API shape uncertainties resolved in Phase 4 reading
6–7. Two minor serde field questions resolved by reading the API response format

All resolved within their respective phases. None required iteration (Phase 7 was empty).

**Lesson:** A well-structured pipeline catches issues at the right level. Dependency problems found in Phase 3, API uncertainties resolved in Phase 4, code validated in Phase 6. The phase separation prevents cascading failures between different concern domains.

---

## 8. Percent-Encoding: A Subtle Bug That Didn't Happen

The agent included a custom `urlencoding()` function rather than adding the `url` crate as a dependency. This was a deliberate choice:

- Cities with spaces (e.g., "New York") need URL encoding
- The custom function handles just enough cases (spaces → `%20`) without adding a crate
- It falls back to byte-percent encoding for non-ASCII characters

**Lesson:** This is good agentic judgment — recognizing when a dependency is overkill and writing a minimal inline implementation instead. The agent correctly identified that URL-encoding a city name is a bounded problem that doesn't warrant a full URL parser crate.

---

## 9. Open-Meteo Was the Right API Choice

The choice of Open-Meteo (free, no API key) had downstream effects:

- Zero friction for users — no signup, no key management
- No rate limiting concerns for CLI usage
- Simple REST endpoints — easy to model in serde structs
- Geocoding + weather in two endpoints — clean separation

**Lesson:** API selection is a pipeline-phase-1 decision that affects every subsequent phase. Choosing an API with no auth, clear docs, and simple JSON responses simplified the entire build.

---

## Summary Table

| Lesson | Category | Impact |
|--------|----------|--------|
| LSP-for-library-consumption | Architecture | Eliminated API hallucinations |
| Dependency decision tree | Troubleshooting | Fast, correct fixes |
| "You choose" pattern | UX | Keeps pipeline moving |
| MCP server composition | Architecture | Richer tool coverage |
| Clean code on first pass | Output quality | Validates the "read first, write later" approach |
| Reusable skill pipeline | Process | Repeatable across projects |
| Phase isolation | Process | Catches issues at the right level |
| Minimal dependencies | Engineering judgment | Smaller binary, fewer CVEs |
| API selection | Strategy | Simplified the entire build |
