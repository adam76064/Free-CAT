# Free-CAT

**Free-CAT** is a free, open-source, professional Computer-Assisted Translation (CAT) tool built with Tauri 2.0 + Rust + SvelteKit.

## Why Free-CAT?

| Feature | Trados Studio | memoQ | **Free-CAT** |
|---|---|---|---|
| Price | ~\$408/yr | ~€396/yr | **Free forever** |
| Translation Memory | ✅ | ✅ | ✅ FTS5 + Lev+Dice |
| Termbase | ✅ | ✅ | ✅ |
| QA Checks | ✅ | ✅ | ✅ 9 checks (growing) |
| Open Source | ❌ | ❌ | ✅ MIT |
| Modern UI | 🟡 | 🟡 | ✅ SvelteKit |
| RAM Usage | 300–500 MB | 200–400 MB | ~50 MB |

## Technology Stack

- **Shell:** [Tauri 2.0](https://tauri.app/) (Rust + WebView2)
- **Frontend:** [SvelteKit](https://kit.svelte.dev/) + TypeScript + TailwindCSS
- **Backend:** Rust — all core logic
- **Database:** SQLite via `rusqlite` with FTS5
- **Fuzzy Matching:** `strsim` Damerau-Levenshtein + word Dice (0.7/0.3 blend)

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.77+
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Windows)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 11)

### Development

```bash
npm install
npm run tauri:dev
```

### Build

```bash
npm run tauri:build
```

## Project Structure

```
Free-CAT/
├── src/                    # SvelteKit frontend
│   ├── lib/
│   │   ├── components/     # UI components
│   │   │   ├── editor/     # SegmentGrid, SegmentRow
│   │   │   ├── panels/     # TmPanel, QaPanel
│   │   │   └── tm/         # TmMatchBadge
│   │   ├── ipc/            # Tauri IPC wrappers
│   │   └── stores/         # Svelte state stores
│   └── routes/             # SvelteKit pages
│       ├── +page.svelte    # Project list / home
│       ├── project/[id]/   # Project dashboard + editor
│       ├── tm/             # TM Manager
│       └── settings/       # User settings
└── src-tauri/              # Rust backend
    └── src/
        ├── commands/       # Tauri IPC command handlers
        └── core/
            ├── db/         # SQLite layers (project, TM, TB)
            ├── models.rs   # Shared data types
            ├── tm/         # Fuzzy matching + TM engine
            ├── qa/         # QA check rules
            ├── parsers/    # TMX, PO, TXT parsers
            └── segmentation/ # SRX-based sentence splitter
```

## Roadmap

- **Phase 0 (current):** Core scaffold — TM engine, QA, TXT/PO parsers
- **Phase 1:** Full editor MVP — XLIFF parser, DOCX import, MT integration
- **Phase 2:** v1.0 — .sdlppx packages, fuzzy repair, MQM review, AI/LLM
- **Phase 3:** v2.0 — Self-hosted collaboration, REST API

## License

MIT © Free-CAT Contributors
