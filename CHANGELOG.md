# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.2.2](https://github.com/nqounet/clara/compare/v0.2.1...v0.2.2) (2026-03-07)

### [0.2.1](https://github.com/nqounet/clara/compare/v0.2.0...v0.2.1) (2026-03-07)

## [0.2.0] - 2026-03-07T12:43:23+09:00

### Added
- プロジェクトの初期化 (Tauri + Svelte + TypeScript)
- `~/.clara/atoms` へのMarkdown保存機能 (YAML Frontmatter付き)
- ローカルLLM CLI（`gemini-cli`等）を標準入力から実行・連携するバックエンドロジック (Rust)
- ユーザープロンプトから `TITLE`, `DESC`, `TAGS` のメタデータを全自動生成する仕組み
- Markdown内で `~~~~~~user` と `~~~~~~ai` で区切る独自のフォーマット
- UI上での `Cmd + Enter` / `Ctrl + Enter` による素早い送信ショートカット
- `mise` タスク定義 (`dev`, `test`, `build`)

### Changed
- READMEの更新（CLARAのアーキテクチャとコンセプト、ロゴの追加）

## [0.0.1] - 2026-03-06

### Added
- 初期リポジトリの作成
