# Gemini CLI Constraints & Guidelines

## 🚨 Critical Constraints (Top Priority)
* **NEVER** commit or push changes to version control unless explicitly instructed by the user. Always stop and wait for a direct command (e.g., "commit this", "push to origin").
* **Language:** Always communicate with the user in **Japanese** (日本語).

## Project: CLARA (Context-Linked Atomic Repository Architecture)
* **Stack:** Rust (Tauri) + Svelte + TypeScript
* **Philosophy:** "対話を資産に変え、思考の軌跡をファイルとして手元に残す"
* **Data Store:** Markdown with YAML Frontmatter, separated by `~~~~~~user` and `~~~~~~ai` blocks. Stored locally (default: `~/.clara/atomics`).
* **LLM Integration:** Direct CLI execution via `stdin` (no API keys managed by the app).
* **Open Source:** Adheres to Semantic Versioning and Keep a Changelog standards.
