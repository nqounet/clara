---
name: release-workflow
description: Automates the release workflow, including CHANGELOG.md updates, version bumping (package.json, Cargo.toml), lockfile synchronization, committing, and git tagging.
---

# Release Workflow Skill

このスキルは、プロジェクトのバージョンバンプ、`CHANGELOG.md` の更新、ロックファイルの同期、コミット作成、および Git タグの付与プロセスを自動化・手順化します。

## 🚨 制約事項 (Constraints)

リリースワークフローを実行する前に、`AGENTS.md` に記載されている以下の制約を厳格に遵守してください：
1. **明示的な指示の確認**: ユーザーから「リリースして」「バージョンを上げて」といった明示的な指示がない限り、絶対に自動でバージョンアップやコミット、プッシュを行わないでください。
2. **クリーンな状態**: 開始前に `git status` がクリーンであることを確認してください。
3. **テストの実行**: コミットやプッシュを行う前に、必ずすべてのテスト（`npm run check`, `npm run test`, `cargo test` など）がパスすることを確認してください。

## 🛠️ 実行手順 (Execution Steps)

ユーザーからリリース指示があった場合、以下の手順を実行します：

### 1. 対象バージョンの決定
ユーザーにインクリメントの種類（`patch`, `minor`, `major`）または具体的なバージョン（例: `0.4.3`）を確認します。指示に含まれている場合はそれを採用します。

### 2. CHANGELOG の確認
`CHANGELOG.md` の `## [Unreleased]` セクションを確認します。前回のリリース以降の変更内容が正しく記載されているか確認してください。

### 3. バージョン更新スクリプトの実行
ワークスペースのルートディレクトリから、以下のヘルパースクリプトを実行します。
これにより、`package.json`、`src-tauri/Cargo.toml`、`CHANGELOG.md` のバージョン表記が更新され、ロックファイルが自動で同期されます。

```bash
node .agents/skills/release-workflow/scripts/release.js <patch|minor|major|x.y.z>
```

### 4. 動作・品質確認（テスト）
ビルドやテストに問題がないか確認します：
```bash
npm run check && npm run test && cargo test --manifest-path src-tauri/Cargo.toml
```

### 5. コミットとタグの作成
変更されたファイルをステージングし、リリース用のコミットを作成します。
```bash
git add package.json package-lock.json CHANGELOG.md src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore(release): vX.Y.Z"
git tag vX.Y.Z
```
*(X.Y.Z は決定した新しいバージョン番号に置き換えてください)*

### 6. ユーザーへの報告とプッシュの確認
コミットとタグの作成が完了したことをユーザーに報告し、リモートへのプッシュ（`git push && git push --tags`）を行ってもよいか確認します。ユーザーの明示的な許可が出るまで、プッシュは絶対に実行しないでください。
