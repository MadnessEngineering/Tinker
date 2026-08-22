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
`src/browser/mod.rs`. `cargo test` reports **164 passed, 0 failed, 3 ignored** (the ignored three
spawn the built binary). Verified on Linux, August 22, 2026.

Note the executed count exceeds the number of `#[test]` functions in the tree: `main.rs` re-declares
modules `lib.rs` already exports, so their tests run in both binaries. See M2.

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
- Keyboard input over the API or MCP (`browser/keyboard.rs` is internal-only).
- Browser profiles — user agent, viewport, timezone, locale.
- Cross-engine result comparison (see Track B, M4).

---

## M1 — Make the repo tell the truth *(done — August 22, 2026)*

The code had outrun its documentation by about a year. Completed:

- [x] **Rewrote the `readme.md` status section.** It claimed performance/debugging was "in progress"
      when it was substantially built, and billed the project "production-ready" without
      qualification. Now states what works, what's missing, and what's unproven.
- [x] **Fixed broken references.** `docs/getting-started.md` didn't exist (now written); the
      `test_visual.py` link pointed at a file named `test_visual_api.py`.
- [x] **Documented the undocumented API surface.** The readme listed ~20 endpoints; the router has
      ~55. Performance, console, recording, and playback were entirely absent.
- [x] **Retired `CURRENT_STATUS.md`.** Dated July 2025 and wrong on nearly every point: claimed
      template files were missing (they exist), visual testing absent (built), WebSocket unstarted
      (live at `/ws`). This roadmap is the single status document now.
- [x] **Deleted the point-in-time summaries.** `MCP_IMPLEMENTATION_SUMMARY.md` and
      `PERFORMANCE_IMPLEMENTATION_SUMMARY.md` were completion reports superseded by the fuller
      guides in `docs/`. `fix_plan.md` described blockers fixed in March 2026.
- [x] **Removed session cruft** from version control and extended `.gitignore` to keep it out.
- [x] **Consolidated the root Python scripts** into `tests/integration/` with a README covering
      prerequisites and what each one exercises.
- [x] **Deleted two dead modules.** `src/browser/tests.rs` and `src/cli/` were declared in neither
      `lib.rs` nor `main.rs` — they had never compiled. `src/cli/mod.rs` was the more dangerous of
      the two: a stub `Args::parse()` returning defaults and ignoring all input, shadowed by the
      real clap parser in `main.rs`, and an easy trap for anyone who found it first.
- [x] **Wrote `docs/getting-started.md`** with the native dependencies, the exact failure they
      cause, and package names for Debian/Fedora/Arch.

`LESSONS_LEARNED.md` was kept — its testing and thread-safety notes are still true and aren't
recorded elsewhere.

---

## M2 — Shared Foundation: prove it works *(current milestone)*

Cross-engine testing (M4) is only meaningful if Tinker reliably runs on more than one platform.
That makes this milestone load-bearing rather than housekeeping.

- [ ] **GitHub Actions: build + test on macOS, Linux, Windows.** The suite passes and nothing runs
      it. Start here.
- [ ] **Install native deps in CI** so the workflow doubles as executable setup documentation.
      A cold clone needs GTK and WebKitGTK headers that `Cargo.toml` can't declare; without them
      `gdk-sys` fails at `pkg-config --libs --cflags gdk-3.0` with a message that never names the
      fix. Now written down in `docs/getting-started.md`, but documentation rots — CI wouldn't.
- [ ] **Headless-capable test lane.** Windowed tests need a display; sort out `xvfb` on Linux or
      gate the windowed suite so the rest can run everywhere.
- [ ] **Resolve `src/platform/`.** With Windows a real target, decide: finish the abstraction for
      what `tao`/`wry` genuinely don't cover (native chrome, theming, window handles), or delete it.
      Don't leave commented-out traits sitting there for another year. M1 removed two other dead
      modules for the same reason; this is the last one, and the only one with a plausible future.
- [ ] **Deduplicate the module tree.** `main.rs` declares `api`, `browser`, `event`, and
      `templates`, all of which `lib.rs` already exports — so the crate is compiled twice and
      shared tests execute twice (48 in the lib binary, 63 in the bin, largely overlapping).
      `main.rs` should depend on the library rather than re-declaring its modules. Note `mcp`
      lives only in `main.rs` and `platform` only in `lib.rs`, so this needs care, not a blind
      delete.
- [ ] **Clear the warning backlog.** A clean build emits 91 warnings — unused imports, unused
      variables, dead constants in `templates/mod.rs`. Enough noise to hide a real one.
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
- [ ] **Expose keyboard input.** `browser/keyboard.rs` handles shortcuts internally but is reachable
      from neither the API nor MCP. Selector-based `click`/`type` can't test tab order, focus
      traversal, or keyboard accessibility — those need real key events. Wanted by both tracks.
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

### M6 — Browser profiles

Salvaged from the retired `fix_plan.md`, which sketched this and never built it. A profile bundles
the variables a test needs to hold steady: user agent, viewport, timezone, language, cookies.

- [ ] **Profile struct and switching**, so one recorded session can run under several profiles.
- [ ] **Locale and timezone control** — currently untestable, and a common source of real bugs.
- [ ] **Viewport presets** for responsive testing, composing with the visual baselines in M4.

Note: `fix_plan.md` framed part of this as "anti-detection" — spoofing fingerprints to evade bot
detection. Deliberately not carried forward. Configurable profiles for testing your own site are
the useful half; evasion tooling is a different product with different obligations.

### M7 — Integrations

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
