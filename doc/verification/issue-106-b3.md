# Issue #106 B3: CLI の pin と起動時の確認（Q）

| 項目 | 結果 |
|---|---|
| 範囲 | CLI の pin（版・`--help` の flag・ログインの状態・Mach-O の実行ファイル）、`--setting-sources` の差分の確認、起動直前の照合（`before_launch`）、Project root と Agent が書ける場所・state ディレクトリ・PATH の分離、probe の custody（macOS の EPERM は死亡を確認したときだけ受け入れる）、probe ごとの 5 秒と CLI 全体の 35 秒の上限 |
| 実装のコミット | c840a57f から 88859cbf まで |
| レビュー | 初回は Sol（T-01〜T-06）と Opus（U-01〜U-14）の 2 系。再レビューは Opus が 2 巡目（V-01〜V-10）、3 巡目（W-01〜W-05）。4 巡目で承認（X-01〜X-03 は Low・任意）。X-01 と X-02 は受け入れ条件・安全性に関わるため修正し、5 巡目で Opus が修正差分を確認し、承認した（新しい指摘なし） |
| macOS のホスト | fa888bf7 で fmt・clippy（workspace・all-targets・`-D warnings`）・build と、`state::` 283、`config::` 12、`project::` 3、`execution::owner::` 8、`execution::process::` 13、`runtime::` 200 が合格。88859cbf で fmt・clippy と `execution::pipeline::` 186 が合格 |
| Linux | rust:1.91.1 のコンテナを非 root で実行（e6545578）。fmt・clippy（`-D warnings`）が合格し、`cargo test --workspace` はすべて合格（lib 819 件、失敗 0） |

## 変異検証

ホストで 1 件ずつ、変異を入れてテストを実行し、元に戻して同じテストが合格することを確かめた。

| 対象 | 件数 | 結果 |
|---|---|---|
| B3 の変異（3e7f14e3） | 33 | 32 件を検出。FIFO の変異（`b3-q-special-files`）はテストが終わらなかったため 7 分で止めた。合格はしなかったので検出とした |
| 生き残り | 1 | `b3-sequential-probe-budget`（probe ごとの上限を CLI 全体の上限に置き換える）。e6545578 で「1 つの probe が 5 秒を超えたら拒否する」テストを足し、検出を確かめた |
| X-02 の変異（88859cbf） | 1 | 「root が無い行は状態に関係なく外す」変異を検出した |

これより前の巡（B3 の 1 巡目と 2 巡目）の変異も、それぞれの巡で検出を確かめている。

## テストの harness の制限

pipeline の owner を使う Store の fixture は、それぞれ本物の Q を実行する（テスト専用の権限は作らない）。150 件ほどが同時に Q を実行すると、harness 自体が probe の期限を超えた。そこで、fixture の Q だけを、4 permits のテスト専用の semaphore（`crates/rrx/tests/support/pipeline_cli.rs`）で絞った。テストを直列にはしておらず、本番のコード・期限・assertion も変えていない。

machine の負荷が高い（load average がおよそ 9 以上）と、probe の期限に近いテストが落ちることがある。88859cbf の 1 回目では 3 件が落ち、負荷が下がってから再実行すると 186 件すべてが合格した。

## 残件

| 項目 | 重大度 | 扱い |
|---|---|---|
| X-03: 本番の `qualify_cli` が `CLI_BOUND` を渡す 1 行の配線を、テストが通らない | Low | 別 Issue に記録する。probe ごとの 5 秒の上限は効いており、全体の上限の値は定数の検査で守っている |
| X-01 の性質（Project の root を Store の mutex を持たずに解決する）を守るテストがない | Low | 設計上の性質として記録し、X-03 と同じ Issue にまとめる |

## 既知の失敗（B3 と無関係）

| テスト | 状態 |
|---|---|
| `execution::native::tests::workflow_source_bootstrap_file_reader…` | main でも失敗する |
| `cli::endpoint` の bind | 全体を負荷の高い状態で実行したときだけ失敗する。Linux の e6545578 では合格した |
