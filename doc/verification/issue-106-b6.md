# Issue #106 B6: テストの段階、Seatbelt、受け付けた TestsProfile、試験の後片付け

| 項目 | 結果 |
|---|---|
| 範囲 | テストの段階（pipeline の custody の下の `/usr/bin/sandbox-exec` で、オーナーが受け付けた TestsProfile のコマンドを 1 つずつ実行）、`ProjectAdmitTests` と `rrx project admit-tests`、profile の hash・decode・validate・recheck、試行ごとの checkout `co/<run>-<seq>-<attempt>` と TMPDIR、採用（pass・採用した fail・held）、Goal の確認の失敗はその step を held、`secret_detected`、試験の後片付け（`pipeline_test_cleanup` の義務と sweep）、rrxd の起動時の環境（`rrx daemon start` の許可した変数だけの環境、トークンに似た名前の拒否） |
| 実装のコミット | 0011a22f から 14576029 まで（D20 の設計と実装 c3395562〜7763e21d を含む） |
| 設計の追記 | D20（環境を消された platform binary の子孫の検出、PID による leader の birth）。§12 の残余（他プロセスの引数と環境の読取り、既存の hardlink を通した書込み、cache の共有）と、段階 1 を始める条件。試験の後片付けは run の状態を変えない（B0 の遷移表はそのまま） |
| オーナーの判断 | 2026-10-20 に書面で受け入れ: 他プロセスの引数と環境を読める残余（資格情報があれば High。段階 1 の開始の条件 4 つ、専用の macOS 利用者は B13 の前の必須の条件）、checkout と TMPDIR の hardlink の残余（Low）、cache の残余（Medium、既定は使わない）、cache の全 Project 共有、managed_native の `rrx daemon start` で API key などが渡らないこと。どの推奨案も Sol（Codex）に 2 巡レビューさせてから確定した |
| レビュー | 初回は Sol（B6-S1〜S3、D20-S4）と Opus（S1〜S10）の 2 系。2 巡目は指摘した側が修正の差分だけを確かめた（Opus N1〜N5、Sol B6-S3 の残り）。3 巡目は Opus（R3-1〜R3-13）。3 巡目で同じ種類の指摘（後片付けの失敗を run の状態に結び付けたことによる）が続いたので、後片付けを状態機械から切り離す形に単純化し、その差分を Opus が 3 回確かめた（最後の 2 回は指摘なし、Low 1 件を修正）。修正は指摘していない側（主に Sol）が書いた |
| macOS のホスト | b01f4ee6 で 140 件を 1 件ずつ実行し 139 件が合格（1 件は load average 46 のときに準備で拒否され、単独の再実行で合格）。古い CLI（`RRX_LEGACY_CLI`）が要る 1 件は未実行。8d12856c で後片付けの試験 15 件が合格。14576029 で直した `b6_checkout_partial` と足した `b6_cleanup_parent_private` は、変異検証の元のコードでの確認として合格。試験の環境にはトークンに似た名前の変数を秘密でない値で置き、daemon の許可した環境を確かめた |
| Linux | rust:1.91.1 のコンテナを非 root（uid 501）で実行（8d12856c）。fmt・clippy（`-D warnings`）・`cargo test --workspace` の 32 対象がすべて合格（失敗 0） |

## ホストで見つけて直した欠陥

| 欠陥 | 影響 | 修正 |
|---|---|---|
| macOS の platform binary は `KERN_PROCARGS2` が成功するのに環境だけが消される | cookie が見えず、group を抜けた platform の子孫が残っても死亡を確認しうる。sandbox-exec の下の試験の leader の birth が読めない | D20（検出して limited、PID による birth、zombie の birth を reap の前に照合） |
| Seatbelt の profile が Mach lookup と `process*` を許していた | 試験から pasteboard などに触れられた | Mach lookup を許さず、process を fork・exec・自分の情報に絞った。ローカルのソケットと network は拒否 |
| まだ無い保護パス（Project の worktree の置き場所）を canonicalize した | すべての試験の起動が拒否された | 一番近い既存の親で解決して重なりを比べる（無いパスを飛ばさない） |
| 起動の途中の合図を待つ試験が、起動が先に終わると永久に待った | 試験が止まった | 起動の完了と合図を競わせ、先に終わったら失敗にする |
| 後片付けで試験の書ける gitfile を信用していた | 試験が後片付けを永久に失敗させられた | rrx が導出したパスだけを link を辿らずに消し、登録は gitdir が一致する管理ディレクトリだけを消す（`git worktree prune` は使わない） |
| 後片付けの失敗で run を held にした | Task と Goal が完了不能になり、B0 の遷移表が崩れた | 後片付けは run の状態を変えない。義務は別表に累積し、sweep が backoff 付きで再試行する |
| sweep が削除の間ずっと境界の lock を持ち、lock の取り合いを失敗として記録した | 採用と起動が直後に失敗し、記録が事実と違った | lock は読み書きの間だけ持ち、取り合いは記録せず次の回に回す |
| daemon が呼び出し元の環境をそのまま受け継いだ | 起動時の拒否が、トークンを環境に持つ試験の実行者の下で daemon の試験をすべて落とした | `rrx daemon start` は `env_clear` と許可した変数で `serve` を起動する（cli_pipeline は最小の集合） |

## Seatbelt で塞げないことの実測

試験の Seatbelt の profile の下から、同じ利用者の Homebrew python3 の親の環境に置いた秘密でない目印を `KERN_PROCARGS2`（MIB で読む）で取得できた。`(allow process-info* (target self))`、sysctl の名前での拒否、`kern.proc*` を除いた sysctl の名前の許可の一覧のどれでも止まらず、`sysctl-read` を外すと python3 と cargo が動かない。この残余はオーナーが受け入れた（§12）。macOS の `nc -z -U` は待ち受け中のソケットに対しても 1 を返すので、試験の対照には perl の connect を使う。

## 変異検証

最後に 1 回、レビュー用の detached worktree で実行した。各変異は、元の文がちょうど 1 回現れることを確かめてから当て、対象のテストを 1 件だけ実行し、hash と clean を確かめて元に戻した。元のコードでの確認は、試験ごとに 1 回行った。

| 対象 | 件数 | 結果 |
|---|---|---|
| B6・D20 の全巡の変異のうち今のコードに当たるもの（209）と、古くなった定義を今のコードに合わせて作り直したもの（22）。8d12856c | 231 | 219 件を検出（コンパイラで 2 件を含む） |
| 直した後に検出 | 4 | `b6-f5-pipeline-minimal-env`・`b6-f5-daemon-mode-wiring`（試験が rrx 本体を起動するので、本体をビルドし直してから再実行）、`b6-f5-existing-partial-existence-ownership`（`b6_checkout_partial` は `co` が 0755 で private の確認に先に止まり、登録の確認に届いていなかった。14576029 で直した）、`b6-f4-drop-parent-private-check`（当てていた試験は後片付けの hold と一緒に消えたので、`b6_cleanup_parent_private` を足した）。どれも 14576029 で検出 |
| 同等の変異 | 5 | `b6-d20-death-api-launch`・`-birth`（Store の検査を外しても、`pipeline.sql:140-141` の trigger が同じ条件で拒否する多層防御）、`b6-d20-he8-launch-wiring`（Agent の leader は platform binary ではないので、cookie の探索でも同じ birth が見つかる。試験の step の側の同じ形の変異 `b6-d20-test-leader-launch-wiring` は `b6_test_flow` で検出）、`b6-f1-restore-process-star`（`process-info* (target self)` は他プロセスの読取りを止めないことを実測しており、`process*` に戻しても観測できる差がない）、`b6-f6-fixture-owner-release-before-daemon`（service を動かしていない fixture の Runtime は drop だけで owner を手放す） |
| 確かめられない | 2 | `b6-f4-drop-recorded-service-validation`・`b6-f5-old-service-diagnostic`。当てた試験は古い CLI（`RRX_LEGACY_CLI`）が無いと何も確かめずに終わる |
| 試験の helper の変異 | 1 | `b6-f5-launch-wait-is-retryable`（helper が `Waiting` をやり直す処理）。sweep との取り合いが起きた実行でだけ効く。製品の性質ではない |
| 設計の変更で守る性質ごと消えたもの（後片付けの hold・`cleanup_terminal_state`・Project 全体の prune など） | 5 | 対象外 |

初めの実行では、`rrx-process-tracker` の試験を `-p rrx` で探したため、元のコードの確認が誤って失敗した。止めて直し、最初からやり直した。

## 既知の限界と残件

| ID | 内容 | 扱い |
|---|---|---|
| R3-6 | 試験が checkout の中で `chmod`・`chflags`・ACL を変えると削除できない（容量の問題だけで、run は止まらない） | Issue #108 |
| B6-L-relative | Project の設定で worktree のパスを相対にすると、登録の照合が一致しない（登録が残る側に倒れる） | Issue #108 |
| B6-L-status | Goal・run の status に後片付けの残りを表示するのは B9 | B9 の受け入れの条件 |
| B6-L-v12 | 開発中の古い v12 の DB には新しい表が無い（v12 は未公開） | 記録だけ |
| B6-L-sweep-log | 結果を書く時点で lock がふさがっているときは、次の回に回す（記録はしない） | 記録だけ |

## テストの harness の制限

- 本物のプロセス、sandbox-exec、daemon を使うテストは macOS のホストで 1 件ずつ実行した。Sol の sandbox では実行できない。
- load average が 25〜100 のとき、Store の fixture が準備で拒否されることがある。単独の実行では合格した。
- 古い CLI（`RRX_LEGACY_CLI`）が無いと、`recorded_new_cli_refuses_old_service_…` は何も確かめずに終わる。
