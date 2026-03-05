// src/lib/types/clara.ts

/**
 * 1回のやり取り（セット）を保存するMarkdownのFrontmatter(YAML)部分の定義
 */
export interface ClaraFrontmatter {
  /** 必須: タイトル（リスト表示用、ファイル名より分かりやすい名前） */
  title: string;
  
  /** 任意: 簡単な説明や要約 */
  description?: string;
  
  /** 必須: 一意のID。保存されるMarkdownファイル名（拡張子抜き）と一致する */
  id: string;
  
  /** 任意: 前提となる親コンテキスト（直前のやり取り）のID。ない場合はnull */
  parent_id: string | null;
  
  /** 必須: 作成日時 (ISO 8601フォーマット) */
  created_at: string;
  
  /** 必須: Obsidianのグラフビューで関係性を明示するためのタグ等。空配列可 */
  tags: string[];
}

/**
 * アプリケーション内で1つのノード（やり取りのセット）として扱うための型
 */
export interface ClaraSet {
  frontmatter: ClaraFrontmatter;
  
  /** ユーザーのプロンプト（送信メッセージ） */
  prompt: string;
  
  /** AIからの回答 */
  response: string;
}
