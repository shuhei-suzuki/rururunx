# Issue #106 要件定義: Plan B — CLI サブプロセスによる Agent パイプライン（Stage 1・1 Project）

Status: 提案中（B0・改訂 1）。オーナーの承認が必要な事項は §8 に挙げる。基準の source は origin/main `79e4d273`。この文書は「何を、なぜ作るか」を定める。「どう作るか」は `doc/design/issue-106-design.md` に書く。ファイルパスは、断りがなければ `crates/rrx/src/` からの相対パスで書く。

## 1. 目的

既存の SQLite Store・rrxd・CLI を使い、オーナーの Mac で次の流れを人手のプロンプトなしで続けられる最小構成を作る。

- Goal に属する Task を Codex（`codex exec`）が実装する。
- テストを実行する。
- Claude（`claude -p`）が独立にレビューする。
- 確認済みの指摘だけを、承認された範囲と回数で修正し、再テストする。
- 結果を保存し、次の READY Task へ進む。

CLI を閉じても状態と実行は残る。障害時は安全側に止まり、照合して復旧する。

strict な native 経路（M0）は `feature/issue-5-m0` の U6a で一時停止している。M0 の branch には U6b まで失敗する 62 件のテストがあり、main へマージできない。Plan B は、Agent を中身を見ない非対話のサブプロセスとして扱い、安全性をプロセスの中身ではなく結果で担保する。担保の手段は次のとおり。

- worktree の分離
- CLI の sandbox flag
- 空から組み立てる環境変数
- テストとレビューによる関門
- rrx 自身が組み立て、SHA を記録した commit だけを採用すること
- 不確かなら Held にすること

これにより、`.rrx/planb/objective.txt` §4 の 8 項目を M0 より早く実 Mac で通す。

## 2. スコープ

- **インストール単位の実行モード**。runtime 設定の `[execution] mode` で、rrxd 全体を `managed_native`（既定）か `cli_pipeline` にする。`cli_pipeline` の rrxd では native の admission を呼ばず、パイプラインが唯一の実行経路になる。Goal は受け入れたモードを記録し、rrxd は自分と違うモードの Goal を受け入れも実行もしない。
- **実行境界**。パイプラインの起動・release と、停止・pause・cancel・回復の操作を、既存の `control_admission` で直列化する。
- パイプラインの永続状態（schema v12）と Store API。run・step・回復記録・ref journal・適格性・provider の制限状態・Goal 完了記録を含む。すべての書込みは、現在の runtime epoch と行ごとの version で検査する。
- CLI の版 pin と、起動時の適格性確認（Q）。
- StepLauncher と StepCustody。起動の意図を記録してから spawn し、spawn の直後に running を記録する。入出力は並行に扱い、死亡を確認してから結果を扱う。
- Executor（Codex による実装・修正）、パイプラインが所有するテスト実行（運用者が admit した TestsProfile の各コマンド）、Reviewer（Claude）、Confirmer（読み取り専用の Codex）。
- 確認と修正のループ（有限回）。
- 同時実行数の制限と、provider の利用上限への対応（待機と backoff）。
- Task branch から Goal branch への release（ref-CAS journal）。
- 次 Task への自動移行。Goal 完了確認の run と、規則による Goal 完了判定。
- rrxd 再起動時の照合と復旧。運用者が step を放棄する command（`rrx pipeline recover --abandon-step`）と、その監査記録。
- 読み取り API。`rrx goal status` / `rrx goal tasks` で、パイプラインの run・step・Held の理由・回復に必要な version・結果 commit・Goal 完了の根拠を照会できること。
- AC-8 の障害注入マトリクスと、実 Mac でオーナーが実行するチェックリスト、`doc/verification/` への記録。

## 3. 非スコープ

| 項目 | 扱い |
|---|---|
| Desktop / TUI | 後続。CLI だけで受け入れる |
| 任意 provider の連携 | 後続。v1 は Codex（Executor・Confirmer）と Claude（Reviewer）に固定する |
| 1 つの rrxd で 2 つの実行モードを同時に動かすこと | 行わない。モードの切替は rrxd の再起動で行い、native の作業が残っていれば `cli_pipeline` を起動しない |
| Linux での完全な P4 適用・安定化 | 延期（#89 に記録済み）。パイプラインは macOS だけで有効にする。Linux では Q で拒否する |
| 同じ Project 内での Task の並列実行 | MVP に入れない。1 Project につき 1 Active Task を維持する |
| M0 の strict 経路の完成 | 対象外。M0 のコードは変更せず、後の強化のために残す |
| 4 Project の並行受入、GitHub sandbox repo での PR・自動マージ（#100・#27・#97） | Stage 2 |
| `attach`（実行中出力のストリーミング） | 後続。v1 は `rrx goal status` / `rrx goal tasks` / `rrx events` で照会する |
| Goal branch を main へ統合すること | Stage 2（PR 経由）。v1 は Goal branch の ref を更新するところまで |
| Git LFS など、filter・diff・merge の driver を使う repo | v1 は拒否する（Q と採用検査で検出する） |
| Agent の読み取り範囲を OS で制限すること（専用 macOS ユーザー、Codex `requirements.toml` の `deny_read`） | 後続（#98）。v1 は残余リスクとして承認を求める（§8 項目 9） |

## 4. 用語

- **実行モード**: rrxd 全体の実行経路。`managed_native` か `cli_pipeline`。
- **run**: 1 つの Task（kind=task）、または Goal の完了確認（kind=goal）に対するパイプラインの実行。Task ごと・Goal ごとに 1 件。
- **step**: run の中の 1 工程。種類は prepare / implement / test / review / confirm / fix / release。step ごとに attempt を持つ。
- **round**: 修正の 1 サイクル（fix → test → review → 必要なら confirm）。最初の実装は round 0。
- **Held**: 自動では進めず、運用者の判断を待つ状態。根拠が足りないときは常にここに倒す。
- **Unknown**: 起動結果やプロセスの生死を確定できない step。再実行しない。
- **rrx commit**: rrx が検査した内容から filter を使わない Git plumbing で組み立て、commit SHA と tree hash を transaction に記録してから branch を進めた commit。結果として採用できる唯一のもの。

## 5. 受入条件

各 AC は、実 rrxd・SQLite・CLI と、実際の `codex` / `claude` を使って実 Mac で観測する。偽 CLI、単体テスト、合成 fixture、CI の成功だけでは合格にしない（偽 CLI は障害注入と回帰の確認に使う）。証跡は `doc/verification/issue-106-acceptance.md` と関連 Issue に残す。記録するのは、対象 SHA、コマンド、結果、DB 行の抜粋（秘密・実リソース ID・生ログを除く）、ファイル digest である。

| ID | 受入条件（観測する事実） | 実 Mac での証跡 |
|---|---|---|
| AC-1 | `[execution] mode = "cli_pipeline"` の設定で `rrx daemon start` すると rrxd が起動し、`rrx project add` で Project を登録できる。Q の 4 項目（platform・native の静止・codex・claude）が qualified として保存される。pin と異なる版、未ログイン、未完了の native 作業のいずれかがあれば、パイプラインを起動しない。 | `rrx daemon status`、`rrx project list`、`pipeline_qualifications` の行。版を偽った負の対照と、その出力 |
| AC-2 | `execution`（Task scope、修正回数の上限、blocking の閾値）と `RequiredTasksVerified` 基準を持つ Goal plan を受け入れる。受入後に Goal・Task DAG・実行設定を読み戻すと同じ内容である。次の 4 つは受入を拒否する: Human / Unverified 基準、scope の欠落、TestsProfile が未 admit、`execution` を持たない Goal（rrxd のモードと違う Goal）。 | `rrx goal --plan` の応答、`rrx goal status` / `rrx goal tasks` の出力、`goal_execution_profiles` の行。拒否例 4 件 |
| AC-3 | Codex が Task worktree で実際にファイルを変更する。rrx は scope・`.git` の完全性・symlink・秘密・サイズを検査し、検査した bytes から commit を組み立て、SHA と tree を記録してから Task branch を進める。TestsProfile の各コマンドを、その commit の clean checkout で、パイプラインの test step として実行する。commit SHA・diff の digest・各テストの結果（終了コード・出力 digest）を保存する。Codex が自分で作った commit（正しい親と trailer を偽ったものを含む）や scope 外の変更は採用しない。filter driver を宣言した repo は拒否する。 | implement / test の `pipeline_steps` の行、`git show --stat`、テスト evidence の digest。scope 外の変更・偽 trailer の commit・filter 宣言の負の対照 |
| AC-4 | Claude が読み取り専用の snapshot で独立にレビューする。厳格な verdict JSON を runtime が decode し、その判定（approve / request_changes と各指摘）で次の step を決める。snapshot が変わった場合、schema に反する場合、model が違う場合は Held にする。 | review step の verdict 原本の digest と decode 結果、次の step の種類。不正な verdict の負の対照 |
| AC-5 | blocking の指摘（`blocking=true`、または重大度が閾値以上）を、規則による事前ふるい分けと読み取り専用の Codex 確認にかける。Confirmed だけが fix に渡る。1 round は fix → rrx commit → 再テスト → 再レビューで、round の数は Goal で決めた上限（0〜2）を超えない。上限で承認に至らなければ Held。Unclear の blocking が 1 件でもあれば Held。request_changes で Confirmed の blocking がなければ Held。Rejected は記録だけ残す。 | confirm step のラベル、fix step の入力 digest（Confirmed の ID だけ）、round の推移、上限で Held になった例 |
| AC-6 | 承認（テスト合格・Reviewer の approve・Confirmed と Unclear の blocking なし）の後、Task branch を Goal branch へ ref-CAS で release する。run は completed になり Project の slot が空く。DAG 上で次に READY な Task が、新しいユーザープロンプトなしに起動する。全 Task の完了後、Goal 完了確認の run が Goal branch の head で TestsProfile を実行し、合格すると、規則の評価で Goal が Completed、その run が completed になり、同じ Project の別の Goal が進める。LLM は完了を判定しない。 | run・ref journal の行、Goal branch の ref の推移、次の run の作成時刻（間に人の操作がない）、`pipeline_goal_completions` の行、`rrx goal status` の Completed、別 Goal の run の作成 |
| AC-7 | 実行中に CLI を終了して再接続しても、`rrx goal status` / `rrx goal tasks` で Goal・run・step・Held の理由・回復に必要な version を読み戻せる。rrxd を再起動すると、新しい epoch が未完了の run を引き継ぎ、永続状態と所有の証跡（epoch・version・cookie・死亡確認）で照合する。計画済みの step は再開し、未確定のものは Held として保持する。 | 再起動前後の `rrx goal status` / 行の比較、引継ぎと再起動の監査イベント、再開した step の新しい epoch |
| AC-8 | 次の 7 つの障害で、二重実行も他 Project への影響も起こさずに、停止・保持・復旧・エスカレーションする。根拠が足りなければ Unknown / Held を維持する。(a) Agent の失敗（非 0 終了・出力不正・timeout）、(b) レビューの拒否（request_changes）、(c) 起動結果の不明（spawn エラー・launching 中の crash）、(d) 停止と再起動、(e) 古い owner（epoch）からの書込み、(f) CAS の競合、(g) 別 Project の継続。Unknown の起動は再実行しない。`rrx pipeline recover --abandon-step` だけが、監査記録を残したうえで新しい attempt を作る。 | 設計書 §14 の障害注入マトリクスの各行（偽 CLI）と、実 Mac の daemon・control 経路での (a)〜(g) すべての観測。(e) は、step 実行中の rrxd を SIGKILL して再起動し、epoch の引継ぎ、旧 custody の孤児の出力を採用しないこと、旧 epoch で組んだ control 要求の再送が拒否されることを記録する。(f) は、同じ期待 version の `rrx pipeline recover` 2 つを同時に送り片方だけが成功すること、起動中の `rrx goal pause` で pause の応答後に新しい起動がないことを記録する。cookie を持つプロセスの残存確認、行の状態、監査イベント |

最初の 1 Project の受入（AC-1〜AC-8）を一区切りとして報告する。4 Project と既存の最終 dogfood は Stage 2 で仕上げる。CLI の部分受入と Issue #16 全体の完了は区別して記録する。

## 6. 制約

- 認証・認可・CAS・owner/epoch fencing・marker・process custody（#19）・quota を緩めない。PID や DB の terminal 表示だけで、死亡・回収・再実行可能を証明したことにしない。
- パイプラインの同時実行数と provider の上限への対応は、native の既存の規則より弱くしない（設計書 §2.6）。
- テストの skip / ignore / 直列化、timeout だけの延長、テスト専用の権限付与、本番の lock 問題を隠す retry を解決策にしない。
- 新しい spawn は `rrx_spawn_hygiene` だけを通す（`crates/rrx-spawn-hygiene/src/lib.rs:31` / `:58`）。rrx 自身の Git 呼び出しも含む。
- SQLite の書込み区間は短くする。CLI の実行や終了待ちをトランザクションや Store lock の中に持ち込まない。
- DB 更新とプロセス起動を 1 つの原子操作とみなさない。起動結果が不明な attempt は再実行しない。
- 秘密情報（`.env`、鍵、token）・本番データ・個人情報・生ログ・実リソース ID を、外部 LLM や公開 Issue に送らない。Agent の環境変数に GH_TOKEN・SSH_AUTH_SOCK・クラウドの資格情報を入れない。表示・保存する抜粋と、次の Agent への prompt は、事前に秘密スキャンを通す。
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
| Runtime epoch と owner lock、書込み時の現在 owner の検査 | `execution/owner.rs:270-300`、`state/execution.rs:537-599`、`state/runtime/goals.rs:14-26` |
| 起動と制御の直列化（`control_admission`） | `runtime/admission.rs:37-41`、`runtime/control.rs:641,667,711`、`runtime/service.rs:170` |
| native の同時実行数と quota の判定 | `execution/native.rs:146-161`、`execution/quota.rs:18-20`、`state/execution/quota_policy.rs:24-83` |
| 本番の経路は native の `start_phase` だけで、Task Driver は最初の Executor phase の後に進まない（SC-N） | `workflow/driven_initial.rs:80-84`、`workflow/driven_settled.rs:30-51` |
| 運用者向けの Task の回復 command がない（`TaskRetry` は Unavailable を返す） | `runtime/control.rs:60-69`、`:761-766` |
| `rrx goal status` / `tasks` はパイプラインの情報を持たない | `runtime/control.rs:275-288,362-370`、`state/runtime/goals.rs:643-647` |

## 8. オーナーの承認事項

各項目は、承認・却下・条件付き承認のどれかで回答してほしい。却下された項目は、設計を直してから再提示する。

| # | 決定事項 | 内容 | 却下された場合の影響 |
|---|---|---|---|
| 1 | インストール単位の実行モードを 2 つ目の実行権限として認める | `[execution] mode` で rrxd 全体を `cli_pipeline` か `managed_native`（既定）にする。`cli_pipeline` では native の admission を呼ばず、パイプラインだけが実行する（native は main でもともと実行できない）。native の作業（driving の Driver・開いた Unit・開いた phase・有効な quota lease）が残っていれば、パイプラインを起動しない。Goal は受け入れたモードを記録し、rrxd は違うモードの Goal の受入・実行・resume を拒否する。2 つの経路が容量を共有しないので、合算の規則は持たない。 | Plan B を実装できない |
| 2 | 実行境界と owner fencing | パイプラインの tick は、native の admission と同じ `control_admission` を、起動意図の記録 → spawn → running の記録まで保持する。release も同じ境界の中で、Goal が Running であることを確かめてから行う。停止・pause・cancel・回復の操作も同じ境界を取るので、起動中の step を取りこぼさない。pause・cancel された Goal は release されない。パイプラインのすべての書込み API は、transaction の中で現在の epoch と行の version を検査する。trigger も、run・step・journal のすべての変更で `owner_epoch == 現在の epoch` を要求する。 | 停止と起動・release の順序が決まらない |
| 3 | step の状態機械と復旧表 | 設計書 §3・§4 のとおり。新しい epoch は起動時に、未完了の run を（Held のものも）状態を変えずに引き継ぐ。spawn エラーと launching 中の crash は Unknown にし、再実行しない。Unknown・Held・Rejected を解くのは `rrx pipeline recover --abandon-step` だけとする。この command は、運用者の申告、cookie による残存確認、作業領域の初期化を伴い、監査記録を残して新しい attempt を作る。 | 起動結果が不明なときの扱いが決まらない |
| 4 | 採用の権限 | 結果として採用するのは rrx commit だけ。rrx は、検査した bytes を filter を使わない plumbing で commit にし、commit SHA と tree hash を記録してから Task branch を進める。再起動時に採用できるのは、記録した SHA だけ。Agent が作った commit・ref・worktree 外のファイルは採用しない。filter・diff・merge の driver を宣言した repo は拒否する。Reviewer と Confirmer からは何も採用しない（判定の JSON だけを読む）。 | Executor の結果を受け取る根拠がなくなる |
| 5 | 承認の意味 | 承認 = すべてのテストが合格 かつ Reviewer が approve かつ Confirmed と Unclear の blocking 指摘がないこと。blocking = `blocking=true` または重大度が閾値以上。修正の上限（0〜2、既定 2）と閾値（既定 high）は Goal の受入時に固定する。上限 N のとき fix は最大 N 回で、N 回目の fix の後のレビューまで行う。そこで承認に至らなければ Held。request_changes で Confirmed の blocking がない場合も Held。 | 修正ループの終わり方が決まらない |
| 6 | release の ref-CAS journal | Task branch から Goal branch への反映は fast-forward だけとする。手順は journal 記録 → `git update-ref <ref> <new> <old>` → 完了記録で、実行境界の中で行う。再起動時は、ref==new なら done、ref==old なら Goal が Running のときだけ再実行（cancel なら中止、pause なら待つ）、それ以外は Held。Goal branch は `rrx/goal/<goal-id>` で、作成も同じ journal で行う。 | 次 Task の base と Goal 完了の根拠がなくなる |
| 7 | 同時実行数と provider 上限の規則 | 上限の値は native と同じ式から作る（全体は `min(global_max_sessions, 6)`、provider ごとは `agents.<name>.max_concurrent` との最小値、executor の予約枠）。判定は native と同じ純粋関数（`quota_policy::decide`）を使う。CLI の終了や出力が provider の上限を示したら、その step は失敗にせず、死亡を確認してから待機し、backoff（60 秒〜30 分）の後に新しい attempt で再開する。上限の判定が付かない失敗は通常の失敗として Held に倒す。 | 実 Agent を並べたときの上限が決まらない |
| 8 | CLI の pin と flag の契約 | 引数の雛形はコードに固定する。設定できるのは、実行ファイルのパス・pin する版・model・effort・PATH の追加ディレクトリ・`CODEX_HOME`・作業ルートだけ。起動時の Q は次を確かめ、結果を DB に残す: 版が pin と一致する、必須 flag が `--help` にある、ログイン状態、実行ファイルの sha256。step の起動直前にも sha256 を照合し直す。 | 実 CLI の挙動が固定されない |
| 9 | 残余リスクの受入（脅威モデル） | 設計書 §12 の残余リスクを、書面で受け入れることを求める。主なものは次の 3 つ。(i) Codex と Claude は OS ユーザーが読めるファイル（`~/.aws` など）を読み、provider へ送りうる。(ii) テストは Agent が書いたコードを、オーナーの権限で実行する。(iii) repo の内容による prompt injection。v1 の対策は次のとおり: すべての出力経路の秘密スキャン、テストの Seatbelt 包み（network 禁止・書込み範囲の限定）、データとしての枠付け、provider をまたいだ確認、テストによる関門、scope 検査。根本の対策（専用の macOS ユーザー、または Codex 管理設定の `deny_read`）は #98 で行う。 | 実 Agent を使う受入（AC-3〜AC-8）を始めない |
