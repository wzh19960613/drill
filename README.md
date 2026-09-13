# Drill

**English** | [简体中文](README_CN.md)

Drill is a self-hosted drilling app that works out of the box. The question bank is just plain Markdown files, so it plays well with Obsidian and various AI tools. Drill through questions one screen at a time in a full-screen immersive interface, or export a PDF for printing or distribution.

## Features

**Question bank**

- The bank is plain Markdown: one question per `.md` file, with formulas and images.
- See [template.md](template.md) for the question format.
- Multiple bank folders (sources), managed in the app.

**Drilling**

- Immersive full-screen drilling with timing; hotkeys are rebindable.
- Review mode: answers always visible — handy for quickly checking answers against a printed copy.
- Mark questions as favorites, mastered, and more.

**Books**

- Single- and multi-subject banks need no configuration — just include the subject in each question's metadata.
- Generate books in a fixed or shuffled order.
- Manage multiple books.

**Export**

- PDF export: for printing and distribution, with rich layout options.
- Markdown export: easy to paste elsewhere, or to use with Obsidian and AI tools.

**Interface**

- Light / dark / system theme.
- Convenient app-wide UI scaling.
- Pair with SVG so images adapt to both light and dark themes.
- Easy copying of LaTeX and Markdown source.
- Friendly to touch and to keyboard/mouse.
- PWA support — "Add to Home Screen" in the browser gives a near-native app experience.

## Usage

Download an archive from [GitHub Releases](https://github.com/wzh19960613/drill/releases) — Windows x64 (zip), Linux x64 (tar.gz), or macOS Apple Silicon (tar.gz) — extract it, and run the backend from inside the folder:

```bash
DRILL_DIST=frontend/dist DRILL_ROOT=/path/to/bank DRILL_DATA=/path/to/data ./drill-backend
# Windows (PowerShell):
#   $env:DRILL_DIST="frontend\dist"; $env:DRILL_ROOT="C:\path\to\bank"; $env:DRILL_DATA="C:\path\to\data"; .\drill-backend.exe
```

Then open http://127.0.0.1:8787; `RELEASE.md` inside the archive has more detail. You can also run the prebuilt Docker image — see [Docker](#docker).

On first run the server writes an embedded example bank and a demo book into the data directory, so every screen works before you configure anything. To drill your own questions, put Markdown files in a folder and point `DRILL_ROOT` at it, or add the folder in the app via the source manager on the bank page; content is re-read from disk live.

Environment variables (all optional):

| Variable     | Default         | Purpose                                                                          |
| ------------ | --------------- | -------------------------------------------------------------------------------- |
| `DRILL_HOST` | `0.0.0.0`       | Listen address                                                                   |
| `DRILL_PORT` | `8787`          | Listen port                                                                      |
| `DRILL_ROOT` | repo root       | Bank root; when set, it is registered as the default source on first run          |
| `DRILL_DATA` | `backend/data`  | Data directory: attempt records, books, sources, mastery marks, export cache     |
| `DRILL_DIST` | `frontend/dist` | Static frontend directory served by the backend                                  |
| `DRILL_MAX_UPLOAD_MB` | `20`            | Editor image upload cap (MB), 1–200                                              |

## PWA

Drill installs like a native app: launch it from a home-screen or desktop icon, and it opens in its own window without the browser's address bar and tabs.

**iOS / iPadOS (Safari)**

1. Open Drill in Safari.
2. Tap the Share button in the toolbar.
3. Choose "Add to Home Screen", then tap "Add".

**Android (Chrome)**

1. Open Drill in Chrome.
2. Open the ⋮ menu at the top right (or the install icon in the address bar).
3. Choose "Install app".

**Desktop browsers**

- Chrome / Edge: click the install icon at the right of the address bar, or use the menu to "Install app" / "Install this site as an app".
- Safari (macOS): menu bar → "File" → "Add to Dock…".

The installed shortcut points at the server address you were visiting, so the backend must be running to use it.

## Docker

```bash
docker run -d --name drill -p 8787:8787 -v /your/data:/data ghcr.io/wzh19960613/drill:latest
```

Put your Markdown bank under `/data/bank` inside the container (the image sets `DRILL_ROOT=/data/bank`); records, books, and exports persist under the mounted `/data` volume. The image is pushed to GHCR on every GitHub release.

## Building

Build from source:

```bash
# frontend; output in frontend/dist is served by the backend
cd frontend && npm ci && npm run build

# backend
cd backend && cargo build --release --locked
```

For development, run the backend directly, with the frontend dev server alongside:

```bash
cd backend && cargo run --release    # serves the built frontend on 0.0.0.0:8787
cd frontend && npm run dev           # hot reload, proxies /api to 8787
```

Tests: `cargo test` in `backend/`, `npm test` in `frontend/`.

Build the Docker image:

```bash
docker build -t drill .
```

Run from the repository root.
