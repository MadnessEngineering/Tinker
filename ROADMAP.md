# Tinker Roadmap

> **Last verified against the code**: August 22, 2026
> **How to re-verify**: every "Built" claim below names the file that implements it and the
> tests that cover it. If you're reading this months from now, spot-check a few before trusting it.

Tinker serves two audiences, and the roadmap is organized around that split:

- **Track A — the agent surface.** A browser an AI agent can drive and observe, over MCP and MQTT.
- **Track B — the testing workbench.** A browser a person uses to record, replay, and diff web tests.

Both sit on one core engine. Work that serves both lives in **Shared Foundation**.

---

## Where Tinker actually stands

The engine dispatches ~70 `BrowserCommand` variants (`src/event/mod.rs`), all handled in
`src/browser/mod.rs`. 147 tests across 27 files. The following is what that adds up to.

### Built and wired

| Capability | Implementation | Tests |
|---|---|---|
| Tabs, navigation, per-tab history | `browser/tabs.rs`, `browser/navigation.rs` | 14 |
| Session persistence across launches | `browser/session.rs` | 3 |
| MQTT event tower + reconnection | `event/mod.rs` | 4 |
| REST API | `api/mod.rs` | — |
| WebSocket live control (`/ws`) | `api/mod.rs` | — |
| MCP server (JSON-RPC 2.0 over stdio) | `mcp/mod.rs` | 34 |
| DOM inspector (CSS/XPath/text), interaction, waits | `browser/inspector.rs` | 2 |
| JavaScript execution | `browser/mod.rs` | — |
| Visual baselines + pixel diffing | `browser/visual.rs` | 2 |
| Network monitoring + HAR export + filters | `browser/network.rs` | 2 |
| Console monitoring + filtering | `browser/console.rs` | 9 |
| Performance: Core Web Vitals, memory, JS profiling, marks/measures | `browser/performance.rs` | 28 |
| Recording + replay: seek, step forward/back, speed, loop | `browser/replay.rs` | 5 |

### Partial

- **Assertions in recordings.** `AddRecordingAssertion` exists and stores expected state; there's
  no authoring UX and no pass/fail surfacing.
- **Platform layer.** `src/platform/` has `common.rs` and `macos.rs` (124 lines), but every trait
  in `mod.rs` is commented out and there is no `windows.rs` or `linux.rs`. Currently dead weight —
  see M2, where cross-engine work forces a decision on it.

### Not started

- CI of any kind. No `.github/workflows`.
- Test generation from recordings.
- Report/export layer.
- Cross-engine result comparison (see Track B, M4).

---

## M1 — Make the repo tell the truth *(current milestone)*

The code outran its documentation by about a year. Nothing new gets built until a reader can trust
what's written down. This is small, and it makes every later milestone safe to start.

- [ ] **Rewrite `readme.md` status section.** It currently claims Phase 4 performance/debugging is
      "in progress" when it's substantially built, and bills the project "production-ready" without
      qualification. State what works, on which platforms, with what gaps.
- [ ] **Fix broken references in `readme.md`.** Links `docs/getting-started.md` (doesn't exist) and
      `test_visual.py` (the file is `test_visual_api.py`).
- [ ] **Retire `CURRENT_STATUS.md`.** Dated July 2025 and wrong on nearly every point: says template
      files are missing (they exist), visual testing is absent (it's built), WebSocket isn't started
      (it's live at `/ws`). Delete it; this roadmap is the single status document now.
- [ ] **Fold the summary docs in.** `MCP_IMPLEMENTATION_SUMMARY.md`,
      `PERFORMANCE_IMPLEMENTATION_SUMMARY.md`, and `fix_plan.md` are point-in-time notes. Move
      anything still true into `docs/`, delete the rest.
- [ ] **Remove session cruft.** `2025-07-23-this-session-is-being-continued-from-a-previous-co.txt`
      and `error_log.txt` don't belong in version control.
- [ ] **Consolidate the root Python scripts.** Seven ad-hoc `test_*.py` files at the repo root
      exercise the live API. Move them to `tests/integration/` with a README explaining that they
      need a running browser, or retire the ones superseded by Rust tests.
- [ ] **Replace `src/browser/tests.rs`.** Ten lines, one test, zero assertions, calls `Browser::new()`
      which may no longer exist. Either write real tests or delete the file.
- [ ] **Write `docs/getting-started.md`,** including the native dependencies below — the single
      most likely thing to stop a new contributor.

### The build trap worth documenting

A cold clone doesn't build without system libraries that aren't obvious from `Cargo.toml`. On Linux
you need the GTK and WebKitGTK development headers; without them `gdk-sys` fails at
`pkg-config --libs --cflags gdk-3.0` with a message that doesn't name the fix clearly. This is
exactly the class of problem CI would catch, which is why M2 follows immediately.

---

## M2 — Shared Foundation: prove it works

Cross-engine testing (M4) is only meaningful if Tinker reliably runs on more than one platform.
That makes this milestone load-bearing rather than housekeeping.

- [ ] **GitHub Actions: build + test on macOS, Linux, Windows.** 147 tests exist and nothing runs
      them. Start here.
- [ ] **Install native deps in CI** so the workflow doubles as executable setup documentation.
- [ ] **Headless-capable test lane.** Windowed tests need a display; sort out `xvfb` on Linux or
      gate the windowed suite so the rest can run everywhere.
- [ ] **Resolve `src/platform/`.** With Windows a real target, decide: finish the abstraction for
      what `tao`/`wry` genuinely don't cover (native chrome, theming, window handles), or delete it.
      Don't leave commented-out traits sitting there for another year.
- [ ] **Tag v0.1.0** once the matrix is green. First point a user can be pointed at.

---

## Track A — Agent-Native Control

Tinker's most distinctive property: an agent can drive the browser *and* observe what it did.
The MCP server (`src/mcp/mod.rs`, 34 tests) already exposes navigation, tabs, screenshots, visual
tests, DOM find/click/type, JavaScript execution, and network monitoring.

### M3 — Close the agent feedback loop

- [ ] **Expose the observability suite over MCP.** Console logs, performance metrics, and Core Web
      Vitals are all built and reachable via REST, but absent from the MCP tool list. An agent
      currently can't ask "did that click throw a console error?" — the highest-value question it
      could ask.
- [ ] **Expose recording/replay over MCP.** Let an agent record its own session and replay it.
- [ ] **Structured errors for agents.** Failures should return machine-readable causes, not prose.
- [ ] **MCP resources and prompts.** `handle_resources_list` and `handle_prompts_list` return empty.
      Resources could expose the live DOM, console buffer, and network log as readable context.
- [ ] **Document the agent loop** in `docs/mcp-server.md`: act → observe → assert.

---

## Track B — Testing Workbench

For a person authoring and running tests. Recording, replay, visual diffing, and network capture
are built; what's missing is everything that turns a session into a durable, shareable test.

### M4 — Cross-engine bug detection

The reason this matters: *"does this bug reproduce on WebKit but not Chromium?"* Tinker can answer
that today and doesn't know it — `wry` binds the OS webview, so the macOS and Linux builds run
WebKit/JavaScriptCore while the Windows build runs Chromium/V8. Two engine families, already
covered, just never compared.

- [ ] **Run the same recorded session across the CI matrix** and collect results per platform.
- [ ] **Compare across engines** — visual baselines, console errors, network behavior — and report
      divergences. The comparison machinery in `browser/visual.rs` already does the pixel work.
- [ ] **Report engine identity** in results, so a divergence names the engine rather than just the OS.
- [ ] **Per-engine baselines**, since minor rendering differences are expected and shouldn't be
      reported as regressions.

This depends on M2 and can't start before it.

### M5 — From recording to test

- [ ] **Generate a runnable test from a recording.** The loop the project was built for and the
      biggest missing piece: recordings replay, but can't become artifacts you commit.
- [ ] **Assertion authoring.** Build on `AddRecordingAssertion` so a person can mark expectations
      during or after a recording.
- [ ] **Run reports.** HTML and JSON output with screenshots, console output, network summary, and
      pass/fail. Nothing today surfaces a result beyond logs.
- [ ] **Export tooling** for CI consumption — JUnit XML or similar.

### M6 — Integrations

- [ ] **CI recipes** for running Tinker suites in GitHub Actions.
- [ ] **Plugin interface** for custom testing tools.
- [ ] **Selenium / Playwright bridges** — *deliberately last.* Revisit only if someone actually wants
      to bring existing suites over; MCP and the REST API cover the automation need, and these
      adapters are a large surface to maintain.

---

## Explicitly not doing

Recorded so these don't get re-proposed.

**Embedding multiple JS engines** (the old "JavaScript Engine Workshop": V8 integration,
SpiderMonkey support, JavaScriptCore bridge, engine switching). Tinker is built on `wry`, which
delegates to the OS webview and its bundled engine. You cannot swap V8 into the macOS build.
SpiderMonkey is unavailable at any price — Gecko ships no embedding API of this kind, so Firefox
coverage would mean abandoning `wry` entirely.

*The underlying goal survives as M4*, which gets cross-engine coverage from the CI matrix instead —
real WebKit and real Chromium, in their shipping configurations, which is better evidence than
embedded engines would have provided anyway.

**Full platform abstraction as originally scoped.** `tao` and `wry` already abstract windowing and
webviews. M2 decides whether the thin remainder is worth keeping.

---

## Keeping this honest

The previous roadmap drifted for about a year: it listed visual testing, network inspection, DOM
tooling, and WebSocket control as unstarted while all four were built and shipping, and never
mentioned the MCP server at all — by then one of the largest modules in the repo.

Two habits prevent a repeat:

1. **Update this file in the same commit as the feature.** A roadmap edited separately is a roadmap
   that drifts.
2. **Cite the code.** Every status claim names a file. A claim that can't name one is a plan, not a
   status — put it under a milestone, not in the standings table.
