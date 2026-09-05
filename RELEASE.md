# Drill — Release Package Notes

This archive contains:

- `drill-backend` (`drill-backend.exe` on Windows) — the backend server. PDF export is built in; no Chrome or any other runtime is required.
- `frontend/dist/` — the built frontend, served automatically by the backend.
- `RELEASE.md` — this file.

## Run

Linux / macOS:

```bash
DRILL_DIST=frontend/dist DRILL_ROOT=/path/to/bank DRILL_DATA=/path/to/data ./drill-backend
```

Windows (PowerShell):

```powershell
$env:DRILL_DIST="frontend\dist"; $env:DRILL_ROOT="C:\path\to\bank"; $env:DRILL_DATA="C:\path\to\data"; .\drill-backend.exe
```

Then open http://127.0.0.1:8787.

## Environment variables

- `DRILL_DIST` — frontend static directory. Inside this package, set it to `frontend/dist` (relative to where you run the binary).
- `DRILL_ROOT` — root of your question bank. **Must be set when running from a release package** (the compiled-in default points at the build machine). The directory itself is registered as the default source on first run; additional folders can be added in the app (bank page → sources).
- `DRILL_DATA` — data directory (attempt records, books, sources, mastery marks, export cache). Set it to a writable path when running from a release package.
- `DRILL_HOST` / `DRILL_PORT` — listen address and port, defaults `0.0.0.0:8787`.

## First run

The server seeds an embedded example bank and a demo book into `DRILL_DATA`, so every screen works before you configure anything. To drill your own questions, point `DRILL_ROOT` at your bank — questions are plain Markdown files (one per file, LaTeX math, Obsidian-style image embeds); see the repository's README for the full format.
