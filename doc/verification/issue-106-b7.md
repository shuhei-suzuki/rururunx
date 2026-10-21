# Issue #106 B7: Reviewer の段階、VerdictDecoder、読み取り専用のスナップショット、秘密情報の走査

| 項目 | 結果 |
|---|---|
| 範囲 | Reviewer の段階（対象のコミットから rrx が作る読み取り専用のスナップショット `work_root/ro/<run>-<seq>-<attempt>`、§5.3 の固定の argv、B4 の launcher と custody の下での起動）、VerdictDecoder（result.json の上限 1 MiB、外側の envelope、strict_json、`deny_unknown_fields`、`rrx.review.v1` の schema と §8.3 の制約）、prompt・生の結果・stderr・デコードした文字列・保存する本文の秘密情報の走査、nonce による Task と diff の囲い込み、共通の launch の境界での必須のレビューの証拠、Goal が Running でないときに書き込まない規則（rule G） |
| 実装のコミット | 0abfec88 から c477238c まで |
| 設計の追記 | D21（Reviewer の argv と Q に `--no-session-persistence` を足す）。§15 の B10 の行に、一時停止と再開のレビューの受け入れ試験と、再開で `PipelineProfile.policy_sha256` を比べることを記録 |
| レビュー | 初回は Sol（S1〜S3）と Opus（O1〜O9）の 2 系。修正は e368f412（Sol）と 347cd91c（S2・S3、Opus）。2 巡目は Sol が S2・S3 の修正を確かめて承認し、Opus が O2-1〜O2-8 を出した（b0b890f2 で Sol が修正）。3 巡目の Opus は、6 件を修正済み、O2-4 を多層防御として受け入れ、O2-2 を本物の CLI での確認待ちとし、新しい Low 3 件を出した（3cb0288e で Sol が修正）。その差分の確認で Opus が Low 1 件（B7-O4-1）を出し、eaa26ecc で Sol が直し、Opus が指摘なしで承認した |
| オーケストレーターの変更 | 8c04ff70（一時停止の試験を B7 が持つ範囲に絞り、再開は B10 に回す）、c477238c（変異の検証で見つけた試験の穴を埋める 1 ケース） |
| macOS のホスト | 0abfec88 で 382 件を 1 件ずつ実行し 379 件が合格。3 件（`b5_f1_binary_tree`・`he08_platform_escape_held`・`b7_snapshot_writable_file`）は高い負荷のときに落ち、単独の再実行で合格。347cd91c で 89 件のうち 87 件が合格（`b7_line_order` は 347cd91c で消した試験、`b7_paused_boundaries_resume` は 8c04ff70 で絞った）。8c04ff70 で 34 件、3cb0288e で 8 件（`b7_runtime_review_diff` は負荷のときに落ち、単独で合格）、eaa26ecc で 5 件がすべて合格。c477238c で `b7_decoder_secrets` が合格。`safe.bareRepository=explicit` の下で git 2.54.0 が bare の形のリポジトリを拒否することを canary で確かめた |
| Linux | rust:1.91.1 のコンテナを非 root（uid 501）で実行（eaa26ecc）。fmt と clippy（`-D warnings`）は合格。`cargo test --workspace` は 1 件を除いて合格（lib は 948 件合格、20 件 ignored）。落ちた `b7_reviewer_git_environment` は、確認用のコンテナに `USER` と `LOGNAME` が無いことが原因で、同じコンテナで 2 つを設定すると合格し、外すと `missing USER` で落ちることを確かめた。macOS の実機と GitHub の Ubuntu のランナーには、どちらもある |

## 変異検証

最後に 1 回、レビュー用の detached worktree（eaa26ecc）で実行した。各変異は、元の文がちょうど 1 回現れることを確かめてから当て、対象のテストを 1 件だけ実行し、hash と clean を確かめて元に戻した。元のコードでの確認は、試験ごとに 1 回行った。

| 対象 | 件数 | 結果 |
|---|---|---|
| B7 の全巡の変異の定義 | 236 | 重複 83 件を除き、今のコードに当たる 140 件と、古くなった 13 件に分けた |
| 古くなった定義を、今のコードに合わせて Sol（読み取り専用）が作り直したもの | 12 | 当てた件数をオーケストレーターが確かめた。残る 1 件 `b7-line-order` は、行の順序の検査が B8 の filtering に移り、`b7_inverted_range_kept` が逆順を保存することを求めるので外した |
| 実行した変異 | 152 | 144 件を検出 |
| 元のコードでの確認が落ちたもの | 5 | `b7_runtime_envelope` など Runtime の試験 4 本が、load average が 50 前後のときに `review_adoption_state`（採用の時点で段階がまだ exited でない）で落ちた。load average が 20 前後のときにやり直すと、元のコードでは合格し、5 件とも検出した |
| 同等の変異 | 2 | `b7-strict-frame`（`review.rs:110` が strict_json の frame より先に同じ 1 MiB の上限で拒否する）、`b7-snapshot-tree-equality`（`runtime/pipeline/review.rs:189-192` が、準備のときに記録した `snapshot_tree` ともう一度照合する） |
| 試験の穴を直した後に検出 | 1 | `b7-decoded-envelope-scan`。生のバイト列の走査は `\uXXXX` しか戻さないので、`\t` を含む `"a=b\tEXAMPLE_SECRET=fixture"` はデコードした文字列の走査でしか見つからない。既存の入力はどれも生の走査で見つかっていた。c477238c でこのケースを足し、デコードした文字列の走査を外すと試験が落ちることを確かめた |

## 既知の限界と残件

| ID | 内容 | 扱い |
|---|---|---|
| B7-O2-2 | 固定した Claude CLI が、子の git に `GIT_CONFIG_*` を渡すか。Reviewer の PATH で `path_prefix` に git があるとき、どの git が動くか | 本物の CLI での確認で記録する |
| B7-L-model | `modelUsage` のキーの形を、本物の claude の envelope で確かめていない | 本物の CLI での確認で記録する |
| B7-L-budget | スナップショットの 100,000 項目の上限での所要時間 | 本物の CLI での確認で測る |
| B7-L-resume | 一時停止の後の再開のレビューの受け入れ試験、再開で accepted policy を比べること | B10 の受け入れの条件 |
| B7-L-order | finding の行の順序（`line` と `end_line`）の扱い | B8 の filtering |
| B7-O2-4 | 入口の検査と handler の guard の重なり | 多層防御として受け入れ |

## テストの harness の制限

- 本物のプロセス、sandbox-exec、daemon を使うテストは macOS のホストで 1 件ずつ実行した。Sol の sandbox では実行できない。
- load average が 40〜90 のとき、Runtime の fixture が採用の時点で段階の終了を確認できずに落ちることがある。単独の実行や負荷の低いときには合格した。
- Linux のコンテナで試験するときは `USER` と `LOGNAME` を設定する（Reviewer の環境の組み立てが、無いときに起動を拒むため）。
