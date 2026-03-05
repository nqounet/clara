<div align="center">
  <img src="assets/logo.png" alt="CLARA Logo" width="600" />
</div>

# CLARA

**Context-Linked Atomic Repository Architecture**

「対話を資産に変え、思考の軌跡をファイルとして手元に残す」

**CLARA（クララ）**は、タイムラインに流れて消えるチャットではなく、1回のやり取り（セット）を独立したMarkdownファイルとして永続化する、Obsidian連動型のノードベースAIチャットエンジンです。

ユーザーは常に「直近の明確な意図」のみをAIに渡し、過去の知見はローカル検索とObsidianによるネットワーク管理で引き出します。トークン浪費ゼロ、誤送信ゼロの、あなた専用の開発者向けナレッジ構築環境を提供します。

---

## 🚀 コア・アーキテクチャ

### 1. 1セット = 1 Markdownファイル

ユーザーの送信メッセージとAIの返信は、必ず1つのMarkdownファイルとして保存されます。

* **Frontmatter (YAML):** セットID、親セットID、作成日時、タグなどを記録。
* **Body:** ユーザーのプロンプトと、AIの回答を記述。
* これにより、Obsidian等のローカルツールでグラフビューとして可視化・編集が可能になります。

### 2. 明示的なコンテキスト・リンク（数珠繋ぎ）

AIには**「今回送信するメッセージ」と「明示的に指定した親セット（Markdownの内容）」のみ**を送信します。

* 過去の履歴を自動でダラダラと送ることは一切しません。
* 「あの件の続き」を話したい時は、SKRで過去のMarkdownを検索し、そのIDを「親」として指定するだけです。

### 3. SKR（セマンティック・ナレッジ・リポジトリ）の役割

ローカルのMarkdownファイル群を常にインデックス化し、ユーザーからの「あの設定どこだっけ？」という検索要求に対して、該当のファイルパス（またはID）を即座に返す、純粋な「検索エンジン」として機能します。

---

## 🔄 システムフロー

```mermaid
sequenceDiagram
    actor User
    participant CLI_GUI as "CLARA アプリ (フロントエンド)"
    participant SKR as "SKR (ローカル検索)"
    participant Files as "Markdown (Obsidian Vault)"
    participant AI as "AI API"

    User->>CLI_GUI: "あのスキルの話の続きを作りたい"
    CLI_GUI->>SKR: "検索: スキル作成の話題"
    SKR-->>CLI_GUI: "該当ファイル: 20260306-skill.md"
    User->>CLI_GUI: "親セットを指定して新メッセージを入力・送信"
    Note over User,CLI_GUI: "※確定エンターは無効化、明示的送信アクションのみ"

    CLI_GUI->>Files: "親セット(20260306-skill.md)の読み込み"
    CLI_GUI->>AI: "親セット内容 + 新メッセージを送信"
    AI-->>CLI_GUI: "AIの回答"

    CLI_GUI->>Files: "新しいセットをMarkdownで保存 (親IDを記録)"
    CLI_GUI->>SKR: "新ファイルをインデックスに追加"
    CLI_GUI-->>User: "画面表示を更新"
```
