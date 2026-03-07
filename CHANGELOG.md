# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [0.2.2] - 2026-03-07T19:40:00+09:00

### Changed
- アプリ名を `temp-app` から `CLARA` に変更
- WorkspaceとVaultのアイコンをそれぞれ `🖥️` と `🏛️` に変更し、UIレイアウトを改善
- エージェントの自律的なコミット・バージョンアップ操作の要件を見直し、`AGENTS.md` のルールを更新

## [0.2.1] - 2026-03-07T13:07:40+09:00

### Added
- YOLOモードの実装（エージェントがファイル編集等を自動実行）
- 設定UIをモーダル化し、Vault・Workspace・CLI・モデル設定を個別に変更可能に改善
- セマンティックバージョニングと `CHANGELOG.md` 更新プロセスの自動化を `AGENTS.md` に定義

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
