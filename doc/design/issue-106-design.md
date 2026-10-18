# Issue #106 設計書: Plan B — CLI サブプロセスによる Agent パイプライン

Status: 提案中（B0・改訂 2）。基準の source は origin/main `79e4d273`。要件は `doc/requirements/issue-106-requirements.md`。ファイルパスは、断りがなければ `crates/rrx/src/` からの相対パスで書く。`crates/` で始まるパスはリポジトリのルートからの相対パスである。

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
| native の exhaustion | ✓ | | | `windows`（`state/execution/quotas.rs:476-502`）が `quota_windows` を decode し、admission が Exhausted の有無を見る（`:181-189`）。回復には新しい観測が要る（`state/execution/quota_observation.rs:4-21`）。パイプラインは読むだけ |
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
| `cli_pipeline` | ・`task_drivers` の `state='driving'` が 0 件。<br>・`execution_units` の `native_effects_open=1 OR result_finalization_open=1` が 0 件。<br>・`managed_phase_operations` の `phase_open=1` が 0 件。<br>・`quota_leases` の `active=1` が 0 件。<br>・`managed_effects` の `state IN ('pending','unknown')` が 0 件。<br>・`cleanup_jobs` を持つ各 Unit の cookie を `rrx_process_tracker::discover` で探し、どれも `coverage.limited()` が false で一致が 0 件（cleanup job は完了で消えない履歴なので、今回の探索の結果で判定する。件数が 4096 を超えれば確認不能として拒否）。<br>・未失効の native exhaustion がない（§2.6 の定義。provider と account を問わない）。 |
| `managed_native` | ・`pipeline_runs` の `state IN ('running','held')` が 0 件。<br>・`pipeline_ref_journal` の `state IN ('pending','held')` が 0 件。<br>・process の step のうち、`launching`・`running`・`unknown`、または `body.death_confirmed` が true でない `held` が 0 件。<br>・死亡を確認済みでない step（`held` で確認済みのものを除く、cookie を持つもの）の cookie の探索が、完全な coverage で 0 件。<br>・どの run の `body.limit.not_before` も未到来でない（パイプラインが観測した provider の上限を native に引き継げないため、待ち終わるまで切り替えない）。 |

満たさなければ、rrxd は epoch を進めずに `ExecutionModeNotQuiescent { reason }` で起動を失敗させ、何も書かない。`managed_native` の起動でパイプラインの行がなければ（既存の環境）、この確認は件数 0 で通り、挙動は変わらない。`cli_pipeline` で通った場合は、epoch E を得た後に `pipeline_qualifications` の `mode_quiescence` を qualified で書き、確認した件数と探索の要約を body に残す。native の痕跡（unknown の effect など）が native 側で解消されない限り、その state でパイプラインは起動できない（安全側）。

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
| 結果を進める書込み | committed・adopted・次の step と attempt の計画・release の journal と done・Goal の完了・回復の `--then retry` | tick か回復の command の中でだけ行う。どちらも guard を保持する。最終 tx で、現在の owner、**Goal が Running であること**、run と step の version（それぞれの期待値）を確かめ直す。Goal が Running でなければ何も書かずに終える。step は exited / committed のまま残り、resume の後の tick がやり直す。cancel なら §4.3 で interrupted にする |
| 観測を記録する書込み | exited・interrupted・rejected・held・unknown・limited、run の held / abandoned、回復の `--then abandon-run` | Goal の状態に依らない。StepCustody の監視タスクは guard を取らずに書く（shutdown が guard を持ったまま step の停止を待つので、取ると互いに待つ）。どれも採用・release・完了・起動をしない |

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

`PIPELINE_TABLES` は次の 7 表である。各表に共通の trigger（epoch・置換・削除）は §2.3 にまとめる。ここには表ごとの列と遷移だけを書く。

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
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=131072      -- held の理由、limit {waits, not_before}
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
   OR (OLD.state='held' AND NEW.state IN ('running','abandoned')))
 OR (OLD.state='held' AND NEW.state='running' AND NOT EXISTS(
     SELECT 1 FROM pipeline_recoveries r WHERE r.run_id=NEW.id AND r.consumed_version=NEW.version))
 -- 放棄は、生きているかもしれない step が 1 つもないときだけ
 OR (NEW.state='abandoned' AND OLD.state<>'abandoned' AND EXISTS(
     SELECT 1 FROM pipeline_steps s WHERE s.run_id=NEW.id
      AND (s.state IN ('planned','launching','running','exited','committed','unknown')
       OR (s.state='held' AND json_extract(s.body,'$.death_confirmed') IS NOT 1))))
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
   AND json_extract(body,'$.epoch') IS owner_epoch),
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
 -- daemon 全体で、生きているかもしれない process の step は 1 つまで（§2.6）
 OR (OLD.state='planned' AND NEW.state='launching' AND EXISTS(SELECT 1 FROM pipeline_steps o
     WHERE NOT (o.run_id=NEW.run_id AND o.seq=NEW.seq AND o.attempt=NEW.attempt)
      AND (o.state IN ('launching','running','unknown')
       OR (o.state='held' AND json_extract(o.body,'$.death_confirmed') IS NOT 1))))
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
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
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
-- 引継ぎは owner_epoch だけを変える、pending→done|held|aborted、held→pending|done|aborted、done と aborted は終端

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
```

### 2.3 全表に共通の trigger 規則

`PIPELINE_TABLES` の 7 表すべてに、次を `pipeline::install_schema` で作る。

| 規則 | 対象 | 内容 |
|---|---|---|
| R-epoch（行の owner） | `pipeline_runs`、`pipeline_steps`、`pipeline_ref_journal` | INSERT と UPDATE で、`NEW.owner_epoch IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1)` なら abort する。**書込み API は、すべての UPDATE で `owner_epoch` に自分の epoch を明示して書く**（変えない列として省略しない）。そのため、引継ぎの後に古い epoch の書き手が行を更新すると、NEW.owner_epoch が古い値になり、trigger だけで拒否される |
| R-epoch（run の owner） | `pipeline_steps`、`pipeline_recoveries` | INSERT と UPDATE で、`(SELECT owner_epoch FROM pipeline_runs WHERE id=NEW.run_id) IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1)` なら abort する |
| R-epoch（記録の epoch） | `goal_execution_profiles`、`pipeline_qualifications`、`pipeline_goal_completions`、`pipeline_recoveries` | INSERT で、`accepted_epoch` / `owner_epoch` / `completed_epoch` が現在の epoch と違えば abort する |
| R-immutable | `goal_execution_profiles`、`pipeline_recoveries`、`pipeline_qualifications`、`pipeline_goal_completions` | `no_update` |
| R-replace | 7 表すべて | `no_replace`: `BEFORE INSERT WHEN EXISTS(…)` の条件を、主キーと、すべての UNIQUE 制約・部分 UNIQUE index の衝突条件の OR にする。例: `pipeline_steps` は `(run_id,seq,attempt)` の一致、または「NEW が未終端の state で、同じ run に未終端の step がある」。`pipeline_runs` は id・`task_id`（kind=task）・`goal_id`（kind=goal）・worktree・branch。`pipeline_ref_journal` は id・`goal_create` の goal・`task_release` の run・開いた ref_name。既存の `goal_authority_no_replace`・`task_drivers_no_replace`（`state/runtime/schema.sql:15-17,99-101`）と同じ形で、`recursive_triggers` が off でも `INSERT OR REPLACE` を拒否する |
| R-delete | 7 表すべて | `no_delete` |
| writer guard | 7 表すべて | `writer_<table>_{INSERT,UPDATE,DELETE}`（`rrx_writer_contract_version()<>12`） |

**参照する TestsProfile の保護**: 既存の `verification_profiles` には `no_replace` がない（`state/execution/verification.sql:1-10`）。FK の検査が有効な接続では、`goal_execution_profiles` の FK `(project_id, tests_profile_digest)` があるので、digest を変える REPLACE は参照先が消えて失敗する。しかし、同じ `project_id`・`digest` のまま body だけを変える REPLACE は、文の終わりに参照先が存在するので FK では防げない。そこでパイプラインは、TestsProfile を読むたびに body と digest を照合する（§7.2）。不一致なら採用も起動もせず held にする。既存の表への `no_replace` の追加は別 Issue に記録する（§13.3）。

### 2.4 Store の書込み API の規則

パイプラインの Store の変更は、すべて `state/execution/pipeline.rs` の API を通す。各 API は次を守る。

1. `BEGIN IMMEDIATE` の短い transaction で行う。Git・プロセス・ファイルの I/O を中に持ち込まない。
2. 最初に `pipeline_owner_current(tx, owner)` を呼ぶ。`owner` は `RuntimeOwner` の instance と epoch。runtime_epoch の行と一致しなければ拒否する（`owner_current` `state/runtime/goals.rs:14-26` と同じ形）。
3. 対象の run の `owner_epoch` が現在の epoch と一致することを確かめる（`validate_unit_authority_facts` の epoch 照合 `state/execution.rs:393-401` と同じ考え方）。
4. 変更する行ごとに、呼び出し側が観測した version を渡す。`UPDATE … SET owner_epoch=?<自分の epoch>, … WHERE <主キー> AND version=?<期待値>` の変更件数が 1 でなければ、`StateGuardError::SnapshotChanged`（`state/mod.rs:75-79`）で中止する。run と step の version は、それぞれ別の期待値で照合する（両者の数値が一致することではない）。
5. 結果を進める API（規則 G）は、同じ tx で Goal が Running であることを確かめる。
6. trigger（§2.3）は、API の検査が抜けたときの第 2 の層である。

API の一覧（B2 で作る。どれも spawn しない）: `pipeline_mode_inventory`（読み取り。epoch の前）、`pipeline_claim_run`、`pipeline_take_over`、`pipeline_begin_launch`（tx1）、`pipeline_record_running`（tx2）、`pipeline_record_launch_unknown`、`pipeline_record_exit`、`pipeline_record_limited`、`pipeline_record_commit`、`pipeline_adopt`、`pipeline_hold`、`pipeline_plan_next`、`pipeline_release_begin`、`pipeline_release_settle`、`pipeline_complete_goal`、`pipeline_cancel_run`、`pipeline_recover`、`pipeline_record_qualification`、読み取りの `pipeline_goal_view` / `pipeline_task_view`。

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
- これは native のどの上限（全体・provider・executor の予約枠）よりも厳しいので、quota の契約を弱めない。provider ごとの枠・公平性の順序・予約枠・probe は持たない。
- 死亡を確認できない step が 1 つでもあれば、どの Project の step も起動しない（安全側）。運用者が `rrx pipeline recover` で解くまで待つ。
- Project ごとの Task は 1（`config.rs:17`）。run の claim の occupied 検査は `pipeline_runs WHERE state IN ('running','held')` で行う。held の run も slot を使い続ける（worktree と残存プロセスが不確かなため）。native の occupied CTE（`candidates.rs:49`、`claim.rs:375`）は変えない。
- 複数の step の並行実行と provider の容量の共有は Stage 2 で別に設計する。

**native の exhaustion（読むだけ）**: tx1 は、未失効の native exhaustion があれば起動せず、step を planned → held（`native_quota_exhausted`、`death_confirmed=true`）、run を held にする。

- 未失効: `windows`（`state/execution/quotas.rs:476-502`。native の admission も同じ関数で Exhausted を判定する `:186-189`）で decode した観測の status が Exhausted で、`now < max(resets_at（なければ 0）, その pool の quota_pools.next_probe_at)` のもの。この時刻より前は native も probe を出さない。この時刻の後は、1 step ずつの起動が native の単一の probe に当たる。
- パイプラインは CLI の account と native の account の対応を証明しないので、provider と account を問わず、1 つでも未失効なら拒否する。decode に失敗した行があっても拒否する。
- パイプラインは quota の表を書かない。`windows` は `pub(super)` にするだけで、挙動を変えない。

**provider の上限（既知の終了だけ）**:

1. step のプロセスが終了し、死亡を確認した（§3.5）。
2. 出力が、pin した版の既知の上限の信号（§5.6）に一致したときだけ、その step を採用しない。exited → limited にし、同じ tx で run の `body.limit` を `waits = waits+1`、`not_before = now + min(60 秒 × 2^(waits−1), 30 分)`（1 回目 60 秒、以後倍）にする。
3. 更新後の `waits` が Goal の `max_limit_waits` を超える場合、または attempt が 16 の場合は、limited ではなく held（`limit_waits_exhausted` / `attempt_limit`）にする。
4. 次の attempt は tick が計画する（§4.2）。Goal が Running で `not_before` を過ぎた後に、§6.4 で step_base に初期化して検証し、attempt+1 を planned で作る。
5. 一致しないものは上限として扱わない。通常の失敗（rejected → run held）か、判断が要るもの（held）になる。起動結果が不明な attempt（unknown）は再実行しない。

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
| running → completed（kind=goal） | Goal 完了の tx | `pipeline_goal_completions` の行（trigger が検査する） |
| running/held → abandoned | Goal cancel の後の tick、または `rrx pipeline recover --then abandon-run` | 生きているかもしれない step が 1 つもない（trigger が検査する。§2.2）。Project の slot が空く |

**不変条件**: run が held のとき、最後の step はちょうど 1 つの held / unknown / rejected の step である。運用者は常にその step を `rrx pipeline recover` の対象にできる。

### 3.2 step

| 遷移 | 主体 | 必要な根拠 |
|---|---|---|
| planned → launching | StepLauncher の tx1 | 次をすべて確認したうえで、cookie（UUIDv4 を英数字にしたもの）、attempt ディレクトリ、argv・env の digest を書く。<br>・現在の owner と、run.owner_epoch = 現在の epoch。<br>・run.state = running、run.version と step.version がそれぞれの期待値と一致。<br>・Goal が Running。<br>・`stopping` が false（実行境界の中で確認）。<br>・他に生きているかもしれない process の step がない（§2.6）。<br>・run の `body.limit.not_before` を過ぎている。<br>・その epoch で該当の CLI が qualified。 |
| planned → held | tick（起動前の検査） | 未失効の native exhaustion（§2.6）、作業領域の検証の失敗（`worktree_drift`。§4.2）、TestsProfile の digest の不一致（§7.2）。spawn していないので `death_confirmed=true` |
| planned → running（prepare / release） | tick | 現在の owner。cookie・pid は持たない |
| planned → abandoned | Goal cancel の後の tick | 起動していない |
| launching → running | StepLauncher の tx2 | spawn が Ok を返したこと。pid、pgid（=pid）、birth（`rrx-process-tracker` の identity による開始時刻） |
| launching → unknown | StepLauncher / 復旧 | spawn が Err を返した（子が残りうる。`crates/rrx-spawn-hygiene/src/lib.rs:5-6`）か、再起動時に launching を見つけた |
| launching → rejected | StepLauncher | spawn を呼ぶ前の失敗（attempt ディレクトリの作成、sha256 の不一致、test の `recheck` の失敗など）。spawn を呼んでいないことを同じプロセスの中で確実に知っている場合だけ |
| launching → exited | StepLauncher | tx2 が失敗したあと、group を kill して死亡を確認した場合（`body.exit=killed_after_record_failure`）。確認できなければ launching のまま残す（再起動時に unknown になる） |
| running → exited | StepCustody | §3.5 の死亡確認がすべて成功した。終了の状態、出力の sha256、上限を超えたかどうか |
| running → interrupted | 停止 / Goal の pause・cancel | kill して死亡を確認した。attempt が 16 なら interrupted ではなく held（`attempt_limit`、`death_confirmed=true`） |
| running → held | StepCustody / 復旧 | 死亡を確認できない（group が空にならない、discovery が limited、cookie を持つプロセスが見つかる）。または古い epoch の running（custody の喪失） |
| running → unknown | StepCustody | 監視タスク自体の異常（panic・join の失敗）で観測が途切れた |
| running → adopted / held / interrupted（prepare / release） | tick | §4.2 と §9 の検証の結果 |
| exited → committed | AdoptionChecker（規則 G） | implement / fix だけ。§6 の検査に合格し、組み立てた commit の SHA と tree を `result_commit` / `result_tree` に書く |
| committed → adopted | AdoptionChecker（規則 G） | Task branch の ref が `result_commit` を指し、HEAD と index と worktree がその commit と一致する（§6.3） |
| committed → held | AdoptionChecker / 復旧 | ref が step_base でも `result_commit` でもない |
| exited → adopted | VerdictDecoder / PipelineTestRunner（規則 G） | test / review / confirm。kind ごとの採用条件（§7・§8）を満たした |
| exited → rejected | 同上 | 非 0 終了（Agent）、出力の上限超過、schema 違反、scope 違反、Agent が自分で commit したなど、確定した不合格 |
| exited → limited | 同上 | provider の上限（§2.6） |
| exited → held | 同上 | 秘密スキャンへの該当、snapshot の変化、model の不一致、`.git` の変化、判定による保留（§8.5）、上限待ちの回数の超過など、運用者の判断が必要なもの |
| exited / committed → interrupted | Goal cancel の後の tick | 採用せずに終える。死亡は確認済み |
| unknown/held/rejected → abandoned | `rrx pipeline recover --abandon-step` | `pipeline_recoveries` の行（trigger が検査する） |

`unknown` から別の状態へ移るのは、運用者の記録による `abandoned` だけ。Unknown の step は、どの経路からも再起動しない。同じ seq の新しい attempt は、`interrupted`・`limited`・`abandoned` の後にだけ、作業領域を初期化して検証してから作る（trigger が検査する）。attempt は 16 までで、16 回目の attempt は interrupted・limited にならず held になるので、自動で 17 を作ろうとすることはない。

### 3.3 起動の手順

次の 1〜6 を、`control_admission` の guard を保持したまま行う（§1.2）。

1. tx1（`BEGIN IMMEDIATE`、短く）: §3.2 の planned → launching の条件を確認して書き、commit する。
2. Store lock を解放する。Store lock は `std::sync::Mutex` で、control 側は `self.owner.store.lock()` として取っている（`runtime/control.rs:657-662`）。spawn の前に、必ず guard を drop する。
3. attempt ディレクトリと TMPDIR を作り、実行ファイルを照合し直す。Agent は sha256 を Q の記録と照合する。test は `AdmittedProfile::recheck`（`execution/verification/plan.rs:216-223`）で、コマンドが admit 時の一覧にあり、program の hash が一致することを確かめる（既存の経路と同じく各起動の直前に行う。`execution/verification/mod.rs:496`）。不一致なら launching → rejected にし、run を held にする（spawn していない）。
4. `tokio::process::Command` を組み立てる。`env_clear()`、許可した環境変数だけ（§5.4）、`process_group(0)`、`kill_on_drop(true)`、stdin・stdout・stderr は pipe。`rrx_spawn_hygiene::spawn_tokio`（`crates/rrx-spawn-hygiene/src/lib.rs:58`）で起動する。
5. Err の場合: tx で launching → unknown と run → held（`launch_unknown`）を書く。
6. Ok の場合: `ProcessGroup::new`（`adapter.rs:482`）で包み、すぐに tx2 で launching → running（pid、birth）を書く。stdin にはまだ書かない。
   - tx2 が失敗した場合: `cleanup_group` → `reap` → group が空になるのを待つ → discovery。確認できたら launching → exited を試みる。書けなければ何もしない（再起動時に unknown になる）。
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
2. leader が zombie のうちに `cleanup_group` で group に SIGKILL を送る（`adapter.rs:1589`。macOS の EPERM は `ps` で全員が zombie と確かめる。`adapter.rs:588-598`）。
3. `reap` する（`adapter.rs:562`）。
4. 2 秒以内に `test_kill_process_group` が ESRCH を返し、group が空であることを観測する（`git.rs:648-662` と同じ方式）。
5. `rrx_process_tracker::discover(cookie)` の `coverage.limited()` が false で、一致するプロセスが 0 件である（`crates/rrx-process-tracker/src/lib.rs:100,152`）。

どれかが失敗したら「確認できない」とし、held にする（`body.death_confirmed=false`）。macOS では、見つかったプロセスに signal を送らない（`crates/rrx-process-tracker/src/macos.rs:12-23`）。

## 4. 復旧

### 4.1 起動時の照合

rrxd は、owner lock を取り `Store::open` した後、モードの静止の確認（§1.1）を行い、通れば `begin_execution_epoch`（`state/execution.rs:537-599`）で epoch E を得る。続いて Q（§5.2）を行う。Q がすべて qualified なら、最初の sweep で `control_admission.lock().await` を取り、次を順に行う。

1. **引継ぎ**（1 つの tx）: `owner_epoch < E` の、running / held の run と、それぞれの最後の step（最大の seq・attempt）、pending / held の journal を、状態を変えずに `owner_epoch=E` にする（version+1）。held の run も引き継ぐので、回復の command は現在の epoch の run を対象にできる。監査イベント `rrx.private.pipeline.taken_over` を残す。引継ぎの後、古い epoch の書き手は、run・step・journal のどれも trigger で拒否される（§2.3）。
2. **step の照合**: §4.2 の表を、引き継いだ各 run の最後の step に当てる。

その後は、各 sweep で変化した行だけを見る。

### 4.2 復旧表

この表は、再起動時の照合と、通常の tick の両方で使う。

| 永続の状態（引継ぎ後） | 処理 | 結果 |
|---|---|---|
| step planned（process の kind） | 起動前に作業領域を検証する（§6.4 の検証。implement / fix は Task worktree、test は checkout、review / confirm は snapshot を作り直す） | 合格すれば通常どおり起動する（§3.3）。不合格なら planned → held（`worktree_drift`）、run を held |
| step planned（prepare / release） | 通常どおり実行する | |
| step launching | 再実行しない | step を unknown、run を held（`launch_unknown`） |
| step running（process の kind） | custody を失ったとして、cookie で探索する | 見つかった → held（`alive_orphan`、pid を記録）。見つからず coverage が完全 → held（`custody_lost`、`death_confirmed=false`）。limited → held（`custody_unobservable`） |
| step running（prepare） | Git の状態を調べる | branch・worktree・登録がどれもない → interrupted（次の行で attempt+1）。branch が base、worktree が登録済みで HEAD == base かつ clean → adopted。それ以外 → held（`materialization_partial`） |
| step running（release） | §9 の settle の規則 | done / pending のまま / held / aborted |
| step exited（implement / fix） | §6 の検査を最初からやり直す（規則 G）。HEAD == step_base かつ branch の ref == step_base でなければ rejected（`agent_commit`）か held（`head_moved`） | committed / rejected / held |
| step committed | Task branch の ref を読む。== step_base なら §6.3 の update-ref からやり直す。== `result_commit` なら、その commit の tree == `result_tree` かつ親 == step_base を確かめて §6.3 を続ける。それ以外は held（`ref_moved`） | adopted / held |
| step exited（test / review / confirm） | 採用の検査をやり直す（規則 G） | adopted / rejected / held / limited |
| step interrupted / limited | Goal が Running で、limited なら run の `not_before` を過ぎてから、§6.4 で初期化して検証し、attempt+1 を作る。検証に失敗したら作らずに run を held（`worktree_drift`。最後の step は新しい planned を held にしたもの） | 新しい planned |
| step unknown / held / rejected | 何もしない | run は held のまま（運用者を待つ） |
| step adopted（最後のもの） | 次の step を計画する（§8。規則 G） | planned |
| ref journal pending / held | §9 の settle の規則 | |
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
- daemon は次の順に行う。
  1. **再送の照会**（副作用の前）: `runtime_control_acks` に同じ request ID があれば、要求の同一性（action と引数の digest）を確かめ、一致すれば保存した応答を返す。違えば拒否する。ここまで何も変えない（`state/runtime/goals.rs:397-408` と同じ順序）。
  2. `control_admission.lock().await` を取る。以下は guard の中で行う。
  3. **前提の検査**（読み取りの tx。ファイルを変えない）: 現在の owner。run が held で `version == v`。対象の step が run の最後の step で、unknown / held / rejected で `version == w`。`retry` なら Goal が Running で `k < 16`。`abandon-run` は Goal の状態を問わない（終わらせる書込みだけで、起動・採用・release・完了をしないため。規則 G の観測の書込み）。一致しなければ `SnapshotChanged` か理由付きで拒否し、何も変えない。
  4. process の kind で cookie があれば、cookie で探索し直す。1 件でも見つかれば、pid を示して拒否する（運用者に止めてもらう）。coverage が limited の場合も拒否する。
  5. `retry` のときだけ、§6.4 で作業領域を初期化して検証する。失敗すれば拒否する（初期化は rrx が所有する領域だけを対象とし、繰り返しても同じ結果になる）。
  6. **最終の tx**: 3 の検査を（ack がまだないことを含めて）すべてやり直し、次の順に書く。この順序は FK と trigger の条件に合わせたものである。
     - `runtime_control_acks`（応答の body。新しい attempt の番号と各 version を含む）
     - `pipeline_recoveries`（UID、request、reason、`then_action`、`consumed_version = v+1`、discovery の要約、初期化の結果）
     - step を abandoned に（期待 version `w` で CAS。trigger が recoveries の行を確かめる）
     - `retry`: run を held → running に（期待 version `v` で CAS。trigger が `consumed_version` を確かめる）。続いて attempt k+1 の planned を INSERT する（trigger は run が running で、前の attempt が abandoned であることを確かめる）
     - `abandon-run`: run を held → abandoned に（期待 version `v` で CAS。trigger が生きているかもしれない step がないことを確かめる）
     - 監査イベント `rrx.private.pipeline.step_abandoned`
- 古い attempt は再実行しない。cookie も再利用しない。最終の tx のどこかで失敗すれば、全体が rollback される。

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

次の 4 つを確かめ、結果を `pipeline_qualifications` に書く。コマンドは環境を空にしてから組み立て（§5.4）、timeout 5 秒、出力 4 KiB までとする。

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
  --setting-sources "" --disable-slash-commands
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

GH_TOKEN・GITHUB_TOKEN・SSH_AUTH_SOCK・`AWS_*`・`ANTHROPIC_*`・`OPENAI_*`・`GIT_*` は入れない。テストでは「実際に渡した集合 ⊆ 許可した集合」を検査する。test の step も同じ集合を使う（§7.2）。

### 5.5 出力の上限とファイル

attempt ディレクトリは `<work_root>/runs/<run>/<seq>-<attempt>/`（0700）に置く。完了後、ファイルを 0400 にする。

| ファイル | 上限 | 超えたとき |
|---|---|---|
| `prompt.md`（sha256 = `input_sha256`） | 512 KiB | 起動しない（launching → rejected） |
| `stdout.jsonl`（codex `--json`）/ `result.json`（claude） | 16 MiB / 1 MiB | group を kill し、rejected（`output_overflow`） |
| `stderr.log` | 1 MiB | 末尾を残し、`truncated` を記録する（不合格にはしない） |
| `last.md` / `last.json` | 256 KiB | rejected |
| `diff.patch`（rrx が生成） | 8 MiB | held（`diff_too_large`） |
| test の stdout / stderr | コマンドの `stdout_bytes` / `stderr_bytes`（`execution/verification/plan.rs:43-44`） | kill し、test の結果を fail（`output_overflow`） |

### 5.6 provider の上限の分類器（既知の終了だけ）

死亡を確認した後に、pin した版ごとにコードへ固定した規則で分類する。

- codex: `--json` の最後の error / turn 失敗の event の、固定したフィールドと、版の文書にある固定の文字列（利用上限・rate limit）。
- claude: 外側の封筒の `is_error=true` と、固定したフィールド（API の status と、利用上限を示す固定の文字列）。

規則は版ごとの fixture（B3 で、偽 CLI と、取得できた実出力から作る）で検査する。どの規則にも当たらない失敗は上限として扱わない（§2.6 の 5）。使用率などの観測は読まない。

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
- 変更の一覧: `git status --porcelain=v2 -z --untracked-files=all --no-renames --ignore-submodules=none`。
- 変更した各パスが Task scope の glob に一致する。固定の拒否リストに当たるパスは、scope が明示していない限り拒否する: `.git*`、`.gitattributes`、`.gitmodules`、`.claude/**`、`.codex/**`、`AGENTS.md`、`CLAUDE.md`、`**/CLAUDE.md`。
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

検証に失敗したら、新しい attempt を起動しない（§4.2 では held、回復では拒否）。

## 7. テスト step

### 7.1 clean checkout

- 検査対象の commit（Task run は run の head、Goal run は Goal branch の head）ごとに、tick が test の step を計画するとき、`git worktree add --detach <work_root>/co/<run>-<seq> <commit>` で checkout を作る（driver の拒否 §6.1 を通した repo だけ）。作った後、HEAD == commit かつ status が空であることを確かめる。
- checkout のパスは決定的で、rrx だけが作る。再起動時に、step を起動する前のパスが残っていれば、検証に合格すれば使い、不合格なら `git worktree remove --force` で消して作り直す（どの step もまだ起動していないことを、step の状態で確かめてから行う）。
- その commit のすべてのコマンドが終わったら、checkout を消す。

### 7.2 各コマンドの実行

TestsProfile（`execution/verification/plan.rs:50`）の各コマンドを、それぞれ 1 つの test step（`body.command_index`）として、agent の step と同じ StepLauncher・StepCustody・死亡確認で実行する。`collector::collect` は使わない（custody を値で受け取り、自分で signal と reap を行い、`pub(super)` であるため。`execution/verification/collector.rs:114-115,149,174`）。

- **profile の読み戻し**: test step を計画するときと tx1 で、`verification_profiles` の行を `(project_id, digest = goal_execution_profiles.tests_profile_digest)` で読み、`hash(body) == digest`（既存の `profile` `state/execution/verification.rs:77-80` と同じ照合）、`AdmittedProfile` への decode、`validate`（`plan.rs:205-215`）を行う。どれかが失敗すれば step を held（`tests_profile_changed`）にする。同じ key のまま body を変えた REPLACE もここで拒否される（§2.3）。
- **起動前の照合**: 各コマンドの起動の直前に `AdmittedProfile::recheck`（`plan.rs:216-223`）を呼び、コマンドが admit 時の一覧にあり、program の hash が一致することを確かめる（§3.3-3）。admit の後に実行ファイルが差し替わっていれば、起動せず rejected にし、run を held にする。
- argv: `/usr/bin/sandbox-exec -f <A>/test.sb -- <program> <args…>`。program は admit 済みの絶対パス、cwd は checkout の中のコマンドの `cwd`。
- Seatbelt の profile: network を拒否する。書込みは checkout・TMPDIR・運用者が宣言したキャッシュディレクトリだけ。
- 環境変数は §5.4 と同じ集合（`CODEX_HOME` を除く）。
- 上限: `timeout_seconds`・`drain_seconds`・`stdout_bytes`・`stderr_bytes`（`plan.rs:35-45`）。
- 結果の採用（exited → adopted、規則 G）: 終了コード 0 なら pass、それ以外・timeout・出力の上限超過は fail。fail も、正しく観測できた結果として adopted にする。`body` に終了コード、signal、stdout / stderr の `StreamEvidence` を書く。
- 観測が欠けた場合（出力の I/O エラー、reap の失敗、死亡を確認できない）は held。

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
  - ref == new: done（update-ref は境界の中で行い、制御は境界を待つので、release は後の pause・cancel より前に起きている）。done を書く tx は Goal の状態に依らない（すでに起きた ref の変化の記録であるため）。run の completed も同じ tx で書く。
  - ref == old: Goal が Running なら手順 3 からやり直す。Paused なら pending のまま待つ。Cancelled なら journal を aborted、step を interrupted にする（run は §4.3 の cancel で abandoned）。
  - それ以外: held。運用者が ref を old か new に戻した場合だけ、回復の command で新しい attempt が読み直す。
- **次の Task**: run が completed になったら `wake.notify_one()` を呼び、次の tick で次の READY な Task を claim する。ユーザーのプロンプトは要らない。
- **Goal の完了**: DAG の全 Task の run が completed になったら、kind=goal の run を claim する（§3.1）。TestsProfile の各コマンドを、Goal branch の head の clean checkout で test step として実行する。最後のコマンドの採用は tick の中（guard を保持）で行い、その tx で次をすべて確かめる: 現在の owner、**Goal が Running**、run と step の version、全コマンドが pass。満たせば、同じ tx で次を書く:
  - `pipeline_goal_completions` の行（全 run の ID と head を canonical にした sha256）
  - Goal run を completed に（trigger が完了の行を検査する）
  - Goal の state を Completed に（`goals` の body の state を version CAS で更新する。定義の trigger `state/runtime/schema.sql:21-30` には当たらない）
  - 監査イベント

  Goal が Paused なら何も書かず、最後の test の step は exited のまま残り、resume の後の tick が完了をやり直す。Cancelled なら §4.3 で interrupted にし、run は abandoned になる。fail のコマンドがあれば、その step を held（`goal_check_failed`）にする。LLM の出力は完了の判定に使わない。Goal run が completed になると Project の slot が空き、同じ Project の別の Goal が進める。

## 10. 読み取り API

`rrx goal status` / `rrx goal tasks` / `rrx events` で、CLI だけで状態を照会し、回復の command に必要な値を得られるようにする。読み取りは既存どおり短い Store lock の中の read-only の tx で行い（`runtime/control.rs:546-548`）、Git・プロセスの I/O をしない。

`PipelineRunView` を 1 つ定義し、Task の run と Goal 完了確認の run の両方に使う。内容: run の ID・kind・version・state・round・head、最後の step の seq・attempt・kind・state・version、Held の理由（固定の定数）、`limit`（`waits` と `not_before`）、他の step の実行待ちかどうか。

| 対象 | 変更 |
|---|---|
| `ControlResponse::GoalFacts`（`runtime/control.rs:275-288`） | `pipeline: Option<PipelineGoalView>` を足す（`#[serde(default, skip_serializing_if = "Option::is_none")]`）。内容: モード、Goal branch の名前と journal の head、run の state ごとの件数、**Goal 完了確認の run の `PipelineRunView`**、完了の記録（head・`runs_sha256`・epoch） |
| `TaskFacts`（`runtime/control.rs:362-370`） | `pipeline: Option<PipelineRunView>` を足す |
| `TaskFacts.state` / `GoalFacts.states` | パイプラインの Goal では、run と最後の step から導く（下の表）。`tasks` の行の state は書き換えない |
| `GoalFacts.attention` / `goal_attention`（`state/runtime/goals.rs:632-649`） | パイプラインの Goal では、`UnavailableReason` の新しい値を返す: `PipelineHeld`（held の run がある）、`PipelineProviderWait`（provider の上限で待機中）、`PipelineUnqualified`（Q が refused）、`ExecutionModeMismatch`（rrxd のモードと違う）、`PipelineActive`（それ以外。`dispatch_available=true` のときの値） |
| `runtime_goal_facts` / `runtime_goal_task_page`（`state/runtime/goals.rs:300-360,520-626`） | パイプラインの Goal では、`waiting::observe` の代わりに上の view を作る。page の上限（`GOAL_TASK_PAGE_BYTES`）の検査は既存どおり |
| `rrx events` | パイプラインの監査イベントを、既存の scope で返す |

| run と最後の step | `TaskFacts.state` |
|---|---|
| run なし | 保存された Task の state |
| run held | WaitingHuman |
| step planned で run の `not_before` が未到来 / 他の step の実行中 | WaitingQuota / WaitingCapacity |
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
| 環境変数からの秘密の漏洩 | 空から組み立てる（§5.4）。渡した集合 ⊆ 許可した集合を、テストで検査する | なし |
| **Agent がユーザーの読めるファイルを何でも読む**（`~/.aws` などを provider に送る） | v1: すべての出力経路の秘密スキャン（§11）と、オーナーの書面による受入 | **残る**。workspace-write も read-only も、ディスク全体を読める。provider への送信そのものは止められない。根本の対策は後続（#98）: (A) 専用の macOS ユーザーで Agent を動かす。(B) Codex の管理設定 `requirements.toml` の `[permissions.filesystem] deny_read`（0.162.1 で使えるかは未確認）。(C) Claude の `--settings` の `permissions.deny` で Read を制限する（部分的）。 |
| テストが、Agent の書いたコードをオーナーの権限で実行する | clean checkout で実行する。`/usr/bin/sandbox-exec` の Seatbelt profile（network 拒否、書込みは checkout・TMPDIR・運用者が宣言したキャッシュディレクトリだけ）で包む。空から組み立てた環境 | 読み取りは制限されない。secret を読んで stdout に出す経路は秘密スキャンで部分的に検出する（受け入れる） |
| admit 後のテストの差し替え | TestsProfile を読むたびに body と digest を照合し、各起動の直前に `recheck` でコマンドと実行ファイルの hash を照合する（§7.2） | 照合から exec までの短い窓での差し替え（既存の経路と同じ） |
| network | Codex のツールは network を拒否する。Claude には Bash も web もない。テストは Seatbelt で拒否する | CLI 自身が provider と通信する（仕様どおり） |
| 信頼しない verdict JSON | strict decode と上限、未知フィールドの拒否、ID の一意性、ラベル集合の一致、存在しない file / line は invalid、quote と行の照合 | なし |
| 古い owner・二重の rrxd | `owner.lock` の flock（`execution/owner.rs:225`）、全書込み API での現在の owner と version の検査（§2.4）、run・step・journal の行に書いた epoch の trigger（§2.3）、control 要求の epoch の照合（`runtime/control.rs:460-469`） | なし |
| モードの切替で、もう一方の経路のプロセスが残る | epoch の前の静止の確認（永続状態と cookie の探索。§1.1）。epoch の更新を死亡の根拠にしない | cookie を消した孤児と、group を抜けた子孫は検出できない（記録して受け入れる） |
| 停止・cancel と起動・採用・release・完了の競合 | `control_admission` による直列化（§1.2）と規則 G（最終 tx での Goal の Running の検査） | なし |
| 起動結果が不明・孤児プロセス | Unknown / Held、cookie による探索、再実行しない。死亡を確認できない間は全 Project の起動を止める | cookie を消した孤児と、group を抜けた子孫は検出できない（記録して受け入れる） |
| provider の上限を使い切る | daemon 全体で 1 step ずつ、既知の上限の終了だけ backoff、native の未失効の exhaustion を読んで起動しない（§2.6） | 分類器が上限を認識できない失敗は、運用者の判断になる（Held に倒す） |
| Goal branch の破損 | ref-CAS の journal、fast-forward だけ、checkout 中なら拒否 | 運用者が手で ref を動かした場合は held になる |

## 13. 影響範囲分析

### 13.1 変更・追加の対象

| 種別 | 対象 | 影響を受ける箇所 | 影響の種類 | 対応 |
|---|---|---|---|---|
| 定数 | `SCHEMA_VERSION` 11→12、`READ_INDEX_LAYOUT_VERSION`=11 を追加 | `state/mod.rs:32,189-193,202-307`、`rrx_writer_contract_version`（`state/mod.rs:135-141`） | 互換性（v12 の DB は旧バイナリが拒否する。仕様どおり） | §2.1 の 4 経路 |
| 関数 | `validate_layout`（版による参照 layout の組立て） | `state/managed_binding/schema.rs:249-281` | 互換性 | §2.1。v10 と actual9 の参照は変えない |
| 定数 | `PIPELINE_TABLES`（新規、7 表）。`MUTABLE_TABLES` は変えない | `MUTABLE_TABLES` の消費者: `state/managed_binding/schema.rs:160,190`、`state/read_index_tests.rs:199`、`state/managed_binding/schema_tests.rs:32,318`、`state/execution/verification.rs:1149,1187`、`state/execution/tests.rs:370,413`、`state/execution/source_recovery.rs:594,633`、`state/runtime/tests.rs:29,65`、`state/execution/native_results_tests.rs:24,83` | テスト（writer guard の総数を数える `read_index_tests.rs:199`・`execution/tests.rs:413` は、v12 で 7 表 ×3 だけ増える） | 総数の検査を `MUTABLE_TABLES` と `PIPELINE_TABLES` の合計にする。各表の guard を照合するテストは、`PIPELINE_TABLES` にも同じ検査を当てる |
| DB | 新しい表 7 つと trigger | `state/execution/pipeline.sql`、`state/sqlite_build_tests.rs` | 機能 | 追加する |
| 関数 | `RuntimeOwner::open`（モードを受け取り、`Store::open` と `begin_execution_epoch` の間で静止を確かめる） | `execution/owner.rs:270-279`。呼び出し元は daemon の起動と多数のテスト | 起動・互換性 | `open` は `managed_native` のまま残し、モード付きの入口を足す。パイプラインの行がない DB では確認が件数 0 で通り、挙動は変わらない |
| 関数 | `quotas::windows` | `state/execution/quotas.rs:476-502`、呼び出し元 `:186` | なし（private → `pub(super)`。挙動は変えない） | パイプラインは読むだけ |
| 型 | `Config.execution`・`Config.pipeline`（任意） | `config.rs:226,376-428`、`runtime/control.rs:540-545`（policy digest） | 互換性 | 既定値は serialize しない。未設定なら digest が変わらないことをテストで確かめる |
| loop | sweep | `runtime/service.rs:76-161`（`:130` をモードで分ける）、`:168-200`、`runtime/stop.rs:7-34`（`PipelineSteps` を足す） | 機能・停止 | tick と停止の site を加える |
| 境界 | `control_admission` | `runtime/admission.rs:37-41`、`runtime/control.rs:641,667,711`、`runtime/service.rs:59,170`、`runtime/task_driver.rs:79`、`runtime/phase_supervisor.rs:1021-1322`、`runtime/installation.rs:110-251`、`runtime/project_control.rs:219` | 停止・応答時間（tick が spawn・短い Git 操作・採用の検査の間だけ保持する） | try_lock で取り、CLI の実行や終了待ちは外で行う。他の消費者の保持の仕方は変えない |
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
| 入力の一覧 | 必須 flag・許可する環境変数・固定の拒否パス・秘密の pattern・上限の分類規則・driver の拒否キー | パイプラインだけ | 安全 | 広がる向きを検出するテストにする |
| 定数 | `global_max_sessions` の既定 12（`config.rs:64`）、`MVP_PROJECT_TASKS`=1（`config.rs:17`）、`PHASE_SLOTS_PER_PROJECT`=4（`runtime/phase_supervisor.rs:22`）、native の上限の既定（`execution/quota.rs:18-20`） | 変えない | | パイプラインは 1 step ずつなので、`MVP_PROJECT_TASKS` だけを使う |

### 13.2 影響なしの根拠

- **native の `start_phase` 経路**: `managed_native` の rrxd では、`cli_pipeline` の Goal を candidate と claim の段階で除くことと、起動時の静止の確認（パイプラインの行がなければ件数 0 で通る）だけで、その他は変えない。既存の Goal には `goal_execution_profiles` の行がない。`cli_pipeline` の rrxd は native の admission を呼ばない。
- **quota の表**: パイプラインは `quota_pools`・`quota_windows`・`quota_leases`・`quota_waiters` を書かない。読むのは、静止の確認と tx1 での `quota_windows`・`quota_pools.next_probe_at` と、静止の確認での `quota_leases` の件数だけ。`quota_policy` は使わない。
- **`GenericCliAdapter` と `ProcessGroup`**: 変えない（§16 D5）。`cleanup_group`・`observe_exit`・`reap` は既存の `pub(crate)` をそのまま使う。
- **`collector::collect`**: 呼ばない。可視性を変えるのは `retain` / `inspect` だけ。
- **Docker・Grok・`ResultStore`・`AttemptManager`**: パイプラインから呼ばない。
- **M0 branch**: 今は影響しない。M0 を再開するときは、M0 の schema を v13 へ番号を振り直す（§16 D1）。

### 13.3 別 Issue に記録するもの

- 既存の `verification_profiles` に `no_replace` の trigger がない（`state/execution/verification.sql:1-10`）。同じ key の REPLACE で body を変えられる。パイプラインは読むたびの digest 照合で拒否する（§2.3・§7.2）が、表そのものの保護は別 Issue で扱う。

### 13.4 回帰の確認（各 unit の最後に行う）

`cargo test --workspace` の全件、`crates/rrx/tests/runtime_cli.rs`・`crates/rrx/tests/daemon.rs`・`crates/rrx/tests/goal_cli.rs`・`crates/rrx/tests/state.rs` の明示実行、v0・v9・v10・v11 の DB を v12 に版上げするテスト（途中の失敗で全体が rollback されることを含む）、`managed_native` の Goal の既存のテストがすべて通ること。

## 14. テスト方針

- **偽の CLI**: `crates/rrx/tests/support/fake_codex.py` と `fake_claude.py`（既存の `crates/rrx/tests/support/grok_fake.py` と同じ流儀）。
  - 環境は空から組み立てるので、シナリオは同じディレクトリの `scenario.json` から読む。テストごとに別のディレクトリへ置く。
  - `--version`、`--help`、`login status` / `auth status`、`exec` / `-p` を実装する。
  - 書くもの、出力、終了コード、遅延、stdin を読まない・先に大量に出力する、子プロセスの fork（cookie を持ったままのもの・消したもの）、フィールドの欠落・重複、provider の上限の出力を、シナリオで指定する。
- **障害注入マトリクス（AC-8）**:

| ID | 注入 | 期待する結果 |
|---|---|---|
| F-a1 | Executor が非 0 で終了する | rejected、run held、他 Project は進む |
| F-a2 | 出力の上限を超える / timeout / stdin を読まない / stdin を読む前に大量に出力する | 期限内に group を kill し、死亡を確認して rejected。launching のまま残らない |
| F-a3 | scope 外を変更する / symlink / 秘密の pattern / 自分で commit する（正しい親と偽の trailer を含む） | rejected または held。commit を採用しない |
| F-a4 | repo に filter driver の宣言（`.gitattributes` の深い階層、repo の設定） | prepare か採用検査で held。driver のコードは実行されない（偽 driver が作る印のファイルがないことで確かめる） |
| F-a5 | provider の上限の出力で終了する / 回数の上限まで繰り返す / 分類できない失敗 / native の `quota_windows` に未失効の exhaustion | limited、run に記録した backoff の後に step_base へ初期化して attempt+1。回数を超えたら held。分類できない失敗は rejected。未失効の exhaustion があれば起動せず held |
| F-b1 | `request_changes`、指摘が Confirmed | fix に入り、round が +1 |
| F-b2 | 上限で承認に至らない（N=0・1・2）/ Unclear の blocking / Confirmed の blocking がない request_changes | すべて held。N 回目の fix の後のレビューまで行われる |
| F-b3 | verdict の未知フィールド・重複キー・重複 ID・存在しない行・model の不一致・snapshot の変化、ラベルの余分・欠落・空 quote・`evidence.line` が 0 / 範囲外 / quote の開始行と不一致 | rejected または held（quote と行の不一致は Unclear に落ちて held） |
| F-c1 | spawn が Err を返す（存在しない実行ファイルではなく、hook で注入する） | unknown、run held、再実行しない |
| F-c2 | tx1 の後、spawn の前に crash する | 再起動時に unknown・held |
| F-c3 | tx2 が失敗する | group を kill し、確認できれば exited、できなければ launching のまま→unknown |
| F-c4 | claim の後、branch 作成の後、worktree の部分作成の後に crash する | 何もなければ再作成、完全なら採用、部分なら held。既存のものを消さない |
| F-c5 | exited の記録の後 / committed の記録の後 / update-ref の後に crash する | §6.3 の表のとおり。記録した SHA だけを採用する |
| F-d1 | running 中に正常停止する | interrupted、attempt+1 で再開する |
| F-d2 | running 中に SIGKILL で rrxd を止める | 再起動時に引継ぎ、custody を失ったとして held。cookie の孤児がいれば pid を記録する |
| F-d3 | release の `update-ref` の前後で crash する | ref==old なら（Goal が Running のときだけ）やり直し、ref==new なら done、それ以外は held |
| F-d4 | tx1 の後・spawn の前、および journal の作成の後・ref 更新の前に、pause / cancel / stop を割り込ませる | 制御は境界を待ち、制御の受理の後に新しい起動も release も起きない |
| F-d5 | step の exited の後・採用の前、および最後の test の exited の後・Goal 完了の前に、pause / cancel を割り込ませる | 採用も Goal の Completed も起きない。pause は resume の後に採用・完了をやり直す。cancel は interrupted と run の abandoned |
| F-e1 | 引継ぎの後に、古い epoch の書き手が tx1 / tx2 / step の終了の記録 / 採用 / run の完了 / recovery / journal の settle / Goal の完了を書く | API の検査と trigger の両方で拒否する（片方を外しても、もう片方で拒否されることを確かめる。step の更新も trigger だけで拒否される）。spawn しない |
| F-e2 | 再起動の前から held の run に `rrx pipeline recover` | 引継ぎ済みの現在の epoch で新しい attempt を作る。旧 attempt は再実行しない |
| F-f1 | tick と `pipeline recover` が同じ step を同時に更新する / 同じ期待 version の recover を 2 つ | 片方だけが成功し、負けた側は何も書かず spawn しない |
| F-f2 | 新しい request ID の初回の recover（retry と abandon-run）、その再送、古い期待 version、途中の失敗 | 初回は成功（tx の順序）。再送は作業領域を変えずに同じ応答。古い version はファイルを変えずに拒否。途中の失敗は全体が rollback |
| F-g1 | Project A の run を死亡確認済みで held にする / A に死亡を確認できない step を残す | 前者は Project B の run が完了まで進む。後者は B も起動を待ち、理由が表示される |
| F-g2 | Goal run の合格 / 不合格 | 合格なら Goal Completed と run completed で、同じ Project の別 Goal が進む。不合格なら held で、Goal の view だけで recover を組み立てて再テストできる |
| F-h1 | `abandon-step` を実行したとき、cookie を持つプロセスが残っている | 拒否する |
| F-h2 | attempt 16 の step が中断・上限で終わる / その run に `--then retry` と `--then abandon-run` / cancel で全 step の死亡を確認できる場合とできない場合 | 16 回目は held。retry は拒否、abandon-run で run が abandoned になり slot が空く。cancel は確認できれば abandoned、できなければ held のまま |
| F-m1 | モードの切替の前に、もう一方の作業を 1 種類ずつ残す（native: open な Unit・pending の effect・cookie を持つ残存プロセス・未失効の exhaustion。pipeline: held の run・unknown の step・開いた journal・未到来の上限待ち） | rrxd は epoch を進めずに起動を失敗させ、何も書かない。片付けた後は起動する |
| F-t1 | admit の後にテストの実行ファイルを差し替える / 同じ key の REPLACE で profile の body を変える | 起動せず rejected / held。合格として採用しない |
| F-i1 | `INSERT OR REPLACE` で各表の主キー・UNIQUE・部分 UNIQUE に衝突させる（`recursive_triggers=false`）/ Agent の kind で provider が NULL の step を INSERT する | すべて拒否され、元の行が残る |

- **mutation の対象**（壊したら落ちることを確かめる）。
  - 配線:
    - native の除外を消す
    - 静止の確認を epoch の後に動かす / cookie の探索を消す
    - tick で `control_admission` を取らないようにする / spawn の前に Store lock を解放しないようにする
    - tx1 の epoch 照合・Goal の Running の検査・1 process の検査を消す
    - 採用・release・Goal 完了・回復の最終 tx の Goal の Running の検査を消す
    - `pipeline_tick` を呼ばないようにする
    - rrx の Git の `GIT_CONFIG_GLOBAL` / `--no-filters` を外す
    - 秘密スキャンを prompt の書込みの前から外す
    - test の起動前の `recheck` / profile の digest 照合を消す
    - 回復で、再送の照会を作業領域の初期化の後に動かす
  - 部分一致と存在の検査:
    - scope の glob を前方一致にする
    - quote の照合を緩める（空を許す / 開始行を見ない）
    - `--help` の flag 検査を部分文字列にする
    - 許可する環境変数を広げる
    - ラベル集合の一致を「部分集合」にする
  - 「この検査が保証する」と書いた箇所:
    - step / run / journal の trigger の各条件（epoch・遷移・完了の EXISTS・放棄の NOT EXISTS・1 process の EXISTS）
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
| B2 | schema v12 と版上げの 4 経路、Store API（spawn しない）、trigger、native の除外、`pipeline_mode_inventory` | AC-2、AC-7、AC-8(e)(f) | (1) v0・v9・v10・v11 → v12 と、途中の失敗での rollback。(2) 各 trigger 規則の正負の SQL テスト（引継ぎ後の古い step の更新を trigger 単独で拒否、Agent の kind の NULL provider の拒否、放棄の条件を含む）。(3) 回復の tx が retry・abandon-run とも初回で成功し、再送が同じ応答を返す | `architecture.md` §13、`goal-runtime.md` §14、`multi-project-runtime.md` §7 |
| B3 | `ExecutionConfig` / `PipelineConfig`、モード付きの `RuntimeOwner::open` と静止の確認、CLI の pin、Q、上限の分類器の fixture | AC-1 | F-m1 の各行 | `agent-adapter.md` §10、`agent-execution.md` の「Process launch boundary」 |
| B4 | StepLauncher と StepCustody、実行境界、1 process の規則、native の exhaustion の読み取り | AC-8(c)(d) | — | `agent-execution.md` §2・§4 |
| B5 | Executor と AdoptionChecker、`PipelineGit`、prepare、作業領域の初期化（§6.4） | AC-3 | — | `agent-execution.md`、`architecture.md` §10・§14 |
| B6 | `PipelineTestRunner`、Seatbelt、`ProjectAdmitTests`、profile の照合 | AC-3 | — | `workflow-engine.md` §17、`agent-execution.md` §5 |
| B7 | Reviewer と VerdictDecoder、snapshot、秘密スキャン | AC-4 | — | `workflow-engine.md` §6、`agent-adapter.md` §7 |
| B8 | ふるい分け、Confirmer、fix のループ、判定 | AC-5 | — | `workflow-engine.md` §6・§7 |
| B9 | `pipeline_tick`、release の journal、次の Task、Goal run と Goal の完了、読み取り API | AC-6 | (1) release と pause / cancel の交差（F-d4）。(2) Paused / Cancelled の Goal が release も Completed もされない（F-d5）。(3) Goal run の view だけで recover の command を組み立てられる（F-g2） | `goal-runtime.md` §6・§7、`multi-project-runtime.md` §7 |
| B10 | 起動時の照合と復旧表、正常停止、pause / cancel、`pipeline recover` | AC-7、AC-8(c)(d)(e)(f) | — | `goal-runtime.md` §14、`architecture.md` §9 |
| B11 | 障害注入マトリクスの完了、オーナーが実 Mac で実行するチェックリストの script、受入の記録 | AC-1〜AC-8 | — | `doc/verification/issue-106-acceptance.md` |

## 16. 承認済みの計画との相違

| ID | 計画 | この設計 | 根拠 |
|---|---|---|---|
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
| D15 | quota lease を使う | v1 は daemon 全体で 1 step ずつ動かし、quota の表を書かない。provider の上限は既知の終了だけを run の backoff で待ち、native の exhaustion は読むだけ | `quota_leases` / `quota_waiters` は `execution_units` への FK を持つ（`state/execution.sql:130-142`）。native の規則（provider ごとの上限の集約、probe、公平性）を条件ごと移し替えると、改訂のたびに弱まりが見つかった。1 step ずつはどの上限よりも厳しく、Stage 1（1 Project・直列の step）の要求を満たす |

**オーケストレーターの簡素化の指示から外した点**

| 指示 | この設計 | 理由 |
|---|---|---|
| repo の設定と global の設定に driver があれば拒否する | repo の設定の driver は拒否する。global / system の設定は、rrx の Git から外す（`GIT_CONFIG_GLOBAL=/dev/null`、`GIT_CONFIG_NOSYSTEM=1`） | オーナーの Mac では global の git-lfs の filter が一般的で、拒否にすると全 Project が使えなくなる。外せば driver は rrx の Git で起動できず、目的（rrx が driver を実行しないこと）は同じに満たす |
| tick は tx1 → spawn → tx2 の間 `control_admission` を保持する | それに加えて、claim・rrx 内部の step・採用・release・Goal の完了も、tick の中で境界を保持して行う | 結果を進める書込みを、pause・cancel と直列化するため（規則 G） |

## 17. Revision 2

| 指摘 | 解決 | 節 |
|---|---|---|
| 範囲の絞り込み | daemon 全体で process の step を 1 つずつ。provider ごとの上限・公平性・予約枠・probe と `pipeline_provider_limits` を削除。並行実行と容量の共有は Stage 2 | §2.6、要件 §3 |
| E-01 | モードの静止を両方向で定義し、`begin_execution_epoch` の前に owner lock を持ったまま確かめる。永続状態（開いた Unit・effect・lease・run・journal・上限待ち）と cookie の探索（完全な coverage で 0 件）を要求し、epoch の更新を根拠にしない | §1.1、§4.1、§5.2 |
| E-02 | 既知の上限の終了だけを run の backoff で待つ。step の起動前に native の未失効の exhaustion（provider と account を問わない）を読み、あれば held。`managed_native` への切替は上限待ちが残る間は拒否 | §2.6、§1.1、§5.6 |
| E-03 | 1 step ずつにして、provider ごとの上限の計算をなくした | §2.6 |
| E-04 | probe をなくした（1 step ずつの起動が、native の probe 時刻の後の単一の起動に当たる） | §2.6 |
| E-05 | 回復の最終 tx を ack → recovery → step の abandoned → run の遷移 → 新しい step の INSERT の順にした | §4.4 |
| E-06 | 規則 G: 採用・release・Goal 完了・回復は境界の中で、最終 tx で Goal の Running と version を確かめ直す。Paused / Cancelled では完了も release もしない | §1.2、§9、§6.3 |
| E-07 | 再送の照会と期待 version の検査を、作業領域の変更より前に行う。最終 tx でやり直す | §4.4 |
| E-08 | step に `owner_epoch` を持たせ、書込み API は毎回自分の epoch を明示して書く。引継ぎの後の古い書込みを trigger だけで拒否する | §2.2、§2.3、§2.4、§4.1 |
| E-09 | `PipelineRunView` を Task の run と Goal 完了確認の run の両方に出す | §10 |
| E-10 | 16 回目の attempt は held。`--then abandon-run` で新しい attempt を作らずに run を放棄して slot を空ける。cancel は死亡を確認できれば run を放棄する | §3.1、§3.2、§4.3、§4.4 |
| E-11 | 各 test の起動直前に `AdmittedProfile::recheck` を呼ぶ | §3.3、§7.2 |
| E-12 | FK で守られるという説明を訂正した。パイプラインは読むたびに body と digest を照合し、不一致なら held | §2.3、§7.2、§13.3 |
| E-13 | 公平性の先頭の選択をなくした | §2.6 |
| E-14 | Agent の kind で `provider IS 'codex'` / `IS 'claude'` を要求し、NULL を CHECK で拒否する | §2.2 |
| E-15 | `evidence.line` は 1 以上で範囲内、quote の出現の開始行と一致すること。不一致は Unclear | §5.7、§8.3 |
| E-16 | 対応なし（指摘は実在しない。改訂 1 の DDL は 8 表で記述と一致していた）。改訂 2 で 1 表を削除し、7 表と記述した | §2.2、§2.3 |
| E-17 | B2 と B9 に、Unit の中の局所の受入を置いた | §15 |
