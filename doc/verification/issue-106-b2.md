# Issue #106 B2: schema v12 と Store の API

| 項目 | 結果 |
|---|---|
| 範囲 | schema v12（`goal_execution_profiles`・`pipeline_runs`・`pipeline_steps`・`pipeline_recoveries`・`pipeline_ref_journal`・`pipeline_qualifications`・`pipeline_goal_completions`・`pipeline_limit_wait`）と、その Store の API。spawn はまだ無い |
| 実装のコミット | 344e99ca、776c0d4a、f19fbe22、e6bc1eeb、7f843c90 |
| レビュー | 初回は Sol（S-01〜S-05）と Opus（O-01〜O-12）の 2 系。再レビューは Opus（P-01〜P-07、Q-01〜Q-02）。確認した指摘はすべて修正し、Opus が 7f843c90 で承認した |
| macOS のホスト | fmt・clippy（workspace・all-targets・`-D warnings`）・build と、`state::` を含む lib の対象範囲が合格。B3 の HEAD f7ca57e6 でも `state::` 283 件が合格し、失敗は 0 件 |
| Linux | rust:1.91.1 のコンテナを非 root で実行（4fe287e7）。fmt・clippy が合格。テストは、Plan B が触れていない既知の不安定な 1 件（`cli::endpoint` の bind）以外が合格 |

## 変異検証

対象は 230 件。225 件でテストが落ちた（検出）。

テストが落ちなかった 5 件は次のとおり。

| 番号 | 変異 | 分類 | 根拠・対応 |
|---|---|---|---|
| 18 | `pipeline_steps` の終端の一覧から `adopted` を外し、`adopted → running` を許す | 等価 | 同じトリガーの「終端の行は、epoch を上げる引継ぎ以外では更新できない」という条件（`pipeline.sql` の `OLD.state IN ('adopted',…) AND NEW.owner_epoch=OLD.owner_epoch`）と、「引継ぎでは state を変えない」という条件が、同じ更新をなお拒否する |
| 26 | run の放棄で、生きているかもしれない step の一覧から `unknown` を外す | 別の層でも拒否 | テストは `write_run` を通すため、同じ経路にある別の検査でも拒否される。トリガーの条件だけを直接突くテストへの強化は任意として、B10（復旧）で扱う |
| 162 | rehold の API | 別の層でも拒否 | B3 の 2 巡目でテストを強化し、変異で落ちることを確認した |
| 174 | 採用元の検査 | 別の層でも拒否 | 同上 |
| 180 | journal の ref の範囲 | 別の層でも拒否 | 同上 |

## 既知の失敗（B2 と無関係）

| テスト | 状態 |
|---|---|
| `execution::native::tests::workflow_source_bootstrap_file_reader…` | main でも失敗する |
| git_owner・grok・`cli::endpoint` の一部 | 全体を負荷の高い状態で実行したときだけ失敗する。B2 は該当するファイルを変更していない |
