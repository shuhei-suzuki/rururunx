# Issue #106 設計書: Plan B — CLI サブプロセスによる Agent パイプライン

Status: 承認済み（B0・改訂 3。設計レビュー 4 巡で承認、2026-10-18 にオーナーが要件 §8 の 9 項目を承認）。任意の G-01（§9 の手動 ref 変更の検知の説明）は残余リスクの記述として B9 で直す。基準の source は origin/main `79e4d273`。要件は `doc/requirements/issue-106-requirements.md`。ファイルパスは、断りがなければ `crates/rrx/src/` からの相対パスで書く。`crates/` で始まるパスはリポジトリのルートからの相対パスである。

改訂 2 では範囲を絞った。v1 は daemon 全体で process の step を同時に 1 つだけ動かし、provider ごとの枠・公平性・予約枠・probe を持たない（§2.6）。

## 0. 再利用する部品の所在（B1 の判定）

各部品が main にあるか、M0 branch（`feature/issue-5-m0`）だけにあるか、どこにもないかを確認した。

| 部品 | main | M0 だけ | 無い | 根拠と注記 |
|---|---|---|---|---|
| `GenericCliAdapter` | ✓ | | | `adapter.rs:624`。本番では使われていない（`AgentRegistry::from_config` `adapter.rs:300-328` を呼ぶのは `crates/rrx/tests/adapter.rs:321` だけ）。パイプラインはこれを使わない（§16 D5） |
| `ProcessGroup` | ✓ | | | `adapter.rs:469`。`new` は `:482`、`observe_exit` は `:503`、`reap` は `:562`、`cleanup_group` は `:1589`。native の Codex（`codex/transport.rs:23,125`、`codex/preparation.rs:295`）と Grok も使っている |
| `execution/process.rs` | ✓ | | | `OwnedProcess::spawn` は `:232`、`stop_and_reap` は `:268`、`capture_child` は `:319` |
| `execution/attempts.rs` | ✓ | | | `AttemptManager` は `:9`。Execution Unit 専用で、パイプラインは使わない |
| `git.rs` `WorktreeManager` | ✓ | | | `:96`。group が空になるのを待つ手順は `:648-662`。パイプラインは `WorktreeManager` を使わず、`PipelineGit` を持つ |
| `execution/verification/collector.rs` | ✓ | | | `collect` は `:114`。custody を値で受け取り、自分で signal と reap を行う（`:114-115,149,174`）ので、パイプラインは使わない（§7）。再利用するのは `StreamEvidence`（`:32`）と、保存・読み戻しの `retain`（`:218`）・`inspect`（`:240`）だけ |
| `execution/verification/plan.rs` | ✓ | | | `TestCommand` は `:35`、`TestsProfile` は `:50`、`hash` / `json_hash` は `:61-66`、`AdmittedProfile` の `validate` / `recheck` は `:205-223` |
| TestsProfile の読み戻し | ✓ | | | `profile`（`state/execution/verification.rs:74-84`）は body の hash と digest を照合してから decode する。起動前の `recheck` は `execution/verification/mod.rs:496` |
| `ManagedVerifier::admit_tests` | ✓ | | | `execution/verification/mod.rs:226`。呼ぶのはテストだけ |
| `verification_profiles` | ✓ | | | `state/execution/verification.sql:1-10`。PK は `project_id`、`digest` は UNIQUE。`no_update` / `no_delete` だけで `no_replace` はない |
| `state/runtime/driver/claim.rs` | ✓ | | | `claim_initial_driver` は `:351`、occupied CTE は `:375` |
| `runtime/service.rs` | ✓ | | | sweep loop は `:76-161`、shutdown は `:168-200` |
| `control_admission` | ✓ | | | `runtime/mod.rs:44`。native admission は `runtime/admission.rs:37-41` で `try_lock` と `service_running` を確かめる。制御側は `runtime/control.rs:641,667,711` で `lock().await`、shutdown は `runtime/service.rs:170` で 5 秒の timeout 付きで取る |
| owner の起動順 | ✓ | | | `RuntimeOwner::open`（`execution/owner.rs:270-279`）が lock → `Store::open` → `begin_execution_epoch` の順に行う。`begin_execution_epoch` は死亡を確かめない（`state/execution.rs:535-536`） |
| 現在の owner の検査 | ✓ | | | `owner_current`（`state/runtime/goals.rs:14-26`）、`validate_unit_authority_facts` の epoch 照合（`state/execution.rs:384-401`）。CAS の負けは `StateGuardError::SnapshotChanged`（`state/mod.rs:75-79`） |
| native の exhaustion | ✓ | | | `windows`（`state/execution/quotas.rs:476-502`）が `quota_windows` を decode し、admission が保存された Exhausted の有無を見る（`:181-189`）。`quota_windows` は provider・account・bucket ごとに 1 行で、`observe_quota` は `applies`（`state/execution/quota_observation.rs:10-22`）が認める観測でだけ行を置き換える（`quotas.rs:76-84`）。Exhausted を置き換えるのは新しい Available の観測（新しい window か、native の probe による観測）だけで、時刻の経過では消えない。書き手は native の実行経路だけ（`execution/native.rs:1523` から `quotas.rs:17,22`、`state/execution/native_phase/live_quota.rs:370`）。パイプラインは読むだけ |
| `crates/rrx-process-tracker` | ✓ | | | `discover` は `lib.rs:152`、cookie の環境変数は `lib.rs:22`、`Coverage::limited` は `lib.rs:100` |
| `crates/rrx-spawn-hygiene` | ✓ | | | `spawn_std` は `lib.rs:31`、`spawn_tokio` は `lib.rs:58`。Tokio が失敗しても子が残りうることを `lib.rs:5-6` に明記している |
| strict JSON decoder | ✓ | | | `execution/strict_json.rs:67`。重複キーと上限を検査する。未知フィールドは DTO の `deny_unknown_fields` で拒否する |
| schema version | 11 | 12 | | main は `state/mod.rs:32`、M0 は `feature/issue-5-m0:crates/rrx/src/state/mod.rs:34` |
| `runtime_startup_probes` | | ✓ | | `feature/issue-5-m0:crates/rrx/src/state/startup.sql:1`。main には無い（§16 D3） |
| `codex_wire` / `claude_wire/decoded` / `runtime/startup.rs` | | ✓ | | M0 で追加されたもの。パイプラインは使わない |

**B1 の結論**: 基準は origin/main とする。M0 だけにある部品で必須のものはない。`runtime_startup_probes` の代わりに、v12 で `pipeline_qualifications` を足す。`79e4d273` の CI が緑であることは、B1 で `gh` を使って確認し、`doc/verification/issue-106-b1.md` に記録する（この文書の作成時点では未確認）。

## 1. アーキテクチャ

### 1.1 実行モード（インストール単位）

```
rrx CLI ──(Unix control socket)──► rrxd Runtime ── sweep loop (runtime/service.rs:76-161)
                                        │
                     [execution] mode = managed_native │ cli_pipeline
                                        │
         ┌──────────────────────────────┴───────────────────────────┐
  admit_ready_tasks（既存・変更なし）                      pipeline_tick（新規）
  runtime/service.rs:130                                  PipelineScheduler → StepLauncher → StepCustody
                                                          → codex exec / テストのコマンド / claude -p
```

- `Config`（`config.rs:226`）に `execution: ExecutionConfig { mode }` と `pipeline: Option<PipelineConfig>` を足す。どちらも既定値のときは serialize しない（`#[serde(default, skip_serializing_if = …)]`）。こうすると、設定を変えない環境の policy digest（`runtime/control.rs:540-545`）は変わらない。
- sweep loop は、モードが `cli_pipeline` なら `runtime/service.rs:130` の `admit_ready_tasks` を呼ばず、`pipeline_tick` を呼ぶ。`managed_native` ならこれまでどおり。1 つの rrxd の中で、2 つの経路が同時に動くことはない。
- **Goal のモードの記録**: `cli_pipeline` の rrxd が受け入れた Goal には、`goal_execution_profiles` の行を受入の transaction の中で作る。行のない Goal は `managed_native` で受け入れたもの。
- **違うモードの Goal の拒否**:
  - `cli_pipeline` の rrxd は、`execution` を持たない Goal plan を拒否する。パイプラインの claim は、行のある Goal しか対象にしない。
  - `managed_native` の rrxd は、`execution` を持つ Goal plan を拒否する。native の候補と claim（`state/runtime/driver/candidates.rs:49`、`claim.rs:365-369`）に `NOT EXISTS(SELECT 1 FROM goal_execution_profiles WHERE goal_id=…)` を加える。モードを戻した rrxd が、パイプラインの Goal を拾わないようにするためである。
  - どちらの rrxd も、違うモードの Goal の resume を `ExecutionModeMismatch` で拒否する。
- 既存の native の 3 つの関門（`Store::managed_phase_required` `state/managed_binding/protection.rs:15`、`require_managed_native_binding_composed` `workflow.rs:539`、`AgentRegistry::from_managed_config` `adapter.rs:332`）は変えない。
- `reconcile_runtime_attention`（`state/runtime/service.rs:25,45-48`）は、パイプラインの Goal の Task に `native_binding_unavailable` を書かない。パイプラインの Task の表示は §10 の読み取りで作る。

**モードの静止の確認**（両方向。起動のたびに行う）:

`RuntimeOwner::open` にモードを渡し、owner lock（`execution/owner.rs:225`）を取って `Store::open` した後、`begin_execution_epoch`（`:278`）の**前**に `Store::mode_inventory(mode)` を行う。owner lock を持っているので、確認から epoch の commit までの間に他の書き手はいない。epoch の更新は、open フラグと lease を閉じるだけで死亡を確かめない（`state/execution.rs:535-536`）ので、静止の根拠にしない。確認は「起動しようとするモードとは**もう一方の**モードの作業」を対象にする。自分のモードの残りは §4 の引継ぎと復旧で扱う（同じモードの再起動で孤児が残る場合は、Held にして保持する）。

| 起動するモード | 静止とみなす条件（すべて満たす） |
|---|---|
| `cli_pipeline` | ・`task_drivers` の `state='driving'` が 0 件。<br>・`execution_units` の `native_effects_open=1 OR result_finalization_open=1` が 0 件。<br>・`managed_phase_operations` の `phase_open=1` が 0 件。<br>・`quota_leases` の `active=1` が 0 件。<br>・`managed_effects` の `state IN ('pending','unknown')` が 0 件。<br>・`cleanup_jobs` を持つ各 Unit の cookie を `rrx_process_tracker::discover` で探し、どれも `coverage.limited()` が false で一致が 0 件（cleanup job は完了で消えない履歴なので、今回の探索の結果で判定する。件数が 4096 を超えれば確認不能として拒否）。<br>・Exhausted の native の観測が残っていない（§2.6 の定義。時刻を見ず、provider と account を問わない）。 |
| `managed_native` | ・`pipeline_runs` の `state IN ('running','held')` が 0 件。<br>・`pipeline_ref_journal` の `state IN ('pending','held')` が 0 件。<br>・process の step のうち、`launching`・`running`・`unknown`、または `body.death_confirmed` が true でない `held` が 0 件。<br>・死亡を確認済みでない step（`held` で確認済みのものを除く、cookie を持つもの）の cookie の探索が、完全な coverage で 0 件。<br>・`pipeline_limit_wait` の行がないか、`state='clear'`（§2.6。waiting と probing は、期限を過ぎていても native に引き継げないので、切り替えない）。 |

満たさなければ、rrxd は epoch を進めずに `ExecutionModeNotQuiescent { reason }` で起動を失敗させ、何も書かない。`managed_native` の起動でパイプラインの行がなければ（既存の環境）、この確認は件数 0 で通り、挙動は変わらない。`cli_pipeline` で通った場合は、epoch E を得た後に `pipeline_qualifications` の `mode_quiescence` を qualified で書き、確認した件数と探索の要約を body に残す。native の痕跡（unknown の effect、Exhausted の観測など）が native 側で解消されない限り、その state でパイプラインは起動できない（安全側）。

### 1.2 実行境界

パイプラインは、native の admission と同じ `control_admission`（`runtime/mod.rs:44`）で、結果を進める処理と制御を直列化する。

- `pipeline_tick` は、`admit_ready_tasks` と同じく `control_admission.try_lock()` を取り、`service_running()` と `stopping` を確かめる（`runtime/admission.rs:37-41` と同じ形）。取れなければ、その sweep では何もしない。取れたら tick の終わりまで保持し、次のどれか 1 つを行う。
  - 新しい run の claim。
  - rrx 内部の step（prepare・release）の実行。Git の各操作には 10 秒の timeout を付ける。
  - 終了した step の採用（§6・§7・§8）と、次の step の計画。最後の test の採用は Goal の完了を含む（§9）。
  - 中断・上限で終わった step の次の attempt の計画（§4.2）。
  - process の step の起動。tx1（起動意図）→ spawn → tx2（running）（§3.3）。stdin の書込みと出力の読取りは guard を離してから行う（§3.4）。
- 回復の command（`PipelineRecover`）と、制御の側（`SetGoalLifecycle`、`RuntimeStop`）は、既存どおり `control_admission.lock().await` を取ってから書く（`runtime/control.rs:711`、`runtime/service.rs:170`）。
- 起動時の照合（§4.1）も、最初の sweep で `control_admission.lock().await` を取ってから行う。
- guard の保持時間は、spawn・短い transaction・timeout 付きの Git 操作・採用の検査のファイル読取りに限る。CLI の実行や終了待ちは guard の外で行う。shutdown は guard を 5 秒で取れなければ、既存どおり `ShutdownPending(Admission)` を返す。

**規則 G（step の終了後の書込み）**: 書込みを 2 種類に分ける。

| 種類 | 対象 | 規則 |
|---|---|---|
| 結果を進める書込み | committed・adopted・limited・次の step と attempt の計画・release の journal と done・Goal の完了・回復の `--then retry` | tick か回復の command の中でだけ行う。どちらも guard を保持する。最終 tx で、現在の owner、**Goal が Running であること**、run と step の version（それぞれの期待値）を確かめ直す。Goal が Running でなければ何も書かずに終える。step は exited / committed のまま残り、resume の後の tick がやり直す。cancel なら §4.3 で interrupted にする |
| 観測を記録する書込み | exited・interrupted・rejected・held・unknown、run の held / abandoned、上限待ちの記録の延長と解除（§2.6）、回復の `--then abandon-run`（journal の決着を含む。§4.4） | Goal の状態に依らない。StepCustody の監視タスクは guard を取らずに書く（shutdown が guard を持ったまま step の停止を待つので、取ると互いに待つ）。どれも採用・起動をしない。`--then abandon-run` が release 済みの journal を done にして run を completed にするのは、すでに起きた ref の変化の記録である（§9 の settle と同じ） |

このため、制御が guard を取った時点で、step は「起動していない（planned）」か「running として記録済み」か「launching から unknown にした」のどれかで、結果を進める書込みの途中ではない。pause・cancel・停止を guard の中で commit した後は、新しい起動も採用も release も Goal の完了も起こらない。launching のまま残るのは、rrxd が crash した場合だけである（§4.2 で unknown にする）。

### 1.3 新しい部品

実行側は `execution/pipeline/`、Store 側は `state/execution/pipeline.rs` と `state/execution/pipeline.sql` に置く。Store 側を `state::execution` の子に置くのは、`quotas::windows` を（`pub(super)` にして）読むためである。

| 部品 | 責務 |
|---|---|
| `PipelineScheduler` | 次を行う。<br>・tick の中で、READY な Task の run と、Goal 完了確認の run を claim する。<br>・各 step の結果から次の step を計画する（§8）。<br>・rrx 内部の step（prepare・release）を実行する。 |
| `StepLauncher` | §3.3 の手順で起動する。spawn は `rrx_spawn_hygiene::spawn_tokio` を通す。 |
| `StepCustody` | 次を行う。<br>・既存の `ProcessGroup`（`adapter.rs:469`）を包む。<br>・stdin の書込み、stdout / stderr の上限付きの読取り、終了の観測、timeout、停止要求を並行に扱う（§3.4）。<br>・group を kill し（`cleanup_group` `:1589`）、reap し（`:562`）、group が空になるのを待つ（`git.rs:648-662` と同じ方式）。<br>・cookie で探索する（`rrx_process_tracker::discover` `lib.rs:152`）。<br>agent の step と test の step は、同じ custody を使う。 |
| `PipelineGit` | rrx 自身の Git 呼び出し（§6.1）。`rrx_spawn_hygiene::spawn_tokio` で起動し、timeout と出力の上限を付ける。 |
| `AdoptionChecker` | Executor の結果を検査し、rrx commit を組み立てる（§6）。 |
| `PipelineTestRunner` | TestsProfile の各コマンドを、パイプラインの test step として実行する（§7）。 |
| `VerdictDecoder` | `strict_json::decode`（`strict_json.rs:67`）の後に、`deny_unknown_fields` の DTO へ変換し、意味の制約を検査する（§8.3）。 |
| `SecretScanner` | すべての出力経路に当てる、固定 pattern の検出（§11）。 |
| `CliQualifier` | 起動時の Q（§5.2）。 |

### 1.4 run と step の流れ

| run の kind | step の並び |
|---|---|
| task | prepare → implement → test ×N → review → (confirm) → [fix → test ×N → review → (confirm)] × 最大 `max_fix_rounds` → release |
| goal | test ×N（Goal branch の head で TestsProfile の全コマンド）→ 最後の test の採用の tx で Goal を完了 |

| step | 入力 | 実行 | 出力として採用するもの |
|---|---|---|---|
| prepare（rrx 内部） | run の base と Goal branch | `PipelineGit` が Goal branch の作成（初回だけ）・Task branch・Task worktree を作る | worktree の検証結果 |
| implement | Task の title・受入条件・scope（運用者が書いたもの）と Task worktree | `codex exec --sandbox workspace-write`（cwd は Task worktree） | rrx commit（§6）。Codex の `last.md` は表示用のデータに留める |
| test | 検査対象の commit と、admit 済みの TestsProfile の 1 コマンド | commit の clean checkout の中で、Seatbelt に包んで実行する（§7） | 合否、終了コード、出力の sha256 |
| review | base..head の diff と、読み取り専用の snapshot | `claude -p --permission-mode plan`（cwd は snapshot） | 検査を通った verdict JSON |
| confirm | ふるい分けを通った blocking 指摘と、別の snapshot | `codex exec --sandbox read-only` | 検査を通ったラベル JSON |
| fix | Confirmed の指摘だけ（またはテスト失敗の要約）と Task worktree | implement と同じ | rrx commit |
| release（rrx 内部） | 承認された run | `PipelineGit` が `update-ref` を行う | ref journal の done |

## 2. Schema v12

### 2.1 版上げの経路

main は v11（`state/mod.rs:32`）。v12 を足す（計画にある「v13」との違いは §16 D1）。

**方針**: パイプラインの表は、既存の `MUTABLE_TABLES`（`state/execution.rs:7-53`）に**加えない**。別の定数 `PIPELINE_TABLES` を持ち、表・trigger・writer guard を `state::execution::pipeline::install_schema` だけで作る。理由は次のとおり。

- `install_writer_guards_at`（`state/execution.rs:129-143`）は、`MUTABLE_TABLES` のすべてに trigger を作る。v5〜v11 の版上げの途中や v10 の参照 layout で呼ばれるので、新しい表を加えると、表がまだない時点で trigger の作成が失敗する。
- `validate_actual9`（`state/managed_binding/schema.rs:142-216`）は、`MUTABLE_TABLES` から過去の版の writer guard の一覧を作る。加えると、過去版の照合が変わってしまう。

`pipeline::install_schema` は、表・各表の trigger・`writer_<table>_{INSERT,UPDATE,DELETE}`（`rrx_writer_contract_version()<>12`、既存と同じ形）を作る。どの経路でも、既存の guard を v12 にした**後**で、最後に 1 回だけ呼ぶ。経路ごとの手順は次のとおり。各経路は既存どおり 1 つの `BEGIN IMMEDIATE` の中で行い、最後に `validate_current_layout` を通してから commit する（`state/mod.rs:306-307`）。

| 元の版 | 手順（`state/mod.rs`） |
|---|---|
| 0（新規） | `:226-230` の後に `pipeline::install_schema`。`user_version=12` |
| 11 | 新しい分岐。`validate_layout(tx, 11)` で v11 の layout と完全に一致することを先に確かめる。続いて `execution::install_writer_guards` → `pipeline::install_schema` → `user_version=12`。legacy の名前空間の検査（`:252`）は通さない |
| 10 | 既存の分岐（`:233-243`）。`validate_exact_v10_layout` → `install_read_indexes` → `execution::install_writer_guards` → `pipeline::install_schema` → `user_version=12` |
| 1〜9 | 既存の版上げの loop（`:271-293`）。`matches!(next, 7..=11)` を `7..=12` にし、`next == 12` で guard の後に `pipeline::install_schema` |

`validate_layout(c, version)`（`state/managed_binding/schema.rs:257-281`）の参照 layout を、版で次のように組み立てる。

- `version >= 11` なら `install_read_indexes`
- `version != SCHEMA_VERSION` なら `install_writer_guards_at(version)`（`MUTABLE_TABLES` だけ）
- `version >= 12` なら `pipeline::install_schema`

v10 の参照（`MUTABLE_TABLES` の guard を版 10 で作り、読み取り index とパイプラインの表なし）と actual9 の照合は、今と同じものになる。定数 `READ_INDEX_LAYOUT_VERSION = 11` を足し、`BINDING_LAYOUT_VERSION = 10`（`state/mod.rs:34`）は変えない。

### 2.2 表（`state/execution/pipeline.sql`）

`PIPELINE_TABLES` は次の 8 表である。各表に共通の trigger（epoch・置換・削除）は §2.3 にまとめる。ここには表ごとの列と遷移だけを書く。

```sql
-- Goal のモードの記録。行がない Goal は managed_native。受入の tx で goal_authority と同時に作る。不変。
CREATE TABLE goal_execution_profiles (
 goal_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL,
 profile TEXT NOT NULL CHECK(profile='cli_pipeline'),
 policy_sha256 TEXT NOT NULL CHECK(length(policy_sha256)=64 AND policy_sha256 NOT GLOB '*[^0-9a-f]*'),
 tests_profile_digest TEXT NOT NULL CHECK(length(tests_profile_digest)=64),
 accepted_epoch INTEGER NOT NULL CHECK(typeof(accepted_epoch)='integer' AND accepted_epoch>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id),
 FOREIGN KEY(project_id,tests_profile_digest) REFERENCES verification_profiles(project_id,digest)
);

CREATE TABLE pipeline_runs (
 id TEXT PRIMARY KEY NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('task','goal')),
 task_id TEXT, goal_id TEXT NOT NULL, project_id TEXT NOT NULL,
 owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
 version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
 state TEXT NOT NULL CHECK(state IN ('running','held','completed','abandoned')),
 round INTEGER NOT NULL CHECK(round BETWEEN 0 AND 2),
 base_commit TEXT NOT NULL CHECK(length(base_commit) IN (40,64)),
 head_commit TEXT NOT NULL CHECK(length(head_commit) IN (40,64)),   -- 最後に採用した rrx commit。初期値は base
 worktree TEXT UNIQUE, branch TEXT UNIQUE,
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=131072      -- held の理由、limit {waits}（この run の上限待ちの回数）
   AND json_extract(body,'$.id') IS id AND json_extract(body,'$.version') IS version
   AND json_extract(body,'$.state') IS state AND json_extract(body,'$.epoch') IS owner_epoch),
 CHECK((kind='task' AND task_id IS NOT NULL AND worktree IS NOT NULL AND branch IS NOT NULL)
    OR (kind='goal' AND task_id IS NULL AND worktree IS NULL AND branch IS NULL)),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
CREATE UNIQUE INDEX pipeline_run_per_task ON pipeline_runs(task_id) WHERE kind='task';
CREATE UNIQUE INDEX pipeline_run_per_goal ON pipeline_runs(goal_id) WHERE kind='goal';
CREATE TRIGGER pipeline_runs_insert BEFORE INSERT ON pipeline_runs
WHEN NEW.version<>1 OR NEW.state<>'running' OR NEW.round<>0 OR NEW.head_commit<>NEW.base_commit
BEGIN SELECT RAISE(ABORT,'invalid pipeline run'); END;
CREATE TRIGGER pipeline_runs_transition BEFORE UPDATE ON pipeline_runs
WHEN OLD.id<>NEW.id OR OLD.kind<>NEW.kind OR OLD.task_id IS NOT NEW.task_id OR OLD.goal_id<>NEW.goal_id
 OR OLD.project_id<>NEW.project_id OR OLD.base_commit<>NEW.base_commit
 OR OLD.worktree IS NOT NEW.worktree OR OLD.branch IS NOT NEW.branch
 OR NEW.version<>OLD.version+1
 OR NEW.owner_epoch<OLD.owner_epoch
 -- 引継ぎ（epoch を上げる更新）は owner だけを変える
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.round<>OLD.round OR NEW.head_commit<>OLD.head_commit))
 OR NEW.round<OLD.round OR NEW.round>OLD.round+1
 OR NOT (NEW.state=OLD.state
   OR (OLD.state='running' AND NEW.state IN ('held','completed','abandoned'))
   OR (OLD.state='held' AND NEW.state IN ('running','abandoned','completed')))
 -- held から running・completed へは、回復の記録があるときだけ（completed は release 済みの journal を決着させた abandon-run。§4.4）
 OR (OLD.state='held' AND NEW.state IN ('running','completed') AND NOT EXISTS(
     SELECT 1 FROM pipeline_recoveries r WHERE r.run_id=NEW.id AND r.consumed_version=NEW.version))
 -- 放棄と held からの完了は、生きているかもしれない step が 1 つもないときだけ
 OR (((NEW.state='abandoned' AND OLD.state<>'abandoned') OR (OLD.state='held' AND NEW.state='completed')) AND EXISTS(
     SELECT 1 FROM pipeline_steps s WHERE s.run_id=NEW.id
      AND (s.state IN ('planned','launching','running','exited','committed','unknown')
       OR (s.state='held' AND json_extract(s.body,'$.death_confirmed') IS NOT 1))))
 -- 放棄は、その run の開いた release の journal がないときだけ（先に決着させる。§4.4・§9）
 OR (NEW.state='abandoned' AND OLD.state<>'abandoned' AND EXISTS(
     SELECT 1 FROM pipeline_ref_journal j WHERE j.run_id=NEW.id AND j.state IN ('pending','held')))
 -- 完了は、kind ごとの完了の証拠が先に同じ tx で書かれていること
 OR (NEW.state='completed' AND OLD.state<>'completed' AND NOT (
     (NEW.kind='task' AND EXISTS(SELECT 1 FROM pipeline_ref_journal j
        WHERE j.run_id=NEW.id AND j.kind='task_release' AND j.state='done'))
  OR (NEW.kind='goal' AND EXISTS(SELECT 1 FROM pipeline_goal_completions c WHERE c.goal_check_run=NEW.id))))
BEGIN SELECT RAISE(ABORT,'invalid pipeline run transition'); END;

CREATE TABLE pipeline_steps (
 run_id TEXT NOT NULL REFERENCES pipeline_runs(id),
 seq INTEGER NOT NULL CHECK(seq BETWEEN 1 AND 256), attempt INTEGER NOT NULL CHECK(attempt BETWEEN 1 AND 16),
 kind TEXT NOT NULL CHECK(kind IN ('prepare','implement','test','review','confirm','fix','release')),
 -- IS は NULL を返さないので、Agent の kind で provider が NULL の行は CHECK で拒否される
 provider TEXT CHECK((kind IN ('implement','fix','confirm') AND provider IS 'codex')
                  OR (kind='review' AND provider IS 'claude')
                  OR (kind IN ('prepare','test','release') AND provider IS NULL)),
 owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),  -- この行を最後に書いた epoch
 round INTEGER NOT NULL CHECK(round BETWEEN 0 AND 2),
 state TEXT NOT NULL CHECK(state IN ('planned','launching','running','exited','committed','adopted',
                                     'rejected','limited','interrupted','unknown','held','abandoned')),
 version INTEGER NOT NULL CHECK(version>0),
 cookie TEXT CHECK(cookie IS NULL OR length(cookie) BETWEEN 16 AND 128),
 pid INTEGER CHECK(pid IS NULL OR pid>1), birth TEXT,
 step_base TEXT NOT NULL CHECK(length(step_base) IN (40,64)),   -- Executor は step 前の HEAD、test / review / confirm は対象の commit
 input_sha256 TEXT NOT NULL CHECK(length(input_sha256)=64),
 result_commit TEXT CHECK(result_commit IS NULL OR length(result_commit) IN (40,64)),
 result_tree TEXT CHECK(result_tree IS NULL OR length(result_tree) IN (40,64)),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=262144
   AND json_extract(body,'$.version') IS version AND json_extract(body,'$.state') IS state
   AND json_extract(body,'$.epoch') IS owner_epoch),     -- Agent の kind は exited で provider_outcome（success / limit / other。§5.6）
 PRIMARY KEY(run_id,seq,attempt)
);
-- run ごとに、未終端の step は同時に 1 つまで
CREATE UNIQUE INDEX pipeline_step_one_open ON pipeline_steps(run_id)
 WHERE state IN ('planned','launching','running','exited','committed');
CREATE TRIGGER pipeline_steps_insert BEFORE INSERT ON pipeline_steps
WHEN NEW.state<>'planned' OR NEW.version<>1 OR NEW.cookie IS NOT NULL OR NEW.pid IS NOT NULL
 OR NEW.result_commit IS NOT NULL OR NEW.result_tree IS NOT NULL
 OR (SELECT state FROM pipeline_runs WHERE id=NEW.run_id) IS NOT 'running'
 -- 同じ seq の次の attempt は、前の attempt が interrupted / limited / abandoned のときだけ
 OR (NEW.attempt>1 AND NOT EXISTS(SELECT 1 FROM pipeline_steps p WHERE p.run_id=NEW.run_id AND p.seq=NEW.seq
      AND p.attempt=NEW.attempt-1 AND p.state IN ('interrupted','limited','abandoned')))
 -- 次の seq は、前の seq のどれかの attempt が adopted のときだけ
 OR (NEW.attempt=1 AND NEW.seq>1 AND NOT EXISTS(SELECT 1 FROM pipeline_steps p WHERE p.run_id=NEW.run_id
      AND p.seq=NEW.seq-1 AND p.state='adopted'))
BEGIN SELECT RAISE(ABORT,'invalid pipeline step insert'); END;
CREATE TRIGGER pipeline_steps_transition BEFORE UPDATE ON pipeline_steps
WHEN OLD.run_id<>NEW.run_id OR OLD.seq<>NEW.seq OR OLD.attempt<>NEW.attempt OR OLD.kind<>NEW.kind
 OR OLD.provider IS NOT NEW.provider OR OLD.round<>NEW.round OR OLD.step_base<>NEW.step_base
 OR OLD.input_sha256<>NEW.input_sha256 OR NEW.version<>OLD.version+1
 OR NEW.owner_epoch<OLD.owner_epoch
 -- 引継ぎは owner だけを変える
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.cookie IS NOT OLD.cookie
     OR NEW.pid IS NOT OLD.pid OR NEW.result_commit IS NOT OLD.result_commit))
 OR (OLD.cookie IS NOT NULL AND OLD.cookie IS NOT NEW.cookie)
 OR (OLD.cookie IS NULL AND NEW.cookie IS NOT NULL AND NOT (OLD.state='planned' AND NEW.state='launching'))
 OR (OLD.pid IS NOT NULL AND (OLD.pid IS NOT NEW.pid OR OLD.birth IS NOT NEW.birth))
 OR (OLD.pid IS NULL AND NEW.pid IS NOT NULL AND NOT (OLD.state='launching' AND NEW.state='running'))
 OR (OLD.result_commit IS NOT NULL AND (OLD.result_commit IS NOT NEW.result_commit OR OLD.result_tree IS NOT NEW.result_tree))
 OR (OLD.result_commit IS NULL AND NEW.result_commit IS NOT NULL
     AND NOT (OLD.state='exited' AND NEW.state='committed' AND NEW.result_tree IS NOT NULL))
 -- provider_outcome は一度書いたら変えない（上限待ちの解除の根拠。§2.6）
 OR (json_extract(OLD.body,'$.provider_outcome') IS NOT NULL
     AND json_extract(NEW.body,'$.provider_outcome') IS NOT json_extract(OLD.body,'$.provider_outcome'))
 -- daemon 全体で、生きているかもしれない process の step は 1 つまで（§2.6）
 OR (OLD.state='planned' AND NEW.state='launching' AND EXISTS(SELECT 1 FROM pipeline_steps o
     WHERE NOT (o.run_id=NEW.run_id AND o.seq=NEW.seq AND o.attempt=NEW.attempt)
      AND (o.state IN ('launching','running','unknown')
       OR (o.state='held' AND json_extract(o.body,'$.death_confirmed') IS NOT 1))))
 -- Agent の step は、daemon 全体の上限待ちが clear か、この step が probe のときだけ起動する（§2.6）
 OR (OLD.state='planned' AND NEW.state='launching' AND NEW.kind IN ('implement','fix','review','confirm') AND EXISTS(
     SELECT 1 FROM pipeline_limit_wait w WHERE w.state<>'clear'
      AND NOT (w.state='probing' AND w.probe_run=NEW.run_id AND w.probe_seq=NEW.seq AND w.probe_attempt=NEW.attempt)))
 OR NOT (NEW.state=OLD.state
   OR (OLD.state='planned'   AND NEW.state='launching' AND NEW.kind NOT IN ('prepare','release'))
   OR (OLD.state='planned'   AND NEW.state='running'   AND NEW.kind IN ('prepare','release'))
   OR (OLD.state='planned'   AND NEW.state IN ('held','abandoned'))
   OR (OLD.state='launching' AND NEW.state IN ('running','unknown','exited','rejected'))
   OR (OLD.state='running'   AND NEW.state IN ('exited','interrupted','held','unknown') AND NEW.kind NOT IN ('prepare','release'))
   OR (OLD.state='running'   AND NEW.state IN ('adopted','interrupted','held') AND NEW.kind IN ('prepare','release'))
   OR (OLD.state='exited'    AND NEW.state='committed' AND NEW.kind IN ('implement','fix'))
   OR (OLD.state='exited'    AND NEW.state IN ('rejected','held','interrupted'))
   OR (OLD.state='exited'    AND NEW.state='limited' AND NEW.kind IN ('implement','fix','review','confirm'))
   OR (OLD.state='exited'    AND NEW.state='adopted' AND NEW.kind NOT IN ('implement','fix'))
   OR (OLD.state='committed' AND NEW.state IN ('adopted','held','interrupted'))
   OR (OLD.state IN ('unknown','held','rejected') AND NEW.state='abandoned'
       AND EXISTS(SELECT 1 FROM pipeline_recoveries r WHERE r.run_id=NEW.run_id AND r.seq=NEW.seq AND r.attempt=NEW.attempt)))
 OR (OLD.state IN ('adopted','interrupted','limited','abandoned') AND NEW.state<>OLD.state)
BEGIN SELECT RAISE(ABORT,'invalid pipeline step transition'); END;

CREATE TABLE pipeline_recoveries (            -- 運用者の放棄記録。追記だけ
 id TEXT PRIMARY KEY NOT NULL, run_id TEXT NOT NULL, seq INTEGER NOT NULL, attempt INTEGER NOT NULL,
 request_id TEXT NOT NULL UNIQUE REFERENCES runtime_control_acks(request_id),  -- ack を同じ tx で先に書く（§4.4）
 ingress_uid INTEGER NOT NULL, accepted_epoch INTEGER NOT NULL CHECK(accepted_epoch>0),
 then_action TEXT NOT NULL CHECK(then_action IN ('retry','abandon_run')),
 consumed_version INTEGER NOT NULL,           -- 回復で更新した後の run の version
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),  -- journal の決着（aborted / done と観測した ref）を含む
 FOREIGN KEY(run_id,seq,attempt) REFERENCES pipeline_steps(run_id,seq,attempt)
);

CREATE TABLE pipeline_ref_journal (           -- Goal branch の作成と release
 id TEXT PRIMARY KEY NOT NULL, goal_id TEXT NOT NULL, project_id TEXT NOT NULL,
 run_id TEXT REFERENCES pipeline_runs(id),
 kind TEXT NOT NULL CHECK(kind IN ('goal_create','task_release')),
 ref_name TEXT NOT NULL, old_commit TEXT NOT NULL, new_commit TEXT NOT NULL,
 owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
 state TEXT NOT NULL CHECK(state IN ('pending','done','held','aborted')),
 version INTEGER NOT NULL CHECK(version>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=16384),
 CHECK((kind='goal_create' AND run_id IS NULL) OR (kind='task_release' AND run_id IS NOT NULL)),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
CREATE UNIQUE INDEX pipeline_ref_one_create ON pipeline_ref_journal(goal_id) WHERE kind='goal_create';
CREATE UNIQUE INDEX pipeline_ref_one_release ON pipeline_ref_journal(run_id) WHERE kind='task_release';
CREATE UNIQUE INDEX pipeline_ref_one_open ON pipeline_ref_journal(ref_name) WHERE state IN ('pending','held');
-- transition: version+1、identity（id・goal・run・kind・ref・old・new）は不変、
-- 引継ぎは owner_epoch だけを変える、pending→done|held|aborted、held→pending|done|aborted、done と aborted は終端。
-- task_release の行は、run が abandoned なら変更できない（`(SELECT state FROM pipeline_runs WHERE id=NEW.run_id) IS 'abandoned'` で abort。放棄の前に決着させる。§4.4）

CREATE TABLE pipeline_qualifications (        -- Q の結果。epoch ごと。不変
 owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
 subject TEXT NOT NULL CHECK(subject IN ('platform','mode_quiescence','codex','claude')),
 state TEXT NOT NULL CHECK(state IN ('qualified','refused')),
 executable_sha256 TEXT, version_text TEXT,
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=16384),
 PRIMARY KEY(owner_epoch,subject)
);

CREATE TABLE pipeline_goal_completions (      -- 完了証明。不変
 goal_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL,
 goal_branch_head TEXT NOT NULL, goal_check_run TEXT NOT NULL UNIQUE REFERENCES pipeline_runs(id),
 runs_sha256 TEXT NOT NULL CHECK(length(runs_sha256)=64),
 completed_epoch INTEGER NOT NULL CHECK(completed_epoch>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);

-- daemon 全体の、既知の provider の上限の待ち（§2.6）。1 行だけ。run に属さない。
-- v1 は account を対応付けないので、provider・Project・run を区別しない。時刻はミリ秒
CREATE TABLE pipeline_limit_wait (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
 version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
 state TEXT NOT NULL CHECK(state IN ('waiting','probing','clear')),
 not_before INTEGER NOT NULL CHECK(typeof(not_before)='integer' AND not_before>0),
 backoff_ms INTEGER NOT NULL CHECK(backoff_ms BETWEEN 60000 AND 1800000),   -- 次の延長の幅
 probe_run TEXT, probe_seq INTEGER, probe_attempt INTEGER,                    -- probing のときの probe の step
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=16384   -- 最後に延長・解除した step の参照と理由
   AND json_extract(body,'$.version') IS version AND json_extract(body,'$.state') IS state
   AND json_extract(body,'$.epoch') IS owner_epoch),
 CHECK((state='probing') = (probe_run IS NOT NULL AND probe_seq IS NOT NULL AND probe_attempt IS NOT NULL)),
 CHECK(state='probing' OR (probe_run IS NULL AND probe_seq IS NULL AND probe_attempt IS NULL)),
 FOREIGN KEY(probe_run,probe_seq,probe_attempt) REFERENCES pipeline_steps(run_id,seq,attempt)
);
-- 最初の上限で作る（1 回目の待ちは 60 秒なので、作った時点の次の幅は 120 秒）
CREATE TRIGGER pipeline_limit_wait_insert BEFORE INSERT ON pipeline_limit_wait
WHEN NEW.version<>1 OR NEW.state<>'waiting' OR NEW.backoff_ms<>120000
BEGIN SELECT RAISE(ABORT,'invalid pipeline limit wait'); END;
CREATE TRIGGER pipeline_limit_wait_transition BEFORE UPDATE ON pipeline_limit_wait
WHEN NEW.version<>OLD.version+1 OR NEW.owner_epoch<OLD.owner_epoch
 -- 期限は短くならない
 OR NEW.not_before<OLD.not_before
 -- 引継ぎは owner だけを変える
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.not_before<>OLD.not_before
     OR NEW.backoff_ms<>OLD.backoff_ms OR NEW.probe_run IS NOT OLD.probe_run
     OR NEW.probe_seq IS NOT OLD.probe_seq OR NEW.probe_attempt IS NOT OLD.probe_attempt))
 OR NOT (NEW.owner_epoch>OLD.owner_epoch
   -- 延長: どの state からでも waiting へ。幅は倍（上限 30 分）
   OR (NEW.state='waiting' AND NEW.backoff_ms=MIN(OLD.backoff_ms*2,1800000))
   -- 期限の後の最初の起動（期限の到来は API が tx1 で確かめる）
   OR (OLD.state='waiting' AND NEW.state='probing'
       AND NEW.not_before=OLD.not_before AND NEW.backoff_ms=OLD.backoff_ms)
   -- 解除: probe の step が終わり、既知の成功に分類されたときだけ。waiting から clear へは移れない（時刻だけでは解除しない）
   OR (OLD.state='probing' AND NEW.state='clear' AND NEW.backoff_ms=60000 AND NEW.not_before=OLD.not_before
       AND EXISTS(SELECT 1 FROM pipeline_steps s WHERE s.run_id=OLD.probe_run AND s.seq=OLD.probe_seq
        AND s.attempt=OLD.probe_attempt AND s.state NOT IN ('planned','launching','running')
        AND json_extract(s.body,'$.provider_outcome') IS 'success')))
BEGIN SELECT RAISE(ABORT,'invalid pipeline limit wait transition'); END;
```

### 2.3 全表に共通の trigger 規則

`PIPELINE_TABLES` の 8 表すべてに、次を `pipeline::install_schema` で作る。

| 規則 | 対象 | 内容 |
|---|---|---|
| R-epoch（行の owner） | `pipeline_runs`、`pipeline_steps`、`pipeline_ref_journal`、`pipeline_limit_wait` | INSERT と UPDATE で、`NEW.owner_epoch IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1)` なら abort する。**書込み API は、すべての UPDATE で `owner_epoch` に自分の epoch を明示して書く**（変えない列として省略しない）。そのため、引継ぎの後に古い epoch の書き手が行を更新すると、NEW.owner_epoch が古い値になり、trigger だけで拒否される |
| R-epoch（run の owner） | `pipeline_steps`、`pipeline_recoveries` | INSERT と UPDATE で、`(SELECT owner_epoch FROM pipeline_runs WHERE id=NEW.run_id) IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1)` なら abort する |
| R-epoch（記録の epoch） | `goal_execution_profiles`、`pipeline_qualifications`、`pipeline_goal_completions`、`pipeline_recoveries` | INSERT で、`accepted_epoch` / `owner_epoch` / `completed_epoch` が現在の epoch と違えば abort する |
| R-immutable | `goal_execution_profiles`、`pipeline_recoveries`、`pipeline_qualifications`、`pipeline_goal_completions` | `no_update` |
| R-replace | 8 表すべて | `no_replace`: `BEFORE INSERT WHEN EXISTS(…)` の条件を、主キーと、すべての UNIQUE 制約・部分 UNIQUE index の衝突条件の OR にする。例: `pipeline_steps` は `(run_id,seq,attempt)` の一致、または「NEW が未終端の state で、同じ run に未終端の step がある」。`pipeline_runs` は id・`task_id`（kind=task）・`goal_id`（kind=goal）・worktree・branch。`pipeline_ref_journal` は id・`goal_create` の goal・`task_release` の run・開いた ref_name。`pipeline_limit_wait` は singleton の行があること。既存の `goal_authority_no_replace`・`task_drivers_no_replace`（`state/runtime/schema.sql:15-17,99-101`）と同じ形で、`recursive_triggers` が off でも `INSERT OR REPLACE` を拒否する |
| R-delete | 8 表すべて | `no_delete` |
| writer guard | 8 表すべて | `writer_<table>_{INSERT,UPDATE,DELETE}`（`rrx_writer_contract_version()<>12`） |

**参照する TestsProfile の保護**: 既存の `verification_profiles` には `no_replace` がない（`state/execution/verification.sql:1-10`）。FK の検査が有効な接続では、`goal_execution_profiles` の FK `(project_id, tests_profile_digest)` があるので、digest を変える REPLACE は参照先が消えて失敗する。しかし、同じ `project_id`・`digest` のまま body だけを変える REPLACE は、文の終わりに参照先が存在するので FK では防げない。そこでパイプラインは、TestsProfile を読むたびに body と digest を照合する（§7.2）。不一致なら採用も起動もせず held にする。既存の表への `no_replace` の追加は別 Issue に記録する（§13.3）。

### 2.4 Store の書込み API の規則

パイプラインの Store の変更は、すべて `state/execution/pipeline.rs` の API を通す。各 API は次を守る。

1. `BEGIN IMMEDIATE` の短い transaction で行う。Git・プロセス・ファイルの I/O を中に持ち込まない。
2. 最初に `pipeline_owner_current(tx, owner)` を呼ぶ。`owner` は `RuntimeOwner` の instance と epoch。runtime_epoch の行と一致しなければ拒否する（`owner_current` `state/runtime/goals.rs:14-26` と同じ形）。
3. 対象の run の `owner_epoch` が現在の epoch と一致することを確かめる（`validate_unit_authority_facts` の epoch 照合 `state/execution.rs:393-401` と同じ考え方）。
4. 変更する行ごとに、呼び出し側が観測した version を渡す。`UPDATE … SET owner_epoch=?<自分の epoch>, … WHERE <主キー> AND version=?<期待値>` の変更件数が 1 でなければ、`StateGuardError::SnapshotChanged`（`state/mod.rs:75-79`）で中止する。run と step の version は、それぞれ別の期待値で照合する（両者の数値が一致することではない）。
5. 結果を進める API（規則 G）は、同じ tx で Goal が Running であることを確かめる。
6. trigger（§2.3）は、API の検査が抜けたときの第 2 の層である。

API の一覧（B2 で作る。どれも spawn しない）: `pipeline_mode_inventory`（読み取り。epoch の前）、`pipeline_claim_run`、`pipeline_take_over`、`pipeline_begin_launch`（tx1）、`pipeline_record_running`（tx2）、`pipeline_record_launch_unknown`、`pipeline_record_exit`、`pipeline_finish_step`、`pipeline_record_limited`、`pipeline_record_commit`、`pipeline_adopt`、`pipeline_hold`、`pipeline_plan_next`、`pipeline_release_begin`、`pipeline_release_settle`、`pipeline_complete_goal`、`pipeline_cancel_run`、`pipeline_recover`、`pipeline_settle_limit_wait`（§2.6 の 4）、`pipeline_record_qualification`、読み取りの `pipeline_goal_view` / `pipeline_task_view`。step を launching / running から出す API（`pipeline_record_exit`・`pipeline_finish_step`・`pipeline_record_launch_unknown`・`pipeline_hold`・中断の記録など）は、その step が probe なら、同じ tx で上限待ちの記録を決着させる（§2.6）。

### 2.5 Goal plan の拡張

`GoalPlan`（`runtime/goal.rs:57`）に、任意の `execution` を足す（`#[serde(default)]`）。`GoalDefinition` の canonical digest（`runtime/goal.rs:124`）は変えない。`execution` は `goal_execution_profiles.policy_sha256` で固定する。

```json
"execution": {
  "max_fix_rounds": 2,
  "blocking_severity": "high",
  "max_limit_waits": 3,
  "task_scopes": { "<task key>": ["src/**", "tests/**"] }
}
```

`cli_pipeline` の rrxd は、受入時（`accept_runtime_goal` `state/runtime/goals.rs:28` の同じトランザクション）に次を検査する。

- `execution` がある。`managed_native` の rrxd は、`execution` がある plan を拒否する。
- 全基準が `RequiredTasksVerified` である（`state/runtime/goals.rs:110-115` の検査に加える）。
- 各 Task に scope がある。scope の形は相対 glob、64 件以下、各 512 byte 以下で、`..` と絶対パスを含まない。
- `max_fix_rounds` は 0〜2、`blocking_severity` は critical / high / medium / low のどれか、`max_limit_waits` は 0〜6（省略時 3）。
- `executor` は `codex`、`reviewers` は `["claude"]` である。`agents` ではなく `pipeline.clis` で検証する（`runtime/goal.rs:189-198` をモードで分ける）。
- Project に admit 済みの TestsProfile がある（`verification_profiles` の行）。

`policy_sha256` は次の 3 つを canonical な JSON にした sha256 とする: `execution`、TestsProfile の digest、CLI pin 設定の digest。

### 2.6 容量と provider の上限

**1 process ずつ（v1）**: daemon 全体で、process の step（implement・test・review・confirm・fix）を同時に 1 つだけ動かす。

- tx1 は、自分以外に「`launching`・`running`・`unknown`、または `body.death_confirmed` が true でない `held`」の step が 1 つでもあれば、起動しない（step は planned のまま、次の tick を待つ）。trigger も planned → launching で同じ条件を検査する（§2.2）。prepare / release は数えない。
- これは native のどの同時実行の上限（全体・provider・executor の予約枠）よりも厳しい。provider ごとの枠・公平性の順序・予約枠は持たない。
- 死亡を確認できない step が 1 つでもあれば、どの Project の step も起動しない（安全側）。運用者が `rrx pipeline recover` で解くまで待つ。
- Project ごとの Task は 1（`config.rs:17`）。run の claim の occupied 検査は `pipeline_runs WHERE state IN ('running','held')` で行う。held の run も slot を使い続ける（worktree と残存プロセスが不確かなため）。native の occupied CTE（`candidates.rs:49`、`claim.rs:375`）は変えない。
- 複数の step の並行実行と provider の容量の共有は Stage 2 で別に設計する。

**native の exhaustion（読むだけ）**:

- **判定**: `windows`（`state/execution/quotas.rs:476-502`）で decode した `quota_windows` の行のうち、status が Exhausted のものが 1 つでもあれば「Exhausted の観測が残る」とする。`resets_at`・`quota_pools.next_probe_at`・現在時刻は見ない。`quota_windows` は provider・account・bucket ごとに 1 行で、native は `quota_observation::applies`（`quota_observation.rs:10-22`）が認める新しい Available の観測（新しい window か、native の probe による観測）でだけ Exhausted の行を置き換える（`quotas.rs:76-84`）。native の admission も、時刻ではなく保存された状態で判定する（`quotas.rs:186-189`）。パイプラインはこの規則に従い、時刻の経過で観測が解けたとはみなさない。
- パイプラインは CLI の account と native の account の対応を証明しないので、provider と account を問わず、1 つでも残っていれば拒否する。decode に失敗した行があっても拒否する。
- **起動時**: `cli_pipeline` の静止の確認で拒否する（§1.1。rrxd は起動しない）。
- **tx1**: 同じ判定をする。残っていれば起動せず、同じ tx で step を planned → held（`native_quota_exhausted`、`death_confirmed=true`）、run を held にする。`cli_pipeline` の rrxd には `quota_windows` の書き手がない（書き手は native の実行経路の `execution/native.rs:1523` から呼ぶ `quotas.rs:17,22` と `state/execution/native_phase/live_quota.rs:370` だけで、`cli_pipeline` は native の admission を呼ばない）。そのため、起動時の確認を通った後にこの held になるのは想定外の書込みがあった場合だけで、tx1 の検査は第 2 の層である。
- **運用者への表示**: `rrx goal status` の run の view に Held の理由 `native_quota_exhausted` を出し、`PipelineGoalView.native_exhaustion` に、残っている観測の provider・bucket・`resets_at` を出す（account は出さない。§10）。起動時の拒否は `rrx daemon start` の `ExecutionModeNotQuiescent { reason: native_quota_exhausted }` で示す。
- **解除**: パイプラインは quota の表を書かない（`quota_pools`・`quota_windows`・`quota_leases`・`quota_waiters` のどれも）。観測を解けるのは native の新しい Available の観測だけである。運用者は、すべての run を終えて（`--then abandon-run` など）`managed_native` で起動し、native が観測を置き換えてから `cli_pipeline` に戻す。`--then retry` は受け付けるが、tx1 が同じ判定でまた held にする。
- `windows` は `pub(super)` にするだけで、挙動を変えない。

**provider の上限（既知の終了だけ・daemon 全体の 1 つの記録）**:

v1 は CLI の account と provider の account を対応付けないので、上限の待ちは、provider・account・Project・run を区別しない 1 つの記録 `pipeline_limit_wait`（§2.2）に置く。run と step から独立しているので、run の放棄・回復・rrxd の再起動の後も残る（再起動時は §4.1 で owner だけを引き継ぐ）。すべての Project の、すべての Agent の step（implement・fix・review・confirm）の tx1 がこの記録を確かめる。test の step は provider を呼ばないので、この待ちの対象にも probe にもならない。

| 記録の state | 意味 | Agent の step の tx1 |
|---|---|---|
| 行なし / `clear` | 待ちなし | 起動できる |
| `waiting` | 既知の上限か、probe の既知の成功以外の終了の後 | `now < not_before` なら起動しない（step は planned のまま。§10 で WaitingQuota）。`now >= not_before` なら、同じ tx1 で記録を `probing` にしてこの step を probe として書き、続いて step を launching にする |
| `probing` | 期限の後の最初の起動が、まだ決着していない | probe の step 以外は起動しない |

1. **分類**: StepCustody が死亡を確認して running → exited を書く tx（観測の書込み）で、Agent の step の出力を §5.6 で分類し、`body.provider_outcome` を `success`・`limit`・`other` のどれかにする（一度書いたら変えない。trigger）。`other` は、認証の失敗・分類できない失敗を含む、`success` と `limit` 以外のすべてである。
2. **延長**: 次のどれかを書く tx は、同じ tx で記録を延長する。延長は `not_before = max(旧 not_before, now + backoff_ms)`、`backoff_ms = min(backoff_ms × 2, 30 分)`、`state='waiting'`、probe を空にする。行がなければ `not_before = now + 60 秒`、`backoff_ms = 120 秒` で作る。
   - どの step でも、`provider_outcome = limit` の exited。
   - probe の step が、launching / running から exited 以外（unknown・held・interrupted・rejected）に出たとき（中断・死亡の未確認・起動結果の不明・起動前の失敗）。
   - probe の step の exited で、`provider_outcome` が `success` でないもの（認証の失敗を含む）。
3. **解除**: probe の step の exited で `provider_outcome = success` なら、同じ tx で記録を `clear` にし、`backoff_ms` を 60 秒に戻す。時刻の経過だけでは解除しない（trigger は waiting → clear を許さず、probing → clear は probe の step の `provider_outcome = success` を要求する）。
4. **決着の取りこぼし**: tick と §4.1 の照合は、記録が `probing` で probe の step が launching / running でなければ、その step の `provider_outcome` が `success` なら 3、それ以外なら 2 の延長を書く（`pipeline_settle_limit_wait`）。
5. **run ごとの回数**: `provider_outcome = limit` の step を、tick が exited → limited にする tx で、run の `body.limit.waits` を +1 する。更新後の `waits` が Goal の `max_limit_waits` を超える場合、または attempt が 16 の場合は、limited ではなく held（`limit_waits_exhausted` / `attempt_limit`）にする。
6. **次の attempt**: Goal が Running なら tick が計画する（§4.2）。§6.4 で step_base に初期化して検証し、attempt+1 を planned で作る。起動を待つのは tx1 で、上の表に従う。
7. 一致しないものは上限として扱わない。通常の失敗（rejected → run held）か、判断が要るもの（held）になる。起動結果が不明な attempt（unknown）は再実行しない。

## 3. 状態機械

### 3.1 run

| 遷移 | 実行する主体 | 必要な永続の根拠 |
|---|---|---|
| （なし）→ running（kind=task） | tick の claim tx | 次をすべて満たし、同じ tx で run 行と seq 1 の prepare step（planned）を作る。初回の Task なら `goal_create` の journal（pending）も作る。<br>・現在の owner（§2.4）。<br>・Goal が Running で、`goal_execution_profiles` の行がある。<br>・hard prerequisite の run がすべて completed。<br>・同じ Project に running / held の run がない。<br>・Project が Registered。 |
| （なし）→ running（kind=goal） | 同上 | Goal の全 Task の run が completed で、同じ Project に running / held の run がない。同じ tx で、TestsProfile の各コマンドの test step の 1 つ目（planned、`step_base` は Goal branch の head）を作る |
| running/held → 同じ state（owner_epoch を上げる） | 起動時の照合（§4.1） | 引継ぎ。owner_epoch と version だけを変える（trigger が検査する） |
| running → held | tick / StepCustody / 復旧 | Held の理由を持つ step（held / unknown / rejected）が同じ tx で書かれていること。`body.held` に理由と step の参照を書き、監査イベント `rrx.private.pipeline.run_held` を残す |
| held → running | `rrx pipeline recover --then retry` | `pipeline_recoveries` の行（`consumed_version` = 新しい version） |
| running → completed（kind=task） | release の settle | `task_release` の journal が done（trigger が検査する） |
| held → completed（kind=task） | `rrx pipeline recover --then abandon-run`（release 済みの journal の決着。§4.4） | `task_release` の journal が done、`pipeline_recoveries` の行（`consumed_version` = 新しい version）、生きているかもしれない step がない（trigger が検査する） |
| running → completed（kind=goal） | Goal 完了の tx | `pipeline_goal_completions` の行（trigger が検査する） |
| running/held → abandoned | Goal cancel の後の tick、または `rrx pipeline recover --then abandon-run` | 生きているかもしれない step が 1 つもなく、その run の `task_release` の journal が pending / held でない（trigger が検査する。§2.2）。Project の slot が空く |

**不変条件**: run が held のとき、最後の step はちょうど 1 つの held / unknown / rejected の step である。運用者は常にその step を `rrx pipeline recover` の対象にできる。

### 3.2 step

| 遷移 | 主体 | 必要な根拠 |
|---|---|---|
| planned → launching | StepLauncher の tx1 | 次をすべて確認したうえで、cookie（UUIDv4 を英数字にしたもの）、attempt ディレクトリ、argv・env の digest を書く。<br>・現在の owner と、run.owner_epoch = 現在の epoch。<br>・run.state = running、run.version と step.version がそれぞれの期待値と一致。<br>・Goal が Running。<br>・`stopping` が false（実行境界の中で確認）。<br>・他に生きているかもしれない process の step がない（§2.6）。<br>・Exhausted の native の観測が残っていない（§2.6。残っていれば下の planned → held）。<br>・Agent の kind なら、daemon 全体の上限待ち（§2.6）が行なし / clear か、waiting で `not_before` を過ぎている（同じ tx1 で probing にし、この step を probe にする）。<br>・その epoch で該当の CLI が qualified。 |
| planned → held | tick（起動前の検査） | Exhausted の native の観測（§2.6）、作業領域の検証の失敗（`worktree_drift`。§4.2）、TestsProfile の digest の不一致（§7.2）。spawn していないので `death_confirmed=true` |
| planned → running（prepare / release） | tick | 現在の owner。cookie・pid は持たない |
| planned → abandoned | Goal cancel の後の tick | 起動していない |
| launching → running | StepLauncher の tx2 | spawn が Ok を返したこと。pid、pgid（=pid）、birth（`rrx-process-tracker` の identity による開始時刻） |
| launching → unknown | StepLauncher / 復旧 | spawn が Err を返した（子が残りうる。`crates/rrx-spawn-hygiene/src/lib.rs:5-6`）か、再起動時に launching を見つけた |
| launching → rejected | StepLauncher | spawn を呼ぶ前の失敗（attempt ディレクトリの作成、sha256 の不一致、test の `recheck` の失敗など）。spawn の前の rejected は、spawn を呼んでいないことを同じプロセスの中で確実に知っている場合だけ（spawn の後の rejected は、次の行の場合だけ） |
| launching → rejected | StepLauncher | tx2 が失敗したあと、group を kill して死亡を確認した場合。`pipeline_finish_step` で死亡の根拠、`body.exit.issue=record_failed`、`reason=record_failed`、step の rejected と run の held、probe の待ちの決着を 1 commit で記録する。確認できなければ launching のまま残す（再起動時に unknown になる） |
| running → exited | StepCustody | §3.5 の死亡確認がすべて成功した。終了の状態、出力の sha256、上限を超えたかどうか、Agent の kind では `provider_outcome`（§2.6・§5.6） |
| running → interrupted | 停止 / Goal の pause・cancel | kill して死亡を確認した。attempt が 16 なら interrupted ではなく held（`attempt_limit`、`death_confirmed=true`） |
| running → held | StepCustody / 復旧 | 死亡を確認できない（group が空にならない、discovery が limited、cookie を持つプロセスが見つかる）。または古い epoch の running（custody の喪失） |
| running → unknown | StepCustody | 監視タスク自体の異常（panic・join の失敗）で観測が途切れた |
| running → adopted / held / interrupted（prepare / release） | tick | §4.2 と §9 の検証の結果 |
| exited → committed | AdoptionChecker（規則 G） | implement / fix だけ。§6 の検査に合格し、組み立てた commit の SHA と tree を `result_commit` / `result_tree` に書く |
| committed → adopted | AdoptionChecker（規則 G） | Task branch の ref が `result_commit` を指し、HEAD と index と worktree がその commit と一致する（§6.3） |
| committed → held | AdoptionChecker / 復旧 | ref が step_base でも `result_commit` でもない |
| exited → adopted | VerdictDecoder / PipelineTestRunner（規則 G） | test / review / confirm。kind ごとの採用条件（§7・§8）を満たした |
| exited → rejected | 同上 | 非 0 終了（Agent）、出力の上限超過、schema 違反、scope 違反、Agent が自分で commit したなど、確定した不合格 |
| exited → limited | 同上 | provider の上限（§2.6。`provider_outcome = limit`） |
| exited → held | 同上 | 秘密スキャンへの該当、snapshot の変化、model の不一致、`.git` の変化、判定による保留（§8.5）、上限待ちの回数の超過など、運用者の判断が必要なもの |
| exited / committed → interrupted | Goal cancel の後の tick | 採用せずに終える。死亡は確認済み |
| unknown/held/rejected → abandoned | `rrx pipeline recover --abandon-step` | `pipeline_recoveries` の行（trigger が検査する） |

`unknown` から別の状態へ移るのは、運用者の記録による `abandoned` だけ。Unknown の step は、どの経路からも再起動しない。同じ seq の新しい attempt は、`interrupted`・`limited`・`abandoned` の後にだけ、作業領域を初期化して検証してから作る（trigger が検査する）。attempt は 16 までで、16 回目の attempt は interrupted・limited にならず held になるので、自動で 17 を作ろうとすることはない。

probe の step（§2.6）を launching / running から出す tx は、どの遷移でも、同じ tx で上限待ちの記録を決着させる（exited で `provider_outcome = success` なら解除、それ以外は延長）。

### 3.3 起動の手順

次の 1〜6 を、`control_admission` の guard を保持したまま行う（§1.2）。

1. tx1（`BEGIN IMMEDIATE`、短く）: §3.2 の planned → launching の条件を確認して書き、commit する。process の kind の step では、body に `launch`（§3.5.2）を書く。
2. Store lock を解放する。Store lock は `std::sync::Mutex` で、control 側は `self.owner.store.lock()` として取っている（`runtime/control.rs:657-662`）。spawn の前に、必ず guard を drop する。
3. attempt ディレクトリと TMPDIR を作り、実行ファイルを照合し直す。Agent は sha256 を Q の記録と照合する。test は `AdmittedProfile::recheck`（`execution/verification/plan.rs:216-223`）で、コマンドが admit 時の一覧にあり、program の hash が一致することを確かめる（既存の経路と同じく各起動の直前に行う。`execution/verification/mod.rs:496`）。不一致なら launching → rejected にし、run を held にする（spawn していない）。
4. `tokio::process::Command` を組み立てる。`env_clear()`、許可した環境変数だけ（§5.4）、`process_group(0)`、`kill_on_drop(true)`、stdin・stdout・stderr は pipe。`rrx_spawn_hygiene::spawn_tokio`（`crates/rrx-spawn-hygiene/src/lib.rs:58`）で起動する。
5. Err の場合: tx で launching → unknown と run → held（`launch_unknown`）を書く。
6. Ok の場合: `ProcessGroup::new`（`adapter.rs:482`）で包み、`owned_leader`（§3.5.3）で PID から読んだ birth と cookie の確認の後に、すぐに tx2 で launching → running（pid、birth）を書く。stdin にはまだ書かない。
   - tx2 が失敗した場合: `cleanup_group` → `reap` → group が空になるのを待つ → discovery。確認できたら、§3.2 の表のとおり、1 回の commit で launching → rejected（`record_failed`）と run の held を記録する。書けなければ何もしない（再起動時に unknown になる）。
7. guard を解放し、プロセスを StepCustody の監視タスクに渡す（§3.4）。

### 3.4 実行中の入出力

StepCustody の監視タスクは、次を 1 つの `tokio::select!` で並行に扱う。stdin の書込みを、出力の読取りや停止の監視より前に済ませることはしない。

| 項目 | 扱い |
|---|---|
| stdin | prompt（最大 512 KiB）を書き、終わったら閉じる。書込み全体に 30 秒の期限を付ける。期限切れか EPIPE なら group を kill し、死亡を確認して rejected（`stdin_failed`） |
| stdout / stderr | 起動の直後から、8 KiB の buffer で上限付きで読み続ける。上限は §5.5。stdout の上限超過は kill して rejected（`output_overflow`）。stderr は末尾を残し `truncated` を記録する |
| 終了 | `observe_exit`（`adapter.rs:503`）。終了後は、出力の残りを drain の期限（agent は 10 秒、test はコマンドの `drain_seconds`）まで読む |
| wall-clock の上限 | implement / fix 3600 秒、review 1800 秒、confirm 1200 秒、test はコマンドの `timeout_seconds` |
| 停止要求 | Goal の pause・cancel と shutdown の通知（`watch` channel）。受けたら kill して死亡を確認し、interrupted |

監視タスクが書くのは観測の記録（規則 G）だけで、guard を取らない。書いた後に `wake.notify_one()` を呼び、採用は次の tick が行う。出力は attempt ディレクトリのファイルへ書き、sha256 を逐次計算する。保存と読み戻しには `retain` / `inspect`（`execution/verification/collector.rs:218,240`）を `pub(crate)` にして使う（挙動は変えない）。stream の digest は `StreamEvidence`（`collector.rs:32`）の形で `body` に記録する。

### 3.5 死亡確認

次をすべて満たしたときだけ「死亡を確認した」とする。

1. `observe_exit` が leader の終了を観測する（`waitid NOWAIT`、`adapter.rs:503-519`）。
2. leader が zombie のうちに `cleanup_group` で group に SIGKILL を送る（`adapter.rs:1589`。macOS の EPERM は `ps` で全員が zombie と確かめる。`adapter.rs:588-598`）。続けて、leader が zombie のうちに、その開始時刻が記録した birth と一致することを確かめる（§3.5.3）。
3. `reap` する（`adapter.rs:562`）。
4. 2 秒以内に `test_kill_process_group` が ESRCH を返し、group が空であることを観測する（`git.rs:648-662` と同じ方式）。
5. `rrx_process_tracker::discover_launched(cookie, launch)` の `coverage.limited()` が false で、一致するプロセスが 0 件である（macOS では §3.5.1・§3.5.2）。

どれかが失敗したら「確認できない」とし、held にする（`body.death_confirmed=false`）。macOS では、見つかったプロセスに signal を送らない（`crates/rrx-process-tracker/src/macos.rs:12-23`）。

5 の探索は、結果が limited のときだけ、独立した新しい探索としてやり直す。やり直すのは、4 の後の同じ期限（`DEATH_BOUND`）の中だけで、各回の間に短く待つ。採るのは limited でない最初の結果だけである。どの回でも一致が 1 件でもあれば、その時点で「確認できない」とし、やり直さない。期限までに limited でない結果が得られなければ「確認できない」とする。各回の探索は §3.5.1 の保証をそれぞれ単独で満たすので、limited でない回を採ることは、最初の回でそれが得られた場合と同じ意味になる。やり直しの回数は `body.death_evidence` に記録する。他のプロセスが起動や exec の途中で一瞬読めないこと（照会は「生きている」を返す）で、死亡確認が held になるのを避けるためである。

#### 3.5.1 macOS の探索での zombie と消えた PID

**問題**: §3.5 の 5 は `coverage.limited()` が false であることを求める。macOS の tracker は、同じ利用者の PID を `proc_listpids(PROC_UID_ONLY)` で列挙し、`proc_pidinfo(PROC_PIDTBSDINFO)` で identity を読む（`crates/rrx-process-tracker/src/macos.rs:24-53,103-127`）。zombie ではこの読取りが失敗し、`identity_unavailable` に数えられる（`macos.rs:133-137`）。そのため `limited()` が true になる（`lib.rs:100-108`）。親が回収しない zombie が 1 つ残るだけで、どの step も死亡を確認できず held になる。オーナーの Mac では、2 日以上残る zombie を観測した（`inspected=766, same_user=765, identity_unavailable=1`）。

**今の tracker にもある観測の穴**: `proc_listpids` は PID だけを返す。そのため、列挙の時点でその PID にいたプロセス（incarnation）と、後で読んだ identity・環境とを結び付けられない（`macos.rs:133-156`）。一覧 L に cookie を持つ A がある。A が子 C を作って exit し、回収され、その PID が cookie を持たない B に割り当てられる。この順序が最初の identity の読取りより前に起きると、B を A として検査してしまう。C は L になく、何の counter も増えない。環境の読取り（`macos.rs:144`）の直前に同じことが起きた場合も同じである。一致しなかったときは identity を読み直さないからだ（`macos.rs:151-153`）。この節では、環境の読取りの前後の穴と、前の巡で記録した PID での穴を閉じる。一覧と、記録のない PID の最初の読取りとの間の穴は閉じず、下の残余 R1 として残す。

**前提**（安全性の論証はこれだけに依る）

| ID | 前提 | 出典と確認 |
|---|---|---|
| P1 | cookie は環境の引継ぎでだけ伝わる。プロセスが自分の環境から cookie を消すことは対象外とする | 今の tracker と同じ（`lib.rs:1-6`） |
| P2 | cookie を持つプロセスは同じ利用者（実効 uid）である | 今の tracker と同じ。走査の範囲が同じ利用者だけである |
| P3 | `proc_listpids(PROC_UID_ONLY, uid)` は、process list の lock を取った 1 つの時点 t0 に、`allproc` にあって実効 uid（`kauth_cred_getuid`）が uid のプロセスの PID を、lock の中で集める。shadow proc は飛ばす。`allproc` の後に空きが残れば、`zombproc` の zombie も集める。集めるのは kernel 内の容量 min(nprocs+20, 出力先の個数) までで、そこで止まる。nprocs は lock の前に読む | XNU のソース（`apple-oss-distributions/xnu` の main、`bsd/kern/proc_info.c`）で確かめた。作成中（`SIDL`）のプロセスが入るかどうかは確かめておらず、論証には使わない |
| P4 | 同じ PID で開始時刻（µs）が同じなら、同じプロセスである | 今の `Identity { pid, uid, birth }` と同じ前提（`lib.rs:83-85`） |
| P5 | プロセスは、fork から回収まで同じ PID を持つ（exec でも変わらない）。存在するプロセス（zombie を含む。shadow proc は除く）の間で PID は重ならない。回収された PID は、PID の counter が一巡してその値に戻るまで割り当てられない。使用中の process group・session の ID も割り当てられない | XNU のソース（`bsd/kern/kern_fork.c` の `forkproc`）で確かめた。counter は直前の PID から 1 ずつ進み、PID_MAX に達すると 100 に戻る。process・process group・session の hash にある値は飛ばす。zombie は回収まで hash に残る。PID_MAX の値はそのファイルになく、99999 は記憶による。論証は 1 文目と 2 文目だけに依る。3 文目以降は、残余 R1 の条件を述べるためだけに使う |
| P6 | 作成を終えて exit していないプロセスは、exec の途中も含めて、shadow でない proc として `allproc` にある | XNU のソース（`bsd/kern/kern_exec.c` の `proc_exec_switch_task`）で確かめた。exec の新しい proc は shadow として作られ、切り替えでは `proc_list_lock()` の中で、pid hash の置き換え（`phash_replace_locked`）と shadow の印の移し替え（新しい proc から外し、古い proc に付ける）を行う。`proc_listpids` は同じ lock の中で集めるので、どの時点でも、その PID の shadow でない proc を 1 つだけ見る（切り替えの前は古い proc、後は新しい proc） |

一覧に開始時刻は含まれない。そのため、`kinfo_proc` の開始時刻と `proc_bsdinfo` の開始時刻が同じ値であることには依らない。開始時刻の比較は、いつも同じ出どころの値どうしで行う（下の記録）。

**列挙**: `proc_listpids(PROC_UID_ONLY)` を今のまま使う（`macos.rs:105-127`）。出力先は `limits.processes` 個の PID とし、`discover` の 1 回ごとに 1 度だけ確保して、各巡で使い回す（既定の 4096 で 16 KiB）。

- 次の一覧は `Err` とする: 返り値が 0 以下・4 の倍数でない・出力先の大きさを超える、または同じ PID が 2 回現れる（今は 0 以下だけを `Err` にしている。P3・P5 により、正しい一覧に同じ PID は現れない）。`pid <= 0` の要素は飛ばす（今と同じ）。
- 返った個数が出力先の個数と等しければ `truncated` とし、返った PID は走査する（今と同じ）。limited になる。
- kernel 内の容量（nprocs+20）が出力先の個数より小さく、そこで止まった場合は、返った個数からは分からない（P3）。これを残余 R2 とする。
- zombie は、`allproc` の後に空きが残ったときだけ一覧に入る（P3）。下の規則と論証は、一覧に zombie が入るかどうかに依らない。一覧にない PID は、記録があっても読まず、新しい不在にも数えない。

**分類の規則**: 第 1 の読取りが失敗した PID にだけ、独立した第 2 の照会 `sysctl({CTL_KERN, KERN_PROC, KERN_PROC_PID, pid})` を行う。第 1 の読取りとは、identity の読取り（`macos.rs:133`）か、環境の読取り（`macos.rs:144`）である。照会の出力先は `kinfo_proc` 1 つ分とする。XNU では、その PID で探せるプロセスがなければ、照会は成功で長さ 0 を返す（XNU のソースで確かめた）。

| 第 2 の照会の結果 | 分類 | 数える先 |
|---|---|---|
| 状態が成功で、返った長さが 0 | 消えた PID | `vanished` |
| 状態が成功で、長さが `size_of::<kinfo_proc>()` とちょうど一致し、`kp_proc.p_pid == pid` かつ `kp_proc.p_stat == SZOMB` | zombie | `zombie` |
| それ以外（errno を問わない失敗。失敗で長さが 0 の場合を含む。短い長さ・長すぎる長さ、pid の不一致、`p_stat` が `SIDL`・`SRUN`・`SSLEEP`・`SSTOP`、または未知の値） | 不明 | 元の失敗に応じて `identity_unavailable` か `environment_unavailable`（今と同じ） |

- `SZOMB` の値は、SDK の `sys/proc.h` では 5 である。`PROC_UID_ONLY` と同じく、SDK の出典を書いた定数として置く（`macos.rs:7-8`）。
- 第 2 の照会を使うのは、失敗した読取りを分類し直すときと、zombie の記録と照合するとき（下の巡の扱いの 2）だけである。照会は、1 つの PID につき 1 巡で 1 回までとする。第 1 の読取りが成功した PID の判断は変えない。
- 照会の結果は、照会した時点のその PID の状態を表す。一覧の時点の incarnation が、その後に子を作った可能性もある。この可能性には、zombie と消えた PID を新しい不在に数え、再列挙することで対応する（下の再列挙）。
- 第 2 の照会が「不明」を返したら、その PID の扱いはそこで終わる。同じ巡で後の検査（環境の読取り・identity の読み直し）に進まず、成功した検査が counter を打ち消すこともない。Coverage の counter は走査全体で増えるだけで、巡ごとに作り直さない。limited になった後は、次の巡を行わない（下の再列挙の 2）。したがって、後で同じ PID の通常の検査が成功しても、不確実さは消えない。
- `Coverage`（`lib.rs:87-97`）に `zombie: usize`、`vanished: usize`、`reused: usize`、`relistings: usize`（下の再列挙の回数）を足す。4 つとも `limited()` の条件に入れない。Linux の tracker はこれらを数えず、常に 0 になる。
- この分類で `processes` に加わる PID はない。zombie と消えた PID は `TrackedProcess` にならず、signal の対象にもならない。macOS では、見つかったプロセスにもともと signal を送らない（`macos.rs:12-23`）。

**zombie と消えた PID が、cookie を持って動けない理由**: zombie は exit 済みで、命令を実行しない。アドレス空間（環境を含む）も fd も持たない。消えた PID とは、照会の時点で、その PID のプロセスを kernel が探せないことを表す。cookie の一致が意味を持つのは、まだ書込み・pipe の保持・fork ができるプロセスだけである。第 2 の照会が「生きている」「作成中」を返した場合、照会そのものが失敗した場合、結果が第 1 の読取りと食い違う場合は、従来どおり limited に倒す。

**記録**: 次の PID には、記録を作る。記録は種類と key からなる。

- 生の記録: 全部の検査で `processes` に入れた PID と、cookie が一致せず identity2 が identity1 と等しかった PID。key は identity の birth である。
- zombie の記録: zombie に分類した PID。key は照会の `p_starttime` である。
- key が 0 のものは記録しない。照合に合わなかった記録は消す（下の巡の扱いの 2）。

**巡の扱い**（一覧の各 PID p に対して）

1. identity を読む（identity1）。
2. p に記録があれば照合する。次のどちらかに当たれば照合済みとし、p の扱いを終える（環境は読まず、counter も増やさない）。
   - 生の記録で、identity1 が読め、その birth が key と等しい。
   - zombie の記録で、identity1 が読めず、第 2 の照会が返した zombie の `p_starttime` が key と等しい。

   どちらにも当たらなければ記録を消し、3 へ進む。identity1 が読めていたら `reused += 1` とする（新しい不在）。identity1 が読めずに照会をした場合は、その結果を 3 の分類に使い、照会はやり直さない。
3. 全部の検査を始める。`inspected += 1`。identity1 が読めていなければ第 2 の照会で分類し、終わる。
4. identity1 の uid が自分と違えば、今と同じく飛ばす（`macos.rs:140-142`）。
5. `same_user += 1`。環境を読む。失敗したら第 2 の照会で分類し、終わる。
6. identity をもう一度読む（identity2）。一致したかどうかに依らず読む。
   - cookie が一致した場合: identity2 が identity1 と等しければ `processes` に入れる。等しくないか読めなければ `identity_changed` とする（今と同じ。一致したプロセスは、その時点で生きていたからである。`macos.rs:154-157`）。
   - 一致しなかった場合: identity2 が identity1 と等しければ検査済みとする。等しくないか読めなければ、読んだ環境が identity1 の incarnation のものだと言えないので、`reused += 1` とする。

**再列挙**: 一覧の後の fork があるので、上の分類だけでは足りない。一覧の時点で生きていた、cookie を持つプロセス A が、その後に子 C を作ってから exit したとする。A は zombie・消えた PID・`reused` のどれかになり、C は一覧に含まれない。今の実装はこの場合を、`identity_unavailable` で limited にするか、上の穴で見落とすかのどちらかになる。zombie と消えた PID を limited から外すなら、この保護を別の手段で保たなければならない。二重 fork でデーモン化するツールが、この形に当たる。そこで、オーナーが選んだ分類に、次の再列挙を加える。

1. 1 巡目は、一覧のすべての PID を上の扱いにかける（記録はまだないので、すべて全部の検査になる）。その巡で増えた `zombie`・`vanished`・`reused` を「新しい不在」と呼ぶ。新しい不在がなければ終わる。PID の主が変わらず zombie もない環境では、1 巡で終わり、結果も今と同じになる（増えるのは、一致しなかった PID の identity2 の読取りだけである）。
2. 新しい不在があり、`limited()` がまだ false なら、その巡の分類をすべて終えた後に一覧をやり直す（`relistings += 1`）。新しい一覧の各 PID を上の扱いにかける。記録と照合された PID については、identity を 1 回読むだけで（zombie の記録では、これに照会が 1 回加わる）、環境は読まない。
3. 新しい不在が出なくなるまで 2 を繰り返す。各巡は同じ `Limits` の中で行う。時間の上限（`limits.time`）に当たれば `time_exceeded`、どれかの一覧が上限に当たれば `truncated` とし、どちらも limited になる。`inspected` と `same_user` は、全巡の全部の検査を数える（照合だけの読取りは数えない）。

**安全性の論証**: 結果が limited でないのに、走査の終わりに、cookie を持つ生きたプロセス X があると仮定する。最後の一覧 L の時点を t0 とし（P3）、L の PID の集合を S とする。以下、残余 R1・R2 に当たらなければ `processes` が空でないことを示す。

- A を決める。P1 により、X から親をたどると、cookie を持つプロセスの列が step の leader まで続く。leader は走査の前に作成を終えている。この列のうち、t0 までに作成を終えた最後のものを A とする。
  - A = X の場合: X は走査の終わりに生きているので、t0 にも生きていた（zombie は生き返らない）。
  - A ≠ X の場合: 列の上での A の子 C は、t0 に作成中だったか、t0 の後に作られた。A は t0 の時点か後に fork の中にいたので、t0 に生きていた。

  どちらの場合も、A は t0 に作成を終えて生きていて、cookie を持ち（P1）、同じ実効 uid である（P2）。
- R2 に当たらず `truncated` でもなければ、t0 の `allproc` の走査は終わっている。したがって P3・P6 により、A の PID p は S に入る。
- 最後の巡での p の扱いは、次のどれかになる。
  - 照合済みの場合: 記録は最後の一覧より前の巡の読取りで作られ、最後の巡の読取りで同じ（PID、開始時刻）を観測した。P4 により両者は同じプロセスで、P5 によりそのプロセスはその間ずっと p にいた。したがって t0 に p にいたのはそのプロセスであり、A である。zombie の記録なら、A は t0 より前に zombie だったことになり、t0 に生きていたことと矛盾する。生の記録なら、A は前の巡で `processes` に入ったか、cookie が一致しなかったとして記録されたかである。後者では、identity1 と identity2 の birth が key と等しいので、その間に読んだ環境は A のものである（P4）。その環境に cookie がないことは P1 と矛盾する。`discover_launched` で環境が absent のときに作る記録（§3.5.2 の系譜の判定の 1・2 だけ）なら、§3.5.2 の安全性の論証により A ではありえない（時計の条件の下で。外れれば R3）。`discover` は absent も不一致として記録するので、この矛盾は環境を観測できる場合だけに成り立つ（下の `discover` の保証）。
  - 記録があって照合されなかった場合: identity1 が読めていれば `reused`（新しい不在）になる。読めなければ、照会で zombie・消えた PID（新しい不在）か、不明（limited）になる。
  - 記録がない場合: identity1 が読めなければ、新しい不在か limited になる。identity1 で読めたプロセス Q が A なら、次のどれかになる。4 は P2 に反する。5 の失敗は、新しい不在か limited になる。6 で cookie が一致すれば、`processes` に入るか `identity_changed`（limited）になる。一致せず identity2 が等しい場合、環境が読めていれば A の環境に cookie がないことになり、矛盾する（引数の領域の偽装は残余 R6）。環境が absent なら、`discover_launched` では §3.5.2 の補題により limited になる（補題の条件が成り立たないのは、残余 R3・R7・R8 の場合である）。`discover` では一致しないとして扱うので、見落とす（下の `discover` の保証）。等しくない場合は `reused`（新しい不在）になる。Q が A でなければ、A は t0 の後、その読取りの前に p を離れ、p は Q に割り当てられた。これが残余 R1 である。
- 最後の巡には新しい不在がなく、結果は limited でない。したがって、残るのは `processes` に入る場合だけである。

したがって、`discover_launched` の結果が limited でなく一致が 0 件なら、残余 R1・R2（この節）と R3・R6・R7・R8（§3.5.2）に当たらない限り、走査の終わりに cookie を持つ生きたプロセスはない。そのときは、その後に cookie を持つプロセスが作られることもない（親になれるものがいない）。R5（§3.5.2）のプロセスは cookie を持たないので、この主張の例外ではないが、cookie の探索では見つからない（今と同じ）。`discover`（launch なし）は、環境が absent のプロセスを今と同じく一致しないとして扱う（absent も不一致の生の記録にする）。そのため `discover` の結果が言えるのは、走査の終わりの X について、上の論証の A（X の祖先の列のうち t0 までに作成を終えた最後のもの）が、どの巡の読取りでも環境を absent と読まれなかった場合に、R1・R2 を除いて同じことだけである。A が absent と読まれれば、X が A 自身でも、A が生きたまま作った、環境を観測できる子孫（一覧になく、新しい不在も生じない）でも、見落とす。そのため `discover` は、環境を観測できるプロセスについても死亡確認を保証しない（今の tracker と同じ）。§3.5 の 5 と、下の消費者のうち死亡確認・復旧表・回復の command が使うのは、`discover_launched` の意味である。macOS での §3.5 の「死亡を確認した」は、R1・R2・R3・R6・R7・R8 を除いた主張である。

保証は P1〜P6 に依る。P1・P2・P4 は今の tracker と同じ前提で、P3・P5・P6 は XNU のソースで確かめた事実である。同じ PID を 2 回読む間（identity1 と identity2 の間、記録と後の巡の読取りの間）の再利用は、P4 により必ず見分けられる。見分けられないのは、t0 から記録のない PID の最初の読取りまでの間だけである（R1）。

**残余**

- **R1（一覧の後、最初の読取りの前の PID の再利用）**: 最後の巡の、記録のない PID p について、次がすべて起きる場合である。
  1. cookie を持つ A が t0 に p で生きていて、一覧に入る。
  2. A が、cookie を持つ子 C を作る。C は t0 に作成中だったか、t0 の後に作られる。
  3. A が t0 の後に exit して回収され、その巡で p を最初に読む前に、p が別のプロセス Q に割り当てられる。Q の実効 uid が自分と同じなら、Q は cookie を持たず、identity1 と identity2 が等しい。Q の実効 uid が違えば、identity1 の時点で検査を終える（identity2 は読まない）。
  4. C か C の子孫が走査の終わりまで生きていて、最後の巡で検査されない。

  **R1 に要る条件**: P5 の割当て器では、3 は、A が回収された後に PID の counter が p まで進んだときだけ起きる。counter は A の誕生のときに p にあった。そのため、A の誕生から p の最初の読取りまでに、counter は 100 から PID_MAX（記憶では 99999）までを一巡しなければならない。そのうち、t0 の counter の値から p までの分は、t0 から最初の読取りまでの間、つまり 1 回の `discover` の中（`limits.time`。既定 500 ms）で進まなければならない。
  - A が t0 の直前に作られた場合（すぐ fork して exit する、デーモン化の中間のプロセスなど）は、ほぼ一巡を 1 回の `discover` の中で進む必要がある。
  - A が長く生きていて、その間に counter が一巡近く進んでいた場合は、1 回の `discover` の中で要る fork は少なくてよい。step が長く、PID を多く使う Mac では、この形がありうる。
  - p が使用中の process group か session の ID なら、p は割り当てられない。A が setsid してから C を作り、C がその session に残る典型の二重 fork では、C が生きている間、R1 は起きない。
  - 1 巡で終わる走査（PID の主が変わらず、zombie もない環境）では、どの PID にも記録がないので、すべての PID に R1 の窓がある。
- **R2（kernel 内の容量で切られた一覧）**: 最後の一覧で、kernel が `allproc` の走査を kernel 内の容量 nprocs+20 で止め、その値が出力先の個数より小さい場合である。返った個数は出力先の個数より小さいので、`truncated` にならない。A が集められなければ見落とす。kernel がそこで止まるには、同じ実効 uid のプロセスの数が、呼出しの中で読んだ nprocs+20 に達しなければならない。nprocs が全利用者のプロセス数なら（記憶による）、nprocs を読んでから lock を取るまでの間に、他の実効 uid のプロセス（root の常駐プロセスなど）の数 + 20 個より多くのプロセスが増える必要がある。
- **起きたときの結果**: 死亡の主張が誤りになる。step は死んだものとして進み、C は作業領域に書きうる。モードの静止の確認・復旧表の running の行・回復の command の 4 も同じ探索を使うので、同じく「いない」と誤りうる。
- **今の tracker との比較**: 今の tracker も `proc_listpids` を 1 回呼び、同じ順に読む。そのため R1・R2 は、今も同じ形で残っている。今の tracker がこれらで見落とさないのは、無関係な zombie などで、もともと limited になっている場合だけである。
- **影響を受ける相手**: 協力しないプロセスは、もともと自分の環境から cookie を消せる（P1 の対象外）。R1・R2 で見落とすのは、たまたま時刻が重なった協力的なプロセスである。
- **閉じる候補（別 Issue）**: R1 は、記録のない PID の identity の開始時刻が、一覧の呼出しの前に測った時刻より前であることを確かめれば、閉じられる見込みがある（P5 により、t0 より前に作られて読取りの時点でも p にいるプロセスは、t0 に p にいた）。ただし、次を確かめる必要があるので、この unit では行わない。
  - 開始時刻とその時刻を同じ時計で比べられること。
  - 開始時刻を記録するのが PID の割当ての後であること。
  - 時計の巻き戻しの扱い。

**忙しい Mac での費用**（同じ `Limits` の中）

- 1 巡ごとに `proc_listpids` を 1 回行う。1 巡目の全部の検査では、一致しなかった同じ利用者の PID ごとに identity の読取りが 1 回増える。2 巡目以降は、記録と照合された PID ごとに identity の読取りが 1 回要る（zombie の記録では、これに照会が 1 回加わる）。オーナーの Mac では、1 巡あたり数百回の `proc_pidinfo` になる。全部の検査は、新しく現れた PID と、記録と合わなかった PID だけに行う。
- 新しい不在の主な出どころは、一覧の後、読取りの前に exit した短命のプロセス（`vanished`・`zombie`）である。記録と合わない PID（`reused`）は、P5 により、counter が一巡しない限り生じない。新しい不在が 1 つ出るたびに 1 巡増える。2 巡目以降の巡は短いので、出入りが止まれば数巡で終わる。
- 新しいプロセスが絶えず現れても、それだけでは巡は増えない。新しい不在を作るのは、一覧に載った後に消えたか、主が変わったものだけである。毎巡それが出続ける場合、探索は `limits.time`（既定 500 ms）で `time_exceeded` になり、limited になる。そのときの費用はその step が held になることであり、見落としではない。
- R1・R2 は受け入れるので、そのための追加の一覧や待ちはない。
- オーナーの Mac での受入で、`relistings`・`reused`・`zombie`・`vanished` と所要時間を記録する（下の受入）。所要時間は、R1 の窓の長さの上限でもある。

実機の zombie で照会の `p_starttime` が 0 なら、開始時刻で同一性を確かめられない。その場合は実装を止めて設計に戻り、黙って PID だけの比較にはしない。実行時に key が 0 のものは記録されず、毎巡全部の検査になる。zombie なら毎巡新しい不在になり、`time_exceeded` で limited に倒れる。

**自分の子の zombie**: 死亡確認では、探索（`custody.rs:126-130`）の前に leader を reap する（`custody.rs:113-114`、§3.5 の 3）。reap が失敗すれば、`?` で探索の前に抜ける。leader の子孫は、leader の exit で launchd に引き取られるので、rrx の子ではない。そのため、この経路で見える zombie は、launchd が回収する前の子孫か、無関係なプロセスの子である。万一、rrx 自身の未回収の子が zombie として見えても、死亡の主張は弱まらない。理由は次の 3 つである。

- zombie は何も実行できない。
- その子孫は、上の論証で A の側として扱われる（R1・R2 を除く。R1・R2 は zombie の扱いとは関係なく残る）。
- leader の reap は、探索とは別の条件（`leader_reaped`）で確かめている。

**消費者への影響**

| 消費者 | 影響 |
|---|---|
| 死亡確認（`custody.rs:126-134`） | 無関係な zombie や、走査中に exit した PID では held にならない。記録した PID の主が変わった場合と、環境の読取りの前後で主が変わった場合は、再列挙するか limited になる（今の tracker より強い）。R1・R2（今の tracker にも同じ形である）と §3.5.2 の R3・R6・R7・R8 は見落とし、死亡を誤って確認しうる。出入りが止まらなければ `time_exceeded` で held になる。条件の式は変えない |
| leader の birth の読取り | 探索を使わない（§3.5.3） |
| モードの静止の確認（`execution/owner.rs:291-301`） | 無関係な zombie で起動を拒否しなくなる。出入りが続けば、今と同じく limited で拒否する。R1・R2 に当たれば、cookie を持つ生きたプロセスがあっても静止とみなしうる。`discover` は §3.5.2 の判定を行わないので、環境が absent の cookie のプロセス（platform binary）も、今と同じく見落とす。渡す組（limited・件数・inspected）の形は変えない。`inspected` は、全巡の全部の検査の数になる（照合だけの読取りは数えない）。`discover` のまま（§13.3） |
| execution cleanup（`execution/cleanup.rs:83-122`） | `cookie_scan` の文字列に `zombie`・`vanished`・`reused`・`relistings` を足す。outcome の決め方（`remaining` だけを見る）は変わらない。zombie は `processes` に入らないので、terminate しない。`discover` のまま（§13.3） |
| 復旧表の running の行（§4.2）と、回復の command の 4（§4.4） | 同じ探索を使う。無関係な zombie では `custody_unobservable` にならず、拒否もされない。保証は死亡確認と同じである（`discover_launched` が limited でなく 0 件なら、R1・R2 と §3.5.2 の R3・R6・R7・R8 を除き、cookie を持つ生きたプロセスはない） |
| Linux（`crates/rrx-process-tracker/src/linux.rs`） | 変えない。今回の観測は macOS だけである。Linux での zombie と PID の再利用の扱いは実機で確かめておらず、この変更の範囲外とする |

**テスト**

決定的なテストは、zombie があるかどうかに依らない。次の 2 つを fake で検査する。

- 第 2 の照会の生の結果（状態・長さ・`kinfo_proc`）を受け取る純粋な関数 `classify_unreadable`。
- 走査の本体。一覧（PID の配列と返り値）・identity・照会・環境の読取りを差し替えられ、どの読取りが呼ばれたかを記録できるようにする。

| ID | 入力 | 期待 |
|---|---|---|
| ZB1 | 照会の状態が成功で、長さが 0 | vanished |
| ZB2 | 長さと pid が一致し、`p_stat = SZOMB` | zombie |
| ZB3 | `p_stat` が SIDL / SRUN / SSLEEP / SSTOP、および未知の値（0・6・`i8::MAX`） | 不明（元の counter） |
| ZB4 | 照会の失敗（ESRCH・EPERM・ENOMEM）、失敗で長さが 0、短い長さ、長すぎる長さ（`size_of::<kinfo_proc>() + 1`）、pid の不一致 | 不明（元の counter）。失敗で長さが 0 でも vanished にはならない |
| ZB5 | `zombie`・`vanished`・`reused`・`relistings` だけが 0 でない Coverage | `limited()` が false。`identity_unavailable` が 1 つでもあれば true |
| ZB6 | 1 巡目に z が zombie（記録 K）。2 巡目でも、z の照会が `p_starttime` K の zombie を返す | `relistings=1`。2 巡目は z の identity の読取りと照会を 1 回ずつだけ行い、環境は読まない。新しい不在にならず、3 巡目の一覧は呼ばれない。limited でなく、`processes` は空 |
| ZB7 | 1 巡目で cookie を持つ A が消え（vanished）、2 巡目の一覧に cookie を持つ新しい C がある | C が `processes` に入る |
| ZB8 | 1 巡目に、p が検査済み（生の記録 K_A）で、q は消えた PID。2 巡目の p の identity1 は birth K_B で、cookie を持つ | `reused=1`。全部の検査を続け、p が `processes` に入る |
| ZB9 | 1 巡目に z が zombie（記録 K_1）。2 巡目の照会は `p_starttime` K_2 の zombie を、3 巡目の照会は K_2 を返す | 2 巡目は `zombie=2` で次の巡に進み、K_2 を記録する。3 巡目で照合され、4 巡目の一覧は呼ばれない |
| ZB10 | 毎巡、新しい消えた PID が出る | `time_exceeded` で limited |
| ZB11 | 環境の読取りが失敗し、照会は「生きている」を返す | `environment_unavailable`（limited） |
| ZB12 | cookie の一致後に identity2 が失敗し、照会は zombie を返す | `identity_changed`（limited） |
| ZB13 | 1 巡目は p と q。p は検査済み（生の記録 K_A）で、q は消えた PID。2 巡目の一覧は p だけで、p の identity は birth K_B（同じ uid、cookie なし、identity2 も K_B）。3 巡目の一覧も p だけで、identity は K_B | 2 巡目は `reused=1` で、p の環境を 1 回読み、K_B を記録する。3 巡目は p の identity を 1 回だけ読み、環境は読まない。4 巡目の一覧は呼ばれない。終わりの値は `relistings=2`・`inspected=3`・`same_user=2` で、limited でない |
| ZB14 | identity1 が読め、環境は一致しない。identity2 は別の birth を返す。2 巡目に cookie を持つ C がある | `reused=1`。再列挙して、C が `processes` に入る |
| ZB15 | identity1 が失敗し、照会も失敗する。fake の環境・identity2 は、呼ばれれば成功を返す。同じ巡に、新しい不在を作る別の PID もある | `identity_unavailable=1`、limited。p の環境は読まない。`relistings=0`（次の巡を行わない） |
| ZB16 | 1 巡目で p が zombie（記録 K）。2 巡目は identity1 が失敗し、照会は短い長さを返す。3 巡目の fake は、呼ばれれば p を成功で返す | `identity_unavailable=1`、limited。3 巡目の一覧は呼ばれない |
| ZB17 | `proc_listpids` が次のどれかを返す: 0 以下・4 の倍数でない長さ・出力先より大きい長さ・同じ PID が 2 つ・出力先ちょうどの長さ | 順に `Err`・`Err`・`Err`・`Err`。最後だけは、返った PID を走査して `truncated`（limited） |
| ZB18 | 照会が返す zombie の `p_starttime` が 0 | 記録されず、毎巡新しい不在になり、`time_exceeded` で limited |
| ZB19 | 1 巡目は p と q。p は検査済み（生の記録 K_p）で、q は消えた PID。2 巡目の一覧は p だけで、identity は K_p | `vanished=1`、`relistings=1`。2 巡目は p の identity を 1 回だけ読み、環境・identity2・照会はない。3 巡目の一覧は呼ばれず、limited でない |
| ZB20 | 1 巡目の一覧は p だけ。identity1 の birth は K で、環境は一致しない。identity2 の読取りは 1 度だけ失敗する（fake の一時的な失敗）。2 巡目の一覧にも p がある。2 巡目の fake では identity の読取りがすべて成功して birth K を返し、環境は一致しない | 1 巡目は `reused=1` で、p は記録しない。2 巡目は `relistings=1` で、p に全部の検査をもう一度行う（identity を 2 回・環境を 1 回読む）。終わりの値は `inspected=2`・`same_user=2`・`reused=1`。3 巡目の一覧は呼ばれず、limited でなく、`processes` は空 |
| ZB21 | 1 巡目は p と q。p は検査済み（生の記録 K）で、q は消えた PID。2 巡目の一覧は p だけで、identity1 が失敗し、照会は `p_starttime` K の zombie を返す。3 巡目の一覧は p と、cookie を持つ C | 2 巡目は `zombie=1`（新しい不在）で、zombie の記録 K を作る。3 巡目で p は照合され、C が `processes` に入る。`relistings=2` |
| ZB22 | 1 巡目は p・z・q。p は検査済み、z は zombie、q は消えた PID。2 巡目の一覧は新しい r だけ（記録済みの zombie と p が入らない一覧の形）で、r は検査済みになる | 2 巡目は r だけを読み、p・z は読まない。`relistings=1` で、3 巡目の一覧は呼ばれず、limited でない |

macOS の実機テスト（`#[cfg(target_os = "macos")]`）では、zombie を自分で作る。

1. 先に探索を 1 回行い、基準の counter を記録する。
2. テスト用の cookie を環境に入れて `/usr/bin/true` を起動し、wait しない。第 2 の照会が zombie を返すまで、最大 5 秒待つ。返らなければ失敗とする。
3. 次を確かめる。
   - その PID の分類が zombie で、`p_starttime` が 0 でない。
   - `discover(cookie)` の `zombie` が 1 以上で、`processes` にその PID がない。
   - 基準が limited でなかったなら、今回も limited でない。基準が limited なら、counter を示して失敗させ、黙って飛ばさない。
4. Drop guard で必ず reap する。reap の後、その PID の分類が vanished になる。PID が再利用されていたら、失敗として報告する。

オーナーの Mac での受入では、長く残る zombie がある状態で次を確かめる。結果は、所要時間と `relistings`・`reused`・`zombie`・`vanished` とともに記録する。

- `discover` を 20 回行い、`zombie=1` かつ `limited()=false` を返した回数と、limited になった回の counter を記録する。
- パイプラインの step が死亡確認を通る。

**変異**

| 変異 | 落ちるテスト |
|---|---|
| `limited()` に `zombie`・`vanished`・`reused` のどれかが 0 でないことを入れる | ZB5、実機テストの 3 |
| 照会の失敗を vanished にする | ZB4 |
| 状態を見ず、長さ 0 なら vanished にする | ZB4（失敗で長さ 0） |
| `p_stat` を見ず、成功ならすべて zombie にする | ZB3 |
| 既知の非 zombie 状態だけを拒否し、それ以外を zombie にする | ZB3（未知の値） |
| 長さを `>=` で比べる | ZB4（長すぎる長さ） |
| pid の一致を見ない | ZB4 |
| 長さ 0 を不明にする | ZB1、実機テストの 4 |
| 再列挙をしない | ZB7、ZB14、ZB21 |
| 再列挙を 1 回で打ち切る | ZB10、ZB13 |
| 記録との照合を PID だけで行う（identity を読まない、または key を見ない） | ZB8、ZB9、ZB13 |
| 記録と合わない identity を新しい不在にしない | ZB13（`relistings=1`） |
| 記録と合わなかったときに、記録を消さない | ZB13（`time_exceeded`） |
| 記録の種類を見ず、key だけで照合する | ZB21 |
| 一致しなかったときに identity2 を読まない（今の形） | ZB14 |
| 照会の「不明」の後に、環境の読取りへ進む | ZB15 |
| limited になった後も再列挙を続ける、または巡ごとに counter を作り直す | ZB15、ZB16 |
| 既知の zombie の照会が「不明」でも照合済みにする | ZB16 |
| 一覧の長さの検査・重複の検査、または出力先ちょうどでの `truncated` を外す | ZB17 |
| key が 0 のものも記録する | ZB18 |
| 照合された PID にも毎巡全部の検査を行う | ZB6、ZB19（読取りの記録） |
| `reused` にした PID も記録する | ZB20（2 巡目に環境の読取りがない。`inspected=1`・`same_user=1`） |
| 一致しなかったとき、identity2 が読めなくても検査済みとして記録する | ZB20（`reused=0`・`relistings=0`） |
| 一覧にない記録済みの PID を新しい不在にする、または読む | ZB22 |
| 環境の読取りが失敗したとき、照会せずに zombie にする | ZB11 |
| cookie の一致後の失敗にも分類を当てる | ZB12 |
| zombie を `processes` に入れる | ZB6、実機テストの 3 |

#### 3.5.2 macOS の探索での環境を消されたプロセス

**問題**: 同じ利用者が起動した Apple の platform binary について、`sysctl(KERN_PROCARGS2)`（`macos.rs:148-180`）は成功する。しかし返るのは実行ファイルのパスと引数だけで、環境の文字列は消されている（オーナーの Mac での実測。§16 D20）。今の tracker は、これを「読めて一致しなかった」とする。`environment_unavailable` にならないので、`limited()` は false のままである。

Agent や test のコマンドが platform binary を起動し、それが group を抜けた場合を考える（例: `/usr/bin/perl -e 'use POSIX; POSIX::setsid(); …'`）。group を kill して空になった後、cookie の探索は一致 0 件で limited にならない。子孫が生きているのに、§3.5 の死亡確認が成り立ってしまう。P1（cookie は環境の引継ぎでだけ伝わる）は保たれているが、観測できない。この形のプロセスを、協力的でも見落とす。

**前提**（P1〜P6 に加える）

| ID | 前提 | 出典と確認 |
|---|---|---|
| P7 | `KERN_PROCARGS2` は、同じ利用者の platform binary について成功し、argc・実行パス・引数だけを返す。sandbox-exec の下でも同じである。platform binary でないプロセスでは、環境が返る | オーナーの Mac で実測した（`/bin/sleep`・`/usr/bin/perl`・setsid の後の perl・sandbox-exec の下の perl では cookie がない。Homebrew の `python3` では、sandbox-exec の下でも cookie がある） |
| P8 | `proc_bsdinfo` は `pbi_ppid`・`pbi_flags`・`pbi_start_tvsec/usec` を持つ。`kinfo_proc` は `kp_proc.p_starttime`・`p_stat`・`p_flag`・`p_pid`・`kp_eproc.e_ppid` を持つ。`PROC_FLAG_TRACED`=2、`P_TRACED`=0x800、`SIDL`〜`SZOMB`=1〜5 | 公開 SDK の `sys/proc_info.h`・`sys/sysctl.h`・`sys/proc.h` で確かめた |
| P9 | プロセスの親が変わるのは、親が exit して launchd（pid 1）に引き取られるときだけである。引き取りは親の exit の処理の中で行われ、親が回収されるより前に終わる。ptrace の attach は除き、追跡中の印で見分ける。subreaper はない | 記憶による（XNU の `proc_exit`）。HE5 で、抜けた子孫の ppid が 1 になることを確かめる。2 文目（引き取りが回収より前に終わる）は、必須の前提確認 HE10 で確かめる。§3.5.2 の系譜の (e) は 2 文目に依る |
| P10 | 利用者でない UID のプロセス（root の `login` など）も、`sysctl(KERN_PROC_PID)` で読める | 未確認。HE2 |
| P11 | exec で開始時刻は変わらない | 未確認。HE4。外れたら §3.5.3 の照合を止めて設計に戻る |
| P12 | `kern.bootsessionuuid` は boot ごとに変わり、同じ boot の中では変わらない。rrxd（sandbox の外）から読める | 未確認。HE3。読めない環境では、別の boot の除外を行わない（安全側。回復が止まりうる） |
| P13 | 開始時刻は fork の時点の壁時計である。起動の記録と開始時刻は同じ時計で比べられる。時計が M より大きく戻らない限り、起動の後に生まれたプロセスの開始時刻は F 以上になる | 開始時刻が壁時計であることは記憶による。戻りは残余 R3 |
| P14 | 任意のコードを実行できる汎用のツールとインタプリタの platform binary（`sh`・`bash`・`zsh`・`perl`・`python3`・`ruby`・`osascript` など）は、`/bin`・`/usr/bin`・`/sbin`・`/usr/sbin` にある。launchd が起動する Apple のバックグラウンドの処理（`mdworker_shared`・`com.apple.iCloudHelper`・`remindd` など）は、`/System/Library` と `/usr/libexec` にある | オーナーの Mac で実測した。step の起動から 5 分・30 分・60 分の間に生まれ、祖先をたどると launchd に達し、環境が absent の同じ利用者のプロセスは 8・9・10 個で、すべて `/System/Library` か `/usr/libexec` にあった。4 つの場所のものは、どの長さでも 0 個だった。`/System/Library` の下にも、任意のコードを実行できるもの（JavaScriptCore の `jsc` など）がある（R7） |

P9〜P12 の確認が外れたら、実装を止めて設計に戻る。黙って別の手段に替えない。P9 の 2 文目（引き取りが回収より前に終わること。HE10）が外れた場合は、設計を見直すまで P9 に依らず、系譜による除外（下の「除外するもの」の 2 つ目）を行わない。系譜の判定の 4 に進むプロセス（環境が absent で、開始時刻が F 以上で、場所が 4 つの場所のどれかか読めないもの）は、すべて「確認できない」（limited）とする。

**起動の記録（launch）**

- launcher は tx1 の直前に `rrx_process_tracker::launch_mark()` を呼ぶ。読むのは次の 2 つである。
  - 壁時計（`CLOCK_REALTIME`）の秒・µs
  - `kern.bootsessionuuid`（読めなければ null）
- tx1（`pipeline_begin_launch`）が、process の kind の step の body に `launch = {"wall":"<sec>:<usec>","boot":"<uuid>"|null}` を書く。壁時計を読めなければ tx1 を行わず、起動しない。
- trigger は次を拒否する。
  - process の kind の planned → launching で `launch.wall` が text でないもの
  - 書いた後に `launch` を変えること
- 「起動の後に生まれた」の境 F を次で定める。M = 60 秒（`LAUNCH_CLOCK_MARGIN`）とする。
  - F = min(`launch.wall`, 記録した leader の birth（あれば）) − M
  - 開始時刻が F 以上のプロセスを「起動の後に生まれた」とする。tx1 は spawn より前なので、時計が戻らなければ、leader とその子孫はすべて F + M 以上になる。

**環境がないことの判定**

- `KERN_PROCARGS2` を読み、`environment_offset` が返した位置から後に、0 でない byte が 1 つもない場合を「環境がない（absent）」とする。末尾までちょうどの場合と、NUL の詰め物だけの場合を含む。
- `cookie_matches` は 3 値（一致・不一致・absent）を返す。
- 本当に空の環境で起動されたプロセスも同じ扱いにする（安全側）。
- argc の解釈に失敗する場合は、今と同じく `environment_unavailable` になる。

**系譜の判定**（`discover_launched` だけで行う）

§3.5.1 の巡の扱いの 5 で環境が absent だったプロセス X を、次の順に扱う。birth・ppid・flags は identity の読取り（`PROC_PIDTBSDINFO`）から取る。

1. `other_boot`（下の開始時の確認）なら、追加の読取りをせず §3.5.1 の 6 に進む。identity2 が identity1 と等しければ、`environment_absent += 1` とし、今と同じく検査済み（生の記録）にして終える。
2. identity1 の birth < F なら、追加の読取りをせず §3.5.1 の 6 に進む。identity2 が等しければ、`environment_absent += 1` とし、除外して生の記録を作る。
3. それ以外は、identity2 の前に、実行ファイルの場所 L1（`proc_pidpath`、`PROC_PIDPATHINFO`）・環境（env2）・場所 L2 をこの順に読む。続けて §3.5.1 の 6 のとおり identity2 を読み、等しくないか読めなければ、今と同じく `reused` とする。等しければ `environment_absent += 1` とし、次のどれかにする。
   - env2 が absent でない（一致・不一致・読めない）か、L1 と L2 がどちらも読めて値が違う: 「確認できない」とする（6）。
   - L1 と L2 がどちらも読めて等しく、その親ディレクトリが `/bin`・`/usr/bin`・`/sbin`・`/usr/sbin` のどれとも一致しない: `environment_absent_elsewhere += 1` として除外する。記録は作らない（残余 R7・R8。オーナーの決定）。
   - それ以外（親ディレクトリが 4 つのどれかと一致する。L1・L2 のどちらかが読めない場合も、安全側としてここに入る）: 4 に進む。

   P11 により、exec は PID と birth を変えない。そのため identity の比較だけでは、環境と場所が同じ実行イメージの観測かどうか分からない。env2 を L1 と L2 の間に読み、L1 = L2 を求めることで、読取りの間の 1 回の exec を見分ける（2 回の exec は残余 R8）。最初の環境の読取りは、この読み直しを始める合図にだけ使う。
4. X の `pbi_flags` に `PROC_FLAG_TRACED` があれば「確認できない」とする。
5. 子 c = X として、次の (a)〜(h) を最大 32 段（`ANCESTRY_DEPTH`）繰り返す。q は c の親の PID である。
   - (a) q が 0・1・自分（rrxd）の PID、またはこの判定で既に訪れた PID なら「確認できない」。
   - (b) `check_time` の後に `sysctl(KERN_PROC_PID, q)` を行い（`query_pid`）、`ancestors_read += 1` とする。
   - (c) 状態が 0 でない、長さが `size_of::<kinfo_proc>()` と一致しない、または `p_pid ≠ q` なら「確認できない」。長さ 0（消えた）もここに入る。
   - (d) `p_stat` が SRUN・SSLEEP・SSTOP のどれでもない（SZOMB・SIDL・未知の値）か、`p_flag` に `P_TRACED` があれば「確認できない」。
   - (e) `p_starttime` が 0 なら「確認できない」。続けて c を読み直す（c = X なら `proc_pidinfo(PROC_PIDTBSDINFO)`、祖先なら `query_pid`。c を前に読んだのと同じ出どころである）。読めない、c が同じプロセスでない（X では identity が identity2 と違い、祖先では開始時刻が c の birth と違う）、親の PID が q でない、追跡中の印がある、祖先で `p_stat` が SRUN・SSLEEP・SSTOP のどれでもない、のどれかなら「確認できない」。これで、q を読んだときの q が c の本当の親であることを確かめる（下の補題）。c の読み直しは `ancestors_read` に数えない。q と c の開始時刻の前後は比べない（時計が少し戻るだけで逆転するので、安全性の根拠にしない）。
   - (f) `p_starttime` < F なら、X を除外する（起動前に生まれた、生きている祖先）。記録は作らない。
   - (g) それ以外は、c =（q、`p_starttime`、`e_ppid`）として続ける。
   - (h) 32 段を超えたら「確認できない」。
6. 「確認できない」場合は次のようにする。
   - `ancestry_unconfirmed += 1` とする。`Discovery.unconfirmed` に X の `Identity` を足す（最大 16 件）。記録は作らない。
   - その巡の残りの PID は今と同じく検査し（一致の発見を続ける）、巡の終わりで limited として返す（再列挙しない）。

**除外するもの**: 次の 3 つだけである。

- 開始時刻が F より前のプロセス（生の記録を作る）
- 系譜をたどって、開始時刻が F より前で生きている祖先 Y に達するプロセス。Y は pid 1 と rrxd 自身ではないものに限る。ほかの UID の祖先も含む。
- 場所 L1 = L2 の親ディレクトリが 4 つの場所の外のプロセス（残余 R7・R8）

途中の祖先がすべて F 以上で pid 1 か rrxd に達するもの、読めない・zombie・親を確かめられない祖先を持つもの、追跡中のもの、環境と場所を同じ実行イメージに結び付けられないものは、limited にする。

**§3.5.1（D17）との関係**

- `ancestry_unconfirmed` と `clock_behind` は `limited()` に入れる。`environment_absent`・`ancestors_read`・`other_boot` は入れない。
- どれも新しい不在ではなく、再列挙を起こさない。limited になった後は、今と同じく次の巡を行わない。
- §3.5.1 の記録との照合（identity だけを読み、環境を読まない）は、記録の根拠が exec の後も変わらない場合にだけ使う。生の記録を作ってよいのは次の 3 つで、zombie の記録は今と同じである。
  - 環境が読めて cookie が一致しなかったもの（P1 により、exec の後も cookie は現れない）と、`processes` に入れたもの（§3.5.1 のまま）
  - absent で birth < F のもの（上の 2。birth は exec で変わらない。P11）
  - `other_boot` の absent のもの（上の 1。前の boot のプロセスは存在しない）
- 場所か系譜で除外した absent のプロセスには記録を作らず、後の巡でも全部の検査（環境・場所・系譜）をやり直す。場所は、PID と birth を保ったままの exec で変わるからである。系譜は exec では変わらないが、記録を使ってよい場合を上の 3 つに絞るため、同じ扱いにする。新しい不在には数えないので、再列挙は起こさない。
- 祖先の読取り、c の読み直し、場所と env2 の読取りは `inspected`・`same_user` に数えず、記録も作らない。時間は同じ `limits.time` の中で使う。
- 開始時の確認（`discover_launched` の最初に 1 度だけ、この順に行う）:
  1. 今の `kern.bootsessionuuid` を読む。launch の boot と今の値がどちらも読めて違えば、`other_boot = true` とし、時計の確認を行わない。前の boot のプロセスは存在できないので（P12）、前の boot の壁時計と今の時計を比べる意味がない。
  2. それ以外（同じ boot、今の値が読めない、launch の boot が null）は、今の壁時計が F より前なら `clock_behind = true`（limited）とする。境は F だけを使う（§16 D20 の要約と同じ）。走査は続け、一致は数える。
- `discover`（launch なし）は、`environment_absent` を数えるだけで、場所・env2 の読取りも、系譜の判定も、時計の確認も行わない。今の挙動と同じである。
- 死亡確認のやり直し（§3.5。`DEATH_BOUND` の中で limited の結果だけを独立に探索し直す）は変えない。短命の無関係なプロセスによる limited は、このやり直しで解ける。

**公開 API の追加**（`crates/rrx-process-tracker/src/lib.rs`）

- `pub struct Launch { pub wall: (u64,u64), pub boot: Option<String>, pub leader_birth: Option<(u64,u64)> }`
- `pub fn launch_mark() -> io::Result<Launch>`（`leader_birth` は None）
- `pub fn discover_launched(cookie: &Cookie, limits: Limits, launch: &Launch) -> io::Result<Discovery>`
- `Coverage` に足すもの: `environment_absent: usize`・`environment_absent_elsewhere: usize`（`limited()` に入れない）・`ancestry_unconfirmed: usize`・`ancestors_read: usize`・`clock_behind: bool`・`other_boot: bool`
- `Discovery` に足すもの: `unconfirmed: Vec<Identity>`（最大 16 件。観測の metadata であり、signal の権限ではない）
- macOS では、見つかったプロセスにも確認できないプロセスにも signal を送らない（D12）。
- Linux は、`launch` を受け取っても使わない。新しい counter は常に 0 / false になる。

**安全性の論証（§3.5.1 の論証への追加）**

§3.5.1 の論証の「記録がない場合」の 6 で、cookie が一致せず identity2 が等しい Q = A について:

- 環境が読めた場合は、今と同じく矛盾する。
- 環境が absent の場合は、次の補題により、A は「確認できない」（`ancestry_unconfirmed`、limited）か `time_exceeded`（limited）になる。補題の条件が成り立たない場合は、残余 R3・R7・R8 である。

「照合済み」の生の記録のうち absent で作るものは、上の 1・2 のときだけである。2 の記録は birth < F で作り、birth は exec で変わらない（P11）。補題の証明の 2 つ目と同じ理由で、時計の条件の下では A のものではありえない。1 の記録では、A が存在しない（補題の boot の条件）。場所か系譜で除外したものは記録しないので、最後の巡で「記録がない場合」として全部の検査を受け、その読取りに補題を当てる。

**補題**: 次の 3 つの条件を満たすとき、cookie を持つ生きたプロセス A（leader か leader の子孫）で、最後の巡に環境が absent と読まれ identity2 が等しいものは、除外されない。到達先は「確認できない」か時間切れ（どちらも limited）だけである。

- 場所: 最後の巡の A の読取りで、L1 と L2 がどちらも読めて等しく、その親ディレクトリが 4 つの場所の外、ということがない。そうであれば A は除外され、残余 R7（A がその場所の platform binary）か R8（読取りの窓の中の 2 回の exec）に当たる。
- boot: `other_boot` でない。`other_boot` なら、P12 により step は前の boot で起動しているので、A は存在しない。
- 時計: step の起動から最後の巡までに、壁時計が M より大きく戻っていない（P13。外れれば残余 R3）。今の時計が F より前なら `clock_behind` で limited になるので、それ以外の場合を考えればよい。

証明:

- 3 で、env2 が absent でないか L1 と L2 が違えば「確認できない」になる。場所の条件により、それ以外は 4 に進む。
- A の fork の系譜は、A → … → leader → rrxd である。leader とその子孫はすべて、leader の誕生の後に生まれている。時計の条件と P13 により、その開始時刻は F 以上である。したがって 2 では除外されない。
- 系譜をたどる各段で、(e) の読み直しは、c が同じプロセスのまま（P4）、q を読む前と後の両方で親の PID が q であることを確かめる。P9 により、c の親が exit すれば、その親が回収されるより前に c は pid 1 に引き取られ、親の PID が q に戻ることはない。そのため、q を読んだとき c の元の親はまだ回収されておらず、P5 により PID q は元の親のものだった。つまり、読んだ q は c の本当の親である。この確かめは、開始時刻の前後にも時計にも依らない。
- 本当の親は、P9 により fork の親か pid 1 である。そのため、A から pid 1 か rrxd に達するまでに出会うのは、leader の子孫か leader だけであり、時計の条件によりその開始時刻はすべて F 以上である。leader が reap されていれば、その子は pid 1 に引き取られている。生きていれば、その親は rrxd である。
- したがって、F より前に生まれた生きた祖先 Y（pid 1・rrxd 以外）に (f) で達することはない。到達先は 3・4・(a)・(c)〜(e)・(h) の「確認できない」か、時間切れだけである。

**残余**（R3・R5・R6・R7・R8 は、どれもオーナーが受け入れた）

- **R3（時計の戻り）**: step の間に、壁時計が M（60 秒）より大きく戻り、その間に子孫が生まれた場合。
  - 戻ったままなら `clock_behind` で limited になる（別の boot と確かめられた場合は、時計を確かめない）。
  - 戻った後に進み直していれば、探索の時点では見分けられない。
  - 時計に依るのは、2 の F との比較と、系譜の (f) の比較である。(e) の親の確かめは時計に依らない。
- **R5（子孫でない代理の起動）**: step が、起動前からあるプロセスにプロセスを作らせた場合（`launchctl submit`・`open -a`・XPC・Terminal への AppleScript など）。そのプロセスは子孫ではなく cookie を持たない。今の tracker と同じく対象外（P1）である。platform binary で launchd の子なら、偶然 limited になる。cookie を持たないので、補題の対象でなく、§3.5.1 の主張（cookie を持つ生きたプロセスがない）の例外にも数えない。死亡確認がこの処理の終わりを確かめないことは、今と同じである。
- **R6（協力しないプロセス）**: platform binary が自分の引数の領域を書き換えて、環境があるように見せる場合。P1 と同じく対象外とする。環境が読めて一致しないとして扱われるので補題は当たらず、§3.5.1 の主張の例外になる。
- **R7（4 つの場所の外の platform binary）**: `/System/Library` や `/usr/libexec` などにある platform binary で、任意のコードを実行できるもの（JavaScriptCore の `jsc` など）を使って group を抜けた子孫は、環境が absent でも除外されるので、見落とす。可用性を保つために、オーナーが受け入れた残余である（4 つの場所に絞らないと、オーナーの Mac では 5 分を超える step がほぼすべて held になると実測した）。
- **R8（読取りの窓の中の 2 回の exec）**: 最後の巡で、cookie を持つ X が、L1 の後に 4 つの場所の platform binary へ exec し、env2 の後、L2 の前に、L1 と同じ実行ファイルへ exec し直した場合である。L1 = L2 が 4 つの場所の外なら除外するので、L1 の実行ファイルが platform binary でなければ、環境を観測できるはずのプロセスを見落とす。P11 により PID と birth では exec を見分けられず、公開 SDK に exec の回数を表す値もないので、閉じない。1 回の exec は L1 ≠ L2 か env2 で見分け（EA24）、巡の間の exec は記録を使わずに毎巡読み直すことで見分ける（EA25）。R1 と同じく、読取りと時刻がたまたま重なった協力的なプロセスだけが当たる。
- **R1・R2**: 変わらない（§3.5.1）。
- **可用性の残余**（見落としではなく held の原因）:
  - 同じ利用者で、launchd の子として F の後に起動した platform binary が、`DEATH_BOUND` を超えて生きている場合。ユーザーが Dock・Spotlight から開いた Apple のアプリ、その XPC service（Safari の WebContent など）、要求で起動するエージェント（`mdworker_shared` など）が当たる。
  - rrxd 自身の子（rrx の Git、EPERM のときの `ps`）は一時的なので、やり直しで解ける。
  - 祖先が読めない環境（P10 が外れる場合）。

**忙しい Mac での費用**（オーナーの Mac での実測。HE9 で引き続き測る）

- 場所で絞る前: step の起動から 10 秒・5 分・30 分・60 分で、確認できないと判定されるプロセスは 0・5・6・7 個だった（`mdworker_shared`・`spotlightknowledged.updater`・`com.apple.iCloudHelper`・`remindd`）。5 分を超える step はほぼすべて held になる。
- 場所で絞った後: どの長さでも 0 個だった。held は、利用者が step の間に 4 つの場所の platform binary を launchd の子として残した場合（Terminal を閉じた後の `nohup` など）に限られる見込みである。
- 1 回の探索で増えるのは、absent で F 以上のプロセスごとに `proc_pidpath` 2 回と環境の読取り 1 回、そのうち 4 つの場所のものでは、さらに 1 段ごとに 2 回の読取り（親の照会と子の読み直し。通常 1〜3 段、最大 32 段）である。場所か系譜で除外したものは記録しないので、巡ごとにこれを繰り返す。`limits.time` は変えない。
- held の step の回復（§4.4 の 4）は、F の後に生まれた対象のプロセスが残る限り拒否される。拒否の応答に `unconfirmed` の PID を示すので、運用者はそれらを終了させるか、再起動する。再起動の後は `other_boot` で判定が外れ、回復できる。

**テスト**（fake。`Source` の環境を 3 値にし、祖先の照会（`e_ppid` を含む）・場所・自分の PID・時計・boot を差し替えられるようにする。読取りは、呼ばれた順に記録する。場所の既定は L1 = L2 = `/usr/bin/x`、env2 の既定は最初の環境の読取りと同じ値、読み直しの既定は前の読取りと同じ値である）

| ID | 入力 | 期待 |
|---|---|---|
| EA1 | absent、birth < F | 除外。`environment_absent=1`、limited でない。2 巡目は identity を 1 回読むだけ |
| EA2 | absent、birth ≥ F、ppid 1 | `ancestry_unconfirmed=1`、limited、`unconfirmed` に X、`relistings=0` |
| EA3 | absent、F 以上、親 q が SSLEEP で開始 < F | 除外。`ancestors_read=1` |
| EA4 | X → q（F 以上、生）→ r（開始 < F、生） | 除外。`ancestors_read=2` |
| EA5 | X → q（F 以上）→ 1 | 確認できない |
| EA6 | ppid が自分の PID / X → q（F 以上）→ 自分 | 確認できない。前者は祖先を読まない |
| EA7 | 祖先の照会が、失敗・長さ 0・短い長さ・長すぎる長さ・pid の不一致・SZOMB・SIDL・未知の状態・開始 0・`P_TRACED` | それぞれ確認できない |
| EA8 | X → q（生）→ r（開始 < F、生）。q を読んだ後の X の読み直しが、ppid 1 / 別の birth（ppid は q のまま）/ 失敗 / `PROC_FLAG_TRACED` を返す。r を読んだ後の q の読み直しが、`e_ppid` 1 / 別の開始（`e_ppid` は r のまま）/ 長さ 0 / SZOMB を返す | それぞれ確認できない（親を確かめられない）。X の読み直しの場合は r を読まない（`ancestors_read=1`） |
| EA9 | 循環（q の ppid が X）/ 33 段の鎖 / 32 段目で開始 < F | 確認できない / 確認できない / 除外 |
| EA10 | 境界: birth = F / birth = F − 1µs。F は min(wall, leader_birth) − M（leader_birth が wall より前の場合を含む） | 確認の対象 / 除外 |
| EA11 | absent・F 以上・ppid 1 の X がある。boot が A と B / 今の boot が読めない / launch の boot が null | `other_boot`・limited でなく、場所も祖先も読まない / 確認できない / 確認できない |
| EA12 | プロセスなし、今の時計 < F。boot が A と A / 今の boot が読めない / launch の boot が null / A と B | `clock_behind`・limited / 同じ / 同じ / `other_boot`、`clock_behind` は false で limited でない |
| EA13 | `discover`（launch なし）、absent・ppid 1 | `environment_absent=1`、limited でなく、祖先を読まない |
| EA14 | 環境があり一致しない、F 以上、ppid 1 | 今と同じく検査済み（系譜を見ない） |
| EA15 | 系譜の途中で時間切れ | `time_exceeded` |
| EA16 | 1 巡目に、birth < F で除外した X1、系譜で除外した X2（EA4 の形）、場所で除外した X3（EA21 の形）がある。別の PID が消えて再列挙し、2 巡目の一覧も X1〜X3 を含む | 2 巡目の X1 は identity の 1 回だけ。X2・X3 は全部の検査をやり直し、環境・場所（X2 は祖先も）を読む |
| EA17 | 新しい counter だけを持つ Coverage | `ancestry_unconfirmed`・`clock_behind` だけが limited にする |
| EA18 | `KERN_PROCARGS2` の byte: 引数まで / 引数＋NUL / 引数＋`A=1` / 引数＋cookie | absent / absent / 不一致 / 一致 |
| EA19 | 確認できないものが 20 件 | `ancestry_unconfirmed=20`、`unconfirmed` は 16 件 |
| EA20 | X の `PROC_FLAG_TRACED` | 確認できない |
| EA21 | absent、F 以上、ppid 1、場所が `/System/Library/...` / `/usr/libexec` | 除外。`environment_absent_elsewhere=1`、limited でない、祖先を読まない |
| EA22 | absent、F 以上、ppid 1。L1 = L2 の親ディレクトリが `/bin` / `/usr/bin` / `/sbin` / `/usr/sbin`（`/bin/x` など） / L1 が読めない / L2 が読めない | それぞれ確認できない |
| EA23 | 親の PID の再利用と小さな時計の戻り: F = 940、X の birth 2000、identity2 の ppid は q。q の照会は、開始 1999.5 の生きた Q を返す（`e_ppid` は開始 100 の生きた Y）。その後の X の読み直しは ppid 1 を返す（ppid の読取りの後に元の親が回収され、時計が 1 秒戻った後に Q が q を得た） | 確認できない、limited。Y を読まない（`ancestors_read=1`） |
| EA24 | 読取りの間の 1 回の exec: absent、F 以上、ppid 1。L1 = `/usr/bin/perl`・env2 absent・L2 = `/opt/homebrew/bin/python3` / L1 = `/opt/homebrew/bin/python3`・env2 absent・L2 = `/usr/bin/perl` / L1 = L2 = `/opt/homebrew/bin/python3`・env2 が cookie を含む / env2 が `A=1` / env2 が読めない | それぞれ確認できない（limited）。どれも除外しない |
| EA25 | 巡の間の exec: 1 巡目の X は absent、F 以上、L1 = L2 = `/System/Library/.../x`（除外）。別の PID が消える。2 巡目の X は同じ PID・birth で、環境に cookie がある | 2 巡目に X の環境を読み、X が `processes` に入る |
| EA26 | identity2 の後の PID の再利用: X は absent、F 以上、L1 = L2 = `/usr/bin/perl`、ppid q。q の照会は開始 < F の生きたプロセスを返す。fake は、identity2 より後の X の読取りに、別の birth と別の親を返し、`proc_pidpath` には `/System/Library/.../x` を返す | 確認できない（(e) の読み直し）。identity2 の後に、X の `proc_pidpath` と環境を呼ばない |
| EA27 | 時計の境: wall 1000、leader_birth 970（F = 910）、同じ boot、プロセスなし。今の時計 930 / 910 / 909.999999 | limited でない / limited でない / `clock_behind`、limited |
| EA28 | absent、F 以上、ppid 1。L1 = L2 が `/usr/bin/x/y`（下位のディレクトリ）/ `/usr/binx/y` / `/binx/y` / `/usr/sbin2/y` / `/usr/local/bin/y`（隣接の名前） | それぞれ除外。`environment_absent_elsewhere=1`、祖先を読まない |

**macOS の実機テスト**（`#[cfg(target_os = "macos")]`。HE1〜HE4 と HE10 は前提の確認で、外れたら実装を止めて設計に戻る。HE10 が外れた場合の扱いは、§3.5.2 の前提の表の後の段落による）

1. **HE1**: cookie を環境に入れて起動した `/bin/sleep` が absent に分類される。テストの実行ファイル自身（platform binary でない）は一致に分類される。
2. **HE2**: pid 1 の `KERN_PROC_PID` が、長さ・pid とも一致して成功する。自分の `e_ppid` が `getppid()` と等しい。`e_ppid` の offset は SDK から求め、layout テスト（`b4_zb_sdk_layout` と同じ流儀）で固定する。
3. **HE3**: `kern.bootsessionuuid` が 36 文字で、2 回の読取りで同じ値になる。
4. **HE4**: Seatbelt の下で `/usr/bin/sandbox-exec … /usr/bin/perl -e '$|=1; print "r"; sysread(STDIN, $b, 1); exec "/bin/sleep", "1"'` を起動する。stdin と stdout は pipe にする。
   - 子が stdout に `r` を書くまで待つ（同期点。sandbox-exec から perl への exec は済み、perl は stdin を待っている）。`owned_leader` の birth B0 と、`proc_pidpath` が `/usr/bin/perl` であることを読む。
   - stdin に 1 byte 書いて exec させる。同じ PID の `proc_pidpath` が `/bin/sleep` になるまで待ち（最大 5 秒。ならなければ失敗）、birth B1 を読む。
   - exit して zombie になった後、同じ PID の birth B2 を読む。
   - B0 = B1 = B2 を確かめる。
5. **HE5**: `launch_mark()` の後、cookie を入れ、`process_group(0)` で `/usr/bin/perl -e 'use POSIX; if (fork) { exit } POSIX::setsid(); sleep 30'` を起動する。leader の exit・group の kill・reap・group が空になることを待つ。
   - 抜けた子の ppid が 1 である（P9）。
   - `discover` は limited でなく、一致 0 件である（欠陥の再現）。
   - `discover_launched` は `ancestry_unconfirmed ≥ 1` で limited になり、`unconfirmed` に子の PID がある。
   - `DEATH_BOUND` の `discover_death` が Err になる。
   - 最後に、記録した PID と birth を照合してから子を kill する。
6. **HE6**: HE5 と同じ形を、テストの実行ファイル（platform binary でない）で行う。`processes` に子が入る。
7. **HE7**: F の後、cookie なしで別の group から、0.3 秒で exit する perl の二重 fork を作る。直後の `discover_death` が `DEATH_BOUND` の中で Ok になる。やり直しの回数を記録する。
8. **HE8**: パイプラインで、Seatbelt の下の test step `/usr/bin/perl -e 'exit 0'` を実行する。
   - tx2 が birth を記録する。
   - `death_evidence.launch_checked`・`leader_birth_matched` が true の exited になる。
   - 合格として採用される。
   - leader が tx2 の前に zombie になっていても、同じ結果になる（テストは spawn の直後の checkpoint で leader が zombie になるのを待ってから進める）。
   - 同じ形で HE5 のコマンドを test にすると、step は held（`death_unconfirmed`）になる。
9. **HE9**（受入、オーナーの Mac）: 普段の利用の中で、F を 10 秒・5 分・30 分・60 分前にして `discover_launched` を 20 回ずつ行う。limited になった回数・`ancestry_unconfirmed`・`ancestors_read`・所要時間を記録する。
10. **HE10**（P9 の 2 文目の前提確認）: テストの実行ファイルが、stdout を pipe にして `/usr/bin/perl -e 'if (my $g = fork) { print "$g\n"; exit } sleep 30'`（親 P）を起動する。P は子 G を作り、G の PID を書いて exit する。テストは P を回収しない（P は zombie のまま残る）。
   - G の PID を読んだら、P の `KERN_PROC_PID` と G の `proc_pidinfo(PROC_PIDTBSDINFO)` を、この順に交互に読み続ける（最大 5 秒）。P の `p_stat` が SZOMB と読めたら、その直後の G の読取りで、`pbi_ppid` が 1 であり、G の birth が最初の読取りと等しい（同じプロセス）ことを確かめる。その後、P をもう一度読み、まだ SZOMB である（回収されていない）ことを確かめる。
   - P を回収し、記録した PID と birth を照合してから G を kill する。
   - これを 1000 回繰り返す。1 回でも、zombie の P の後に読んだ G の `pbi_ppid` が 1 でなければ、または P が 5 秒の中で SZOMB と読めなければ、失敗とする。繰り返した回数と、P がまだ zombie でないときに G の `pbi_ppid` が P だった観測の回数（競合の窓を実際に読んだことの目安）を記録する。
   - P9 が要るのは、引き取りが回収より前に終わることである。HE10 は、それより強い「zombie として見える時点で、引き取りが済んでいる」を確かめる（回収は zombie の後にしか起きない）。XNU のソースは行まで確かめていないので根拠に数えず、この実機の確認を必須とする。
   - 外れたら、P9 の 2 文目に依らない。§3.5.2 の前提の表の後の段落のとおり、系譜による除外を行わず、設計に戻る。

**変異**

| 変異 | 落ちるテスト |
|---|---|
| absent を今と同じ不一致にする | EA2、HE5 |
| `ancestry_unconfirmed` を `limited()` から外す / `environment_absent` を入れる | EA17、EA2 / EA1、EA13 |
| pid 1 を起動前の祖先として受け入れる | EA2、EA5 |
| 自分の PID を祖先として受け入れる | EA6 |
| (e) の子の読み直しを外す / 読み直しで親の PID を比べない / 読み直しで同じプロセスかを比べない | EA8、EA23 / EA8（ppid 1）、EA23 / EA8（別の birth） |
| zombie・SIDL・未知の状態の祖先を生きているとする | EA7 |
| `P_TRACED` / `PROC_FLAG_TRACED` を見ない | EA7、EA20 |
| 深さの上限・訪問済みの検査を外す | EA9 |
| M を引かない / ≥ を > にする / leader_birth との min を取らない | EA10 |
| boot が読めないときも別の boot とする | EA11、EA12 |
| 時計の確認を boot の比較より前に行う（別の boot でも `clock_behind` にする） | EA12（A と B） |
| 時計の確認の境を `launch.wall` − M にする | EA27 |
| `clock_behind` を limited にしない | EA12 |
| birth < F で除外した X を記録しない（毎巡読む） | EA1、EA16 |
| 場所か系譜で除外した X を記録し、後の巡で照合だけにする | EA16、EA25 |
| env2 を読まない（最初の環境の読取りを使う） | EA24（env2 が cookie を含む） |
| L1 か L2 の片方だけを読む / L1 と L2 を比べない | EA24（L1 と L2 の 2 つの順） |
| 場所を identity2 の後に読む | EA26 |
| `discover` でも系譜を判定する | EA13 |
| `unconfirmed` の上限を外す | EA19 |
| 場所の判定を外す（すべてを対象にする） / 4 つの場所を除外にする | EA21 / EA22 |
| 4 つの場所のどれか 1 つを、単独で対象から外す（4 通り） | EA22（その場所の場合） |
| 親ディレクトリでなく、前方一致か祖先のディレクトリで比べる | EA28 |
| `proc_pidpath` の失敗を除外にする | EA22 |

#### 3.5.3 leader の birth と cookie の確認

- **読む場所**: launcher は、`ProcessGroup::new` の直後、tx2 の前に、test の step なら `rrx_process_tracker::owned_test_leader(pid, cookie)`、Agent の step なら `owned_leader(pid, cookie)` を `spawn_blocking` で呼ぶ。cookie の探索で leader を探す `custody::birth` は使わない。
- **読み方（macOS）**:
  - `query_pid(pid)`（`KERN_PROC_PID`）を読む。受け入れるのは、状態 0・長さが一致・`p_pid == pid`・`p_stat` が SRUN / SSLEEP / SSTOP / SZOMB・開始が 0 でない、の全部を満たす場合だけである。birth は `p_starttime`、`zombie` は SZOMB で返す。それ以外は Err にする。
  - 記録の形（`"<sec>:<usec>"`）は変えない。`b4_host_listed_start_time_matches_identity` により、`proc_bsdinfo` の値と等しい。
- **cookie の確認（既存の確認を残す）**: 今の読取りは、cookie が leader の環境に届いたことも確かめている。zombie でなければ、leader の `KERN_PROCARGS2` を 1 回読む。
  - 一致: 受け入れる。
  - absent（platform binary の leader。sandbox-exec の test）: test の step だけ受け入れる。Agent の step は Err にする。
  - 不一致: Err にする。
  - 読めない: 照会し直す。zombie になっていれば受け入れ、そうでなければ Err にする。
  - zombie（すぐ exit した test）: 環境を読まずに受け入れる。exit したプロセスは何も実行しない。子孫は死亡確認が探す。
- **Linux**: 既存の `/proc/<pid>/stat`・`environ` の読取りで同じ関数を実装する。
- **Err の場合**: 既存の tx2 失敗の経路（§3.3 の 6、`record_failed`）に進む。
- **PID が reap の前に再利用されない理由**:
  - P5 により、zombie を含め、回収まで PID は使用中である。
  - leader は rrxd の子で、回収するのは `ProcessGroup::reap` だけである。
  - rrxd は次のどれも行わない（本番のコードの `waitid` は、どれも `WaitId::Pid` である）。
    - `SIGCHLD` の無視
    - `SA_NOCLDWAIT`
    - 任意の子・group を対象にした wait
  - `observe_exit` は `WNOWAIT` で回収しない（`adapter.rs:503-519`）。
  - `kill_on_drop` の回収は `ProcessGroup` を drop した後だけで、その後に birth は読まない。
  - したがって、spawn から reap までの間、`owned_leader` が読む PID は leader のものである。
- **死亡確認での照合**:
  - `confirm_death` は、step の `launch` と、記録した birth（なければ None）を受け取る。
  - `observe_exit` と 2 回目の `cleanup_group` の後、`reap` の前に、`owned_leader` で leader を読む。zombie で、birth が記録と等しいことを確かめる（`death_evidence.leader_birth_matched`）。等しくなければ「確認できない」（held）にする。
  - 探索は `discover_launched(cookie, launch)` で行う。`launch.leader_birth` には記録した birth を入れる。
  - trigger は、launching / running から `death_confirmed` を立てるとき、`death_evidence.launch_checked = 1` を要求する。`OLD.birth` が NULL でなければ、`leader_birth_matched = 1` も要求する。
  - tx2 の失敗の経路では、birth を読めていればそれを渡し、なければ照合を行わない（birth の列は NULL なので、trigger も要求しない）。

## 4. 復旧

### 4.1 起動時の照合

rrxd は、owner lock を取り `Store::open` した後、モードの静止の確認（§1.1）を行い、通れば `begin_execution_epoch`（`state/execution.rs:537-599`）で epoch E を得る。続いて Q（§5.2）を行う。Q がすべて qualified なら、最初の sweep で `control_admission.lock().await` を取り、次を順に行う。

1. **引継ぎ**（1 つの tx）: `owner_epoch < E` の、running / held の run と、それぞれの最後の step（最大の seq・attempt）、pending / held の journal、`pipeline_limit_wait` の行（あれば）を、状態を変えずに `owner_epoch=E` にする（version+1）。held の run も引き継ぐので、回復の command は現在の epoch の run を対象にできる。上限待ちの記録は、期限・幅・probe を変えずに引き継ぐ。監査イベント `rrx.private.pipeline.taken_over` を残す。引継ぎの後、古い epoch の書き手は、run・step・journal・上限待ちの記録のどれも trigger で拒否される（§2.3）。
2. **step の照合**: §4.2 の表を、引き継いだ各 run の最後の step に当てる。
3. **上限待ちの照合**: §2.6 の 4（probing のまま probe の step が launching / running でなければ決着させる）。

その後は、各 sweep で変化した行だけを見る。

### 4.2 復旧表

この表は、再起動時の照合と、通常の tick の両方で使う。

| 永続の状態（引継ぎ後） | 処理 | 結果 |
|---|---|---|
| step planned（process の kind） | 起動前に作業領域を検証する（§6.4 の検証。implement / fix は Task worktree、test は checkout、review / confirm は snapshot を作り直す） | 合格すれば通常どおり起動する（§3.3）。不合格なら planned → held（`worktree_drift`）、run を held |
| step planned（prepare / release） | 通常どおり実行する | |
| step launching | 再実行しない | step を unknown、run を held（`launch_unknown`）。probe なら同じ tx で上限待ちを延長（§2.6） |
| step running（process の kind） | custody を失ったとして、`body.launch` と記録した birth で `discover_launched` を行う | 見つかった → held（`alive_orphan`、pid を記録）。見つからず coverage が完全 → held（`custody_lost`、`death_confirmed=false`）。limited → held（`custody_unobservable`）。probe ならどれも同じ tx で上限待ちを延長（§2.6） |
| step running（prepare） | Git の状態を調べる | branch・worktree・登録がどれもない → interrupted（次の行で attempt+1）。branch が base、worktree が登録済みで HEAD == base かつ clean → adopted。それ以外 → held（`materialization_partial`） |
| step running（release） | §9 の settle の規則 | done / pending のまま / held / aborted |
| step exited（implement / fix） | §6 の検査を最初からやり直す（規則 G）。HEAD == step_base かつ branch の ref == step_base でなければ rejected（`agent_commit`）か held（`head_moved`） | committed / rejected / held |
| step committed | Task branch の ref を読む。== step_base なら §6.3 の update-ref からやり直す。== `result_commit` なら、その commit の tree == `result_tree` かつ親 == step_base を確かめて §6.3 を続ける。それ以外は held（`ref_moved`） | adopted / held |
| step exited（test / review / confirm） | 採用の検査をやり直す（規則 G） | adopted / rejected / held / limited |
| step interrupted / limited | Goal が Running なら、§6.4 で初期化して検証し、attempt+1 を作る。起動の待ちは tx1 が daemon 全体の上限待ち（§2.6）で行う。検証に失敗したら作らずに run を held（`worktree_drift`。最後の step は新しい planned を held にしたもの） | 新しい planned |
| step unknown / held / rejected | 何もしない | run は held のまま（運用者を待つ） |
| step adopted（最後のもの） | 次の step を計画する（§8。規則 G） | planned |
| ref journal pending / held | §9 の settle の規則 | |
| `pipeline_limit_wait` が probing で、probe の step が launching / running でない | §2.6 の 4 | clear または waiting |
| run completed / abandoned | 何もしない | |
| `pipeline_qualifications` に E の行がない | Q を実行する | qualified でなければパイプラインは起動しない |

interrupted / limited の後に検証が失敗した場合は、新しい attempt を planned で作り、同じ tx でそれを held（`worktree_drift`）にして run を held にする。これで不変条件（§3.1）を保つ。採用できるのは記録した SHA（`result_commit`）だけである。trailer や親の一致は採用の根拠にしない。

### 4.3 停止・pause・cancel

**正常な停止**（`Runtime::shutdown`、`runtime/service.rs:168-200`）: guard を取り（`:170`）、`stopping` を立てた後は新しい起動がない。StepCustody に停止を通知し、running の step を kill して死亡を確認し、interrupted にする（監視タスクは guard を取らずに書く）。全 step の終了を 5 秒まで待ち、終わらなければ新しい `ShutdownSite::PipelineSteps`（`runtime/stop.rs:7-34`）で pending を返す。確認できない step は running のまま残す（再起動時に held になる）。

**Goal の pause**: `set_runtime_goal_lifecycle`（`state/runtime/goals.rs:361-516`）が guard の中で Goal を Paused にした後は、規則 G により新しい起動・採用・release・完了はない。commit の後に control 側が `wake` と StepCustody への停止通知を送る。running の step は kill して死亡を確認し、interrupted にする。exited / committed の step はそのまま残る。resume の後、tick が §4.2 の表で、採用をやり直すか attempt+1 を作って再開する。resume の拒否（`preparation_history` `goals.rs:442`）は native の表だけを数えるので、パイプラインの Goal には当たらない。policy digest の一致（`goals.rs:431-439`）は維持する。違うモードの Goal の resume は `ExecutionModeMismatch` で拒否する。

**Goal の cancel**: pause と同じく、新しい起動と結果を進める書込みを止める。tick（`pipeline_cancel_run`）は、running の step を kill して死亡を確認し interrupted（確認できなければ held）、planned の step を abandoned、exited / committed の step を interrupted にする。生きているかもしれない step がなければ、run を abandoned にして Project の slot を空ける。release も Goal の完了も行わない（§9）。死亡を確認できない step や unknown の step があれば、run は held のまま残る。運用者は `rrx pipeline recover --then abandon-run` で、残存を確かめてから run を放棄できる。

### 4.4 回復の command

`rrx pipeline recover <run-id> --abandon-step --seq <n> --attempt <k> --expected-run-version <v> --expected-step-version <w> --then retry|abandon-run --reason <text>`

kind=task と kind=goal の両方の run を対象にする（Goal 完了確認の run は Task を持たないため、command を run で指定する）。必要な値はすべて `rrx goal status` / `rrx goal tasks` で得られる（§10）。

- 新しい `ControlAction::PipelineRecover` を足す（`runtime/control.rs:24-139`）。運用者は `HumanIngress` の OS UID で識別する（`runtime/control.rs:439-470`）。要求の instance と epoch が現在のものと違えば、`HumanIngress::check` が拒否する（`:460-469`）。
- **再送の照会**は、次の 3 か所で行う。どれも同じ規則で、`runtime_control_acks` に同じ request ID があれば、要求の同一性（instance・action と引数の digest・UID）を確かめ、一致すれば保存した応答を返して以降を何も行わない。違えば拒否する。どれも現在の状態の検査より先に行う（`state/runtime/goals.rs:397-408` と同じ順序）。
- daemon は次の順に行う。
  1. **再送の照会**（guard の前、副作用の前）。ここまで何も変えない。
  2. `control_admission.lock().await` を取る。以下は guard の中で行う。
  3. **前提の検査**（読み取りの tx。ファイルを変えない）: **最初に再送の照会をやり直す**（1 と 2 の間に、同じ request ID の別の要求が guard の中で成功していれば、ここで保存した応答を返す）。ack がなければ: 現在の owner。run が held で `version == v`。対象の step が run の最後の step で、unknown / held / rejected で `version == w`。`retry` なら Goal が Running で `k < 16`。`abandon-run` は Goal の状態を問わない（終わらせる書込みだけで、起動・採用をしないため。規則 G の観測の書込み）。`abandon-run` なら、その run の `task_release` の journal が pending / held かを読み、あれば id・version・old・new を控える。一致しなければ `SnapshotChanged` か理由付きで拒否し、何も変えない。
  4. process の kind で cookie があれば、`body.launch` と記録した birth で `discover_launched` を行い直す。1 件でも見つかれば、pid を示して拒否する（運用者に止めてもらう）。coverage が limited の場合も拒否し、応答に `unconfirmed` の PID を含める。
  5. `retry` のときだけ、§6.4 で作業領域を初期化して検証する。失敗すれば拒否する（初期化は rrx が所有する領域だけを対象とし、繰り返しても同じ結果になる）。
  6. `abandon-run` で開いた journal があれば、`PipelineGit` で journal の ref を読む（guard の中、10 秒の timeout、ref は変えない）。ref == old なら「中止」、ref == new なら「release 済み」とする。それ以外なら `ref_unexpected` で拒否し、何も変えない（run は held のまま。journal は §9 の settle の規則に従う）。
  7. **最終の tx**: **最初に再送の照会をやり直す**（ack があれば何も書かずに保存した応答を返す）。続いて 3 の検査（journal の version を含む）をすべてやり直し、次の順に書く。この順序は FK と trigger の条件に合わせたものである。
     - `runtime_control_acks`（応答の body。新しい attempt の番号、各 version、`abandon-run` の結果（abandoned / completed）を含む）
     - `pipeline_recoveries`（UID、request、reason、`then_action`、`consumed_version = v+1`、discovery の要約、初期化の結果、journal の決着と観測した ref）
     - `abandon-run` で開いた journal があれば: 「中止」は journal を aborted に、「release 済み」は done に（期待 version で CAS。どちらも journal の更新の権限をここで閉じる）
     - step を abandoned に（期待 version `w` で CAS。trigger が recoveries の行を確かめる）
     - `retry`: run を held → running に（期待 version `v` で CAS。trigger が `consumed_version` を確かめる）。続いて attempt k+1 の planned を INSERT する（trigger は run が running で、前の attempt が abandoned であることを確かめる）
     - `abandon-run`（journal なし、または「中止」）: run を held → abandoned に（期待 version `v` で CAS。trigger が、生きているかもしれない step と開いた journal がないことを確かめる）
     - `abandon-run`（「release 済み」）: run を held → completed に（期待 version `v` で CAS。trigger が done の journal・recoveries の行・生きているかもしれない step がないことを確かめる）。Goal branch はすでにこの Task の承認済みの commit を指している（journal は §9 の手順 1 で承認の後にだけ作る）ので、放棄すると Git と DB が食い違う。completed にしてその事実を記録する。応答は completed（`release_applied`）で、`wake` で次の Task へ進む
     - 監査イベント `rrx.private.pipeline.step_abandoned`
- 同じ request ID の要求を重複して送った場合、guard が要求を直列にし、後の要求は 3 の再照会で保存した応答を受け取る。cookie の探索・作業領域の初期化・ref の読取り・書込みは、最初の要求の 1 回だけである。
- 古い attempt は再実行しない。cookie も再利用しない。最終の tx のどこかで失敗すれば、全体が rollback される。
- 放棄の後は、その run に開いた journal は残らず、§9 の settle は abandoned の run の journal に `update-ref` を実行しない。どの tick・再起動も、その run を理由に ref を変えない。
- `goal_create` の journal は run ではなく Goal に属するので、run の放棄では変えない（Goal が Running なら §9 の settle が作成を終え、Cancelled なら aborted にする）。

## 5. CLI の契約

### 5.1 設定（runtime の設定だけ。Project の overlay では変えられない。`config.rs:235`）

```toml
[execution]
mode = "cli_pipeline"                        # 既定は managed_native
[pipeline]
work_root = "/Users/<user>/rrx-pipeline"     # 絶対パス。[A-Za-z0-9_/-] だけ、48 byte 以下、0700、Project の root と state の外
path_prefix = ["/opt/homebrew/bin"]          # node などの絶対ディレクトリ。world-writable は拒否
[pipeline.clis.codex]
executable = "/opt/homebrew/bin/codex"
version = "<pinned version text>"
model = "<model>"
effort = "high"
codex_home = "/Users/<user>/.rrx/codex-home" # 専用。運用者がここで login する
[pipeline.clis.claude]
executable = "/Users/<user>/.local/bin/claude"
version = "<pinned version text>"
model = "<model>"
```

同時実行数の設定は持たない（v1 は 1 step ずつ。§2.6）。`mode = "cli_pipeline"` で `[pipeline]` がなければ、起動を拒否する。

### 5.2 起動時の適格性確認（Q）

次の 4 つを確かめ、結果を `pipeline_qualifications` に書く。コマンドは環境を空にしてから組み立て（§5.4）、timeout 5 秒、出力は help を要求する probe（codex `exec --help`、claude `--help`、claude `--setting-sources bogus|"" --help`）は 64 KiB まで、それ以外（`--version`、login status）は 4 KiB までとする。

| subject | 内容 |
|---|---|
| platform | macOS であること。それ以外は refused |
| mode_quiescence | §1.1 の静止の確認（epoch の前に行ったもの）が通ったこと。通らなければ rrxd は起動しないので、この行は qualified だけになる |
| codex / claude | 1. `executable` の realpath と sha256 を記録する。<br>2. `--version` の出力を trim したものが `version` と完全に一致する。<br>3. 次の出力に §5.3 の必須 flag がすべてある（行単位の完全一致）: codex は `exec --help`、claude は `--help`。<br>4. ログインの確認。codex は `CODEX_HOME` を指定して `codex login status` の終了コードが 0。さらに `codex_home/config.toml` を TOML として読み、許可したキーだけであること（`mcp_servers`・`notify`・`hooks`・`profiles` は拒否）。claude は `claude auth status` の終了コードが 0。pin した版にこのサブコマンドがなければ、その版は pin の対象にしない。 |

起動のたびに Q をやり直す。step を起動する直前には実行ファイルを照合し直す（§3.3-3）。

### 5.3 引数の雛形（コードに固定する）

`<W>` は Task worktree か snapshot、`<A>` は attempt ディレクトリ。どちらも `work_root` の下に置き、パスは `[A-Za-z0-9_/-]` だけにする（Codex の `-c` はキーを `.` で区切り、引用符を解釈しない）。

**Executor（implement / fix）**

```
<codex> exec --sandbox workspace-write
  -c sandbox_workspace_write.network_access=false
  -c projects.<W>.trust_level="untrusted"
  -m <model> -c model_reasoning_effort=<effort>
  -C <W> -o <A>/last.md --json --color never -
```

**Confirmer**

```
<codex> exec --sandbox read-only
  -c projects.<W>.trust_level="untrusted"
  -m <model> -c model_reasoning_effort=<effort>
  -C <W> --output-schema <A>/confirm.schema.json -o <A>/last.json --json --color never -
```

**Reviewer**

```
<claude> -p --permission-mode plan
  --tools Read,Grep,Glob
  --disallowedTools Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Task
  --strict-mcp-config --mcp-config <A>/mcp.json        # {"mcpServers":{}}
  --setting-sources "" --disable-slash-commands --no-session-persistence
  --model <model> --output-format json --json-schema <schema JSON>
```

- prompt は stdin で渡す（§3.4）。cwd は `<W>`。
- `--setting-sources ""` を pin した版が受け付けない場合、Q を不合格にする（B3 で実機確認する。§16 D9）。

### 5.4 環境変数

`env_clear()` の後に、次だけを入れる。

| 変数 | 値 |
|---|---|
| `HOME` / `USER` / `LOGNAME` | 運用者のもの |
| `PATH` | `path_prefix` + `/usr/bin:/bin:/usr/sbin:/sbin` |
| `LANG` | `en_US.UTF-8` |
| `TMPDIR` | `<work_root>/t/<attempt の短い ID>`。60 byte 以下。AF_UNIX のパス長の上限 104 byte への配慮 |
| `TERM` / `NO_COLOR` | `dumb` / `1` |
| `RRX_PROCESS_COOKIE` | step の cookie（`crates/rrx-process-tracker/src/lib.rs:22`） |
| `CODEX_HOME` | codex のときだけ |
| `GIT_CONFIG_COUNT` / `GIT_CONFIG_KEY_0` / `GIT_CONFIG_VALUE_0` | Reviewer のときだけ `1` / `safe.bareRepository` / `explicit`（暗黙の bare repository を Git 自身が拒否する） |
| `GIT_CONFIG_NOSYSTEM` | Reviewer のときだけ `1` |

GH_TOKEN・GITHUB_TOKEN・SSH_AUTH_SOCK・`AWS_*`・`ANTHROPIC_*`・`OPENAI_*`・上表以外の `GIT_*` は入れない。テストでは「実際に渡した集合 ⊆ 許可した集合」を検査する。test の step も同じ集合を使う（§7.2）。

### 5.5 出力の上限とファイル

attempt ディレクトリは `<work_root>/runs/<run>/<seq>-<attempt>/`（0700）に置く。完了後、ファイルを 0400 にする。

| ファイル | 上限 | 超えたとき |
|---|---|---|
| `prompt.md`（sha256 = `input_sha256`） | 512 KiB | 起動しない（Reviewer は prepare / launch の前検査で planned → held、`review_prompt_too_large`。他の Agent は launching → rejected） |
| `stdout.jsonl`（codex `--json`）/ `result.json`（claude） | 16 MiB / 1 MiB | group を kill し、rejected（`output_overflow`） |
| `stderr.log` | 1 MiB | 末尾を残し、`truncated` を記録する（不合格にはしない） |
| `last.md` / `last.json` | 256 KiB | rejected |
| `diff.patch`（rrx が生成） | 8 MiB | held（`diff_too_large`） |
| test の stdout / stderr | コマンドの `stdout_bytes` / `stderr_bytes`（`execution/verification/plan.rs:43-44`） | kill し、test の結果を fail（`output_overflow`） |

### 5.6 provider の上限の分類器（既知の終了だけ）

死亡を確認した後に、pin から選んだ wire profile の、コードへ固定した規則で分類し、`provider_outcome` を書く（§2.6 の 1）。Q の適格性と wire の対応は別に検査する。Q を通っても、対応のない pin の出力は `other` である。

#### 5.6.1 pin と wire profile

`execution/pipeline/classify.rs` に、provider と pin の完全一致から、閉じた enum の profile を選ぶ `match` の表を置く。pin は Q で照合した `cli.version`（`--version` の出力を trim した文字列そのもの）である。prefix・semver の範囲・部分一致・最新版への fallback・設定による追加は使わない。表は outcome を持たず、profile の実装だけが outcome を返す。provider が違う profile は選べない型にし、未知 pin の arm は decode より前に `other` を返す。Q を通った未対応 pin も観測のための起動は認めるが、課金を伴い、結果は常に `other` となる。custody の exit evidence に固定の `wire: "unmapped"` を残し、`rrx goal status` の held の理由でも未対応 pin と、それ以外の `agent_failed` を区別できるようにする。観測を理由に採用や後続起動を許可しない。

| provider | pin（完全一致） | wire profile | success の根拠と確認 | limit の根拠と確認 |
|---|---|---|---|---|
| codex | `rrx-fixture-codex-v1` | `codex-jsonl-v1` | 偽 CLI の既存 fixture 用 | 合成（実 CLI では未観測） |
| claude | `rrx-fixture-claude-v1` | `claude-result-v1` | 偽 CLI の既存 fixture 用 | 合成（実 CLI では未観測） |
| codex | `codex-cli 0.164.0` | `codex-jsonl-v1` | 文書化された `exec --json` の形式と、それをモデルにした fixture。次の実 CLI pass で shape を照合する | `error.code` / `error.message` は合成（未観測）。実版の limit の根拠にはしない |
| claude | `2.1.312 (Claude Code)` | `claude-result-v1` | 文書化された `-p --output-format json` の封筒と、それをモデルにした fixture。次の実 CLI pass で shape を照合する | `api_error.status` / `result` の固定文言は合成（未観測）。`claude_wire.rs:378,384-389,396` が読む top-level の `api_error_status` と文字列配列 `errors` とは異なる。success の矛盾検査ではこれらも読むが、limit の一致規則には代用しない |

この 2 つの実 pin は、観測を始めるための初期対応として今回追加する。実 wire との一致を確認済みとは扱わない。既存の「pin した版ごとにコードへ固定する」規則をこの表で具体化するもので、未知の版を許可する変更ではない。既存 fixture は合成データなので、これを実 CLI の保存出力と呼ばない。文書の出典・対象 pin・fixture の対応と、まだ実観測のない分岐を検証記録に残す。検証記録では、文書由来・実観測の shape 由来・合成した分岐を区別し、原本の本文を fixture に持ち込んでいないことも記録する。

#### 5.6.2 profile の分類規則と安全条件

既存 fixture の規則を上の 2 profile に移し、次の安全条件を加える。両方とも、終了コードが得られなければ `other` とし、完全な出力に strict_json の重複キー・深さ・サイズの検査と、profile が読むフィールドの型検査を当てる。現行の `classify.rs:64-69,94-99` は limit の判定で終了コードを見ないため、終了コード `None` と limit の形の組は、今回 `limit` から `other` に変わる。codex は JSONL の全非空行、claude は単一の JSON object を対象にする。途中まで parse できたことを成功の根拠にしない。`classify.rs:17-31` の strict_json の上限（1 文字列 256 KiB、1 frame の文字列の合計 1 MiB など）は維持する。codex の `command_execution` item が出力全体を含む場合など、正常終了でも decode の上限に達して `other` になる偽陰性は、安全側の動作として受け入れる。

- `limit`:
  - `codex-jsonl-v1`: 最後の event が `error` または `turn.failed` で、`error.code=rate_limit_exceeded`、`error.message=Rate limit reached` が完全一致する。
  - `claude-result-v1`: 外側の封筒が `type=result`・`is_error=true`・`api_error.status=429` で、`result=Rate limit exceeded` が完全一致する。
  - 自由文や stderr の部分一致、code / status だけでは上限にしない。実版が異なるエラー形式を使えば `other` になる。
- `success`（既知の成功。上限待ちの解除に使える唯一の分類）:
  - `codex-jsonl-v1`: 終了コード 0、最後の event が `turn.completed` で、それが唯一の完了 event であり、その前に `turn.started` がちょうど 1 回あり、全体に `error` / `turn.failed` がない。途中の event は `thread.started`・`turn.started`・`item.started`・`item.updated`・`item.completed` だけを認める。未知の event や完了後の event があれば `other` にする。item の本文にある「成功」の文字列は使わない。
  - `claude-result-v1`: 終了コード 0、外側の封筒が `type=result`・`subtype=success`・`is_error=false` で、`structured_output` が object である。`structured_output` の欠落・null・文字列など object 以外は `other` にする。`api_error` と top-level の `api_error_status` はそれぞれ欠落または null だけを許し、非 null なら `other` とする（`api_error_status` が null でも数値でもなければ型違いとして `other`）。存在する `errors` は文字列だけの配列に限り、空でなければ成功と矛盾するので `other` にする。`result` の本文や structured verdict の内容は成功の根拠にせず、verdict の妥当性は別に検査する。
- `other`: 上のどれにも当たらないもの（未知の pin / wire、認証の失敗、分類できない失敗、終了コードの欠落、出力の欠落・不正・上限超過を含む）。

StepCustody の検査を profile で迂回しない。死亡の確認、出力の完全な回収、上限、timeout・I/O・秘密の検出などの issue の不在が前提となる。codex の Executor / Confirmer には `last.md` / `last.json` の検査も必要である。現行の `check_last`（`custody.rs:562-567`）は欠落時に `Ok(None)` を返すが、今回、両 role では欠落も `last_output_invalid` にする製品変更を行う。不正・欠落・上限超過などで `last_output_invalid` があれば、JSONL が成功形でも `other` にする。

`success` は provider の呼び出しが上限なしで終わったことだけを表す。これだけで §2.6 の probe は daemon 全体の上限待ちを解除するので、採用検査が後で落ちることを安全策に数えない。失敗の成功誤認を避けるため、未知・矛盾・不完全な観測は `other` に倒す。Executor の採用はさらに終了コード 0 と `provider_outcome=success`（`AdoptionChecker::build`）、Reviewer / Confirmer はそれに加えて verdict の検査を要求する（§6〜§8）。使用率などの観測は読まない。

#### 5.6.3 実 CLI harness の shape 観測（テスト専用）

`state/execution/pipeline/tests/real_cli.rs` は、各 Agent step / attempt の custody の終了後、採用を試みる前に shape を採取し、`summary.json` のその step に記録する。失敗・rejected でも、回収済みの出力があれば対象にする。先行 step の失敗で未起動なら固定の `not_run`、起動したが観測できなければ固定の状態を残す。prepare と test は Agent の shape 観測の対象にしない。今の pass の implement / review に適用し、fix / confirm を通す pass を足す場合も同じ関門を使う。

読取り先は設定済み `work_root/runs/<run>/<seq>-<attempt>/` の `stdout.jsonl` / `result.json` だけで、run・seq・attempt は Store の step から導出する。symlink をたどらない通常ファイルの bounded read と custody の出力 receipt の照合を使い、stdout は §5.5 の上限を超えて読まない。解析も製品と同じ strict_json の上限と重複キー拒否を使う。Agent の出力に書かれたパスは開かない。

| 対象 | 記録する shape（Agent の本文は記録しない） |
|---|---|
| 共通 | receipt の終了コード（数値。signal 等で得られなければ null）、選択した profile の固定名または `unmapped`、`provider_outcome` の固定値。生の issue やエラー文字列を写さない |
| codex | JSONL の全非空行の `type` を順序どおりに最大 4096 件。文字列は ASCII の `^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*$`、64 byte 以下に限る。`error` / `turn.failed` の位置に、その event の top-level keys と、`error` が object ならその子の key set を記録する。各集合は辞書順、最大 128 個、各キーは ASCII の `^[A-Za-z_][A-Za-z0-9_]{0,63}$` に完全一致とし、キーに対応する値は写さない。ただし既存の観測項目である `error.code` は ASCII の `^[a-z][a-z0-9_]{0,63}$` に完全一致すれば値を記録する。任意フィールドの欠落は固定の `absent` とし、値があるのに型・pattern が合わなければ shape 全体を固定の `redacted` にする。`error.message`・top-level の message・item・thread ID・usage の値は写さない |
| claude | 外側の `type`・`subtype`（上の type と同じ pattern・64 byte 以下）、`is_error`（bool）、`api_error.status` と top-level の `api_error_status`（それぞれ 100〜599 の整数）、`errors` の件数（文字列配列なら 0〜4096）を記録する。top-level keys と、`api_error` が object ならその子の key set も記録する。各集合は辞書順、最大 128 個、各キーは ASCII の `^[A-Za-z_][A-Za-z0-9_]{0,63}$` に完全一致とし、キーに対応する値は写さない。任意フィールドの欠落は固定の `absent`、存在する値の型・範囲違い（status の null を含む）は固定の `redacted` とする。`result`・`structured_output`・`errors` の本文は写さない。既存のスキャン済み `model_usage_keys` の観測は維持する |
| last-output | `passed` / `failed` / `not_applicable` / `not_checked` の固定値。codex の有効な last receipt があるときだけ `passed`、custody の issue が `last_output_invalid` なら `failed`、claude は `not_applicable`、確認できなければ `not_checked`。issue がないだけで `passed` と推測しない。ファイルの本文は読取り結果に含めない |

shape 全体は 512 KiB までとする。shape の event・key の件数、文字列・全体の bound を超えたら、部分列を完全な観測として残さず `shape_overflow` だけにする。これとは別に、製品と同じ strict_json の decode 上限に達した場合は固定の `strict_json_limit` だけにし、read / 構文不正・重複キーなどの parse / receipt の照合の失敗を表す `absent_or_invalid` と区別する。どちらも部分 shape や parse error の診断は残さない。これらはテスト用の観測状態で、製品の出力上限は広げない。

既存の `envelope_keys` と同じ秘密スキャンを、raw（Unicode escape を含む）と decode 後の全キー・全文字列、最後に組み立てた shape に当てる。codex は bounded な JSONL frame ごとに検査し、frame をまたぐ検出は既存の stream scan と同じ方式で行う。抽出しない本文も検査を省かない。1 件でも検出したら、その shape 全体を `{"state":"secret_detected"}` だけに置き換え、先に集めた値も捨てる。スキャン不能・型違い・曖昧さは固定の状態に倒し、raw・parse error・秘密の一致文字列・Agent の本文を診断に添えない。終了コードと last-output は信頼した receipt から別に記録できるが、出力由来の shape を復元しない。秘密スキャンは未知の秘密をすべて検出する保証ではなく、投影するフィールドと長さも制限する。

次の実 CLI pass は、この shape と製品の `provider_outcome`、last-output、既存の採用・test・review の結果を合わせて初期対応を検証する。harness の shape 判定は「記録済み（`recorded`）か、検証不能か」だけであり、profile への適合は製品の `provider_outcome` だけで判断する。harness に profile の分類規則を再実装しない。製品が `success` 以外、秘匿・欠落・上限超過などで shape が検証不能、codex の last-output が `passed` 以外なら pass は失敗とし、後続の Agent を起動しない。未知 pin でも shape は記録できるが、製品は `other` と exit evidence の `wire: "unmapped"` を残す。harness は outcome を書き換えず、表を自動学習・追加しない。オーナーへ返すのは従来どおり `summary.json` と `send-record.json` だけで、attempt の原本や last の本文をコピーしない。

次の pass の wire 照合には、(a) codex と claude の両 profile を確認できる、(b) codex が合わず implement が `other` → `agent_failed` で止まり、review は `not_run` で claude の shape は観測できない、(c) codex は合うが claude が合わない、の 3 通りがある。(b) は codex を修正・ローカルで再検証してから claude を観測するもう 1 窓が必要で、(c) も claude の修正後の再確認にもう 1 窓を要しうる。成功条件は緩めない。通常のローカルテストで記録器と失敗経路を先に完成させ、初期対応の照合には最大 2 窓が必要になりうる計画とする。各窓はオーナーが確保し、D22 の二重 gate と開始条件を毎回満たす。2 窓で確認できなければ受入を未完了として扱う。B11 の受入記録には、この 3 通りと最大 2 窓の可能性、実際に観測できた provider と未観測の provider、各 pass の結果を明記する。

#### 5.6.4 版の追加と fixture

以後の実 pin の追加・対応 profile の変更は、保守担当が通常のコード変更と独立レビューで行う。オーナーが承認した実 CLI pass の、同じ完全一致 pin のスキャン済み shape と終了コード・last-output の証跡、wire の文書の出典、profile の全分岐の fixture / 負の対照を根拠にする。初期の 2 pin 以外には、版番号が近いことや偽 CLI の合格だけで対応を広げない。未知の pin でも harness は観測できるが、製品の分類は `other` のままとし、観測のために採用や後続起動を許可しない。今回の初期対応が次の pass と合わなければ、当該 entry を除くか profile を修正して再検証する。成功条件を緩めて通さない。

fixture は、記録した終了コード・event の順序・固定フィールド・キーの構造を保ち、本文・ID・パス等を固定の合成値で新しく組み立てる。原本の出力を fixture やレビュー資料にコピーしない。実 pin ごとに success / limit / other の組を持ち、共通 profile のデータは共有できるが、その pin を引数に分類器を通す。既存の `codex-success.jsonl` にない `turn.started` を `turn.completed` の前に 1 件加え、合成 pin と実 pin の両方で厳格な成功条件を検査する。出典の shape と、文書から補った部分・合成した分岐の区別は、§5.6.1 の検証記録の要件として残す。shape は `error.message` / `result` の本文を含まないので、上限の固定文言は文書由来の合成値であり、成功 pass で実版の limit 分岐まで確認したとは扱わない。失敗 event を観測できた場合も、key set は構造の根拠にだけ使い、省いた code / status / 本文の値を推測して実観測扱いにしない。実際の上限形が異なれば `other` に倒れ、probe の待ちは延長される。上限を起こすための追加の課金・実行はこの設計では要求しない。

### 5.7 verdict の JSON schema

**Reviewer**（`rrx.review.v1`）。strict_json の上限は frame 1 MiB、depth 8、nodes 4096、string 64 KiB。

```json
{"type":"object","additionalProperties":false,"required":["schema","verdict","summary","findings"],
 "properties":{
  "schema":{"const":"rrx.review.v1"},
  "verdict":{"enum":["approve","request_changes"]},
  "summary":{"type":"string","maxLength":4000},
  "findings":{"type":"array","maxItems":32,"items":{"type":"object","additionalProperties":false,
    "required":["id","severity","blocking","file","line","title","rationale"],
    "properties":{"id":{"type":"string","pattern":"^F[0-9]{1,2}$"},
      "severity":{"enum":["critical","high","medium","low"]},"blocking":{"type":"boolean"},
      "file":{"type":"string","maxLength":512},"line":{"type":"integer","minimum":1,"maximum":1000000},
      "end_line":{"type":"integer","minimum":1,"maximum":1000000},
      "title":{"type":"string","maxLength":200},"rationale":{"type":"string","maxLength":4000}}}}}}
```

外側の封筒（`--output-format json`）について:

- `type=result`、`subtype=success`、`is_error=false` と、structured output の存在を要求する。
- pin した model が主 model として記録されていることを要求する。ほかの model は診断用に記録する。
- 外側の未知フィールドは、pin した版の範囲で記録だけ残す。内側の verdict は `deny_unknown_fields` の DTO で decode する。

**Confirmer**（`rrx.confirm.v1`）

```json
{"schema":"rrx.confirm.v1","labels":[{"id":"F1","label":"confirmed|rejected|unclear",
  "evidence":{"file":"…","line":1,"quote":"1〜512 文字"},"reason":"≤2000 文字"}]}
```

`evidence.line` は schema でも `minimum: 1` とする。意味の制約は §8.3 に書く。

## 6. Executor の採用（rrx commit）

### 6.1 rrx の Git 呼び出し（`PipelineGit`）

- `env_clear()` の後、`HOME`・`PATH`（`/usr/bin:/bin`）・`LANG` と、次だけを入れる: `GIT_CONFIG_NOSYSTEM=1`、`GIT_CONFIG_GLOBAL=/dev/null`、`GIT_ATTR_NOSYSTEM=1`。オーナーの global / system の設定（例: git-lfs の filter）は、rrx の Git に効かない。
- 常に `-c core.hooksPath=/dev/null -c core.fsmonitor=false -c core.untrackedCache=false -c gc.auto=0 -c maintenance.auto=false -c commit.gpgsign=false -c core.autocrlf=false` を付ける。
- diff は常に `--no-ext-diff --no-textconv` を付ける。
- `rrx_spawn_hygiene::spawn_tokio` で起動し、10 秒の timeout と 4 MiB の出力上限を付ける（diff は §5.5 の上限）。
- **driver の拒否**（prepare のとき、各 Executor の step の前、採用検査のとき）。次のどれかがあれば、その repo を拒否する（prepare なら held `git_driver_declared`、採用検査なら held）。
  - repo の設定（`config`・`config.worktree`、include 先を含む `git config --list --show-origin -z --local --worktree`）の `filter.*`・`diff.*.textconv`・`diff.*.command`・`diff.external`・`merge.*.driver`・`core.attributesFile`。
  - 対象の tree と worktree の、深さを問わないすべての `.gitattributes`（追跡外を含む）と `info/attributes` の、`filter`・`diff`・`merge`・`working-tree-encoding` の属性。
- 拒否が通った repo では、status・checkout・reset が外部のコードを起動しない。

### 6.2 検査

検査は、死亡を確認した後、tick の中（guard を保持）で Task worktree に対して行う。

- HEAD は Task branch への symbolic ref で、branch の ref == step_base（Agent が commit していない）。違えば rejected（`agent_commit`）。
- `.git` ファイルの内容、common dir の `config`・`hooks/`・`info/`・その branch の ref、`git config --list --show-origin` の digest が、step の前と同じ。違えば held（`git_control_changed`）。
- driver の拒否（§6.1）を、もう一度当てる。
- 変更の一覧: `git status --porcelain=v2 -z --untracked-files=all --no-renames --ignore-submodules=all`（§16 D18）。対象の tree または Task worktree の index に gitlink（mode 160000）があれば、status より前に拒否する。
- 変更した各パスが Task scope の glob に一致する。固定の拒否リストに当たるパスは、scope が明示していない限り拒否する: `.git*`、`.gitattributes`、`.gitmodules`、`.claude/**`、`.codex/**`、`.mcp.json`、`AGENTS.md`、`AGENTS.override.md`、`CLAUDE.md`、`CLAUDE.local.md`、`**/CLAUDE.md`（名前の照合は、ASCII の大文字と小文字を区別せず、さらに Unicode の大文字小文字の畳み込みでも行う。§16 D19）。`.gitignore` に当たって変更の一覧に出ないパスも、`git status --porcelain=v1 -z --ignored=matching --untracked-files=all --ignore-submodules=all` の ignored の項目として列挙し、保護する名前の成分を持つものがあれば拒否する（`protected_path`）。build と、clean の確認（`clean_at`）の両方で行う。ignored の項目は変更として扱わず、clean の判定にも数えない（rename の検出は `--no-renames` で止め、UTF-8 でない成分は保護する名前に一致しないものとして読む）。`.gitignore`（深さを問わず、名前は同じ規則で畳む）の追加・変更・削除は、scope が明示していても拒否する（`protected_path`）。Task worktree の index に `assume-unchanged` か `skip-worktree` のフラグを持つ項目があれば、gitlink と同じく status の前に拒否する。
- 各パスを `lstat` し、通常ファイルだけを許す。mode は 100644 か 100755。symlink（120000）と gitlink（160000）は拒否する。
- 各ファイルを 1 回だけ読み、bytes を保持する。変更は 200 ファイル以下、追加は合計 8 MiB 以下、1 ファイル 1 MiB 以下。NUL を含むもの（バイナリ）は拒否する。
- 変更が 0 件なら rejected（`no_change`）。
- 秘密スキャン（§11）を、`last.md`・`stdout.jsonl`・stderr に当てる。

### 6.3 組み立てと採用

1. 一時 index（`GIT_INDEX_FILE=<A>/index`）に `git read-tree <step_base>`。
2. 変更・追加の各パスについて、§6.2 で読んだ bytes を `git hash-object -w --no-filters --stdin` に渡して blob を作り、`git update-index --index-info` に `<mode> <blob>\t<path>` を渡す。削除は mode 0 の行を渡す。worktree は読まない。
3. `git write-tree` で tree、`git commit-tree <tree> -p <step_base>` で commit を作る。author と committer は `rrx <rrx@localhost>`、メッセージに `Rrx-Run:` と `Rrx-Step: <seq>-<attempt>` を付ける（人が読むための表示で、採用の根拠ではない）。この時点では、どの ref も指していない。
4. `git diff-tree -p --no-ext-diff --no-textconv <step_base> <commit>` で diff を作り、追加行に秘密スキャンを当て、`diff.patch` の上限を確かめる。該当すれば held（commit object は参照されないまま残るだけ）。
5. tx（規則 G）: step を exited → committed にし、`result_commit` と `result_tree` を書く。
6. `git update-ref refs/heads/<Task branch> <result_commit> <step_base>`（旧値を指定した CAS）。続いて `git read-tree <result_commit>`（本来の index。worktree は書かない）。`rev-parse HEAD` == `result_commit`、`status` が空であることを確かめる。worktree のファイルは手順 2 で使った bytes と同じなので、差分は出ない。
7. tx（規則 G）: step を committed → adopted、run の `head_commit` を `result_commit` にする（それぞれ期待 version で CAS）。

1〜7 は 1 回の tick の中で行う。手順 5 か 7 の tx で Goal が Running でなければ、そこで止める。§4.2 の表が、resume の後に同じ位置から続ける。

**crash の窓**（§4.2 の表と同じ規則）:

| crash の位置 | 再起動時の扱い |
|---|---|
| exited の記録の後、手順 5 の前 | 手順 1 からやり直す（HEAD と ref が step_base のままのはず。違えば rejected / held）。作り直した commit は別の SHA になるが、どれも記録されていないので問題ない |
| 手順 5 の後、手順 6 の前 | ref == step_base なので、手順 6 から行う |
| 手順 6 の後、手順 7 の前 | ref == `result_commit` なので、tree と親を確かめて手順 6 の残りと手順 7 を行う |
| どの時点でも、ref がそれ以外 | held（`ref_moved`） |

### 6.4 作業領域の初期化と検証

新しい attempt の前（tick の §4.2、回復の `retry`）に、死亡を確認した後で、rrx が所有する作業領域だけを初期化する。

| step の kind | 初期化 | 検証 |
|---|---|---|
| implement / fix | Task worktree を step_base に戻す（`reset --hard <step_base>`、`clean -ffdx`） | HEAD == step_base、branch の ref == step_base、`git status --porcelain=v1 -z --untracked-files=all` が空、Git 設定の digest が step の前と同じ |
| test | rrx が作った checkout を削除して作り直す（§7.1） | HEAD == 対象の commit、status が空 |
| review / confirm | snapshot を削除して作り直す | tree digest が対象の commit と一致 |
| prepare | 何もしない（rrx は既存のものを消さない。運用者が片付ける） | branch・worktree のパス・登録がどれもない |
| release | 何もしない（新しい attempt が ref を読み直す） | — |

snapshot の実効上限は、root・ディレクトリ・ファイルを合わせた 100,000 項目、または `git ls-tree -r -z` の一覧出力 4 MiB のうち、先に達する方である。どちらの上限を超えても review は `review_snapshot_limit` で held にする。一覧の上限は snapshot の呼び出し箇所で名前付き理由に変換し、他の Git 呼び出しの扱いと既存の上限は変えない。driver の検査も同じ一覧を読むため、snapshot の一覧取得をその前に行う。driver の検査はファイルの書き出し前に必ず通す。

検証に失敗したら、新しい attempt を起動しない（§4.2 では held、回復では拒否）。

## 7. テスト step

### 7.1 clean checkout

- 検査対象の commit（Task run は run の head、Goal run は Goal branch の head）ごとに、tick が test の step を計画するとき、`git worktree add --detach <work_root>/co/<run>-<seq>-<attempt> <commit>` で checkout を作る（driver の拒否 §6.1 を通した repo だけ）。作った後、HEAD == commit かつ status が空であることを確かめる。
- checkout のパスは決定的で、rrx だけが作る。再起動時に、step を起動する前のパスが残っていれば、検証に合格すれば使い、不合格なら `git worktree remove --force` で消して作り直す（どの step もまだ起動していないことを、step の状態で確かめてから行う）。
- 再利用の前に、通常ファイルの gitfile が `<common>/worktrees/<checkout の名前>` を直接指すことを確かめる。別 attempt の checkout は再利用しない。
- cleanup は run の状態・version・step receipt を変えない。checkout は attempt ごと、TMPDIR は cookie ごとに一意で再利用しないので、削除の成否は実行結果の正しさに影響しない。死亡確認済み test の全 attempt を、採用・abandon・retry・起動前の放棄と同じ transaction で `pipeline_test_cleanup` に累積して記録する。key は `(run_id, seq, attempt)`。done の行も保持し、後の commit の採用で前の義務を上書き・再登録しない。この独立テーブルの書込みは run / step の version を変えないので、launch / monitor の cursor CAS は従来どおりである。
- Runtime sweep だけが、生きている `launching` / `running` step のない run の削除待ちを処理する。失敗は義務ごとに固定の区分（`checkout_remove` / `tmpdir_remove` / `cleanup_sweep`）と rrx が導出したパスだけを記録し、2 秒から倍増して最大 256 秒の backoff を置く。ファイル内容は記録しない。sweep の在庫読取りなどのエラーは固定の `cleanup_sweep` と導出した checkout パスを削除待ちに保存し、同じ backoff を置く。記録自体が失敗しても固定の区分だけをログに出して次の run に進み、service loop を止めない。採用・起動は cleanup を先に実行せず、cleanup の失敗によって held にしない。既存の goal / run view の `pending_cleanup` に各義務と最後の失敗区分を表示する。
- 削除は checkout の gitfile を信用せず、canonical な private parent `work_root/co` と導出した名前だけを検証し、symlink を辿らずにその entry を消す。その後 canonical common dir の `worktrees/<導出名>` が symlink でない directory、その `gitdir` が symlink でない通常ファイルで、内容が `<checkout>/.git` と Git の改行 1 文字だけであることを照合し、この管理 directory だけを消す。不一致は登録を残して `checkout_remove` として記録する。Project 全体の prune は行わない。
- cleanup の部分的な epoch 引継ぎは completed / abandoned の run だけを対象にする。running / held は journal・limit wait を含む既存の全体引継ぎを使う。終端の run を held に戻す遷移は認めない。

### 7.2 各コマンドの実行

TestsProfile（`execution/verification/plan.rs:50`）の各コマンドを、それぞれ 1 つの test step（`body.command_index`）として、agent の step と同じ StepLauncher・StepCustody・死亡確認で実行する。`collector::collect` は使わない（custody を値で受け取り、自分で signal と reap を行い、`pub(super)` であるため。`execution/verification/collector.rs:114-115,149,174`）。

- **profile の読み戻し**: test step を計画するときと tx1 で、`verification_profiles` の行を `(project_id, digest = goal_execution_profiles.tests_profile_digest)` で読み、`hash(body) == digest`（既存の `profile` `state/execution/verification.rs:77-80` と同じ照合）、`AdmittedProfile` への decode、`validate`（`plan.rs:205-215`）を行う。どれかが失敗すれば step を held（`tests_profile_changed`）にする。同じ key のまま body を変えた REPLACE もここで拒否される（§2.3）。
- **起動前の照合**: 各コマンドの起動の直前に `AdmittedProfile::recheck`（`plan.rs:216-223`）を呼び、コマンドが admit 時の一覧にあり、program の hash が一致することを確かめる（§3.3-3）。admit の後に実行ファイルが差し替わっていれば、起動せず rejected にし、run を held にする。
- argv: `/usr/bin/sandbox-exec -f <A>/test.sb -- <program> <args…>`。program は admit 済みの絶対パス、cwd は checkout の中のコマンドの `cwd`。
- Seatbelt の profile: Mach lookup の許可は設けず、process は fork・exec と自己を target にした `process-info*`、signal は same-sandbox だけを許可する（この process-info 指定で他プロセスの読取りを遮断できるとは扱わない。親への libproc `proc_pidinfo(PROC_PIDTASKALLINFO)` は成功した。引数・環境も無条件の `sysctl-read` の許可で読める。§12）。network は localhost の bind / listen と AF_UNIX（rrx control socket を含む）も拒否する。テストはローカルソケット、POSIX IPC（例: Python の `multiprocessing.Lock`）、`/dev/tty` を使えない。書込みは checkout・TMPDIR・運用者が宣言したキャッシュディレクトリと `/dev/null` の data write だけ。
- checkout・TMPDIR・cache の grant は、既存の保護対象に加え、canonicalize した CODEX_HOME・全 PATH prefix・両 CLI の実体とその親との重なりを拒否する。cache は全 Project で共有する。program は work_root・cache・CODEX_HOME の下では admission / recheck とも拒否し、hash は `spawn_blocking` で計算する。
- 環境変数は §5.4 と同じ集合（`CODEX_HOME` を除く）。
- 上限: `timeout_seconds`・`drain_seconds`・`stdout_bytes`・`stderr_bytes`（`plan.rs:35-45`）。
- 結果の採用（exited → adopted、規則 G）: 終了コード 0 なら pass、それ以外・timeout・出力の上限超過は fail。fail も、正しく観測できた結果として adopted にする。`body` に終了コード、signal、stdout / stderr の `StreamEvidence` を書く。
- 観測が欠けた場合（出力の I/O エラー、reap の失敗、死亡を確認できない）は held。
- Goal run の fail は採用しない。§9 のとおり、その step を held（`goal_check_failed`）にする。
- 各 step の TMPDIR（tx1 の intent に記録した `<work_root>/t/<cookie の先頭 8 文字>`）は、§7.1 の義務で checkout と一緒に消す。対象は死亡確認済み step だけで、確認できない step の TMPDIR は残す。retry の新 attempt は死亡確認済み集合に含めない。TMPDIR が link に置き換わっていれば、link を辿らずに link だけを消す。

全コマンドの結果がそろったら、その commit のテストの合否を決める: すべて pass なら合格。

## 8. レビュー・確認・修正

### 8.1 blocking の規則

指摘が **blocking** であるのは、`blocking=true` か、重大度が Goal の `blocking_severity` 以上のときである。どちらかを満たせば、重大度にかかわらず blocking として扱う。blocking でない指摘は記録して報告するが、v1 では修正しない。

### 8.2 規則によるふるい分け

blocking の指摘のうち、次をすべて満たすものだけを confirm に渡す。満たさないものは `invalid` として記録する。

- `file` が正規化した相対パスで、レビューした tree にある。
- `line`（と `end_line`）がそのファイルの行数以下で、`line <= end_line`。
- ファイルが base..head の変更集合にあるか、変更されたファイルが import しているファイルとして rrx が検出できる。
- Task の scope 内にある。

### 8.3 VerdictDecoder の意味の制約

**Reviewer**

- finding の `id` は verdict の中で一意。重複があれば schema 違反として rejected。
- `approve` で blocking の指摘を含むものは、矛盾として `request_changes` 扱いにする。

**Confirmer**

- ラベルの `id` の集合が、渡したふるい分け済みの blocking 指摘の `id` の集合と、重複なく完全に一致する。余分・欠落・重複があれば rejected。
- `confirmed` のラベルは、次をすべて満たすこと。満たさなければ、そのラベルを `unclear` に落とす。
  - `evidence.file` が指摘と同じファイル。
  - `evidence.line` が 1 以上でそのファイルの行数以下、かつ指摘の `[line-3, end_line+3]` の範囲にある。
  - `quote` が空白以外の文字を 8 文字以上含む。
  - snapshot のそのファイルに `quote` の出現があり、その出現の開始位置の行が `evidence.line` である（複数行にわたる quote も開始行で照合する）。

### 8.4 round

| 用語 | 定義 |
|---|---|
| round 0 | implement → test ×N → （合格なら）review → （必要なら）confirm |
| round r（1 ≤ r ≤ N） | fix → test ×N → （合格なら）review → （必要なら）confirm。fix の step を計画するときに `run.round` を r にする（trigger は +1 だけを許す） |
| N | Goal の `max_fix_rounds`（0〜2） |

- テストが不合格なら、その round では review を行わず、判定へ進む。
- N=0 では fix を行わない。N=2 では fix は最大 2 回で、2 回目の fix の後の test と review までを行う。

### 8.5 判定

各 round の最後（test が不合格、または review と confirm が終わったとき）に、次の順で判定する。

| 条件 | 結果 |
|---|---|
| テストがすべて合格、verdict が approve、Confirmed と Unclear の blocking が 0 | 承認。release の step を計画する |
| Unclear の blocking が 1 件以上 | 最後の step を held（`unclear_finding`） |
| verdict が request_changes で、Confirmed の blocking が 0 | 最後の step を held（`unsupported_request_changes`）。修正も再レビューも自動では行わない |
| テストが不合格、または Confirmed の blocking が 1 件以上で、round < N | round+1 の fix を計画する。入力は Confirmed の指摘の ID と内容だけ（テスト不合格なら失敗の要約） |
| 同上で、round = N | 最後の step を held（`round_limit`） |

「最後の step を held」は、判定を下した step（test・review・confirm のどれか）を exited → held にすることを指す。運用者がその step を `rrx pipeline recover --then retry` で放棄すると、同じ step の attempt+1（再テスト・再レビュー・再確認）になる。round の数と上限は変わらない。

## 9. release と Goal の完了

- **Goal branch**: 最初の Task の claim tx で、`goal_create` の journal（`refs/heads/rrx/goal/<goal-id>`、old=0、new=Project の `base_branch` の head）を作る。その run の prepare の step で、実行境界の中で `git update-ref <ref> <new> 0000…` を行い、ref == new を観測したら done にする。
- **Task branch と worktree**: prepare の step で、`git worktree add -b rrx/pipeline/<task-id> <work_root>/wt/<run> <Goal head>` を行う。run の base は claim tx で記録した Goal の head。
- **release の step**（tick の中、規則 G）:
  1. tx: Goal が Running、run が running で承認済み、を確かめて journal（old=run.base、new=run.head）を pending で書く。
  2. Goal branch がどこかの worktree で checkout されていれば（`git worktree list --porcelain`）、held。
  3. `git update-ref refs/heads/rrx/goal/<id> <new> <old>` を実行し、結果の ref を読む。
  4. tx: Goal が Running であることを確かめ直し、ref == new なら journal を done、step を adopted、run を completed にする。ref が old でも new でもなければ held。
  - Project の slot は 1 つなので、Goal branch が release 以外で動くことはなく、常に fast-forward になる。
- **settle の規則**（再起動時と、held の journal の各 sweep）: 実行境界の中で ref を読む。
  - `task_release` の journal で、その run が abandoned なら、何もしない（`update-ref` を実行しない。§4.4 で放棄の前に決着させるので、通常は起こらない。trigger も abandoned の run の journal の変更を拒否する）。
  - ref == new: done（update-ref は境界の中で行い、制御は境界を待つので、release は後の pause・cancel より前に起きている）。done を書く tx は Goal の状態に依らない（すでに起きた ref の変化の記録であるため）。run の completed も同じ tx で書く。
  - ref == old: Goal が Running なら手順 3 からやり直す。Paused なら pending のまま待つ。Cancelled なら journal を aborted、step を interrupted にする（run は §4.3 の cancel で abandoned）。
  - それ以外: held。運用者が ref を old か new に戻した場合だけ、回復の command で新しい attempt が読み直す（`--then abandon-run` は §4.4 の 6 で決着させる）。
- **次の Task**: run が completed になったら `wake.notify_one()` を呼び、次の tick で次の READY な Task を claim する。ユーザーのプロンプトは要らない。
- **Goal の完了**: DAG の全 Task の run が completed になったら、kind=goal の run を claim する（§3.1）。TestsProfile の各コマンドを、Goal branch の head の clean checkout で test step として実行する。最後のコマンドの採用は tick の中（guard を保持）で行い、その tx で次をすべて確かめる: 現在の owner、**Goal が Running**、run と step の version、全コマンドが pass。満たせば、同じ tx で次を書く:
  - `pipeline_goal_completions` の行（全 run の ID と head を canonical にした sha256）
  - Goal run を completed に（trigger が完了の行を検査する）
  - Goal の state を Completed に（`goals` の body の state を version CAS で更新する。定義の trigger `state/runtime/schema.sql:21-30` には当たらない）
  - 監査イベント

  Goal が Paused なら何も書かず、最後の test の step は exited のまま残り、resume の後の tick が完了をやり直す。Cancelled なら §4.3 で interrupted にし、run は abandoned になる。fail のコマンドがあれば、その step を held（`goal_check_failed`）にする。LLM の出力は完了の判定に使わない。Goal run が completed になると Project の slot が空き、同じ Project の別の Goal が進める。

  fail のコマンドは、その結果を採用する時点で held にし、残りのコマンドを計画しない。回復は最後の step だけを対象にするので、`rrx pipeline recover --then retry` は、失敗したそのコマンドの attempt+1 を作る。合格すれば、残りのコマンドを順に計画する。

## 10. 読み取り API

`rrx goal status` / `rrx goal tasks` / `rrx events` で、CLI だけで状態を照会し、回復の command に必要な値を得られるようにする。読み取りは既存どおり短い Store lock の中の read-only の tx で行い（`runtime/control.rs:546-548`）、Git・プロセスの I/O をしない。

`PipelineRunView` を 1 つ定義し、Task の run と Goal 完了確認の run の両方に使う。内容: run の ID・kind・version・state・round・head、最後の step の seq・attempt・kind・state・version、Held の理由（固定の定数）、`limit_waits`（この run の上限待ちの回数）、他の step の実行待ちか daemon 全体の上限待ちかどうか。

| 対象 | 変更 |
|---|---|
| `ControlResponse::GoalFacts`（`runtime/control.rs:275-288`） | `pipeline: Option<PipelineGoalView>` を足す（`#[serde(default, skip_serializing_if = "Option::is_none")]`）。内容: モード、Goal branch の名前と journal の head、run の state ごとの件数、**Goal 完了確認の run の `PipelineRunView`**、完了の記録（head・`runs_sha256`・epoch）、`provider_wait`（daemon 全体の上限待ちの記録: state・`not_before`・`backoff_ms`・probe の step の参照。daemon 全体のものなので、どの Project の Goal にも同じ値を出す。行なし / clear なら省略）、`native_exhaustion`（Exhausted の native の観測の provider・bucket・`resets_at` の一覧。account は出さない。なければ省略） |
| `TaskFacts`（`runtime/control.rs:362-370`） | `pipeline: Option<PipelineRunView>` を足す |
| `TaskFacts.state` / `GoalFacts.states` | パイプラインの Goal では、run と最後の step から導く（下の表）。`tasks` の行の state は書き換えない |
| `GoalFacts.attention` / `goal_attention`（`state/runtime/goals.rs:632-649`） | パイプラインの Goal では、`UnavailableReason` の新しい値を返す: `PipelineHeld`（held の run がある。理由が `native_quota_exhausted` のときは `native_exhaustion` も出る）、`PipelineProviderWait`（daemon 全体の上限待ちが clear でなく、run の最後の step が planned の Agent の step）、`PipelineUnqualified`（Q が refused）、`ExecutionModeMismatch`（rrxd のモードと違う）、`PipelineActive`（それ以外。`dispatch_available=true` のときの値） |
| `runtime_goal_facts` / `runtime_goal_task_page`（`state/runtime/goals.rs:300-360,520-626`） | パイプラインの Goal では、`waiting::observe` の代わりに上の view を作る。page の上限（`GOAL_TASK_PAGE_BYTES`）の検査は既存どおり |
| `rrx events` | パイプラインの監査イベントを、既存の scope で返す |

| run と最後の step | `TaskFacts.state` |
|---|---|
| run なし | 保存された Task の state |
| run held | WaitingHuman |
| step planned（Agent の kind）で daemon 全体の上限待ちが clear でない / 他の step の実行中 | WaitingQuota / WaitingCapacity |
| prepare / implement | Implementing |
| test | Testing |
| review / confirm | Reviewing |
| fix | Fixing |
| release | ReadyForPr（Goal branch への統合待ち。PR は作らない） |
| run completed / abandoned | Completed / Cancelled |

表示する文字列は、固定の定数か、秘密スキャンを通して保存した抜粋だけ（§11）。

## 11. 秘密スキャン

`SecretScanner` は固定の pattern（秘密鍵の header、AWS のアクセスキー、GitHub の token、`xox[bp]-`、汎用の `*_SECRET=` / `*_TOKEN=` など）を持つ。次の 3 つの関門の前に必ず当てる。

| 関門 | 対象 |
|---|---|
| 次の Agent への prompt | 組み立てた prompt の全体（Task の文、diff、指摘、テスト失敗の要約を含む）。`prompt.md` に書く前に当てる |
| DB への抜粋の保存 | `body` に入れるすべての文字列（verdict の summary・title・rationale、ラベルの reason・quote、テスト失敗の要約（失敗したコマンドの stdout / stderr の末尾 8 KiB）、stderr の末尾、`last.md` の抜粋） |
| 採用の検査 | diff の追加行（§6.3-4）、`last.md`、`stdout.jsonl`、stderr、verdict と confirm の原本、test の stdout / stderr |

- 該当したら、その step を held（`secret_detected`）にする。抜粋は保存せず、pattern の ID と位置（ファイルと offset の digest）だけを記録する。表示もしない。
- 原本の出力ファイルは attempt ディレクトリに 0400 で残すが、表示・送信はしない。
- 表示（§10）は、固定の定数とスキャン済みの保存抜粋だけから作る。

## 12. 脅威モデル

**守るもの**: オーナーの資格情報（`~/.aws`、`~/.ssh`、keychain、gh の token）、repo と Goal branch の正しさ、rrx の状態（SQLite・epoch）、他 Project の作業、オーナーの Mac の可用性。

**主体**: オーナー（運用者）、rrxd、Codex / Claude の CLI とその provider、repo の内容（信頼しない）、Agent が書いたコード（信頼しない）。

**信頼の境界**: rrxd ↔ Agent のプロセス、Agent ↔ provider（外部への送信）、Task worktree ↔ repo の共通 dir、テストのプロセス ↔ ホスト。

| 脅威 | 対策 | 残余リスク |
|---|---|---|
| repo の内容による prompt injection（指示の書き換え、偽の verdict の誘導） | ・信頼しない内容は nonce 付きの区切りでデータとして枠付けする。<br>・verdict の schema を固定する。<br>・指摘は provider をまたいで確認する。<br>・テストを関門にする。<br>・scope を検査する。<br>・Reviewer の approve だけでは承認しない。 | 両方の provider が同じ誘導に乗り、テストも通る変更は検出できない（受け入れる） |
| repo 側の設定がコードを実行する（Git hook、filter・diff・merge の driver、`.claude/settings.json` の hook、`.codex/config.toml`） | ・rrx の Git は global / system の設定を読まず、hook・fsmonitor を止め、diff の外部 driver と textconv を使わない（§6.1）。<br>・repo の設定と、深さを問わない `.gitattributes` に driver の宣言があれば拒否する。<br>・commit は検査した bytes から `--no-filters` で組み立てる。<br>・Claude は `--setting-sources ""`。<br>・Codex は `trust_level="untrusted"` と専用の `CODEX_HOME`。<br>・step の前後で Git 設定の digest を比べる。<br>・指示ファイルの変更は既定で拒否する。 | Codex は `AGENTS.md` を必ず読み、Claude も `CLAUDE.md` を読みうる（指示による誘導が残る） |
| Reviewer / Confirmer による書込み | 読み取り専用の snapshot（chmod a-w）、書込みツールの拒否、前後の tree digest の照合。Reviewer と Confirmer の結果は判定の JSON だけを読む | なし（変化を検出したら held） |
| Agent が自分で commit・ref の変更・hook の設置を行う | 採用するのは、記録した SHA の rrx commit だけ。HEAD・ref・`.git` の digest を照合する。trailer は根拠にしない | なし |
| 環境変数からの秘密の漏洩 | 空から組み立てる（§5.4）。渡した集合 ⊆ 許可した集合を、テストで検査する。`rrx daemon start` は `env_clear()` とモード別の固定 allowlist で `serve` の環境を組み立てる。cli_pipeline は HOME・USER・LOGNAME・PATH・TMPDIR・LANG・LC_ALL・LC_CTYPE・LC_MESSAGES・TERM・XDG_CONFIG_HOME・XDG_DATA_HOME・XDG_STATE_HOME・XDG_CACHE_HOME だけを継承する。managed_native は従来の固定リストを使う。直接の `rrx serve` と `rrx` CLI 自体も運用者が資格情報のない環境から実行する。rrxd の両実行モードの起動時に、環境の変数名を補助的に検査して拒否する（安全の保証ではない）。拒否する名前は、`GITHUB_TOKEN`・`GH_TOKEN`・`GH_ENTERPRISE_TOKEN`・`GITHUB_ENTERPRISE_TOKEN`、ASCII 大文字化した名前が `AWS_` で始まるもの、ASCII 大文字化した名前に `TOKEN`・`SECRET`・`PASSWORD`・`API_KEY` を含むもの。エラーは名前だけを示し、値は示さず、環境を unset しない | 空から組み立てる環境は秘密の直接の継承を防ぐが、他プロセスから取得できる引数と環境の読取りは防がない。自己を target にした process-info の許可でも、親への libproc `proc_pidinfo(PROC_PIDTASKALLINFO)` は成功したため、他プロセスの読取りを遮断できるとは扱わない。引数・環境の読取りは無条件の `(allow sysctl-read)` を通る。オーナー側の測定では、その後に `(deny sysctl-read (sysctl-name "kern.procargs2") (sysctl-name "kern.procargs") (sysctl-name-prefix "kern.proc."))` を置いても、`kern.proc*` を除く名前の allowlist にしても、MIB による `KERN_PROCARGS2` / `KERN_PROC_PID` の読取りを遮断しなかった。`sysctl-read` をなくすと Homebrew python3 と cargo が動かない。採用した試験用 Seatbelt profile の下から、同一利用者の Homebrew python3 の親の環境に置いた秘密でない目印を取得できることを実測した。Apple platform binary は環境が除去されるが、引数は保護されない。Agent の段階での同じ読取りは未確認である。一時的な資格情報も対象となり、出力・cache・後続の Agent を通る流出を、network の拒否と秘密スキャンで防ぐことは保証できない。資格情報が存在する運用での影響は High である。段階 1 を始める条件は、許可した変数だけで rrxd を起動すること、起動時の補助的な拒否（トークンに似た名前の変数があれば起動しない）、同一利用者の資格情報を環境か引数に持つプロセスと併存しないこと（短時間のプロセスも対象）、この残余へのオーナーの書面による受入とする。専用 macOS 利用者による分離とその検証は、B13 を始める前の必須条件とする。完了の条件は、Agent と試験の利用者を token を持つプロセスと helper から UID とアクセス権で分離すること、秘密でない目印で境界を確認すること、token を rrxd と子の環境と引数に置かない設計である。オーナーは 2026-10-20 にこの残余を書面で受け入れた。 |
| **Agent がユーザーの読めるファイルを何でも読む**（`~/.aws` などを provider に送る） | v1: すべての出力経路の秘密スキャン（§11）と、オーナーの書面による受入 | **残る**。workspace-write も read-only も、ディスク全体を読める。provider への送信そのものは止められない。根本の対策は後続（#98）: (A) 専用の macOS ユーザーで Agent と試験を分離し、検証する unit を B13 開始前の必須条件とする（完了条件は上の行）。(B) Codex の管理設定 `requirements.toml` の `[permissions.filesystem] deny_read`（0.162.1 で使えるかは未確認）。(C) Claude の `--settings` の `permissions.deny` で Read を制限する（部分的）。 |
| テストが、Agent の書いたコードをオーナーの権限で実行する | clean checkout で実行する。`/usr/bin/sandbox-exec` の Seatbelt profile（Mach lookup の許可なし、network / ローカルソケット拒否、書込みは checkout・TMPDIR・運用者が宣言した cache と `/dev/null` の data write だけ）で包む。空から組み立てた環境。再利用の前に symlink をたどらず、`.git` gitfile 以外の ignored・untracked を含むツリー全体を検査する | ファイルの読み取りは制限されない。秘密スキャンは部分的な検出であり、他プロセスの引数と環境の残余と開始条件は上の行のとおり。cache は全 Project で共有し、汚染で別 Project のテスト結果に影響しうる。この Project 間の共有 cache の残余も、オーナーは 2026-10-20 に書面で受け入れた。書込みの grant は inode 単位の隔離を保証しない。sandbox の中から write grant の外のファイルへの hardlink の作成は、ln と perl の link() で拒否を確認した。checkout の再利用の前には、ignored を含む通常のファイルの link 数を検査し、2 以上か検査できなければ再利用せず作り直す。TMPDIR は試行ごとに新しく作る。これらと外部からの同時の変更がない条件の下で、checkout と TMPDIR の残余を Low とする。cache は既定で使わない。有効にする場合は、新しい専用の cache か、既存の外部との hardlink を除いた cache に限り、外部のツールによる利用、外部との hardlink の共有、外部からの同時の変更を禁止する。この運用に依存する残余を Medium として受け入れる。オーナーは 2026-10-20 にこの残余を書面で受け入れた。 |
| admit 後のテストの差し替え | TestsProfile を読むたびに body と digest を照合し、各起動の直前に `recheck` でコマンドと実行ファイルの hash を照合する（§7.2） | 照合から exec までの短い窓での差し替え（既存の経路と同じ） |
| network | Codex のツールは network を拒否する。Claude には Bash も web もない。テストは Seatbelt で拒否する | CLI 自身が provider と通信する（仕様どおり） |
| 信頼しない verdict JSON | strict decode と上限、未知フィールドの拒否、ID の一意性、ラベル集合の一致、存在しない file / line は invalid、quote と行の照合 | なし |
| 実 pin の wire を成功と誤認し、採用や daemon 全体の上限待ちを進める | exact pin → profile の閉じた表、完全な出力と矛盾の検査、custody の last-output 検査を維持する。未知・不完全な形は `other`。harness で秘匿した shape を観測し、対応を照合する（§5.6、F-w1〜F-w6・F-a7） | 初期の 2 pin は文書と合成 fixture を根拠にし、実 wire との一致は次の pass で確かめる。本文を除いた shape だけでは実版の上限の固定文言まで確認できず、その相違は `other` となって待ちが延長される |
| 古い owner・二重の rrxd | `owner.lock` の flock（`execution/owner.rs:225`）、全書込み API での現在の owner と version の検査（§2.4）、run・step・journal・上限待ちの記録の行に書いた epoch の trigger（§2.3）、control 要求の epoch の照合（`runtime/control.rs:460-469`） | なし |
| モードの切替で、もう一方の経路のプロセスが残る | epoch の前の静止の確認（永続状態と cookie の探索。§1.1）。epoch の更新を死亡の根拠にしない | cookie を消した孤児と、group を抜けた子孫は検出できない。環境を消された platform binary の子孫も、`discover` のままなので検出できない（記録して受け入れる。§13.3） |
| 停止・cancel と起動・採用・release・完了の競合 | `control_admission` による直列化（§1.2）と規則 G（最終 tx での Goal の Running の検査） | なし |
| 回復の要求の重複と、放棄した run の ref の更新 | 再送の照会を guard の前・guard の中の読み取り・最終 tx の 3 か所で、現在の状態の検査より先に行う。`--then abandon-run` は開いた release の journal を先に決着させ、abandoned の run の journal は trigger で変更できない（§4.4・§9） | 決着の ref の読取りと最終 tx の間に、運用者が手で ref を動かす窓（記録した ref と食い違えば、次の settle が held にする） |
| 起動結果が不明・孤児プロセス | Unknown / Held、cookie による探索、再実行しない。死亡を確認できない間は全 Project の起動を止める | cookie を自分で消した孤児は検出できない。環境を消された platform binary の子孫は、実行ファイルが 4 つの場所にあれば確認できないとして held にする（§3.5.2。R3・R5・R6・R7・R8 は残る） |
| provider の上限を使い切る | daemon 全体で 1 step ずつ。既知の上限の終了は daemon 全体の 1 つの記録で backoff を待ち、期限の後は 1 つの起動が既知の成功で終わるまで次を起動しない（それ以外の終わり方は延長）。native の Exhausted の観測が残る間は起動しない（§2.6） | 分類器が上限を認識できない失敗は、上限待ちを始めない（運用者の判断になり、Held に倒す） |
| Goal branch の破損 | ref-CAS の journal、fast-forward だけ、checkout 中なら拒否 | 運用者が手で ref を動かした場合は held になる |

## 13. 影響範囲分析

### 13.1 変更・追加の対象

| 種別 | 対象 | 影響を受ける箇所 | 影響の種類 | 対応 |
|---|---|---|---|---|
| 定数 | `SCHEMA_VERSION` 11→12、`READ_INDEX_LAYOUT_VERSION`=11 を追加 | `state/mod.rs:32,189-193,202-307`、`rrx_writer_contract_version`（`state/mod.rs:135-141`） | 互換性（v12 の DB は旧バイナリが拒否する。仕様どおり） | §2.1 の 4 経路 |
| 関数 | `validate_layout`（版による参照 layout の組立て） | `state/managed_binding/schema.rs:249-281` | 互換性 | §2.1。v10 と actual9 の参照は変えない |
| 定数 | `PIPELINE_TABLES`（新規、8 表）。`MUTABLE_TABLES` は変えない | `MUTABLE_TABLES` の消費者: `state/managed_binding/schema.rs:160,190`、`state/read_index_tests.rs:199`、`state/managed_binding/schema_tests.rs:32,318`、`state/execution/verification.rs:1149,1187`、`state/execution/tests.rs:370,413`、`state/execution/source_recovery.rs:594,633`、`state/runtime/tests.rs:29,65`、`state/execution/native_results_tests.rs:24,83` | テスト（writer guard の総数を数える `read_index_tests.rs:199`・`execution/tests.rs:413` は、v12 で 8 表 ×3 だけ増える） | 総数の検査を `MUTABLE_TABLES` と `PIPELINE_TABLES` の合計にする。各表の guard を照合するテストは、`PIPELINE_TABLES` にも同じ検査を当てる |
| DB | 新しい表 8 つと trigger | `state/execution/pipeline.sql`、`state/sqlite_build_tests.rs` | 機能 | 追加する |
| 関数 | `RuntimeOwner::open`（モードを受け取り、`Store::open` と `begin_execution_epoch` の間で静止を確かめる） | `execution/owner.rs:270-279`。呼び出し元は daemon の起動と多数のテスト | 起動・互換性 | `open` は `managed_native` のまま残し、モード付きの入口を足す。パイプラインの行がない DB では確認が件数 0 で通り、挙動は変わらない |
| 関数 | `quotas::windows` | `state/execution/quotas.rs:476-502`、呼び出し元 `:76,94,186` | なし（private → `pub(super)`。挙動は変えない） | パイプラインは読むだけ。`quota_windows` の書き手（`quotas.rs:83`、`native_phase/live_quota.rs:370`）は変えない |
| 型 | `Config.execution`・`Config.pipeline`（任意） | `config.rs:226,376-428`、`runtime/control.rs:540-545`（policy digest） | 互換性 | 既定値は serialize しない。未設定なら digest が変わらないことをテストで確かめる |
| loop | sweep | `runtime/service.rs:76-161`（`:130` をモードで分ける）、`:168-200`、`runtime/stop.rs:7-34`（`PipelineSteps` を足す） | 機能・停止 | tick と停止の site を加える |
| 境界 | `control_admission` | `runtime/admission.rs:37-41`、`runtime/control.rs:641,667,711`、`runtime/service.rs:59,170`、`runtime/task_driver.rs:79`、`runtime/phase_supervisor.rs:1021-1322`、`runtime/installation.rs:110-251`、`runtime/project_control.rs:219` | 停止・応答時間（tick が spawn・短い Git 操作・採用の検査の間だけ保持する。回復の `abandon-run` は journal の ref の読取り 1 回を足す） | try_lock で取り、CLI の実行や終了待ちは外で行う。他の消費者の保持の仕方は変えない |
| SQL | native の除外 | `state/runtime/driver/candidates.rs:49`、`claim.rs:365-369` | セキュリティ（経路の分離） | `NOT EXISTS goal_execution_profiles` |
| 関数 | `reconcile_runtime_attention` | `state/runtime/service.rs:25,45-48,70` | 表示 | パイプラインの Goal の Task を除く |
| 関数 | `accept_runtime_goal` | `state/runtime/goals.rs:28-207` | 機能 | モードの検査、プロファイルの行の挿入、検査の追加 |
| 関数 | `set_runtime_goal_lifecycle` | `state/runtime/goals.rs:361-516`、`runtime/control.rs:710-733` | 機能 | 違うモードの resume の拒否。commit の後に StepCustody へ停止を通知する（行の更新はパイプラインの tx が行う） |
| 型 | `GoalPlan.execution`（任意） | `runtime/goal.rs:57,166-235`、`runtime/control.rs:29-33,638` | 互換性（省略したときは従来どおり） | `#[serde(default)]` |
| API | `ControlAction::PipelineRecover` / `ProjectAdmitTests` | `runtime/control.rs:24-139,752-768`、`main.rs:142,487` | 機能・認可（OS UID・epoch） | 追加する。既存の `TaskRetry` は Unavailable のまま |
| API（読み取り） | `GoalFacts`・`TaskFacts`・`UnavailableReason` | `runtime/control.rs:275-288,362-370,389-421`、`state/runtime/goals.rs:300-360,520-649`、`runtime/control/recorded.rs:60-77`、`runtime/control/read.rs`、`state/runtime/read.rs`（attention queue） | 互換性（任意フィールドと enum の値の追加） | §10。native の Goal の応答は変えない。CLI の表示（`main.rs` の status / tasks）に pipeline の欄を足す |
| 関数 | `ManagedVerifier::admit_tests` | `execution/verification/mod.rs:226-247`、`state/execution/verification.rs:638,669-677` | 機能（初めて本番から呼ぶ） | `ProjectAdmitTests` から呼ぶ。1 Project に 1 プロファイルで不変（`state/execution/verification.sql:1-10`） |
| 関数（再利用） | `hash`・`AdmittedProfile::validate` / `recheck` | `execution/verification/plan.rs:61,205-223` | なし（呼ぶだけ） | パイプラインの profile の読み戻しと、各 test の起動前に呼ぶ |
| 可視性 | `retain` / `inspect` | `execution/verification/collector.rs:218,240` | なし（`pub(super)` → `pub(crate)`。挙動は変えない） | 既存の呼び出し元（verification の中）はそのまま |
| 環境変数 | `RRX_PROCESS_COOKIE` | 既存の消費者は `execution/resources.rs:75`、`execution/cleanup.rs:85` | なし（値の空間は UUID で別） | パイプラインの値も UUID から作る。静止の確認は native の cookie を探索するだけで signal を送らない |
| 環境変数 | `CODEX_HOME`・`TMPDIR`・`PATH`、rrx の Git の `GIT_CONFIG_NOSYSTEM`・`GIT_CONFIG_GLOBAL`・`GIT_ATTR_NOSYSTEM`・`GIT_INDEX_FILE` | 子プロセスの中だけ（rrxd 自身の環境は変えない） | パスの長さ（TMPDIR は 60 byte 以下） | 起動時に検査する |
| 入力の一覧 | 必須 flag・許可する環境変数・固定の拒否パス・秘密の pattern・上限と成功の分類規則・driver の拒否キー | パイプラインだけ | 安全 | 広がる向きを検出するテストにする |
| 定数・型・関数 | provider / exact pin → wire profile の閉じた表、profile enum、`classify_codex` / `classify_claude`（`execution/pipeline/classify.rs`） | `execution/pipeline/custody.rs:438-440`、`classify/tests.rs`、既存の fixture pin を使う偽 CLI | 機能・安全（実 pin が分類可能になる） | §5.6.1〜§5.6.2。未知 pin は `other`、enum の profile を通らず outcome を返す表は作らない。既存の合成 pin と fixture を維持し、実 pin でも同じ分岐を検査する |
| 関数・消費者 | `check_last`、`provider_outcome=success`、custody の `last_output_invalid` と last receipt | `execution/pipeline/custody.rs:562-567`（欠落時の `Ok(None)`）・`:421-431`（last の検査と issue の設定）・`:435-440`（issue による `other`）、`execution/pipeline/adoption.rs:38`、`state/execution/pipeline/review.rs:113`・`rounds.rs:607`、上限待ちの解除 trigger（`state/execution/pipeline.sql:294`）と §2.6 の settle | 機能・採用・安全（Executor / Confirmer の last 欠落も拒否する。成功誤認は、採用前に daemon 全体の待ちを解除する） | 欠落も `last_output_invalid` にする製品変更を行う。採用・解除の条件は緩めず、profile と custody の正負の検査で守る。両 role で last 不正・欠落なら成功形が `other` になり、probe の待ちが延長されることまで確認する |
| 証跡・表示の消費者 | 未対応 pin の exit evidence の `wire: "unmapped"` | `execution/pipeline/custody.rs` の exit 記録、Goal/run status と held の理由の表示 | 運用（未対応 pin を他の `agent_failed` と区別する） | Q を通った観測起動は維持し、閉じた固定値だけを既存の証跡と表示経路で伝える。生の pin や出力本文を理由に写さず、分類は `other` のままとする |
| テスト専用の関数・出力 | `state/execution/pipeline/tests/real_cli.rs` の `envelope_keys`・Agent step の観測・`summary.json` | 同ファイルの `pass_flow` / `run_pass`・偽 CLI pass・秘密の秘匿テスト、`scripts/real-cli-pass.sh` が返す要約 | 検証・秘密保護（出力由来の値が要約に増える） | §5.6.3。採用の前と失敗時にも bounded な shape を採り、raw / decoded / 投影後を scan。`review::scan_raw` / `scan_strings` と通常ファイルの検査を再利用し、製品の分類器・DB schema・Runtime API に観測機能を足さない |
| 定数 | `global_max_sessions` の既定 12（`config.rs:64`）、`MVP_PROJECT_TASKS`=1（`config.rs:17`）、`PHASE_SLOTS_PER_PROJECT`=4（`runtime/phase_supervisor.rs:22`）、native の上限の既定（`execution/quota.rs:18-20`） | 変えない | | パイプラインは 1 step ずつなので、`MVP_PROJECT_TASKS` だけを使う |
| 定数 | 上限待ちの backoff（初回 60 秒、倍、上限 30 分）。native の `quota_pools.backoff` の範囲（`quotas.rs:317`、`:508`）と同じ値 | `pipeline_limit_wait` の CHECK と trigger、§2.6 | なし（パイプラインだけ。native の値は読まない・変えない） | trigger とテストで固定する |
| 関数・型（tracker の公開 API） | `discover_launched`・`launch_mark`・`owned_leader`、`Launch`・`OwnedLeader`（新規） | `execution/pipeline/custody.rs:189-237`、`execution/pipeline/launcher.rs:298-329`、§4.2 の running の行と §4.4 の 4（B10） | 機能・安全 | 追加する |
| 型（公開） | `Coverage` に 5 つの欄、`limited()` に `ancestry_unconfirmed`・`clock_behind`、`Discovery.unconfirmed` | `Coverage` / `Discovery` の literal: `custody/tests.rs:92-94`。`limited()` の消費者: `custody.rs:71`、`execution/owner.rs:297`、`execution/cleanup.rs:116`。文字列: `cleanup.rs:113` | 互換性（`Default` で埋まる。`discover` は新しい limited の欄を立てない） | literal を直す。cleanup の文字列は変えない |
| 関数（tracker の内部） | `scan`・`Source`・`cookie_matches`（3 値）・`KinfoProc`（`e_ppid` の型付け、`rest` の分割） | `rrx-process-tracker/src/macos.rs:13-27,148-216,248-380` と同じファイルのテスト | 機能 | fake の `Source` を広げる。ZB1〜ZB22 は変わらずに通ること |
| 関数（既存） | `discover` | `execution/owner.rs:291-301`、`execution/cleanup.rs:83-125`、`state/execution/pipeline/tests/b3.rs:460`・`b4.rs:118`、`rrx-process-tracker/tests/discovery.rs` | なし（`environment_absent` を数えるだけ） | 変えない |
| 関数 | `custody::birth`（削除）、`confirm_death`（`launch` と birth を受け取る） | `launcher.rs:300,310`、`custody.rs:337`（監視タスク。停止・pause・cancel の経路を含む） | 機能・安全 | §3.5.3 |
| Store API | `pipeline_begin_launch`（`launch` を受け取る）、`PipelineDeathEvidence`（`launch_checked`・`leader_birth_matched`） | launcher と Store のテスト（`pipeline_begin_launch` と death evidence を使うもの） | 機能 | 追加する |
| DB の trigger | `pipeline_steps_transition`（`state/execution/pipeline.sql:113-160`） | v12（この branch だけ。未リリース） | 安全 | §3.5.2・§3.5.3 の条件を足し、正負の SQL テストを書く |
| 定数 | `LAUNCH_CLOCK_MARGIN`=60 秒、`ANCESTRY_DEPTH`=32、`unconfirmed` の上限 16、`SIDL`〜`SSTOP`・`P_TRACED`・`PROC_FLAG_TRACED` | tracker だけ | 可用性（M を大きくすると held が増える） | EA10 で境界を固定する |
| 外部の値 | `kern.bootsessionuuid`、壁時計 | tracker だけ | 可用性 | HE3。読めなければ別の boot を除外しない |
| マスター設計書 | `agent-execution.md` の leader の birth（162-166 行）と死亡確認・macOS の探索（238-280 行） | — | 文書 | 実装 PR で更新する |
| 共有のコード | native の cleanup と backlog（`execution/cleanup.rs:83-125`）、モードの静止の確認の native の cookie（`owner.rs:291-301`） | `origin/main` の同じ tracker | 安全（同じ見落としが残る） | 変えない。native の Unit には `launch` の記録がない。§13.3 |

モードの静止の確認で pipeline の cookie を `discover` のまま使ってよい理由: 生きたプロセスを持ちうる step（launching・running・unknown・死亡を確認していない held）は `pipeline_processes` に数えられ、探索の前に Busy になる（`state/execution/pipeline.rs:762-803`）。探索に回るのは、spawn しなかった step の cookie だけである。

### 13.2 影響なしの根拠

- **native の `start_phase` 経路**: `managed_native` の rrxd では、`cli_pipeline` の Goal を candidate と claim の段階で除くことと、起動時の静止の確認（パイプラインの行がなければ件数 0 で通る）だけで、その他は変えない。既存の Goal には `goal_execution_profiles` の行がない。`cli_pipeline` の rrxd は native の admission を呼ばない。
- **quota の表**: パイプラインは `quota_pools`・`quota_windows`・`quota_leases`・`quota_waiters` を書かない。読むのは、静止の確認と tx1 と読み取り API での `quota_windows`（`windows` 経由）と、静止の確認での `quota_leases` の件数だけ。`quota_pools` と `quota_policy` は使わない。
- **`GenericCliAdapter` と `ProcessGroup`**: 変えない（§16 D5）。`cleanup_group`・`observe_exit`・`reap` は既存の `pub(crate)` をそのまま使う。
- **`collector::collect`**: 呼ばない。可視性を変えるのは `retain` / `inspect` だけ。
- **Docker・Grok・`ResultStore`・`AttemptManager`**: パイプラインから呼ばない。
- **M0 branch**: 今は影響しない。M0 を再開するときは、M0 の schema を v13 へ番号を振り直す（§16 D1）。
- **D26 の wire 対応**: 製品の変更先は pipeline の分類器と入力規則、Executor / Confirmer の last 欠落の拒否、未対応 pin の固定の exit evidence とその理由表示であり、CLI の argv・Q の pin 照合・DB schema・native adapter は変えない。shape は `#[cfg(test)]` の harness が既存の receipt と原本から作る検証用要約で、shape を DB に保存する契約や製品の表示 API は増やさない。`wire: "unmapped"` は既存の exit evidence と status の理由表示で伝える。送信記録の対象も従来の入力 packet のままとする。

### 13.3 別 Issue に記録するもの

- 既存の `verification_profiles` に `no_replace` の trigger がない（`state/execution/verification.sql:1-10`）。同じ key の REPLACE で body を変えられる。パイプラインは読むたびの digest 照合で拒否する（§2.3・§7.2）が、表そのものの保護は別 Issue で扱う。
- macOS の tracker では、環境を消された platform binary の子孫を、`origin/main` の native の経路でも同じ形で見落とす。対象は、execution cleanup とその backlog（`execution/cleanup.rs:83-125`）と、`cli_pipeline` への切替の静止の確認（native の cookie）である。native の Unit には起動の記録（`launch`）がないので、この unit では変えない。別 Issue で扱う。

### 13.4 回帰の確認（各 unit の最後に行う）

`cargo test --workspace` の全件、`crates/rrx/tests/runtime_cli.rs`・`crates/rrx/tests/daemon.rs`・`crates/rrx/tests/goal_cli.rs`・`crates/rrx/tests/state.rs` の明示実行、v0・v9・v10・v11 の DB を v12 に版上げするテスト（途中の失敗で全体が rollback されることを含む）、`managed_native` の Goal の既存のテストがすべて通ること。

## 14. テスト方針

- **偽の CLI**: `crates/rrx/tests/support/fake_codex.py` と `fake_claude.py`（既存の `crates/rrx/tests/support/grok_fake.py` と同じ流儀）。
  - 環境は空から組み立てるので、シナリオは同じディレクトリの `scenario.json` から読む。テストごとに別のディレクトリへ置く。
  - `--version`、`--help`、`login status` / `auth status`、`exec` / `-p` を実装する。
  - 書くもの、出力、終了コード、遅延、stdin を読まない・先に大量に出力する、子プロセスの fork（cookie を持ったままのもの・消したもの）、フィールドの欠落・重複、provider の上限の出力、認証の失敗の出力を、シナリオで指定する。
- **障害注入マトリクス（AC-8）**:

| ID | 注入 | 期待する結果 |
|---|---|---|
| F-a1 | Executor が非 0 で終了する | rejected、run held、他 Project は進む |
| F-a2 | 出力の上限を超える / timeout / stdin を読まない / stdin を読む前に大量に出力する | 期限内に group を kill し、死亡を確認して rejected。launching のまま残らない |
| F-a3 | scope 外を変更する / symlink / 秘密の pattern / 自分で commit する（正しい親と偽の trailer を含む） | rejected または held。commit を採用しない |
| F-a4 | repo に filter driver の宣言（`.gitattributes` の深い階層、repo の設定） | prepare か採用検査で held。driver のコードは実行されない（偽 driver が作る印のファイルがないことで確かめる） |
| F-a5 | provider の上限の出力で終了する / 回数の上限まで繰り返す / 分類できない失敗 / native の `quota_windows` に Exhausted の観測（`resets_at` と `next_probe_at` を過ぎたものを含む） | limited、上限待ちの記録の期限の後に step_base へ初期化した attempt+1 が起動する。回数を超えたら held。分類できない失敗は rejected で、上限待ちを始めない。Exhausted の観測があれば、時刻にかかわらず起動せず held（`native_quota_exhausted`）で、`rrx goal status` に理由と観測の要約が出る。パイプラインは quota の表を書かない |
| F-a6 | Project A の step が既知の上限で終了した後、Project B の Agent の step を計画する / A の run を `--then abandon-run` で放棄する / rrxd を再起動する | B の step は期限の前に起動しない（WaitingQuota、`provider_wait` が表示される）。放棄と再起動の後も、記録の期限・幅・state が変わらない。期限の後は Agent の step が 1 つだけ起動する |
| F-a7 | 期限の後の最初の step（probe）を、中断（停止）・認証の失敗・unknown（spawn の Err）・死亡の未確認・分類できない失敗で終わらせる / 既知の成功で終わらせる | 前者はどれも記録が同じ backoff で延長され、延長した期限まで次の Agent の step を起動しない（probe の終了の直後に別の Project の step も起動しない）。後者は clear になり、通常どおり起動する。時刻の経過だけで clear にはならない |
| F-b1 | `request_changes`、指摘が Confirmed | fix に入り、round が +1 |
| F-b2 | 上限で承認に至らない（N=0・1・2）/ Unclear の blocking / Confirmed の blocking がない request_changes | すべて held。N 回目の fix の後のレビューまで行われる |
| F-b3 | verdict の未知フィールド・重複キー・重複 ID・存在しない行・model の不一致・snapshot の変化、ラベルの余分・欠落・空 quote・`evidence.line` が 0 / 範囲外 / quote の開始行と不一致 | rejected または held（quote と行の不一致は Unclear に落ちて held） |
| F-c1 | spawn が Err を返す（存在しない実行ファイルではなく、hook で注入する） | unknown、run held、再実行しない |
| F-c2 | tx1 の後、spawn の前に crash する | 再起動時に unknown・held |
| F-c3 | tx2 が失敗する | group を kill し、確認できれば 1 回の commit で rejected（`record_failed`）と run の held、できなければ launching のまま（再起動時に unknown） |
| F-c4 | claim の後、branch 作成の後、worktree の部分作成の後に crash する | 何もなければ再作成、完全なら採用、部分なら held。既存のものを消さない |
| F-c5 | exited の記録の後 / committed の記録の後 / update-ref の後に crash する | §6.3 の表のとおり。記録した SHA だけを採用する |
| F-c6 | test の leader が platform binary である | 起動し、合格として採用される（HE8） |
| F-d1 | running 中に正常停止する | interrupted、attempt+1 で再開する |
| F-d2 | running 中に SIGKILL で rrxd を止める | 再起動時に引継ぎ、custody を失ったとして held。cookie の孤児がいれば pid を記録する |
| F-d3 | release の `update-ref` の前後で crash する | ref==old なら（Goal が Running のときだけ）やり直し、ref==new なら done、それ以外は held |
| F-d4 | tx1 の後・spawn の前、および journal の作成の後・ref 更新の前に、pause / cancel / stop を割り込ませる | 制御は境界を待ち、制御の受理の後に新しい起動も release も起きない |
| F-d5 | step の exited の後・採用の前、および最後の test の exited の後・Goal 完了の前に、pause / cancel を割り込ませる | 採用も Goal の Completed も起きない。pause は resume の後に採用・完了をやり直す。cancel は interrupted と run の abandoned |
| F-d6 | setsid で group を抜けた、4 つの場所の platform binary の子孫 | held（`death_unconfirmed`）（HE5・HE8） |
| F-e1 | 引継ぎの後に、古い epoch の書き手が tx1 / tx2 / step の終了の記録 / 採用 / run の完了 / recovery / journal の settle / Goal の完了 / 上限待ちの記録の更新を書く | API の検査と trigger の両方で拒否する（片方を外しても、もう片方で拒否されることを確かめる。step の更新も trigger だけで拒否される）。spawn しない |
| F-e2 | 再起動の前から held の run に `rrx pipeline recover` | 引継ぎ済みの現在の epoch で新しい attempt を作る。旧 attempt は再実行しない |
| F-f1 | tick と `pipeline recover` が同じ step を同時に更新する / 同じ期待 version の recover を 2 つ（request ID は別） | 片方だけが成功し、負けた側は何も書かず spawn しない |
| F-f2 | 新しい request ID の初回の recover（retry と abandon-run）、その再送、古い期待 version、途中の失敗 | 初回は成功（tx の順序）。再送は作業領域を変えずに同じ応答。古い version はファイルを変えずに拒否。途中の失敗は全体が rollback |
| F-f3 | 同じ request ID の recover を 2 つ同時に送る（どちらも guard の前の照会で ack を見ない状態にする） | 両方が同じ保存応答を受け取る。cookie の探索・作業領域の初期化・ref の読取り・書込みは 1 回だけ（後の要求は guard の中の照会で返る） |
| F-g1 | Project A の run を死亡確認済みで held にする / A に死亡を確認できない step を残す | 前者は Project B の run が完了まで進む。後者は B も起動を待ち、理由が表示される |
| F-g2 | Goal run の合格 / 不合格 | 合格なら Goal Completed と run completed で、同じ Project の別 Goal が進む。不合格なら held で、Goal の view だけで recover を組み立てて再テストできる |
| F-h1 | `abandon-step` を実行したとき、cookie を持つプロセスが残っている | 拒否する |
| F-h2 | attempt 16 の step が中断・上限で終わる / その run に `--then retry` と `--then abandon-run` / cancel で全 step の死亡を確認できる場合とできない場合 | 16 回目は held。retry は拒否、abandon-run で run が abandoned になり slot が空く。cancel は確認できれば abandoned、できなければ held のまま |
| F-h3 | `task_release` の journal が pending / held の run（checkout 中で held にしたもの、ref を手で動かしたもの）に `--then abandon-run`。ref が old / new / それ以外 | old: journal が aborted、run が abandoned。その後の tick・再起動で ref が変わらず、開いた journal が残らない。new: journal が done、run が completed（`release_applied`）で次の Task へ進む。それ以外: 拒否し、何も変わらず Held のまま。abandoned の run の journal は trigger で変更できず、settle は `update-ref` を実行しない |
| F-m1 | モードの切替の前に、もう一方の作業を 1 種類ずつ残す（native: open な Unit・pending の effect・cookie を持つ残存プロセス・Exhausted の観測（期限を過ぎたものを含む）。pipeline: held の run・unknown の step・開いた journal・waiting（期限を過ぎたものを含む）/ probing の上限待ちの記録） | rrxd は epoch を進めずに起動を失敗させ、何も書かない。片付けた後は起動する |
| F-t1 | admit の後にテストの実行ファイルを差し替える / 同じ key の REPLACE で profile の body を変える | 起動せず rejected / held。合格として採用しない |
| F-i1 | `INSERT OR REPLACE` で各表の主キー・UNIQUE・部分 UNIQUE に衝突させる（`recursive_triggers=false`）/ Agent の kind で provider が NULL の step を INSERT する | すべて拒否され、元の行が残る |
| F-w1 | pin → profile の閉じた `match` と enum の構造を確認し、未登録 pin、既知 pin の prefix・suffix・大小文字違い、別 provider の pin に既知の success / limit 出力を渡す | 未知 pin の arm は decode 前に `other`。既知 arm は provider と exact pin から実装済み profile だけを選び、pin 単独から success を返さない。表の構造を確認し、列挙した非対応入力はすべて `other` となる |
| F-w2 | §5.6.1 の各実 pin を引数に、shape に忠実な success / limit / other fixture を通す（合成 pin の既存テストも維持） | 完全な成功形だけ success、固定した上限形だけ limit、認証の失敗・未知形は other |
| F-w3 | success fixture を非 0 / 終了コードなし、重複キー、未知の codex event、`turn.started` の欠落・重複・完了後への移動、error / turn.failed の後の完了、完了の重複・後続 event、claude の型違い・エラーとの矛盾、`structured_output` の欠落・null・文字列、途中切れ・overflow・strict_json 上限超過に変える。Claude の矛盾は `api_error`・top-level の `api_error_status` の非 null と型違い、`errors` の非空・文字列配列以外を個別に渡す。両 provider の limit fixture に終了コード `None` も渡す | いずれも `other`。上限の code / status / 固定文言の一つを変えたものも limit にならない。終了コード `None` + limit の形も `other`。probe での `other` が待ちを解除しないことは F-a7 と合わせて確認する |
| F-w4 | Executor / Confirmer のそれぞれに、実 pin の成功 JSONL と、不正・欠落・上限超過の `last.md` / `last.json` を custody に渡す / 有効な last receipt を渡す | 前者は欠落も含め `last_output_invalid` と `provider_outcome=other`、採用不可、probe は待ちを延長。後者だけ `passed` を要約に記録する。未確認を passed にせず、claude は `not_applicable`。harness の shape は不採用の attempt にも残る |
| F-w5 | receipt と許可する shape のフィールドを固定し、本文・ID・パス・error.message・top-level の message・structured_output・modelUsage と、codex の失敗 event / `error` 子・claude の `api_error` 子の未投影の値、件数を固定した `errors` の各文字列に互いに異なる秘密でない目印を入れる。さらに各段の raw / Unicode escape / decode 後のキー・値に秘密 pattern を注入する | 目印を変えても key set・件数を含む要約の shape は変わらず、目印は一つも要約に出ない。秘密検出なら shape 全体が固定の `secret_detected` だけになり、検出までに集めた type / key / code も残らない。原本や parse error の診断を要約・console に写さない |
| F-w6 | 偽 CLI pass で Agent を採用前に失敗させる、未知 pin / shape、観測の不正な型・identifier・status、4096 event / 各集合 128 key / `errors` 件数 / 文字列 / 全体の bound の境界、strict_json の decode 上限、ファイル欠落・symlink・receipt 不一致、観測前の失敗を注入する | event の順序、辞書順の key set、`errors` 件数と許可フィールドだけが recorded。shape の上限超過は部分列を残さず `shape_overflow`、decode 上限は `strict_json_limit`、読めない・構文不正・receipt 不一致は `absent_or_invalid`、未起動は not_run。harness は shape の記録可否だけを判定し、製品の `provider_outcome` が success 以外なら pass failed で後続の Agent を起動しない。未知 pin は shape を記録しても `other` で、exit evidence の `wire: "unmapped"` と status の held の理由で区別できる。outcome の上書きと profile 規則の再実装をせず、shape 検証不能も pass failed。実 CLI の二重 gate とオーナーの開始条件も維持される |

- **mutation の対象**（壊したら落ちることを確かめる）。
  - 配線:
    - native の除外を消す
    - 静止の確認を epoch の後に動かす / cookie の探索を消す
    - tick で `control_admission` を取らないようにする / spawn の前に Store lock を解放しないようにする
    - tx1 の epoch 照合・Goal の Running の検査・1 process の検査を消す
    - tx1 の上限待ちの検査を自分の run・自分の Project の記録だけにする / probe の決着で既知の成功以外を解除にする / 時刻の経過で native の Exhausted を外す
    - 採用・release・Goal 完了・回復の最終 tx の Goal の Running の検査を消す
    - `pipeline_tick` を呼ばないようにする
    - rrx の Git の `GIT_CONFIG_GLOBAL` / `--no-filters` を外す
    - 秘密スキャンを prompt の書込みの前から外す
    - pin の完全一致を prefix / fallback にする / profile の成功条件から失敗・矛盾の拒否を外す / `last_output_invalid` で成功を返す（F-w1〜F-w4）
    - Executor / Confirmer の last 欠落を `Ok(None)` に戻す（両 role の F-w4 が落ちる）
    - shape に本文を混ぜる / raw または decoded の秘密スキャンを外す / 秘密検出前の部分 shape を残す / shape 不明で後続を起動する（F-w5・F-w6）
    - test の起動前の `recheck` / profile の digest 照合を消す
    - 回復で、再送の照会を作業領域の初期化の後に動かす / guard の中と最終 tx の再送の照会を消す
    - `abandon-run` の journal の決着を消す / settle の abandoned の run の検査を消す
  - 部分一致と存在の検査:
    - codex の途中 event の許可一覧を外して未知 event を許す / 完了 event が 1 回だけという検査を外す（F-w3）
    - `turn.started` が完了前にちょうど 1 回ある検査 / claude の `structured_output` が object である検査を外す（F-w3）
    - scope の glob を前方一致にする
    - quote の照合を緩める（空を許す / 開始行を見ない）
    - `--help` の flag 検査を部分文字列にする
    - 許可する環境変数を広げる
    - ラベル集合の一致を「部分集合」にする
  - 「この検査が保証する」と書いた箇所:
    - step / run / journal / 上限待ちの記録の trigger の各条件（epoch・遷移・完了の EXISTS・放棄の NOT EXISTS（step と journal）・1 process の EXISTS・上限待ちの EXISTS・期限の単調性・解除の `provider_outcome`）
    - 書込み API が `owner_epoch` を明示して書くこと（省略すると F-e1 の trigger 単独の拒否が落ちる）
    - 死亡確認の 5 条件の一つずつ
    - `unknown` から `abandoned` への EXISTS
    - 再起動時に記録した SHA だけを採用する条件
    - ref journal の old / new の判定
- **実 Mac**（AC-8 のすべて）: オーナーが実行するチェックリストの script を B11 で作る。(e)(f) は daemon と control の経路で行う（要件 AC-8 の証跡の欄）。

## 15. Unit の分割

| Unit | 内容 | 進める AC | Unit の中の局所の受入 | 更新するマスター設計書 |
|---|---|---|---|---|
| B1 | 基準の決定（§0）、main の CI を `gh` で確認、偽 CLI の基盤 | 前提 | — | なし（`doc/verification/issue-106-b1.md`） |
| B2 | schema v12 と版上げの 4 経路、Store API（spawn しない）、trigger、native の除外、`pipeline_mode_inventory` | AC-2、AC-7、AC-8(e)(f) | (1) v0・v9・v10・v11 → v12 と、途中の失敗での rollback。(2) 各 trigger 規則の正負の SQL テスト（引継ぎ後の古い step の更新を trigger 単独で拒否、Agent の kind の NULL provider の拒否、放棄の条件、上限待ちの記録の遷移を含む）。(3) 回復の tx が retry・abandon-run とも初回で成功し、再送が同じ応答を返す | `architecture.md` §13、`goal-runtime.md` §14、`multi-project-runtime.md` §7 |
| B3 | `ExecutionConfig` / `PipelineConfig`、モード付きの `RuntimeOwner::open` と静止の確認、CLI の pin、Q、上限と成功の分類器の fixture | AC-1 | F-m1 の各行 | `agent-adapter.md` §10、`agent-execution.md` の「Process launch boundary」 |
| B4 | StepLauncher と StepCustody、実行境界、1 process の規則、native の exhaustion の読み取り、daemon 全体の上限待ち | AC-8(c)(d) | — | `agent-execution.md` §2・§4 |
| B5 | Executor と AdoptionChecker、`PipelineGit`、prepare、作業領域の初期化（§6.4） | AC-3 | — | `agent-execution.md`、`architecture.md` §10・§14 |
| B6 | `PipelineTestRunner`、Seatbelt、`ProjectAdmitTests`、profile の照合 | AC-3 | — | `workflow-engine.md` §17、`agent-execution.md` §5 |
| B7 | Reviewer と VerdictDecoder、snapshot、秘密スキャン | AC-4 | — | `workflow-engine.md` §6、`agent-adapter.md` §7 |
| B8 | ふるい分け、Confirmer、fix のループ、判定 | AC-5 | — | `workflow-engine.md` §6・§7 |
| B9 | `pipeline_tick`、release の journal、次の Task、Goal run と Goal の完了、読み取り API | AC-6 | (1) release と pause / cancel の交差（F-d4）。(2) Paused / Cancelled の Goal が release も Completed もされない（F-d5）。(3) Goal run の view だけで recover の command を組み立てられる（F-g2）。(4) Goal/run status に pending cleanup の義務（run/seq/attempt）、失敗区分と rrx 導出パスを表示する | `goal-runtime.md` §6・§7、`multi-project-runtime.md` §7 |
| B10 | 起動時の照合と復旧表、正常停止、pause / cancel、`pipeline recover` | AC-7、AC-8(c)(d)(e)(f) | review 境界で pause し、同じモードの pipeline Goal を resume すると review が進む | `goal-runtime.md` §14、`architecture.md` §9 |
| B11 | 障害注入マトリクスの完了、オーナーが実 Mac で実行するチェックリストの script、受入の記録 | AC-1〜AC-8 | — | `doc/verification/issue-106-acceptance.md` |

B10 設計入力: lifecycle Resume は Runtime config hash（`runtime/control.rs` の `policy_digest`）でなく、pipeline の受入済み policy（`PipelineProfile.policy_sha256`、`state/execution/pipeline.rs`）と照合する。

D26 の追加作業は、製品の pin → profile 表・分類規則・実 pin の fixture を **B3 の補修**（F-w1〜F-w3）、Executor / Confirmer の last 欠落も `last_output_invalid` にする変更と上限待ちの消費者の回帰を **B4 / B5 の補修・確認**（F-w4・F-a7）、未対応 pin の exit evidence と理由表示を **B4 / B9 の補修**、テスト専用の shape 記録器を **B7 の実 CLI harness の補修**（F-w5・F-w6）に置く。ローカルでこれらを実装・検証してから、**B11 の次の実 CLI pass** で 2 pin の対応と既存の Executor → test → Reviewer の採用を確認する。結果は §5.6.3 の (a) 両 profile を確認、(b) codex が合わず claude は未観測、(c) codex は合い claude が合わない、の 3 通りで、(b) は修正・ローカル再検証後にもう 1 窓が必要となり、(c) も再確認にもう 1 窓を要しうる。初期対応の照合には最大 2 窓が必要になりうることを計画に含め、各窓で D22 の gate と開始条件を満たす。成功条件は緩めず、2 窓で確認できなければ受入は未完了とする。新しい製品 step や Unit は増やさず、最初の実 pass まではローカルで実装・検証を済ませる。実装時に `agent-adapter.md` の pin と分類規則、`agent-execution.md` の custody と上限待ちの説明を更新する。B11 の受入記録には、3 通りの結果と最大 2 窓の可能性、各 pass で観測済み・未観測の provider、実際の結果に加え、文書由来・実観測の shape 由来・合成した fixture 分岐を区別して記載する。

## 16. 承認済みの計画との相違

| ID | 計画 | この設計 | 根拠 |
|---|---|---|---|
| D26 | pin した版ごとに固定した分類規則を使う | provider と exact pin から閉じた wire profile を選ぶ。`codex-cli 0.164.0` → `codex-jsonl-v1`、`2.1.312 (Claude Code)` → `claude-result-v1` を初期対応に加え、未知 pin は `other` と exit evidence の `wire: "unmapped"` で理由を区別する。Executor / Confirmer の last 欠落も `last_output_invalid` とし、終了コード `None` の limit 形も `other` にする。実 CLI harness は各 Agent の秘匿した shape と last-output の結果を記録し、適合は製品の `provider_outcome` だけで判断する。次の pass は (a) 両 profile を確認、(b) codex が合わず claude は未観測、(c) codex は合い claude が合わない、の 3 通りで、修正・再確認を含め最大 2 窓を要しうる。各窓で D22 の gate と開始条件を維持し、成功条件は緩めず、未確認なら受入未完了とする。B11 の受入記録にも 3 通りと窓の可能性・実際の観測結果を残す。以後の実 pin の追加には実 pass の shape を要求する（§5.6） | fixture pin だけを受け付ける分類器では、Q を通った実 CLI も常に `other` となり、Executor の採用が `agent_failed` で止まる。success は文書化された形式、limit は未観測の合成 fixture を根拠に初期対応を作り、未知・矛盾・不完全な成功形を拒否して上限待ちの誤解除も防ぐ。codex が失敗すると claude は起動されず shape を取れないため、1 窓で両方を確認できるとは限らない。本文を持ち出さずに照合し、再確認の窓を計画に含める |
| D25 | 起動直前に両 executable の全体を 5 秒以内に再 hash する | 5 秒の bound と全体の再 hash を維持し、dev profile の sha2 を `opt-level = 3` でビルドする（test profile も継承）。安全性の検査自体は変えない | 2026-10-29 の実 CLI 検証では、Q を通過して prepare を採用した後、implement が Agent の起動前に `launch_preflight_failed` で拒否された。オーナーの Mac で `executable_until` は未最適化だと codex（260 MB）が 13.2 秒、claude（236 MB）が 10.6 秒、sha2 の dev profile を `opt-level = 3` にするとそれぞれ 0.62 秒、0.56 秒だった。両 executable は共有の 5 秒の期限内に再 hash する。実 CLI harness と debug binary は dev profile を使い、release は既に最適化される。各計測値はそれぞれ 1 回の実行によるもので、負荷とページキャッシュの状態は記録しておらず、bound を超えた場合は意図した fail-closed の挙動として起動を拒否する |
| D24 | Q の全 probe の出力は 4 KiB まで | help を要求する probe（codex `exec --help`、claude `--help`、claude `--setting-sources bogus\|"" --help`）だけ 64 KiB まで、それ以外（`--version`、login status）は 4 KiB までとし、不正値の settings probe の parser error は stderr だけから取る | 2026-10-29 の実 CLI 検証で、Claude Code 2.1.312 の help が 22,116 byte あり、必須 flag の検査前に `help_probe` で拒否された。timeout・cleanup・出力を保存しない規則は変えない |
| D23 | snapshot の 100,000 項目の上限 | 実効上限は 100,000 項目または `ls-tree -r -z` の一覧出力 4 MiB のうち先に達する方とする。どちらの超過も review を `review_snapshot_limit` で held にする | 100,000 項目の計測では、既存の 4 MiB の Git 出力上限で準備が先に失敗した。上限や budget は広げず、snapshot の一覧取得だけで出力超過を名前付き理由に変換する。長いパスを使った、項目数の上限より十分小さい tree で、一覧出力の上限直前は準備でき、直後は名前付き理由で保留することを検査する |
| D22 | B7 の本物の CLI と snapshot 上限の確認 | 製品機能ではなく検証用の harness を追加する。macOS の通常テストは native fake CLI で Executor → admitted TestsProfile の Seatbelt test → Reviewer の Runtime API を通す。実 CLI は ignored test と `RRX_REAL_CLI_PASS=1` の二重ゲートを要求し、owner が他の資格情報を持つ同一利用者のプロセスを止めた新しい端末から `scripts/real-cli-pass.sh` を実行する | 秘密を含まない合成 Git project で 1 pass だけ行う。要約は各 step の状態・理由・時間、prompt と `git archive` の SHA256・bytes、Reviewer envelope と `modelUsage` のキー、TMPDIR 内の git wrapper による固定 `GIT_CONFIG_*` の転送、probe を除いた PATH の git を記録する。Agent の本文は要約に転記しない。snapshot の時間計測も ignored と専用 gate を要求し、製品と同じ entry budget・rebuild・verify を使う。Q と製品の制約は変えない |
| D21 | §5.3 の Reviewer 引数 | `--no-session-persistence` を追加して §5.3 を強化する。Q でも必須 flag として検査する | pin した Claude CLI 2.1.304 は print モードでこの flag を受け付け、session の保存と再開を止める。rrx の秘密スキャンを通らない CLI 自身の transcript 保存を抑止する |
| D1 | schema v13 | v12 | main は v11（`state/mod.rs:32`）。v12 は M0 branch（`feature/issue-5-m0:crates/rrx/src/state/mod.rs:34`）にしかない。M0 を再開するときは v13 へ番号を振り直す |
| D2 | `goals.execution_profile` 列、Goal ごとのプロファイル、両 claim 経路での容量の合算 | インストール単位のモードと、不変の表 `goal_execution_profiles`（行がなければ `managed_native`）。容量は合算しない | 1 つの rrxd で 2 経路が同時に動かず、モードの起動にもう一方の静止を要求するので、容量の共有も、2 つの admission の間の競合もなくなる。`goals` 表を変えると、全 catalog の byte 比較（`state/managed_binding/schema.rs:249-281`）と定義の trigger（`state/runtime/schema.sql:21-30`）に波及する |
| D3 | Q の記録先は `runtime_startup_probes`（kind cli） | `pipeline_qualifications` | `runtime_startup_probes` は M0 にしかない（`startup.sql:1`）。その `step` の CHECK も native 用 |
| D4 | テストは `collector::collect` と `verification_runs` で記録する | テストの各コマンドを test step としてパイプラインの custody で実行する。`verification_profiles` は再利用し、記録は `pipeline_steps` に残す | `collect` は custody を値で受け取り、自分で signal と reap を行う（`execution/verification/collector.rs:114-115,149,174`）。`verification_runs` は `execution_units` などへの FK を要求する（`state/execution/verification.sql:26,36,52`） |
| D5 | `ProcessGroup` を共有の module へ移す | 移さずに、`StepCustody` で包む | `ProcessGroup` は native の Codex（`codex/transport.rs:23,125`、`codex/preparation.rs:295`）と Grok が使っている custody（#19）で、別の担当の範囲に触れる。必要な `observe_exit`・`cleanup_group`・`reap` はすでに `pub(crate)` |
| D6 | 死亡の確認 = group の死亡と cookie の探索 | group が空になることを観測する手順を、条件として明示する | 既存の `ProcessGroup` は、SIGKILL の配送と leader の reap までしか確かめない（`adapter.rs:520-568`）。group が空になるまで待つのは `git.rs:648-662` だけ |
| D7 | strict_json で未知フィールドを拒否する | strict_json（上限と重複キー）＋ `deny_unknown_fields` の DTO ＋ 意味の制約（§8.3） | `strict_json.rs:65-66` に、型の検査は後で行うと明記されている |
| D8 | step の状態: planned…held | `committed`・`limited`・`interrupted`・`abandoned` を明示する。`unknown` からは `abandoned` だけ | 記録した SHA による採用、provider の上限、正常停止、放棄の command を表すため。unknown を held へ移せる経路を作らないため |
| D9 | Claude で「project / user の setting sources を使わない」 | `--setting-sources ""` を Q で確かめる。受け付けない版は pin しない | 空の指定を受け付けるかは実機で未確認（この文書の作成中、`codex` / `claude` の `--help` の実行は hook で拒否された） |
| D10 | 計画にないもの | テストを Seatbelt で包む。`ProjectAdmitTests` の command | `admit_tests` は本番から呼ぶ経路がない（`execution/verification/mod.rs:226`。呼ぶのはテストだけ）。テストは Agent のコードを、sandbox なしで実行する（§12）。要件の承認事項 9 に含めて判断を求める |
| D11 | Goal の完了 = 型付きの条件 | 新しい evaluator を足さない。`cli_pipeline` では `RequiredTasksVerified` だけを受け付け、Goal run の test を完了の固定条件にする | evaluator を足すと、canonical digest（`runtime/goal.rs:149-157`）と既存の Goal に波及する |
| D12 | `ProcessGroup` の macOS での「孤児は Held のまま」 | 計画どおり。加えて、見つかった pid に signal を送らない | `crates/rrx-process-tracker/src/macos.rs:12-23` は、探索で見つけたプロセスへの signal を無効にしている |
| D13 | `rrx task recover --abandon-step` | `rrx pipeline recover <run-id> --abandon-step --then retry|abandon-run` | Goal 完了確認の run は Task を持たない。run で指定すれば、両方の kind に同じ command を使える。attempt の上限に達した run を終わらせる出口が要る |
| D14 | rrx commit を trailer で識別する | 記録した commit SHA と tree だけで識別する | trailer と親は Agent も作れるので、作成主体の証明にならない |
| D15 | quota lease を使う | v1 は daemon 全体で 1 step ずつ動かし、quota の表を書かない。provider の上限は既知の終了だけを daemon 全体の 1 つの記録（`pipeline_limit_wait`）の backoff で待ち、期限の後は 1 つの起動が既知の成功で終わるまで次を起動しない。native の Exhausted の観測は読むだけで、残る間は起動しない | `quota_leases` / `quota_waiters` は `execution_units` への FK を持つ（`state/execution.sql:130-142`）。native の規則（provider ごとの上限の集約、probe、公平性）を条件ごと移し替えると、改訂のたびに弱まりが見つかった。1 step ずつはどの同時実行の上限よりも厳しく、Stage 1（1 Project・直列の step）の要求を満たす。時間をまたぐ制約（待ちと、期限後の単一の起動）は、account を区別しない 1 つの記録で保守的に保つ |
| D16 | CLI の入口のファイルを hash で固定する | Q は Mach-O の native binary だけを受け付け、`#!` の入口は拒否する（制約の強化）。npm Codex では `…/node_modules/@openai/codex-darwin-arm64/vendor/aarch64-apple-darwin/bin/codex` を設定する | interpreter・vendor の実体が入口の hash と独立に入れ替わる経路を pin から除く |
| D17 | §3.5 の 5: 探索の `coverage.limited()` が false であること（読めない PID はすべて limited） | macOS の tracker が、読めない PID を独立した `sysctl(KERN_PROC_PID)` で zombie・消えた PID・不明に分ける。前の 2 つは `limited()` から外す。列挙は `proc_listpids` のまま（lock の中の 1 時点の PID の集合）とする。検査した PID の identity の開始時刻と、zombie の照会の開始時刻を、種類とともに記録する。後の巡で記録と合わない PID と、identity と環境の読取りが同じ incarnation のものだと確かめられない PID（`reused`）も、limited から外して新しい不在に数える。新しい不在がなくなるまで再列挙し、limits を超えれば limited にする（§3.5.1） | オーナーの Mac では、親が回収しない同じ利用者の zombie が 1 つあるだけで、探索が常に limited になり、どの step も死亡を確認できない。zombie と消えた PID は何も実行できない。一覧の後の fork と、記録した PID や読取りの間での PID の再利用は、再列挙か limited に倒す。保証は「limited でなく一致 0 件なら、残余 R1・R2 を除き、走査の終わりに cookie を持つ生きたプロセスはない」であり、§3.5.1 の前提 P1〜P6 に依る（P3・P5・P6 は XNU のソースで確かめた）。R1 は、最後の一覧の後、記録のない PID を初めて読む前に、cookie を持つプロセスが一覧にない子を残して exit し、その PID が別のプロセスに割り当てられる順序である。XNU の割当て器では、そのプロセスの誕生から PID の counter が一巡し、その残りを 1 回の discover の中で進む必要がある。R2 は、一覧が kernel 内の容量（nprocs+20）で切られ、返った長さからそれが分からない順序である。どちらも今の tracker に同じ形であり、R1・R2 ともオーナーが受け入れた残余とする。出入りが止まらない Mac では、`time_exceeded` で held になりうる |
| D18 | §6.2 の変更の一覧は `--ignore-submodules=none` | 対象の tree の gitlink（160000）を status より前に拒否し、すべての status を `--ignore-submodules=all` にする。共通の `-c` に `diff.ignoreSubmodules=all` と `submodule.recurse=false` を加える。あわせて、rrx の Git は `core.attributesFile=/dev/null`・`core.excludesFile=/dev/null`・`GIT_NO_LAZY_FETCH=1` で動かし、`diff-tree` には `--text` を付ける。拒否する属性に組み込みの macro `binary` と `ident` を加え、`.gitattributes` の名前を ASCII の大文字と小文字を区別せずに照合する。Task の worktree は `--no-checkout` で登録し、その context の設定と属性を検査してから checkout する | gitlink のパスに Agent が `<path>/.git` を置くと、status がその中で子の Git を起動し、submodule 自身の設定の filter を rrx の権限で実行しうる（§6.1 の前提が崩れる）。global の attributes・ignore、`binary`・`core.bigFileThreshold`・`ident`、大文字小文字の違う `.gitattributes` は、diff の追加行や status から変更を隠し、秘密スキャンや scope の検査を素通りさせる。Task の worktree だけで有効な条件付き include は、Project 側の検査の後の checkout で効く。どれも制約の強化で、秘密スキャンで見つかった内容を含む blob と commit は、参照のない loose object として common の objects に残る（§12） |
| D19 | §6.2 の変更の一覧は ignored のファイルを含まない。保護する名前と `.gitattributes` の名前は ASCII の大文字と小文字だけを区別しない | build と `clean_at` は、`--ignored=matching` の status で ignored の項目を列挙し、保護する名前の成分を持つ項目を `protected_path` で拒否する（ignored の項目は変更にも drift にも数えない。ignored のディレクトリは 1 項目に畳まれる）。保護する名前と `.gitattributes` の名前の照合に、ASCII に加えて Unicode の大文字小文字の畳み込み（大文字にしてから小文字にする）を使う。`.gitignore` は scope の明示でも Executor に変えさせない。index の `assume-unchanged`・`skip-worktree` は拒否する | ignored のファイルは status に出ないので、scope と保護の検査を受けず、次の step の作業領域に残る（Agent が置いた `AGENTS.override.md` を Codex が読む）。Agent が置く `.gitignore` は変更か ignored の項目として必ず列挙され、scope を問わず拒否されるので、効く ignore 規則は Goal の base から来たものだけになり、隠せるのは対象の tree の ignore 規則が畳むディレクトリ（`node_modules/` など）の中だけになる。これは残余として受け入れる。通常の採用 → 次の step の経路では持ち越され、消えるのは retry の初期化（`clean -ffdx`）のときだけである。root は畳まれないので、Codex が読む root の指示ファイルは必ず列挙される。逆に、ディレクトリ自体に一致しない規則（`node_modules/**`・`*.pyc` など）は中のファイルを 1 件ずつ列挙させるので、依存物に含まれる `.github/`・`CLAUDE.md` などで `protected_path` に倒れることがある（安全側の誤検出）。index のフラグを付けた追跡ファイルは書き換えても status に出ないので、gitlink と同じく拒否する。retry の初期化（`reset --hard`）はフラグを消さないので、フラグが残った run は held のままになる（安全側）。運用者は Task worktree の index を消してから reset して復旧する。macOS の APFS は Unicode の畳み込みで名前を比べるので、`ſ`（U+017F）を含む名前が `.gitattributes`・`AGENTS.md` として開かれる（オーナーの Mac で確認）。どれも制約の強化 |
| D20 | §3.5 の 5: 探索の `coverage.limited()` が false で一致 0 件。§3.3 の 6: leader の birth を cookie の探索で読む | macOS の tracker は、`KERN_PROCARGS2` が引数の後に何も返さないプロセスを「環境がない」とする。そのうち、実行ファイルの場所（`proc_pidpath`）の親が `/bin`・`/usr/bin`・`/sbin`・`/usr/sbin` のどれかのもの（`proc_pidpath` が読めないものを含む）について、次を limited（`ancestry_unconfirmed`）にする: (1) 開始時刻が起動の境 F 以上で、`e_ppid` をたどった系譜が、F 以上のプロセスだけを通って pid 1 か rrxd 自身に達するもの、(2) 祖先が読めない・zombie・親を確かめられない（祖先を読んだ後の子の読み直しで、子が変わるか親の PID が変わる）・追跡中・32 段を超えるもの、(3) 環境と場所を同じ実行イメージの観測に結び付けられないもの（場所を環境の読み直しの前後で読み、違えば limited）。F は、tx1 の前に記録した壁時計（`body.launch`）と leader の birth の早い方から 60 秒を引いた値である。除外は、F より前に生まれたプロセスと、F より前に生まれた生きた祖先（pid 1・rrxd 以外）を系譜に持つプロセスだけとする。場所か系譜で除外したものは記録せず、巡ごとに読み直す（exec は PID と開始時刻を変えない）。別の boot で起動したと確かめられた step には、判定も時計の確認も当てない。それ以外で、今の時計が F より前なら limited にする。limited の結果は、今と同じく `DEATH_BOUND` の中でやり直し、得られなければ held にする。見つけたプロセスに signal は送らない（D12）。leader の birth は、spawn の直後に PID から `KERN_PROC_PID` で読む。cookie が leader に届いたことは、環境を 1 回読んで別に確かめる。死亡確認の reap の前に、zombie の開始時刻と照合する（§3.5.2・§3.5.3） | オーナーの Mac で、同じ利用者の platform binary の `KERN_PROCARGS2` が成功し、環境だけが消えることを実測した（`/bin/sleep`・`/usr/bin/perl`・setsid の後の perl・sandbox-exec の下の perl で cookie なし。Homebrew の python3 では cookie あり）。今の tracker はこれを「一致しない」とするので、setsid で group を抜けた platform binary の子孫が生きていても、死亡を確認してしまう（`origin/main` の tracker も同じ）。また、leader が platform binary の test（sandbox-exec の下の perl）は、birth の探索で見つからず、tx2 が `record_failed` になって held になっていた。オーナーは、残余として受け入れず、専用の macOS 利用者も今は使わず、検出して held に倒すことを選んだ。決定に加えた点は 4 つある。rrxd 自身を launchd と同じに扱う（生きた leader の除外を防ぐ）。別の boot を除外する（前の boot のプロセスは存在できない。これがないと再起動の後の回復が永久に拒否される）。時計に 60 秒の余裕を取る。起動時の cookie の確認を残す（既存の確認を弱めない）。公開 SDK に unique ID がないので、起動との前後は壁時計の開始時刻で比べる。親の取り違えは、開始時刻の順序（時計が少し戻るだけで逆転する）ではなく、子の読み直しで見分ける。残余は R3（時計が 60 秒より大きく戻る）・R5（子孫でない代理の起動）・R6（引数の領域の偽装）で、R1・R2 は変わらない。費用として、step の間に起動した Apple のアプリ・XPC service が残ると held になり、held の回復はそれらの終了か再起動まで拒否される（HE9 で測る）。その後、オーナーの Mac で、場所で絞らないと 5 分を超える step がほぼすべて held になる（無関係な macOS の処理が 5 分で 5 個、60 分で 7 個）ことと、4 つの場所に絞ると 0 個になることを実測し、オーナーが場所で絞ることを選んだ。4 つの場所の外の platform binary で抜けた子孫は残余 R7 とする。場所を環境の前後で読んでも、読取りの窓の中で 2 回 exec して元の場所に戻ったものは見分けられず、残余 R8 とする（オーナーが受け入れた） |

**オーケストレーターの簡素化の指示から外した点**

| 指示 | この設計 | 理由 |
|---|---|---|
| repo の設定と global の設定に driver があれば拒否する | repo の設定の driver は拒否する。global / system の設定は、rrx の Git から外す（`GIT_CONFIG_GLOBAL=/dev/null`、`GIT_CONFIG_NOSYSTEM=1`） | オーナーの Mac では global の git-lfs の filter が一般的で、拒否にすると全 Project が使えなくなる。外せば driver は rrx の Git で起動できず、目的（rrx が driver を実行しないこと）は同じに満たす |
| tick は tx1 → spawn → tx2 の間 `control_admission` を保持する | それに加えて、claim・rrx 内部の step・採用・release・Goal の完了も、tick の中で境界を保持して行う | 結果を進める書込みを、pause・cancel と直列化するため（規則 G） |

## 17. Revision 3

| 指摘 | 解決 | 節 |
|---|---|---|
| F-01 | 時刻（`resets_at`・`next_probe_at`）で native の Exhausted を外す判定を削除した。`quota_windows` に Exhausted の観測が 1 つでも残る間（`applies` が認める新しい Available の観測で置き換わるまで）は、`cli_pipeline` の起動を拒否し、tx1 でも step を held（`native_quota_exhausted`）にする。運用者は `rrx goal status` の Held の理由と `native_exhaustion`（provider・bucket・`resets_at`）で知る。パイプラインは quota の表を書かず、解除は native の観測だけ。単一の起動を native の probe とみなす記述を削除した | §0、§1.1、§2.6、§3.2、§10、§12、§14 F-a5・F-m1、要件 §6・§7・§8 項目 1・7 |
| F-02 | 上限待ちを run の body から、daemon 全体の 1 行の表 `pipeline_limit_wait`（state・`not_before`・`backoff_ms`・probe の step）に移した。trigger は R-epoch・`no_replace`・`no_delete`、期限の単調性、waiting → clear の禁止、解除に probe の step の `provider_outcome = success` を要求する。すべての Project の Agent の step の tx1 が確かめる。期限の後は最初の 1 つ（probe）だけを起動し、既知の成功以外の終わり方（中断・認証の失敗・unknown・死亡の未確認・分類できない失敗）は同じ backoff で延長する。run の放棄・回復・再起動で失われない（再起動時は owner だけを引き継ぐ）。`rrx goal status` に `provider_wait` を出す。`managed_native` への切替は記録が clear のときだけ | §1.1、§1.2、§2.2、§2.3、§2.4、§2.6、§3.2、§4.1、§4.2、§5.6、§10、§12、§13.1、§14 F-a6・F-a7・F-m1、§16 D15、要件 §2・§5 AC-8・§6・§8 項目 1・7 |
| F-03 | guard の中の読み取りの tx と最終の tx で、現在の状態の検査より先に ack と要求の同一性を照会し直す（`goals.rs:397-408` と同じ順序）。同じ request ID の重複は、どれも同じ保存応答を受け取り、副作用は 1 回だけ | §4.4、§12、§14 F-f3、要件 §7・§8 項目 3 |
| F-04 | `--then abandon-run` は、その run の開いた `task_release` の journal を先に決着させる。guard の中で ref を読み、old なら journal を aborted にしてから run を abandoned、new なら journal を done にして run を held → completed（Goal branch は承認済みの commit を指しているので、放棄すると Git と DB が食い違う）、それ以外は拒否して Held のまま。run の trigger は、開いた journal が残る放棄を拒否し、held → completed には回復の記録を要求する。abandoned の run の journal は trigger で変更できず、settle は `update-ref` を実行しない | §2.2、§3.1、§4.4、§9、§12、§14 F-h3、要件 §8 項目 3・6 |
