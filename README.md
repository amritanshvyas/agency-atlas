# Agency Atlas desktop catalog

A read-only Windows desktop catalog for this repository's agent definitions. It uses Tauri 2, React, TypeScript, and Vite. It does not run agents or change source Markdown, generated TOMLs, or installed Codex files.

## Requirements

- Windows with WebView2 and Visual Studio C++ Build Tools
- Node.js 22, pnpm 11, and Rust/Cargo
- A checkout of this repository

## Setup and development

From `apps/desktop/`:

```powershell
pnpm install
pnpm tauri dev
```

The catalog locates the repository from `AGENCY_AGENTS_REPO`, the checkout used to build the app, the current directory, or the executable's ancestor directories (in that order). If you move the checkout after building, set `AGENCY_AGENTS_REPO` to its current absolute path before launching. A missing repository appears as an error with a retry action.

```powershell
$env:AGENCY_AGENTS_REPO = 'C:\path\to\agency-agents'
pnpm tauri dev
```

The Vite page alone cannot read local files; use `pnpm tauri dev` for the functioning catalog.

## Tests and build

```powershell
pnpm test
pnpm typecheck
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

`pnpm tauri build` produces a Windows NSIS installer in `src-tauri/target/release/bundle/nsis/`. The app reads from the checkout at runtime; it does not package the agent definitions. The installed TOML directory is resolved from the current user's profile and is optional.

## Scope

The catalog reads `divisions.json` for the 18 canonical divisions and recursively scans Markdown files in them. A file must start with YAML frontmatter to appear. This includes future agents in nested division folders, while excluding docs, examples, strategy files, integration copies, and generated output elsewhere. Missing or malformed metadata is shown on the agent with a warning and fallback title; unreadable files are listed as catalog warnings. The relative Markdown path is the stable ID.

For TOML mapping, `scripts/lib.sh` lowercases the source `name`, changes runs of non-ASCII-alphanumeric bytes to `-`, and trims edge hyphens. `scripts/convert.sh` writes `integrations/codex/agents/<slug>.toml`, while installed files are looked up under the current user's `.codex/agents/`. This app checks file presence only. Shared slugs are marked ambiguous, since a filename cannot identify which source produced it. Missing or inaccessible installed directories show **Not available**.
