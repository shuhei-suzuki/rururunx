# Issue #106 B4: StepLauncher、StepCustody、実行境界、1 process の規則

| 項目 | 結果 |
|---|---|
| 範囲 | StepLauncher（tx1・Store の lock の外での spawn・tx2）、StepCustody（並行の入出力・上限・死亡確認）、Runtime の入口と規則 G、daemon 全体の 1 process の規則（prepare・release は数えない）、native の Exhausted、daemon 全体の上限待ちと probe、macOS の process tracker の zombie・消えた PID・`reused` の分類と再列挙（設計 §3.5.1・D17）、死亡確認の探索のやり直し |
| 実装のコミット | 0af87b8f から 8bbd0bd5 まで |
| 設計の追記 | §3.5.1 と D17 は Opus が書き、Sol が 5 巡で承認した。前提 P3・P5・P6 は XNU のソースで確かめた。残余 R1（走査の間の PID の再利用）と R2（kernel 内の容量で切られた一覧）はオーナーが受け入れた。死亡確認の探索のやり直し（§3.5 の 5 の後）は orchestrator が書き、実装のレビューで確かめた |
| レビュー | 初回は Sol（B4-SOL-01〜04）と Opus（F1〜F13）の 2 系。再レビューは Opus が 2 巡目（G-01〜G-05）、3 巡目（H-01〜H-04、コードの欠陥なし）、4 巡目（H-01・H-02 の確認）。F7〜F11、F4 の残り、H-03 は後の unit の残件として Issue #108 に記録した |
| macOS のホスト | 26b3663e で fmt・clippy・build と `state::` 304、`execution::pipeline::` 216、`codex::` 124 などが合格。8bbd0bd5 までの修正の後、`execution::pipeline::` 224、`runtime::` 200、tracker 29（実機の zombie テストを含む）が合格 |
| Linux | rust:1.91.1 のコンテナを非 root で実行（26b3663e）。fmt・clippy（`-D warnings`）・`cargo test --workspace` がすべて合格 |

## ホストで見つけて直した欠陥

| 欠陥 | 影響 | 修正 |
|---|---|---|
| Q が `<work_root>/t` を既定の権限（0755）で作っていた | launcher の private ディレクトリの規則で、すべての起動が拒否された | Q と launcher で、0700 で作り、既存のものは所有者と 0700 を確かめる関数を共有した |
| 子の stdin を閉じていなかった（tokio の `shutdown` はパイプを閉じない） | prompt を EOF まで読む CLI が始まらなかった | 書込みの後に stdin を drop する |
| 同じ利用者の長く残る zombie 1 つで、macOS の探索が常に limited になった | どの step も死亡を確認できず、held になった | tracker が kernel への独立した照会で zombie・消えた PID を分ける（§3.5.1） |
| 起動や exec の途中の他のプロセスが一瞬読めず、探索が limited になった | 死亡確認が held になりやすかった | limited のときだけ、同じ期限の中で探索をやり直す |

## 変異検証

ホストで 1 件ずつ、変異を入れてテストを実行し、元に戻して同じテストが合格することを確かめた（各回 `running 1 test` と、復元後の hash・clean を確認）。

| 対象 | 件数 | 結果 |
|---|---|---|
| B4 の変異（8bbd0bd5） | 92 | 91 件を検出 |
| 生き残り | 1 | `b4-before-launch-wiring-r`（ファイルを作る前の 1 回目の `before_launch` を消す）。spawn の直前の 2 回目の照合で拒否されるので、別の層でも拒否される冗長な層とした |

`b4_before_launch_rechecks_q` が守るのは、2 か所の照合のどちらか一方が働くことまでである。設計 §3.3 が求める spawn の直前の照合（2 回目）だけを外す変更は、1 回目の照合が拒否するので、このテストでは見分けられない。照合を何もせずに spawn する状態にはならない。2 回目だけを守るテストは Issue #108 の残件とした。

後の修正でコードが変わり、元の operators が当たらなくなった 7 件は除いた。そのうち配線の 2 件は今のコードに合わせて書き直し、残る 5 件は fix2 の operators に置き換わっている。

実行の道具（レビュー用 worktree の `.rrx/planb/b4mut.py`）は、各回の出力に `running 1 test` がなければ NOTRUN と記録し、変異の後に元のファイルの hash が一致しないか作業ツリーが clean でなければ止まる。92 件とも、そのどちらにも当たらなかった。各回の出力は `.rrx/planb/b4-mut-out/<番号>/` に残している。

## 既知の失敗（B4 と無関係）

| テスト | 状態 |
|---|---|
| `adapter::grok::environment_tests::unrelated_foreign_changes_do_not_revoke_checkpoint`・`..._resume` | B4 の前（9faaabfe）でも同じく失敗する。B4 は adapter・grok を変えていない |
| `runtime::installation::tests::success::sc10_four_projects_close_independently` | 負荷の高いときだけ失敗する。単独では B4 の前も後も合格し、8bbd0bd5 までの修正の後の全体の実行でも合格した |

## テストの harness の制限

load average がおよそ 10 を超えると、期限に近いテストが落ちることがある。落ちたときは、負荷が下がってから同じ範囲を 1 回だけ実行し直して確かめた。
