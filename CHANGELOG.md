# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [Unreleased]

## [0.3.0] - 2026-03-12T08:55:54+09:00

### Added
- implement local vector search for SKR without external API dependencies

### Changed
- implement code review suggestions for local vector search

## [0.2.9] - 2026-03-12T02:41:28+09:00

### Added
- add workspace history to config and UI

### Changed
- fix duplicate CI runs on PRs and formatting errors

### Fixed
- resolve path traversal vulnerability

## [0.2.8] - 2026-03-09T09:37:53+09:00

### Removed
- 不要になった旧ストアファイル `src/lib/clara.svelte.ts` を削除（0.2.7 のストア分割に伴うクリーンアップ）

## [0.2.7] - 2026-03-09T09:09:35+09:00

### Changed
- Migrate stores to Svelte 5 runes and configure Vitest

### Fixed
- Address PR review comments for atom mock and config validation

## [0.2.6] - 2026-03-08T22:30:00+09:00

### Changed
- Atom（Markdown）生成時に末尾へ付与していた余分な親リンク（`**Parent:** [[id]]`）ブロックを削除（フロントマターで管理されているため）

## [0.2.5] - 2026-03-08T16:53:59+09:00

### Added
- AI回答のリアルタイム・ストリーミング表示機能を実装。回答の生成に合わせてUIが逐次更新されるように改善。

### Changed
- `AGENTS.md` にナレッジ管理（Atomic Thinking）のガイドラインを追加。LLMがVault内の過去のAtom（ID）を自律的に参照し、文脈を補完する設計を明文化。

### Fixed
- ストリーミング表示の遅延を解消。標準出力の読み取りを `BufReader`（行単位）からバイト単位の直接読み取りに変更し、バッファリングによる「まとめ出し」を防止。

## [0.2.4] - 2026-03-08T02:30:00+09:00

### Added
- SKR(セマンティック・ナレッジ・リポジトリ)を利用した過去のAtomの検索機能を実装

### Fixed
- 検索ボックスでのIME入力確定時（Enterキー）に意図せず検索が実行されてしまう不具合を、ネイティブの`<form>`要素を使用することで修正
- 検索対象がプロジェクト全体になっていた問題を修正し、Vaultディレクトリ内のみを検索するように変更
- `gemini` CLI呼び出し時の引数エラー（`--search`, `--prompt`フラグの誤用）を修正

## [0.2.3] - 2026-03-07T19:50:00+09:00

### Changed
- `CHANGELOG.md` の日付フォーマットの修正と更新

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
