# Milestone 1 — read-only catalog

## Scope

The app provides a searchable list, category filtering, agent details, metadata warnings, and generated/installed Codex TOML presence. It reads files only. There is no agent execution, task flow, shell access from the UI, listener, or change to global Codex configuration.

## Source discovery

`divisions.json` defines the only source divisions. The loader recursively scans `.md` files inside those folders and accepts files beginning with a YAML frontmatter fence. This mirrors the directory boundary and recursive scan in `scripts/convert.sh`. The repository currently has 282 Markdown files in those divisions. Other Markdown files, including `strategy/`, `examples/`, `integrations/`, root docs, and generated output, are outside that boundary. In particular, `integrations/mcp-memory/backend-architect-with-memory.md` shares a display name with a source agent but is an integration-specific copy, so it is excluded.

The stable ID and shown source path are the repository-relative Markdown path with `/` separators. A missing name gets a filename-based display title and no TOML attribution. Malformed YAML and missing fields do not stop discovery. Individual read errors appear in catalog warnings.

## TOML mapping

The app follows `slugify` in `scripts/lib.sh` and the Codex output path in `scripts/convert.sh`: slug from the frontmatter `name`, then `<slug>.toml`. It checks the generated directory and the current user's `.codex/agents/` directory without modifying files. If multiple included source agents produce one slug, both show **Ambiguous** and list conflicting source paths. A TOML's presence is not proof that its contents are current or that it came from a particular source. The existing integration copy with the `Backend Architect` name does not create ambiguity in the canonical source roster.

## Limitations

- Repository discovery needs a readable checkout; set `AGENCY_AGENTS_REPO` when the build-time checkout is unavailable.
- The app checks TOML filenames and presence, not TOML contents or sync freshness.
- The installed directory depends on the current Windows user profile. If unavailable or inaccessible, the UI reports **Not available**.
- The initial catalog has no file watcher; use Refresh to reread files.

## Verification

Verified on Windows on 2026-10-14:

- `pnpm test`: 3 frontend tests passed.
- `pnpm typecheck`: passed.
- `pnpm build`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 7 Rust tests passed, including live source discovery.
- `pnpm tauri build`: passed; produced the release `.exe` and an NSIS installer.

Cargo was invoked with `C:\Users\Delg\.cargo\bin` added to the command's `PATH`, because that directory was not on PowerShell's initial `PATH`. The GUI has not been manually exercised; the native build and catalog loader test are the runtime-level checks performed here.
