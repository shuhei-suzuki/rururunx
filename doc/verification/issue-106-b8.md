# Issue #106 B8: 確認と修正のループ

| 項目 | 結果 |
|---|---|
| 範囲 | §8.2 の規則による絞り込み（変更したファイルだけ、レビューしたコミットの tree に完全一致するパスだけ、重大度の閾値）、Confirmer の段階（読み取り専用の `codex exec`、`rrx.confirm.v1` のデコーダ。ラベルの ID の集合が完全一致すること、証拠の検査に外れたものは Unclear）、修正の段階（Confirmed だけを prompt に入れ、`run.round` を 1 つ上げる）、§8.5 の判定。巡ごとの採用は、判定が「採用して次を計画する」か「止める」かを決めるまで、判定の対象の段階を `exited` と `round_result_validated` のまま保つ |
| Store の規則 | 巡の段階を通常の採用で終わらせない（`round_adoption_required`）、二重の staging を拒否する（`round_result_staged`）、判定の無い計画を拒否する（`round_decision_required`）。保存の大きさは 1 つの関数 `staged_body` で、staging の印を含めて測る。TestsProfile の catalog の不一致は、6 つの境界のどれでも同じ transaction の中で `tests_profile_changed` の held にする。Confirmer と修正の段階の出力に秘密情報があれば、`secret_detected` の held にする |
| 実装のコミット | 465338b1 から c84ef638 まで。共有の pre-commit hook が 3,014 行・20 ファイルのコミットで 6 分以上止まったため、実装を 4 つのコミット（465338b1、92319794、c4dce266、277e1688）に分けた |
| マスター設計書 | `doc/design/master/workflow-engine.md` の §6 と §7（465338b1） |
| レビュー | 初回は Sol（S-H1、S-M1〜S-M3）と Opus（O1〜O3）の 2 系。S-H1 と O2（判定は呼び出し側の約束でしかなく、判定なしで採用や次の計画ができる）は同じ指摘で、2/2 が指摘した。8e6c9c4b で Opus が S-H1・S-M1・S-M2・S-M3 を直し、f3e820da で Sol が O1（Confirmer と修正の出力の秘密情報が held にならない）と O3（大文字小文字だけが違うパスがスナップショットの drift として held になる）を直した。2 巡目は、Opus が O1 と O3 の修正を確かめて承認し、Sol が S-M2（Store が足す staging の印の 30 バイトが検査に入っていない）と S-M3（catalog の不一致の held が一部の境界だけ）の残りを出した。c84ef638 で Opus が直し、3 巡目の Sol が指摘なしで承認した |
| 記録する判断 | 試験だけが使う `Store::pipeline_seed_step` と `Fixture::adopt_staged` は、規則の文言（試験専用の権限を作らない）には当たる。Sol は、本番から届かず、B8 の判定・絞り込み・Confirmer・held の主な試験は本物の経路を通るので、重大な指摘には数えないと判断した。Runtime の `decide_pipeline_round` が、遅れて catalog が変わったときに held を保存した後で Err を返すことは、古い cursor が CAS で拒否されるので後へ進まないとして Sol が 3 巡目で受け入れた |
| macOS のホスト | c84ef638 で、B4〜B8 と real-CLI の harness の 215 件を 1 件ずつ実行した。fmt と clippy は合格し、210 件が合格した。落ちた 5 件のうち 4 件（`b7_executor_oversized_prompt_rejected`・`b7_prepared_generic_launch`・`b8_runtime_round_limits`・`fake_cli_post_spawn_packet`）は、load average が 9〜13 のときに単独で再実行して合格した。残る `b8_runtime_confirmation_secrets` は、使い捨ての worktree に表示を足して原因を調べた（下の「終了の確認の時間切れ」）。load average が 3〜5 のときの 10 回で 5 回が合格し、落ちた 5 回のうち 4 回は、段階が `death_unconfirmed` の held になったことを表示で確かめた。残る 1 回はテストの段階の採用で落ちたもので、理由を表示する前の実行だった。表示を足した後の同じ形の失敗は `death_unconfirmed` だった |
| Linux | rust:1.91.1 のコンテナを非 root（uid 501）で実行（c84ef638、レビュー用の clean な detached worktree を読み取り専用で mount）。fmt と clippy（`-D warnings`）は合格。`cargo test --workspace` はすべて合格（lib は 966 件合格、20 件 ignored） |

## 終了の確認の時間切れ

`b8_runtime_confirmation_secrets` は、3 つの繰り返しのそれぞれで Executor・テスト・レビュー・Confirmer の段階を起動するので、1 回の実行で段階の終了の確認を 12 回以上行う。落ちた回で表示した理由は `cookie death unconfirmed: deadline` だった。終了の確認（`execution/pipeline/custody.rs` の `confirm_death`）は、reap、process group が空になったことの確認、cookie による残りの process の探索のすべてを、合わせて 2 秒（`DEATH_BOUND`）の中で行う。このホストでは常に数百の process が動いていて、探索が時間内に完全な結果を返せないことがある。そのときは、設計どおりに証明できないものとして段階を held にする。安全の性質は保たれていて、採用や次の段階へは進まない。

上限は安全の契約の一部なので、B8 では変えない。混んだ Mac で `death_unconfirmed` の held が起きやすいことは、B10（再起動の後の再開と held の扱い）と B11（実 Mac での受け入れ）で扱う残件として記録する。

## 変異検証

レビュー用の clean な detached worktree（c84ef638）で、104 個の変異を 1 つずつ入れ、割り当てた試験を流した（`.rrx/planb/b8/mut_run.py`）。試験ごとに変えていない状態で 1 回流し、合格しなければ 1 回だけ流し直した。

| 結果 | 件数 | 内容 |
|---|---|---|
| 検出 | 91 | 割り当てた試験が落ちた |
| コンパイラで検出 | 1 | — |
| 他の試験で検出 | 2 | `b8-decoder-size`・`b8-decoder-decoded-scan` は、割り当てた `b8_decoder_schema` が拒否の例だけを見るので生き残ったが、`confirm::tests` の全体を流すと `b8_exact_label_set` と `b8_evidence_checks` が落ちた |
| 流し直しで検出 | 7 | 変えていない状態の試験（`b8_runtime_round_limits`・`b8_runtime_fix_input_guard`・`b8_runtime_confirmation_secrets`・`b8_runtime_plan_scan`）が負荷による既知の揺れで落ちた分。変えていない状態で合格するまで流し直してから変異を入れ、7 個とも、helper の揺れ（`state changed`・`death unconfirmed` など）ではない、性質を確かめる assert の行で落ちた |
| 試験の穴 | 3 | 77623d8d で試験を足し、3 つとも落ちることを確かめた（下の表） |

| 変異 | 足した試験 |
|---|---|
| `filter` で、レビューした tree にあるのにスナップショットのファイルが読めない（無い・UTF-8 でない）指摘を有効にする | `b8_filter_conditions` に、無いファイルと UTF-8 でないファイルが `MissingFile` になる検査 |
| `validate` のラベルの ID の比較を「部分集合＋件数」にする（指摘に無い ID を受け付ける） | `b8_exact_label_set` で `validate` を直接呼ぶ検査 |
| 引用の空白を除いた文字数の下限を 8 から 7 にする | `b8_evidence_quote_length_boundary`（7 文字は unclear、8 文字は confirmed） |

足した試験は Opus が承認した（Critical / High / Medium なし）。任意の Low 2 件は、既存の試験の書き方の改善で、対応しない：`b8_evidence_checks` の `"abcdefg "` の例は引用が行に無いことで unclear になっていて、文字数の境界とは別の理由で通る。`b8_exact_label_set` の `extra` の例は、F2 を `confirmed` にしているので、`decoded` 側の assert は `validate` が無くても同じ文字列で落ちる。

confirm の試験のスナップショットは、TMPDIR が正規のパスでないと `review_snapshot_changed` で落ちる（macOS の `/var` は symlink）。変異検証と host での実行では、worktree の中の正規のパスを TMPDIR にした。

## 既知の限界と残件

| ID | 内容 | 扱い |
|---|---|---|
| B8-L-seed | 試験だけが使う seed と採用の helper | 上の判断のとおり記録。B11 の障害の行列で本物の経路の試験を足すときに置き換えを検討する |
| B8-L-death | 混んだホストでは、2 秒の中で終了を証明できず、段階が `death_unconfirmed` の held になることがある | B10 と B11 の残件。held を解除できるのは今は `rrx task recover --abandon-step` だけなので、実 Mac での頻度と、後で同じ証拠を取り直せるかを B10 で決める |
| B8-L-snapshot | 作り直しが拒否されたときに、古いスナップショットが残る | 後片付けの残件 |
| B8-L-tree | 絞り込みの `ls-tree -r -z` は、`PipelineGit::run` の 4 MiB の出力上限の下にある。tree のどこかに UTF-8 でないファイル名があると、絞り込みが失敗して held になる | レビューのスナップショットと同じ上限なので受け入れ |
