# Issue #106 要件定義: Plan B — CLI サブプロセスによる Agent パイプライン（Stage 1・1 Project）

Status: 提案中（B0）。オーナーの承認が必要な事項は §8 に挙げる。基準の source は origin/main `79e4d273`。この文書は「何を、なぜ作るか」を定める。「どう作るか」は `doc/design/issue-106-design.md` に書く。

## 1. 目的

既存の SQLite Store・rrxd・CLI を使い、オーナーの Mac で次の流れを人手のプロンプトなしで続けられる最小構成を作る。

- Goal に属する Task を Codex（`codex exec`）が実装する。
- テストを実行する。
- Claude（`claude -p`）が独立にレビューする。
- 確認済みの指摘だけを、承認された範囲と回数で修正し、再テストする。
- 結果を保存し、次の READY Task へ進む。

CLI を閉じても状態と実行は残る。障害時は安全側に止まり、照合して復旧する。

strict な native 経路（M0）は `feature/issue-5-m0` の U6a で一時停止している。M0 の branch には U6b まで失敗する 62 件のテストがあり、main へマージできない。Plan B は、Agent を中身を見ない非対話のサブプロセスとして扱う。安全性はプロセスの中身ではなく結果で担保する。担保の手段は次のとおり。

- worktree の分離
- CLI の sandbox flag
- 空から組み立てる環境変数
- テストとレビューによる関門
- rrx 自身が作った commit だけを採用すること
- 不確かなら Held にすること

これにより、`.rrx/planb/objective.txt` §4 の 8 項目を M0 より早く実 Mac で通す。

## 2. スコープ

- Goal ごとの実行プロファイル（`cli_pipeline` / `managed_native`）。受入時に固定する。各経路は他方の Goal を拒否する。
- パイプラインの永続状態（schema v12）と Store API。run・step・回復記録・ref journal・CLI 適格性・Goal 完了記録を含む。
- CLI の版 pin と起動時の適格性確認（Q）。
- StepLauncher。CAS で launching を記録してから spawn し、spawn 後に running を記録する。死亡を確認してから採用する。
- Executor（Codex 実装・修正）、Tests（運用者が admit した TestsProfile）、Reviewer（Claude）、Confirmer（読み取り専用の Codex）。
- 確認と修正のループ（有限回）。
- Task branch から Goal branch への release（ref-CAS journal）。
- 次 Task への自動移行と、型付き規則による Goal 完了判定。
- rrxd 再起動時の復旧。運用者が step を放棄する command（`rrx task recover --abandon-step`）と、その監査記録。
- AC-8 の障害注入マトリクス。実 Mac でオーナーが実行するチェックリストと、`doc/verification/` への記録。

## 3. 非スコープ

| 項目 | 扱い |
|---|---|
| Desktop / TUI | 後続。CLI だけで受け入れる |
| 任意 provider の連携 | 後続。v1 は Codex（Executor・Confirmer）と Claude（Reviewer）に固定する |
| Linux での完全な P4 適用・安定化 | 延期（#89 に記録済み）。パイプラインは macOS だけで有効にする。Linux では起動時に拒否する |
| 同じ Project 内での Task の並列実行 | MVP に入れない。1 Project につき 1 Active Task を維持する |
| M0 の strict 経路の完成 | 対象外。M0 のコードは変更せず、後の強化のために残す |
| 4 Project の並行受入、GitHub sandbox repo での PR・自動マージ（#100・#27・#97） | Stage 2 |
| `attach`（実行中出力のストリーミング） | 後続。v1 は `rrx goal status` / `rrx goal tasks` / `rrx events` で照会する |
| Goal branch を main へ統合すること | Stage 2（PR 経由）。v1 は Goal branch の ref を更新するところまで |
| Agent の読み取り範囲を OS で制限すること（専用 macOS ユーザー、Codex `requirements.toml` の `deny_read`） | 後続（#98）。v1 は残余リスクとして承認を求める（§8 項目 7） |

## 4. 用語

- **run**: 1 つの Task（または Goal 完了確認）に対するパイプラインの実行。1 Task に 1 件。
- **step**: run の中の 1 工程。種類は implement / tests / review / confirm / fix / goal_check。step ごとに attempt を持つ。
- **Held**: 自動では進めず、運用者の判断を待つ状態。根拠が足りないときは常にここに倒す。
- **Unknown**: 起動結果やプロセスの生死を確定できない step。再実行しない。
- **rrx commit**: rrx 自身が検査したうえで作った commit。結果として採用できる唯一のもの。

## 5. 受入条件

各 AC は、実 rrxd・SQLite・CLI と、実際の `codex` / `claude` を使って実 Mac で観測する。偽 CLI、単体テスト、合成 fixture、CI の成功だけでは合格にしない（偽 CLI は §6 の障害注入と回帰の確認にだけ使う）。証跡は `doc/verification/issue-106-acceptance.md` と関連 Issue に残す。記録するのは、対象 SHA、コマンド、結果、DB 行の抜粋（秘密・実リソース ID・生ログを除く）、ファイル digest である。

| ID | 受入条件（観測する事実） | 実 Mac での証跡 |
|---|---|---|
| AC-1 | `rrx daemon start` で rrxd が起動し、`rrx project add` で Project を登録できる。CLI 適格性（Q）の結果が qualified として保存されている。pin と異なる版や未ログインでは、パイプラインの起動を拒否する（native 経路は影響を受けない）。 | `rrx daemon status`、`rrx project list`、`pipeline_cli_qualifications` の行。版を偽った負の対照の出力 |
| AC-2 | `execution.profile = "cli_pipeline"`、Task scope、修正回数の上限、`RequiredTasksVerified` 基準を持つ Goal plan を受け入れる。受入後に Goal・Task DAG・実行プロファイルを読み戻すと同じ内容である。Human / Unverified 基準、scope の欠落、TestsProfile 未 admit の Goal は受入を拒否する。 | `rrx goal --plan` の応答、`rrx goal status` / `rrx goal tasks` の出力、`goal_execution_profiles` の行。拒否例 3 件 |
| AC-3 | Codex が Task worktree で実際にファイルを変更する。rrx が scope・`.git` 完全性・symlink・秘密・サイズを検査してから commit し、TestsProfile のテストをその commit の clean checkout で実行する。commit SHA・diff の digest・テスト結果（終了コード・出力 digest）を保存する。Codex が自分で作った commit や scope 外の変更は採用しない。 | `pipeline_steps` の implement / tests 行、rrx commit の trailer、`git show --stat`、テスト evidence の digest。scope 外変更の負の対照 |
| AC-4 | Claude が読み取り専用の snapshot で独立にレビューする。厳格な verdict JSON を runtime が decode し、その判定（approve / request_changes と各指摘）で次の step を決める。snapshot が変わった場合、schema に反する場合、model が違う場合は Held にする。 | review step の verdict 原本の digest と decode 結果、次 step の種類。不正 verdict の負の対照 |
| AC-5 | review の blocking 指摘を、規則による事前ふるい分けと読み取り専用の Codex 確認にかける。Confirmed だけが fix に渡り、修正→rrx commit→再テスト→再レビューを行う。回数は Goal で決めた上限（0〜2）を超えない。超えたら Held にする。Unclear の blocking 指摘も Held にする。Rejected は記録だけ残す。 | confirm step のラベル、fix step の入力 digest（Confirmed の ID だけ）、round の推移、上限超過の Held の例 |
| AC-6 | 承認（テスト合格・レビュー承認・Confirmed の blocking 指摘なし）の後、Task branch を Goal branch へ ref-CAS で release する。run は completed になり Project の slot が空く。DAG 上で次に READY な Task が、新しいユーザープロンプトなしに起動する。全 Task が完了し、Goal branch で TestsProfile が合格したとき、規則の評価で Goal が Completed になる。LLM は完了を判定しない。 | run・ref journal の行、Goal branch の ref の推移、次 Task の run 作成時刻（間に人の操作がない）、`pipeline_goal_completions` の行、`rrx goal status` の Completed |
| AC-7 | 実行中に CLI を終了して再接続しても、Goal・run・step を読み戻せる。rrxd を再起動すると、永続状態と所有の証跡（epoch・version・cookie・死亡確認）から照合する。そのうえで、計画済みの step は再開し、未確定のものは Held として保持する。 | 再起動前後の `rrx goal status` / 行の比較、再起動の監査イベント、再開した step の新しい epoch |
| AC-8 | 次の 7 つの障害で、二重実行も他 Project への影響も起こさずに、停止・保持・復旧・エスカレーションする。根拠が足りなければ Unknown / Held を維持する。(a) Agent の失敗（非 0 終了・出力不正・timeout）、(b) レビューの拒否（request_changes）、(c) 起動結果の不明（spawn エラー・launching 中の crash）、(d) 停止と再起動、(e) 古い owner（epoch）からの書込み、(f) CAS の競合、(g) 別 Project の継続。Unknown の起動は再実行しない。`rrx task recover --abandon-step` だけが、監査記録を残したうえで新しい attempt を作る。 | 設計書 §10 の障害注入マトリクスの各行（偽 CLI）と、実 Mac での (a)(b)(c)(d)(g) の観測。cookie を持つプロセスの残存確認、行の状態、監査イベント |

最初の 1 Project の受入（AC-1〜AC-8）を一区切りとして報告する。4 Project と既存の最終 dogfood は Stage 2 で仕上げる。CLI の部分受入と Issue #16 全体の完了は区別して記録する。

## 6. 制約

- 認証・認可・CAS・owner/epoch fencing・marker・process custody（#19）・quota を緩めない。PID や DB の terminal 表示だけで、死亡・回収・再実行可能を証明したことにしない。
- テストの skip / ignore / 直列化、timeout だけの延長、テスト専用の権限付与、本番の lock 問題を隠す retry を解決策にしない。
- 新しい spawn は `rrx_spawn_hygiene` だけを通す（`crates/rrx-spawn-hygiene/src/lib.rs:31` / `:58`）。
- SQLite の書込み区間は短くする。CLI の実行や終了待ちをトランザクションや Store lock の中に持ち込まない。
- DB 更新とプロセス起動を 1 つの原子操作とみなさない。起動結果が不明な attempt は再実行しない。
- 秘密情報（`.env`、鍵、token）・本番データ・個人情報・生ログ・実リソース ID を、外部 LLM や公開 Issue に送らない。Agent の環境変数に GH_TOKEN・SSH_AUTH_SOCK・クラウドの資格情報を入れない。
- main へ直接 push しない。新しい権限・所有・fencing の契約は、オーナーの承認（§8）を得てから実装する。
- 通常の進行はイベント駆動にする（既存の `wake` 通知と 5 秒の sweep）。高頻度の polling を足さない。新しい DB・メッセージ基盤・汎用 parser は足さない。
- 修正ラウンド中はホストで影響テストと単独の再実行だけを行い、全件実行と mutation は各 unit の最後に 1 回行う。
- runtime の試験先に rururunx 自身を使わない。
- macOS を主な検証環境とする。Linux はパイプラインを拒否し、共有コードの安全性と必須 CI は維持する。

## 7. 前提（main 上で確認済み）

| 前提 | 根拠 |
|---|---|
| Project の登録・Goal plan の受入・DAG の保存が動いている | `state/runtime/goals.rs:28`、`main.rs:487`、`main.rs:193-201` |
| 1 Project 1 Active Task と全体上限 | `config.rs:17`、`state/runtime/driver/claim.rs:375-381` |
| Runtime epoch と owner lock | `execution/owner.rs:270-300`、`state/execution.rs:537-599` |
| 本番の経路は native の `start_phase` だけで、Task Driver は最初の Executor phase の後に進まない（SC-N） | `workflow/driven_initial.rs:80-84`、`workflow/driven_settled.rs:30-51` |
| 運用者向けの Task の回復 command がない（`TaskRetry` は Unavailable を返す） | `runtime/control.rs:60-69`、`:761-766` |

## 8. オーナーの承認事項

各項目は、承認・却下・条件付き承認のどれかで回答してほしい。却下された項目は、設計を直してから再提示する。

| # | 決定事項 | 内容 | 却下された場合の影響 |
|---|---|---|---|
| 1 | Goal ごとの実行プロファイルを 2 つ目の実行権限として認める | 受入時に `cli_pipeline` か `managed_native`（既定）を固定し、変更できない。native の 3 つの関門は `cli_pipeline` の Goal を拒否し、パイプラインは `managed_native` の Goal を拒否する。容量（全体の session 数・provider ごとの同時数・1 Project 1 Active Task）は両経路の合計で数え、両方の claim 経路が同じ数え方をする。 | Plan B を実装できない |
| 2 | step の状態機械と復旧表 | 設計書 §3・§4 のとおり。spawn エラーと launching 中の crash は Unknown にし、再実行しない。Unknown・Held・Rejected を解くのは `rrx task recover --abandon-step` だけとする。この command は、運用者の申告、cookie による残存確認、worktree の base への reset を伴い、監査記録を残して新しい attempt を作る。 | 起動結果が不明なときの扱いが決まらない |
| 3 | 採用の権限 | 結果として採用するのは、rrx が検査して作った commit だけ。Agent が作った commit・ref・worktree 外のファイルは採用しない。Reviewer と Confirmer からは何も採用しない（判定の JSON だけを読む）。 | Executor の結果を受け取る根拠がなくなる |
| 4 | 承認の意味 | 承認 = テスト合格 かつ Reviewer が approve かつ Confirmed の blocking 指摘が残っていないこと。Reviewer の approve は必要条件だが十分条件ではない。修正の上限（0〜2、既定 2）と blocking の重大度（既定 high 以上）は Goal の受入時に固定する。上限を超えたら Held にする。 | 修正ループの終わり方が決まらない |
| 5 | release の ref-CAS journal | Task branch から Goal branch への反映は fast-forward だけとする。手順は journal 記録 → `git update-ref <ref> <new> <old>` → 完了記録。再起動時は、ref==new なら done、ref==old なら再実行、それ以外なら Held にする。Goal branch は `rrx/goal/<goal-id>` で、作成も同じ journal で行う。 | 次 Task の base と Goal 完了の根拠がなくなる |
| 6 | CLI の pin と flag の契約 | 引数の雛形はコードに固定する。設定できるのは、実行ファイルのパス・pin する版・model・effort・PATH の追加ディレクトリ・`CODEX_HOME`・作業ルートだけ。起動時の Q は次を確かめ、結果を DB に残す: 版が pin と一致する、必須 flag が `--help` にある、ログイン状態、実行ファイルの sha256。step の起動直前にも sha256 を再確認する。 | 実 CLI の挙動が固定されない |
| 7 | 残余リスクの受入（脅威モデル） | 設計書 §8 の残余リスクを、書面で受け入れることを求める。主なものは次の 3 つ。(i) Codex と Claude は OS ユーザーが読めるファイル（`~/.aws` など）を読み、provider へ送りうる。(ii) テストは Agent が書いたコードを、オーナーの権限で実行する。(iii) repo の内容による prompt injection。v1 の対策は次のとおり: 差分と成果物の秘密スキャン、テストの Seatbelt 包み（network 禁止・書込み範囲の限定）、データとしての枠付け、provider をまたいだ確認、テストによる関門、scope 検査。根本の対策（専用の macOS ユーザー、または Codex 管理設定の `deny_read`）は #98 で行う。 | 実 Agent を使う受入（AC-3〜AC-8）を始めない |
