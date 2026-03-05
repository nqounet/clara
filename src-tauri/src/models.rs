use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 1回のやり取り（セット）を保存するMarkdownのFrontmatter(YAML)部分の定義
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaraFrontmatter {
    /// 必須: タイトル（リスト表示用、ファイル名より分かりやすい名前）
    pub title: String,

    /// 任意: 簡単な説明や要約
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 必須: 一意のID。保存されるMarkdownファイル名（拡張子抜き）と一致する
    pub id: String,

    /// 任意: 前提となる親コンテキスト（直前のやり取り）のID。ない場合はnull
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// 必須: 作成日時 (ISO 8601フォーマット)
    pub created_at: DateTime<Utc>,

    /// 必須: Obsidianのグラフビューで関係性を明示するためのタグ等。空配列可
    #[serde(default)]
    pub tags: Vec<String>,
}

/// アプリケーション内で1つのノード（やり取りのセット）として扱うための型
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaraSet {
    pub frontmatter: ClaraFrontmatter,

    /// ユーザーのプロンプト（送信メッセージ）
    pub prompt: String,

    /// AIからの回答
    pub response: String,
}
