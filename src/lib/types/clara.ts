// src/lib/types/clara.ts

/**
 * アプリケーションのグローバル設定（Vault Rootの場所を管理）
 */
export interface AppConfig {
  root_dir: string;
}

/**
 * CLIツールとLLMの実行設定
 */
export interface ClaraConfig {
  cli_command: string;
  cli_args: string[];
  model?: string;
  working_dir?: string;
}
/**
 * 1回のやり取り（Atom）を保存するMarkdownのFrontmatter(YAML)部分の定義
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
  
  /** 任意: 親Atom（コンテキスト元）へのObsidianリンク。例: "[[parent-id]]" */
  parent?: string;

  /** 必須: Obsidianのグラフビューで関係性を明示するためのタグ等。空配列可 */
  tags: string[];

  /** 任意: 使用したCLIコマンド名（例: gemini） */
  cli_command?: string;

  /** 任意: 使用したモデル名（例: gemini-2.5-pro） */
  model?: string;

  /** 任意: 実行時の作業ディレクトリ（Workspace） */
  workspace?: string;

  /** 任意: YOLOモード（ツール自動実行）で実行されたかどうか */
  yolo?: boolean;
}

/**
 * アプリケーション内で1つのノード（やり取りのAtom）として扱うための型
 */
export interface ClaraAtom {
  frontmatter: ClaraFrontmatter;
  
  /** ユーザーのプロンプト（送信メッセージ） */
  prompt: string;
  
  /** AIからの回答 */
  response: string;
}

/**
 * SKR検索の結果を表す型
 */
export interface SkrSearchResult {
  id: string;
  title: string;
  score: number;
  snippet: string;
}