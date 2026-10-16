# Free-CAT — Development Checkpoint

**Date:** 2026-10-16  
**Milestone:** Phase 0 Complete / Phase 1 MVP Baseline (~80%)  
**Attribution:** Developed by **Claude**  
**Handover Status:** Checkpoint prior to **Gemini** starting development work  

---

## 📌 Context & Purpose

This checkpoint records the exact state of the Free-CAT repository as created, implemented, and verified by **Claude**, before handover to **Gemini** for subsequent feature additions and Phase 1 completion.

---

## 🛠️ Work Completed Under Claude's Authorship

### 1. Architecture & Core Infrastructure (Phase 0)
- **Tauri 2.0 + Rust + SvelteKit** desktop architecture.
- **SQLite Database System**:
  - `projects.db`: Central project metadata, files, segment states, analysis, and QA logs.
  - Dedicated TM databases (`.db` per TM with SQLite FTS5 + WAL mode).
  - Dedicated Termbase databases (`.db` per TB with SQLite FTS5).
- **Fuzzy Matching Algorithm**: Blended scoring formula (`0.7 × Levenshtein (strsim) + 0.3 × word-Dice`), with 102% context, 101% document, 100% exact, and fuzzy bands.
- **SRX 2.0 Segmentation Engine**: Language-aware break and no-break regex rules, abbreviations, and sentence boundary detection.
- **Production Standalone Release**: Solved desktop offline packaging (`custom-protocol` feature flag in `src-tauri/Cargo.toml` and Windows GUI subsystem), producing a verified self-contained executable at `target/release/free-cat.exe` (~8.1 MB) that operates completely offline without Node/Vite.

### 2. Implemented Features (Phase 1 Baseline)
- **Bilingual Editor Grid**: Side-by-side source/target grid with contenteditable editing, row selection, inline tag protection (`{1}`, `{2}`), keyboard confirmation (`Tab`/`Enter`), and status workflow (New → Draft → Translated → Approved).
- **Translation Memory (TM)**: Interactive TM inspector panel with top-5 match scoring, 1-click match application, auto-propagation across repeated segments, and automatic TM write-back on confirmation.
- **Termbase / Glossary**: Live term matching against active source segments, interactive glossary sidebar, 1-click term insertion, and in-editor term creation.
- **File Parsers**:
  - XLIFF 1.2 and XLIFF 2.0
  - Trados `.sdlxliff`
  - memoQ `.mqxliff`
  - TMX 1.4b (import and export)
  - TBX-Basic ISO 30042 (import and export)
  - GNU PO / POT
  - Plain text (`.txt`)
- **Automated QA Engine**: 10 automated quality checks (empty targets, tag mismatches, whitespace, double spaces, missing numbers, repeated words, forbidden terms, length violations, untranslated identical segments) with a dedicated QA panel.
- **Analysis & Pre-translation Backend**: Exact and fuzzy match volume calculation, TM savings analysis, and batch pre-translation.
- **Machine Translation Clients**: Backend API integration for DeepL, LibreTranslate (offline/self-hosted), and Google Translate.
- **Settings & UI**: Dark and light mode theming, persistent user settings, and project dashboard.

---

## 📋 Open Tasks Scheduled for Subsequent Development (Gemini)

1. **DOCX Document Parser**: OpenXML extraction preserving runs and inline formatting tags into `{1}`, `{2}` placeholders.
2. **HTML Parser**: Web document parser with tag protection.
3. **Editor Capabilities**:
   - Segment Split & Merge (`Ctrl+Enter` / `Ctrl+Backspace`).
   - Find & Replace in target text.
   - Status filtering dropdown (untranslated, translated, QA warnings).
4. **UI Connectors**:
   - Visual Analysis Report dialog displaying TM leverage and savings table.
   - MT suggestion cards displayed directly inside the TM sidebar panel.
5. **Spreadsheet Glossary I/O**: Direct CSV and Excel XLSX import/export for termbases.

---

## 🏷️ Checkpoint References

- **Git Tag:** `checkpoint-claude-pre-gemini`
- **Git Branch:** `checkpoint/claude-baseline`
- **Standalone Binary:** `target/release/free-cat.exe` (~8.1 MB, verified working)
