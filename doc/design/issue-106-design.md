# Issue #106 設計書: Plan B — CLI サブプロセスによる Agent パイプライン

Status: 提案中（B0）。基準の source は origin/main `79e4d273`。要件は `doc/requirements/issue-106-requirements.md`。ファイルパスは、断りがなければ `crates/rrx/src/` からの相対パスで書く。

## 0. 再利用する部品の所在（B1 の判定）

各部品が main にあるか、M0 branch（`feature/issue-5-m0`）だけにあるか、どこにもないかを確認した。

| 部品 | main | M0 だけ | 無い | 根拠と注記 |
|---|---|---|---|---|
| `GenericCliAdapter` | ✓ | | | `adapter.rs:624`。本番では使われていない（`AgentRegistry::from_config` `adapter.rs:300-328` を呼ぶのはテスト `tests/adapter.rs:321` だけ）。パイプラインはこれを使わない（§12 D5） |
| `ProcessGroup` | ✓ | | | `adapter.rs:469`。`observe_exit` は `:503`、`reap` は `:562`、`cleanup_group` は `:1589`。native の Codex（`codex/transport.rs:23,125`、`codex/preparation.rs:295`）と Grok も使っている |
| `execution/process.rs` | ✓ | | | `OwnedProcess::spawn` は `:232`、`stop_and_reap` は `:268`、`capture_child` は `:319` |
| `execution/attempts.rs` | ✓ | | | `AttemptManager` は `:9`。Execution Unit 専用で、パイプラインは使わない |
| `git.rs` `WorktreeManager` | ✓ | | | `:96`。reset はない。hook を止めていない（`git.rs:141`、`adapter.rs:1306`） |
| `execution/results.rs` `ResultStore` | ✓ | | | `:18`。snapshot の手順（`:630`）を参考にする |
| `verification/collector.rs` `collect` | ✓ | | | `:114`。`retain` は `:218` |
| `ManagedVerifier` | ✓ | | | `verification/mod.rs:206`。`admit_tests` は `:226` で、呼ぶのはテストだけ |
| `state/runtime/driver/claim.rs` | ✓ | | | `claim_initial_driver` は `:351`、occupied CTE は `:375` |
| `runtime/service.rs` | ✓ | | | sweep loop は `:76-161`、shutdown は `:168-200` |
| `crates/rrx-process-tracker` | ✓ | | | `discover` は `lib.rs:152`、cookie の環境変数は `lib.rs:22`、`Coverage::limited` は `lib.rs:100` |
| `crates/rrx-spawn-hygiene` | ✓ | | | `spawn_std` は `lib.rs:31`、`spawn_tokio` は `lib.rs:58`。Tokio が失敗しても子が残りうることを `lib.rs:5-6` に明記している |
| strict JSON decoder | ✓ | | | `execution/strict_json.rs:67`。重複キーと上限を検査する。未知フィールドは DTO の `deny_unknown_fields` で拒否する |
| schema version | 11 | 12 | | main は `state/mod.rs:32`、M0 は `feature/issue-5-m0:crates/rrx/src/state/mod.rs:34` |
| `runtime_startup_probes` | | ✓ | | `feature/issue-5-m0:crates/rrx/src/state/startup.sql:1`。main には無い（§12 D3） |
| `codex_wire` / `claude_wire/decoded` / `runtime/startup.rs` | | ✓ | | M0 で追加されたもの。パイプラインは使わない |

**B1 の結論**: 基準は origin/main とする。M0 だけにある部品で必須のものはない。`runtime_startup_probes` の代わりに、v12 で `pipeline_cli_qualifications` を足す。`79e4d273` の CI が緑であることは、B1 で `gh` を使って確認し、`doc/verification/issue-106-b1.md` に記録する（この文書の作成時点では未確認）。

## 1. アーキテクチャ

### 1.1 2 つの実行経路

```
rrx CLI ──(Unix control socket)──► rrxd Runtime ── sweep loop (runtime/service.rs:76-161)
                                        │
             ┌──────────────────────────┴──────────────────────────┐
     managed_native Goal                                    cli_pipeline Goal
     admit_ready_tasks (admission.rs:33)                    pipeline_tick（新規）
     Task Driver / start_phase（既存・変更なし）             PipelineScheduler → StepLauncher
                                                             → codex exec / tests / claude -p
             └──────── 共通: Store（SQLite）、runtime_epoch、容量の数え方 ────────┘
```

- 実行プロファイルは、`goal_execution_profiles` の行の有無で決める（§2）。行がある Goal は `cli_pipeline`、行がない Goal（既存の Goal すべて）は `managed_native`。
- native 側は `cli_pipeline` の Goal を拒否する。`ready_driver_candidates`（`state/runtime/driver/candidates.rs:49`）と `claim_initial_driver`（`claim.rs:365-369`）の両方に `NOT EXISTS(SELECT 1 FROM goal_execution_profiles WHERE goal_id=…)` を加える。既存の 3 つの関門（`Store::managed_phase_required` `state/managed_binding/protection.rs:15`、`require_managed_native_binding_composed` `workflow.rs:539`、`AgentRegistry::from_managed_config` `adapter.rs:332`）は変えない。
- パイプライン側は、`goal_execution_profiles` の行がない Goal を claim しない。
- `reconcile_runtime_attention`（`state/runtime/service.rs:25,45-48`）は、`cli_pipeline` の Task に `native_binding_unavailable` を書かない。代わりにパイプラインが自分の attention（`pipeline_waiting` / `pipeline_held`）を書く。

### 1.2 新しい部品（`execution/pipeline/` に置く）

| 部品 | 責務 |
|---|---|
| `PipelineScheduler` | 次の 4 つを行う。<br>・sweep の中で READY な Task を選ぶ。<br>・run を claim する。<br>・各 step の後に次の step を計画する。<br>・release journal を settle する。 |
| `StepLauncher` | §3.3 の 2 段トランザクションで起動する。spawn は `rrx_spawn_hygiene::spawn_tokio` を通す。 |
| `StepCustody` | 次を行う。<br>・既存の `ProcessGroup`（`adapter.rs:469`）を包む。<br>・終了を観測する（`observe_exit` `:503`）。<br>・group を kill する（`cleanup_group` `:1589`）。<br>・reap する（`:562`）。<br>・group が空になるのを待つ（`git.rs:648-662` と同じ方式）。<br>・cookie で探索する（`rrx_process_tracker::discover` `lib.rs:152`）。 |
| `PipelineGit` | rrx 自身の Git 呼び出し。`env_clear` で空の環境から組み立て、常に `-c core.hooksPath=/dev/null -c core.fsmonitor=false -c gc.auto=0 -c maintenance.auto=false -c commit.gpgsign=false` を付ける。Git 設定の digest をとる。 |
| `AdoptionChecker` | Executor の結果を rrx commit にする前の検査（§5.3）。 |
| `PipelineVerificationPort` | `TestsProfile`（`verification/plan.rs:50`）を Seatbelt で包み、`collector::collect`（`collector.rs:114`）で実行する。 |
| `VerdictDecoder` | `strict_json::decode`（`strict_json.rs:67`）の後に、`deny_unknown_fields` の DTO へ変換する。 |
| `CliQualifier` | 起動時の Q（§5.2）。 |

### 1.3 step の種類ごとのデータの流れ

| step | 入力 | 実行 | 出力として採用するもの |
|---|---|---|---|
| implement | Task の title・受入条件・scope（運用者が書いたもの）と Task worktree | `codex exec --sandbox workspace-write`（cwd は Task worktree） | rrx commit（§5.3）。Codex の `last.md` は表示用のデータに留める |
| tests | rrx commit と、admit 済みの TestsProfile | 各コマンドを、commit の clean checkout の中で Seatbelt に包んで実行する | 終了コード、出力の sha256、evidence ファイル |
| review | base..head の diff と、読み取り専用の snapshot | `claude -p --permission-mode plan`（cwd は snapshot） | 検査を通った verdict JSON |
| confirm | ふるい分けを通った blocking 指摘と、別の snapshot | `codex exec --sandbox read-only` | 検査を通ったラベル JSON |
| fix | Confirmed の指摘だけ（またはテスト失敗の要約）と Task worktree | implement と同じ | rrx commit |
| goal_check | Goal branch の head | tests と同じ | 合否 |
| release（step ではない） | 承認された run | `PipelineGit` が `update-ref` を行う | ref journal の done |

## 2. Schema v12

main は v11（`state/mod.rs:32`）。v12 を足す（計画にある「v13」との違いは §12 D1）。v12 の追加は次の 5 か所で行う。

- 新規作成の手順（`state/mod.rs:214-232`）
- `next=12` の版上げ（`state/mod.rs:271-293`）
- `MUTABLE_TABLES`（`state/execution.rs:7-53`）への新しい表の追加
- writer guard の再生成（`state/execution.rs:129-143`）
- 全 catalog を照合する参照 schema（`state/managed_binding/schema.rs:249-257`）

v11 の DB では、`execution::install_writer_guards` を呼んだあとに `pipeline.sql` を実行する。既存の表は変えない。

### 2.1 表（`state/execution/pipeline.sql`）

```sql
-- 実行プロファイル。行がない Goal は managed_native。受入トランザクションの中で goal_authority と同時に作る。
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
-- 変更・置換・削除を禁止する（goal_authority と同じ形の 3 trigger: no_update / no_replace / no_delete）

CREATE TABLE pipeline_runs (
 id TEXT PRIMARY KEY NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('task','goal')),
 task_id TEXT, goal_id TEXT NOT NULL, project_id TEXT NOT NULL,
 owner_epoch INTEGER NOT NULL CHECK(typeof(owner_epoch)='integer' AND owner_epoch>0),
 version INTEGER NOT NULL CHECK(typeof(version)='integer' AND version>0),
 state TEXT NOT NULL CHECK(state IN ('running','held','completed','abandoned')),
 round INTEGER NOT NULL CHECK(round BETWEEN 0 AND 2),
 base_commit TEXT NOT NULL CHECK(length(base_commit) IN (40,64)),
 head_commit TEXT CHECK(head_commit IS NULL OR length(head_commit) IN (40,64)),
 worktree TEXT UNIQUE, branch TEXT UNIQUE,
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=131072
   AND json_extract(body,'$.id') IS id AND json_extract(body,'$.version') IS version
   AND json_extract(body,'$.state') IS state AND json_extract(body,'$.epoch') IS owner_epoch),
 CHECK((kind='task' AND task_id IS NOT NULL) OR (kind='goal' AND task_id IS NULL)),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
CREATE UNIQUE INDEX pipeline_run_per_task ON pipeline_runs(task_id) WHERE kind='task';
CREATE UNIQUE INDEX pipeline_run_per_goal ON pipeline_runs(goal_id) WHERE kind='goal';
CREATE TRIGGER pipeline_runs_transition BEFORE UPDATE ON pipeline_runs
WHEN OLD.id<>NEW.id OR OLD.kind<>NEW.kind OR OLD.task_id IS NOT NEW.task_id OR OLD.goal_id<>NEW.goal_id
 OR OLD.project_id<>NEW.project_id OR OLD.base_commit<>NEW.base_commit
 OR OLD.worktree IS NOT NEW.worktree OR OLD.branch IS NOT NEW.branch
 OR NEW.version<>OLD.version+1
 OR NEW.owner_epoch<OLD.owner_epoch
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.owner_epoch<>(SELECT epoch FROM runtime_epoch WHERE singleton=1)))
 OR NEW.round<OLD.round OR NEW.round>OLD.round+1
 OR NOT (NEW.state=OLD.state
   OR (OLD.state='running' AND NEW.state IN ('held','completed','abandoned'))
   OR (OLD.state='held' AND NEW.state IN ('running','abandoned')))
 OR (OLD.state='held' AND NEW.state='running' AND NOT EXISTS(
     SELECT 1 FROM pipeline_recoveries r WHERE r.run_id=NEW.id AND r.consumed_version=NEW.version))
BEGIN SELECT RAISE(ABORT,'invalid pipeline run transition'); END;
-- no_replace / no_delete trigger（task_drivers と同じ形。runtime/schema.sql:99-103）

CREATE TABLE pipeline_steps (
 run_id TEXT NOT NULL REFERENCES pipeline_runs(id),
 seq INTEGER NOT NULL CHECK(seq BETWEEN 1 AND 64), attempt INTEGER NOT NULL CHECK(attempt BETWEEN 1 AND 16),
 kind TEXT NOT NULL CHECK(kind IN ('implement','tests','review','confirm','fix','goal_check')),
 round INTEGER NOT NULL CHECK(round BETWEEN 0 AND 2),
 state TEXT NOT NULL CHECK(state IN ('planned','launching','running','exited','adopted','rejected',
                                     'interrupted','unknown','held','abandoned')),
 version INTEGER NOT NULL CHECK(version>0),
 launch_epoch INTEGER CHECK(launch_epoch IS NULL OR launch_epoch>0),
 cookie TEXT CHECK(cookie IS NULL OR length(cookie) BETWEEN 16 AND 128),
 pid INTEGER CHECK(pid IS NULL OR pid>1), birth TEXT,
 step_base TEXT NOT NULL CHECK(length(step_base) IN (40,64)),
 input_sha256 TEXT NOT NULL CHECK(length(input_sha256)=64),
 result_commit TEXT,
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=262144),
 PRIMARY KEY(run_id,seq,attempt)
);
-- run ごとに、未終端の step は同時に 1 つまで
CREATE UNIQUE INDEX pipeline_step_one_open ON pipeline_steps(run_id)
 WHERE state IN ('planned','launching','running','exited');
CREATE TRIGGER pipeline_steps_transition BEFORE UPDATE ON pipeline_steps
WHEN OLD.run_id<>NEW.run_id OR OLD.seq<>NEW.seq OR OLD.attempt<>NEW.attempt OR OLD.kind<>NEW.kind
 OR OLD.round<>NEW.round OR OLD.step_base<>NEW.step_base OR OLD.input_sha256<>NEW.input_sha256
 OR NEW.version<>OLD.version+1
 OR (OLD.cookie IS NOT NULL AND OLD.cookie IS NOT NEW.cookie)
 OR (OLD.cookie IS NULL AND NEW.cookie IS NOT NULL AND NOT (OLD.state='planned' AND NEW.state='launching'))
 OR (OLD.pid IS NOT NULL AND (OLD.pid IS NOT NEW.pid OR OLD.birth IS NOT NEW.birth))
 OR (OLD.pid IS NULL AND NEW.pid IS NOT NULL AND NOT (OLD.state='launching' AND NEW.state='running'))
 OR (OLD.launch_epoch IS NOT NULL AND OLD.launch_epoch IS NOT NEW.launch_epoch)
 OR (OLD.result_commit IS NOT NULL AND OLD.result_commit IS NOT NEW.result_commit)
 OR NOT (NEW.state=OLD.state
   OR (OLD.state='planned'   AND NEW.state IN ('launching','abandoned'))
   OR (OLD.state='launching' AND NEW.state IN ('running','unknown','exited','rejected'))
   OR (OLD.state='running'   AND NEW.state IN ('exited','interrupted','held','unknown'))
   OR (OLD.state='exited'    AND NEW.state IN ('adopted','rejected','held'))
   OR (OLD.state IN ('unknown','held','rejected') AND NEW.state='abandoned'
       AND EXISTS(SELECT 1 FROM pipeline_recoveries r WHERE r.run_id=NEW.run_id AND r.seq=NEW.seq AND r.attempt=NEW.attempt)))
 OR (OLD.state IN ('adopted','interrupted','abandoned') AND NEW.state<>OLD.state)
BEGIN SELECT RAISE(ABORT,'invalid pipeline step transition'); END;
-- no_replace / no_delete trigger。INSERT は state='planned'、version=1、cookie と pid が NULL の行だけを許す

CREATE TABLE pipeline_recoveries (            -- 運用者の放棄記録。追記だけ
 id TEXT PRIMARY KEY NOT NULL, run_id TEXT NOT NULL, seq INTEGER NOT NULL, attempt INTEGER NOT NULL,
 request_id TEXT NOT NULL UNIQUE REFERENCES runtime_control_acks(request_id),
 ingress_uid INTEGER NOT NULL, accepted_epoch INTEGER NOT NULL CHECK(accepted_epoch>0),
 consumed_version INTEGER NOT NULL,           -- 回復で held→running にしたときの run の version
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
 FOREIGN KEY(run_id,seq,attempt) REFERENCES pipeline_steps(run_id,seq,attempt)
);
-- no_update / no_delete trigger

CREATE TABLE pipeline_ref_journal (           -- Goal branch の作成と release
 id TEXT PRIMARY KEY NOT NULL, goal_id TEXT NOT NULL, project_id TEXT NOT NULL,
 run_id TEXT REFERENCES pipeline_runs(id),
 kind TEXT NOT NULL CHECK(kind IN ('goal_create','task_release')),
 ref_name TEXT NOT NULL, old_commit TEXT NOT NULL, new_commit TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('pending','done','held')),
 version INTEGER NOT NULL CHECK(version>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=16384),
 CHECK((kind='goal_create' AND run_id IS NULL) OR (kind='task_release' AND run_id IS NOT NULL))
);
CREATE UNIQUE INDEX pipeline_ref_one_create ON pipeline_ref_journal(goal_id) WHERE kind='goal_create';
CREATE UNIQUE INDEX pipeline_ref_one_release ON pipeline_ref_journal(run_id) WHERE kind='task_release';
CREATE UNIQUE INDEX pipeline_ref_one_pending ON pipeline_ref_journal(ref_name) WHERE state<>'done';
-- transition: version+1、identity は不変、pending→done|held、held→pending|done、done は終端。no_delete

CREATE TABLE pipeline_cli_qualifications (    -- Q の結果。epoch ごと
 owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
 cli TEXT NOT NULL CHECK(cli IN ('codex','claude')),
 state TEXT NOT NULL CHECK(state IN ('qualified','refused')),
 executable_sha256 TEXT, version_text TEXT,
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=16384),
 PRIMARY KEY(owner_epoch,cli)
);
-- no_update / no_delete trigger。前の epoch の行は残す（監査のため）

CREATE TABLE pipeline_goal_completions (      -- 完了証明。不変
 goal_id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL,
 goal_branch_head TEXT NOT NULL, goal_check_run TEXT NOT NULL REFERENCES pipeline_runs(id),
 runs_sha256 TEXT NOT NULL CHECK(length(runs_sha256)=64),
 completed_epoch INTEGER NOT NULL CHECK(completed_epoch>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=65536),
 FOREIGN KEY(goal_id,project_id) REFERENCES goals(id,project_id)
);
-- no_update / no_delete / no_replace trigger
```

### 2.2 Goal plan の拡張

`GoalPlan`（`runtime/goal.rs:57`）に、任意の `execution` を足す。`GoalDefinition` の canonical digest（`runtime/goal.rs:124`）は変えない。`execution` は `goal_execution_profiles.policy_sha256` で固定する。

```json
"execution": {
  "profile": "cli_pipeline",
  "max_fix_rounds": 2,
  "blocking_severity": "high",
  "task_scopes": { "<task key>": ["src/**", "tests/**"] }
}
```

`cli_pipeline` の Goal は、受入時（`accept_runtime_goal` `state/runtime/goals.rs:28` の同じトランザクション）に次を検査する。

- 全基準が `RequiredTasksVerified` である（`goals.rs:110-115` の検査に加える）。
- 各 Task に scope がある。scope の形は相対 glob、64 件以下、各 512 byte 以下で、`..` と絶対パスを含まない。
- `executor` は `codex`、`reviewers` は `["claude"]` である。これは `pipeline.clis` で検証する（`runtime/goal.rs:189-198` を経路で分ける）。
- Project に admit 済みの TestsProfile がある（`verification_profiles` の行）。
- CLI の pin の digest。

`policy_sha256` は次の 3 つを canonical な JSON にした sha256 とする: `execution`、TestsProfile の digest、CLI pin 設定の digest。

### 2.3 容量の数え方（両経路で共通）

- **Task 単位（1 Project 1 Active Task と全体上限）**: occupied CTE を 1 つの定数 `OCCUPIED_TASKS_SQL` にまとめる。既存の 3 つの源に `UNION SELECT coalesce(task_id,id),project_id FROM pipeline_runs WHERE state IN ('running','held')` を加え、`candidates.rs:49`、`claim.rs:375`、パイプラインの claim の 3 か所で同じ定数を使う。held の run も Project の slot を使い続ける（worktree と残存プロセスが不確かなため）。
- **session 単位（全体の `global_max_sessions` と provider の `max_concurrent`）**: パイプラインで session を使う step は、state が `launching`、`running`、`unknown`、`held` のものとする。`held` は死亡を確認できていない場合だけ数える（`body.process_confirmed_dead=false`）。
  - native の数え方（`state/execution/quotas.rs:190-200`、`state/execution/native_phase/quota.rs:509-516`）に、この件数を provider（`codex` / `claude`）ごとに加える。
  - パイプラインの step を起動する tx1（§3.3）では、次の 2 つの合計を上限と比べる: `quota_leases WHERE active=1` の件数（provider ごと）と、パイプラインの件数。
  - tests と goal_check は provider を使わないので、全体の session だけを数える。

## 3. 状態機械

### 3.1 run

| 遷移 | 実行する主体 | 必要な永続の根拠 |
|---|---|---|
| （なし）→ running | `PipelineScheduler` の claim tx | 次をすべて満たし、同じ tx で `pipeline_ref_journal`（初回の Task なら goal_create）と run 行を作る。<br>・runtime_epoch が現在のもの。<br>・Goal が Running で、`goal_execution_profiles` の行がある。<br>・hard prerequisite の run がすべて completed で、その release が done。<br>・`OCCUPIED_TASKS_SQL` の容量。<br>・Project が Registered。 |
| running → running（owner_epoch を上げる） | 新しい epoch の Scheduler（採用） | §3.4 の採用条件を満たす |
| running → held | Scheduler / StepCustody / 復旧 | `body.held` に理由、step の参照、根拠（discovery の要約・検査の結果）を書き、監査イベント `rrx.private.pipeline.run_held` を残す |
| held → running | `rrx task recover --abandon-step` | `pipeline_recoveries` の行（`consumed_version` = 新しい version）。trigger が検査する |
| running → completed | release の settle | `pipeline_ref_journal` の task_release が done で、ref==new を観測していること |
| running/held → abandoned | Goal cancel | 全 step が終端（adopted / rejected / interrupted / abandoned）で、死亡を確認済みであること。確認できない step があれば held のまま |

### 3.2 step

| 遷移 | 主体 | 必要な根拠 |
|---|---|---|
| planned → launching | StepLauncher の tx1 | 次をすべて確認したうえで、cookie（UUIDv4 を英数字にしたもの）、`launch_epoch`、attempt ディレクトリ、argv・env の digest を書く。<br>・run.state=running。<br>・run.owner_epoch = runtime_epoch。<br>・run.version と step.version が一致。<br>・Goal が Running（Paused ではない）。<br>・session の容量。<br>・その epoch で該当 CLI が qualified。 |
| planned → abandoned | Goal cancel | 起動していない |
| launching → running | StepLauncher の tx2 | spawn が Ok を返したこと。pid、pgid（=pid）、birth（`rrx-process-tracker` の identity による開始時刻）、同じ epoch と version |
| launching → unknown | StepLauncher / 復旧 | spawn が Err を返した（子が残りうる。`rrx-spawn-hygiene/src/lib.rs:5-6`）か、再起動時に launching を見つけた |
| launching → rejected | StepLauncher | spawn を呼ぶ前の失敗（attempt ディレクトリの作成など）。spawn を呼んでいないことを同じプロセスの中で確実に知っている場合だけ |
| launching → exited | StepLauncher | tx2 が失敗したあと、group を kill して死亡を確認した場合（`body.exit=killed_after_record_failure`）。確認できなければ launching のまま残る（再起動時に unknown になる） |
| running → exited | StepCustody | §3.5 の死亡確認がすべて成功した。終了の状態、出力の sha256、上限を超えたかどうか |
| running → interrupted | 正常な停止 / Goal pause の kill | kill して死亡を確認した |
| running → held | StepCustody / 復旧 | 死亡を確認できない（group が空にならない、discovery が limited、cookie を持つプロセスが見つかる）。または古い epoch の running（custody の喪失） |
| running → unknown | StepCustody | 監視タスク自体の異常（panic・join の失敗）で観測が途切れた |
| exited → adopted | AdoptionChecker / VerdictDecoder | kind ごとの採用条件（§5.3〜§6）をすべて満たした。Executor は `result_commit` を書く |
| exited → rejected | 同上 | 非 0 終了、timeout、出力の上限超過、schema 違反、scope 違反など、確定した不合格 |
| exited → held | 同上 | 秘密スキャンへの該当、snapshot の変化、model の不一致、`.git` の変化など、運用者の判断が必要なもの |
| unknown/held/rejected → abandoned | `rrx task recover --abandon-step` | `pipeline_recoveries` の行（trigger が検査する） |

`unknown` から別の状態へ移るのは、運用者の記録による `abandoned` だけ。Unknown の step は、どの経路からも再起動しない。新しい attempt は、`interrupted` か `abandoned` の後にだけ、worktree を `step_base` に reset して検証してから `attempt+1` の planned として作る。

### 3.3 起動の手順

1. tx1（`BEGIN IMMEDIATE`、短く）: §3.2 の planned → launching の条件を CAS で確認して書き、commit する。
2. Store lock を解放する。Store lock は `std::sync::Mutex`。lock は control 側で `self.owner.store.lock()` として取っている（`runtime/control.rs:657-662`）。spawn の前に、必ず guard を drop する。
3. attempt ディレクトリと TMPDIR を作り、実行ファイルの sha256 を、Q の記録と照合し直す。不一致なら launching → rejected にする（spawn していない）。
4. `tokio::process::Command` を組み立てる。`env_clear()`、許可した環境変数だけ、`process_group(0)`、`kill_on_drop(true)`、stdin/stdout/stderr は pipe。`rrx_spawn_hygiene::spawn_tokio`（`lib.rs:58`）で起動する。
5. Err の場合: tx で launching → unknown と run → held（`launch_unknown`）を書き、wake を通知する。
6. Ok の場合: `ProcessGroup::new`（`adapter.rs:482`）で包み、stdin に prompt を書いて閉じる。tx2 で launching → running（pid、birth）を書く。
7. tx2 が失敗した場合: `cleanup_group` → `reap` → group が空になるのを待つ → discovery。確認できたら launching → exited を試みる。書けなければ何もしない（再起動時に unknown になる）。
8. 監視: `observe_exit`、kind ごとの wall-clock 上限（implement / fix 3600 秒、review 1800 秒、confirm 1200 秒。tests はプロファイルの値に従う）、停止要求、出力の上限、の 4 つを select する。

### 3.4 採用の条件（別の epoch が残した run を引き継ぐとき）

新しい epoch E の Scheduler は、次をすべて満たす run だけ `owner_epoch=E` にする（`state` は変えない。trigger の `NEW.owner_epoch=(SELECT epoch FROM runtime_epoch)` で検査する）。

- run.state が running で、owner_epoch < E。
- run のどの step も launching / running / unknown ではない。
- Task worktree の HEAD が、最新の adopted step の `result_commit`（または run の base）と一致する。
- `git status --porcelain=v1 -z --untracked-files=all` が空である。
- Git 設定の digest が run の記録と一致する。

満たさない場合は held にする。

### 3.5 死亡確認（step が終わったとき）

次をすべて満たしたときだけ「死亡を確認した」とする。

1. `observe_exit` が leader の終了を観測する（`waitid NOWAIT`、`adapter.rs:503-519`）。
2. leader が zombie のうちに `cleanup_group` で group に SIGKILL を送る（`adapter.rs:1589`。macOS の EPERM は `ps` で全員が zombie と確かめる。`adapter.rs:588-598`）。
3. `reap` する（`adapter.rs:562`）。
4. 2 秒以内に `test_kill_process_group` が ESRCH を返し、group が空であることを観測する（`git.rs:648-662` と同じ方式）。
5. `rrx_process_tracker::discover(cookie)` の `coverage.limited()` が false で、一致するプロセスが 0 件である（`lib.rs:100,152`）。

どれかが失敗したら「確認できない」とし、held にする。macOS では、見つかったプロセスに signal を送らない（`rrx-process-tracker/src/macos.rs:12-23`）。

## 4. 復旧表

rrxd が起動すると、`begin_execution_epoch`（`state/execution.rs:537-599`）で epoch E を得る。パイプラインの照合は、Q（§5.2）の後、最初の sweep の前に 1 回行う。その後は各 sweep で、変化した行だけを見る。

| 永続の状態（epoch < E） | 処理 | 結果 |
|---|---|---|
| step planned | run を採用（§3.4）してから再開する | 新しい epoch で launching へ進む |
| step launching | 再実行しない | step を unknown、run を held（`launch_unknown`）にする |
| step running | custody を失ったとして、cookie で探索する | 見つかった → held（`alive_orphan`、pid を記録）。見つからず coverage が完全 → held（`custody_lost`）。limited → held（`custody_unobservable`） |
| step exited | 採用の検査をやり直す。Executor の場合、HEAD==step_base なら検査と commit をやり直す。HEAD の親が step_base で、trailer の step ID が一致するなら、その commit を検査し直して採用する。それ以外は held | adopted / rejected / held |
| step interrupted | worktree を step_base に reset して検証し、attempt+1 を作る | 新しい planned |
| step unknown / held / rejected | 何もしない | run は held のまま（運用者を待つ） |
| step adopted（最後のもの） | 次の step を計画する（§6） | planned / release |
| ref journal pending | ref の値を読む: ==new なら done、==old なら `update-ref <new> <old>` をやり直す、それ以外は held | done / pending / held |
| ref journal held | 毎 sweep、ref を読み直す。運用者が ref を old か new に戻したら、pending か done に戻す | |
| run held | 何もしない | 運用者を待つ |
| run completed / abandoned | 何もしない | |
| `pipeline_cli_qualifications` に E の行がない | Q を実行する | qualified でなければパイプラインは起動しない（native は影響を受けない） |

**正常な停止**（`Runtime::shutdown`、`runtime/service.rs:168-200`）: 新しい起動を止め、running の step を kill して死亡を確認し、interrupted にする。確認できない step は running のまま残す（再起動時に held になる）。`ShutdownPending` に `PipelineSteps` を足す（`runtime/stop.rs:7-34`）。

**Goal の pause**: 新しい step を起動しない。running の step は kill して死亡を確認し、interrupted にする。resume の後は attempt+1 で再開する。`set_runtime_goal_lifecycle`（`goals.rs:361-516`）の resume の拒否（`preparation_history`、`goals.rs:442`）は native の表だけを数えるので、パイプラインの Goal には当たらない。ただし policy digest の一致（`goals.rs:431-439`）は維持する。

**Goal の cancel**: kill して死亡を確認できたら、step を interrupted、run を abandoned にする。確認できなければ held にする。

**放棄の command**: `rrx task recover <task> --abandon-step --run <id> --seq <n> --attempt <k> --expected-version <v> --reason <text>`

- 新しい `ControlAction::TaskRecover` を足す（`runtime/control.rs:24-139`）。運用者は `HumanIngress` の OS UID で識別する（`control.rs:619`）。冪等にするため `runtime_control_acks` を使う。
- daemon の処理は次の順に行う。
  1. run が held で、対象の step が unknown / held / rejected であることを確かめる。
  2. cookie で探索し直す。1 件でも見つかれば、pid を示して拒否する（運用者に止めてもらう）。limited の場合も拒否する。
  3. worktree を step_base に reset する（`reset --hard`、`clean -ffdx`）。HEAD・status・Git 設定の digest を検証する。
  4. 1 つの tx で次を書く:
     - `pipeline_recoveries`（UID、request、reason、discovery の要約、reset の結果）
     - step を abandoned に
     - attempt+1 の planned
     - run を held → running に
     - 監査イベント `rrx.private.pipeline.step_abandoned`
     - ack
- 古い attempt は再実行しない。cookie も再利用しない。

## 5. CLI の契約

### 5.1 設定（runtime の設定だけ。Project の overlay では変えられない。`config.rs:235`）

```toml
[pipeline]
work_root = "/Users/<user>/rrx-pipeline"   # 絶対パス。[A-Za-z0-9_/-] だけ、48 byte 以下、0700、Project の root と state の外
path_prefix = ["/opt/homebrew/bin"]         # node などの絶対ディレクトリ。world-writable は拒否
[pipeline.clis.codex]
executable = "/opt/homebrew/bin/codex"
version = "<pinned version text>"
model = "<model>"
effort = "high"
codex_home = "/Users/<user>/.rrx/codex-home"  # 専用。運用者がここで login する
max_concurrent = 2
[pipeline.clis.claude]
executable = "/Users/<user>/.local/bin/claude"
version = "<pinned version text>"
model = "<model>"
max_concurrent = 2
```

`Config`（`config.rs:226`）に `pipeline: Option<PipelineConfig>` を足し、`#[serde(skip_serializing_if = "Option::is_none")]` を付ける。こうすると、`pipeline` を設定していない環境の policy digest（`runtime/control.rs:540-545`）が変わらない。

### 5.2 起動時の適格性確認（Q）

CLI ごとに、次の手順を行う。各コマンドは環境を空にしてから組み立て（§5.4）、timeout 5 秒、出力 4 KiB までとする。結果は `pipeline_cli_qualifications` に書く。

1. `executable` の realpath と sha256 を記録する。
2. `--version` の出力を trim したものが `version` と完全に一致する。
3. 次の 2 つの出力に、§5.3 の必須 flag がすべてある: codex は `exec --help`、claude は `--help`。
4. ログインの確認。
   - codex: `CODEX_HOME` を指定して `codex login status` の終了コードが 0。さらに `codex_home/config.toml` を TOML として読み、許可したキーだけであること（`mcp_servers`・`notify`・`hooks`・`profiles` は拒否）。
   - claude: `claude auth status` の終了コードが 0。pin した版にこのサブコマンドがなければ、その版は pin の対象にしない。
5. macOS 以外の OS では refused にする。

起動のたびに Q をやり直す。step を起動する直前には sha256 を照合し直す（§3.3-3）。

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

- prompt は stdin で渡す。cwd は `<W>`。
- `--setting-sources ""` を pin した版が受け付けない場合、Q を不合格にする（B3 で実機確認する。§12 D9）。

### 5.4 環境変数

`env_clear()` の後に、次だけを入れる。

| 変数 | 値 |
|---|---|
| `HOME` / `USER` / `LOGNAME` | 運用者のもの |
| `PATH` | `path_prefix` + `/usr/bin:/bin:/usr/sbin:/sbin` |
| `LANG` | `en_US.UTF-8` |
| `TMPDIR` | `<work_root>/t/<attempt の短い ID>`。60 byte 以下。AF_UNIX のパス長の上限 104 byte への配慮 |
| `TERM` / `NO_COLOR` | `dumb` / `1` |
| `RRX_PROCESS_COOKIE` | step の cookie（`rrx-process-tracker/src/lib.rs:22`） |
| `CODEX_HOME` | codex のときだけ |

GH_TOKEN・GITHUB_TOKEN・SSH_AUTH_SOCK・`AWS_*`・`ANTHROPIC_*`・`OPENAI_*`・`GIT_*` は入れない。テストでは「実際に渡した集合 ⊆ 許可した集合」を検査する。

### 5.5 出力の上限とファイル

attempt ディレクトリは `<work_root>/runs/<run>/<seq>-<attempt>/`（0700）に置く。完了後、ファイルを 0400 にする。

| ファイル | 上限 | 超えたとき |
|---|---|---|
| `prompt.md`（sha256 = `input_sha256`） | 512 KiB | 起動しない |
| `stdout.jsonl`（codex `--json`）/ `result.json`（claude） | 16 MiB / 1 MiB | group を kill し、rejected（`output_overflow`） |
| `stderr.log` | 1 MiB | 末尾を残し、`truncated` を記録する（不合格にはしない） |
| `last.md` / `last.json` | 256 KiB | rejected |
| `diff.patch`（rrx が生成） | 8 MiB | held（`diff_too_large`） |
| tests の evidence | プロファイルの `stdout_bytes` / `stderr_bytes`（`plan.rs:35`） | collector の `OutputOverflow` |

### 5.6 verdict の JSON schema

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
- `approve` なのに blocking 指摘を含むものは、矛盾として `request_changes` 扱いにする。

**Confirmer**（`rrx.confirm.v1`）

```json
{"schema":"rrx.confirm.v1","labels":[{"id":"F1","label":"confirmed|rejected|unclear",
  "evidence":{"file":"…","line":1,"quote":"≤512 文字"},"reason":"≤2000 文字"}]}
```

ふるい分けを通った各 ID に、ラベルがちょうど 1 つずつ付いていることを要求する。Confirmed の場合、evidence の `quote` が snapshot の該当行の前後 3 行の部分文字列でなければ、Unclear に落とす。

## 6. 確認と修正のループ

1. **implement → tests**。テストが不合格なら、fix（入力はテスト失敗の要約。rrx が観測したものなので Confirmed として扱う）を経て tests へ戻る。
2. **tests 合格 → review**。
3. **規則によるふるい分け**。次をすべて満たす指摘だけを残し、それ以外は `invalid` として記録する。
   - `file` が正規化した相対パスで、レビューした tree にある。
   - `line` がそのファイルの行数以下。
   - ファイルが base..head の変更集合にあるか、変更されたファイルが import しているファイルとして rrx が検出できる。
   - Task の scope 内にある。
   - 重大度が `blocking_severity` 以上、または `blocking=true`。
4. **confirm**（ふるい分けを通った blocking 指摘が 1 件以上あるときだけ、別の snapshot で行う）。
5. **判定**。
   - テスト合格、`approve`、Confirmed の blocking が 0、Unclear の blocking が 0 → 承認して release へ。
   - Unclear の blocking が 1 件以上 → held（`unclear_finding`）。
   - Confirmed の blocking が 1 件以上 → fix へ（入力は Confirmed の ID と内容だけ）。
   - `request_changes` で、blocking がすべて Rejected / invalid → 修正せずに再レビュー（1 ラウンドを使う）。
6. **ラウンド**。fix と再レビューのたびに `run.round` を 1 増やす（trigger で +1 だけを許す）。`round = max_fix_rounds` で承認に至らなければ held（`round_limit`）。fix の結果も、§5.3 の採用検査と tests・review をすべて通す。

medium と low の指摘は記録して報告するが、v1 では修正しない。

### 5.3 の補足: Executor の採用検査（AdoptionChecker）

検査は、死亡を確認した後、Task worktree で `PipelineGit` が行う。

- HEAD == step_base（Agent が commit していない）。`.git` ファイルの内容、common dir の `config`・`hooks/`・`info/`・その branch の ref、`git config --list --show-origin` の digest が、step の前と同じ。
- 変更した各パスが Task scope の glob に一致する。固定の拒否リストに当たるパスは、scope が明示していない限り拒否する: `.git*`、`.gitattributes`、`.gitmodules`、`.claude/**`、`.codex/**`、`AGENTS.md`、`CLAUDE.md`、`**/CLAUDE.md`。
- mode は 100644 か 100755 だけ。symlink（120000）と gitlink（160000）は拒否する。
- 変更は 200 ファイル以下、追加は合計 8 MiB 以下、1 ファイル 1 MiB 以下。バイナリは拒否する。
- 秘密スキャン（固定の pattern: 秘密鍵の header、AWS のアクセスキー、GitHub の token、`xox[bp]-`、汎用の `*_SECRET=` など）を、追加行・`last.md`・stderr に当てる。該当したら held。
- 変更が 0 件なら rejected（`no_change`）。

合格したら、`git add -- <検査したパス>`、`git -c user.name=rrx -c user.email=rrx@localhost commit` を行う。trailer は `Rrx-Run:`、`Rrx-Step: <seq>-<attempt>`。作った commit を `result_commit` に書いて adopted にする。

## 7. release と Goal の完了

- **Goal branch**: 最初の Task の claim tx で、`goal_create` の journal（`refs/heads/rrx/goal/<goal-id>`、old=0、new=Project の `base_branch` の head）を作る。`git update-ref <ref> <new> 0000…` で作成する。
- **Task branch**: `rrx/pipeline/<task-id>`。worktree は `git worktree add -b <branch> <work_root>/wt/<run> <Goal head>` で作り、run.base_commit は Goal の head とする。
- **release**: 承認されたら、1 つの tx で journal（old=run.base_commit、new=run.head_commit）を pending で書く。続いて `git update-ref refs/heads/rrx/goal/<id> <new> <old>` を実行し、結果の ref を読む。==new なら done にして、run を completed にする。
  - Project の slot は 1 つなので、Goal branch が release 以外で動くことはない。そのため常に fast-forward になる。
  - ref が old でも new でもなければ held にする。
  - Goal branch がどこかの worktree で checkout されていれば（`git worktree list --porcelain`）、held にする。
- **次の Task**: run が completed になったら `wake.notify_one()` を呼び、同じ sweep の中で次の READY な Task を claim する。ユーザーのプロンプトは要らない。
- **Goal の完了**: DAG の全 Task の run が completed（release は done）になったら、kind=goal の run を作り、Goal branch の head で goal_check（TestsProfile）を実行する。合格したら、1 つの tx で次を書く:
  - `pipeline_goal_completions` の行（全 run の ID と head を canonical にした sha256）
  - Goal の state を Completed に（`goals` の body の state を version CAS で更新する。定義の trigger `runtime/schema.sql:21-30` には当たらない）
  - 監査イベント

  不合格なら held。Task の run が abandoned のときは、Goal を Blocked にする。LLM の出力は完了の判定に使わない。

## 8. 脅威モデル

**守るもの**: オーナーの資格情報（`~/.aws`、`~/.ssh`、keychain、gh の token）、repo と Goal branch の正しさ、rrx の状態（SQLite・epoch）、他 Project の作業、オーナーの Mac の可用性。

**主体**: オーナー（運用者）、rrxd、Codex / Claude の CLI とその provider、repo の内容（信頼しない）、Agent が書いたコード（信頼しない）。

**信頼の境界**: rrxd ↔ Agent のプロセス、Agent ↔ provider（外部への送信）、Task worktree ↔ repo の共通 dir、テストのプロセス ↔ ホスト。

| 脅威 | 対策 | 残余リスク |
|---|---|---|
| repo の内容による prompt injection（指示の書き換え、偽の verdict の誘導） | ・信頼しない内容は nonce 付きの区切りでデータとして枠付けする。<br>・verdict の schema を固定する。<br>・指摘は provider をまたいで確認する。<br>・テストを関門にする。<br>・scope を検査する。<br>・Reviewer の approve だけでは承認しない。 | 両方の provider が同じ誘導に乗り、テストも通る変更は検出できない（受け入れる） |
| repo 側の設定がコードを実行する（Git hook、`.claude/settings.json` の hook、`.codex/config.toml`、filter driver） | ・rrx の Git は常に `core.hooksPath=/dev/null`・`core.fsmonitor=false` を付け、clone には `--template=` を付ける。<br>・Claude は `--setting-sources ""`。<br>・Codex は `trust_level="untrusted"` と専用の `CODEX_HOME`。<br>・step の前後で Git 設定の digest を比べる。<br>・指示ファイルの変更は既定で拒否する。 | Codex は `AGENTS.md` を必ず読み、Claude も `CLAUDE.md` を読みうる（指示による誘導が残る） |
| Reviewer / Confirmer による書込み | 読み取り専用の snapshot（chmod a-w）、書込みツールの拒否、前後の tree digest の照合。Reviewer と Confirmer の結果は判定の JSON だけを読む | なし（変化を検出したら held） |
| Agent が自分で commit・ref の変更・hook の設置を行う | 採用するのは rrx commit だけ。HEAD・ref・`.git` の digest を照合する | なし |
| 環境変数からの秘密の漏洩 | 空から組み立てる（§5.4）。渡した集合 ⊆ 許可した集合を、テストで検査する | なし |
| **Agent がユーザーの読めるファイルを何でも読む**（`~/.aws` などを provider に送る） | v1: 差分と成果物の秘密スキャン（出口での検出）と、オーナーの書面による受入 | **残る**。workspace-write も read-only も、ディスク全体を読める。根本の対策は後続（#98）: (A) 専用の macOS ユーザーで Agent を動かす。(B) Codex の管理設定 `requirements.toml` の `[permissions.filesystem] deny_read`（0.162.1 で使えるかは未確認）。(C) Claude の `--settings` の `permissions.deny` で Read を制限する（部分的）。 |
| テストが、Agent の書いたコードをオーナーの権限で実行する | clean checkout で実行する。`/usr/bin/sandbox-exec` の Seatbelt profile（network 拒否、書込みは checkout・TMPDIR・運用者が宣言したキャッシュディレクトリだけ）で包む。空から組み立てた環境 | 読み取りは制限されない。secret を読んで stdout に出す経路は秘密スキャンで部分的に検出する（受け入れる） |
| network | Codex のツールは network を拒否する。Claude には Bash も web もない。テストは Seatbelt で拒否する | CLI 自身が provider と通信する（仕様どおり） |
| 信頼しない verdict JSON | strict decode と上限、未知フィールドの拒否、存在しない file / line は invalid、quote の照合 | なし |
| 古い owner・二重の rrxd | `owner.lock` の flock（`execution/owner.rs:225`）、全 CAS での epoch 照合、run の epoch trigger | なし |
| 起動結果が不明・孤児プロセス | Unknown / Held、cookie による探索、再実行しない | cookie を消した孤児と、group を抜けた子孫は検出できない（記録して受け入れる） |
| Goal branch の破損 | ref-CAS の journal、fast-forward だけ、checkout 中なら拒否 | 運用者が手で ref を動かした場合は held になる |

## 9. 影響範囲分析

### 9.1 変更・追加の対象

| 種別 | 対象 | 影響を受ける箇所 | 影響の種類 | 対応 |
|---|---|---|---|---|
| 定数 | `SCHEMA_VERSION` 11→12 | `state/mod.rs:32,189-193,271-293`、`state/execution.rs:129-143`（writer guard の版の literal）、`state/managed_binding/schema.rs:249-257`、`state/read_index_tests.rs`、`sqlite_build_tests.rs` | 互換性（v12 の DB は旧バイナリが拒否する。仕様どおり） | 版上げの手順を足す。catalog のテストを更新する |
| DB | 新しい表 6 つと trigger | `MUTABLE_TABLES`（`state/execution.rs:7-53`） | 機能 | 追加する |
| SQL | occupied CTE | `state/runtime/driver/candidates.rs:49`、`claim.rs:375` | 機能（容量の合算） | 1 つの定数にして、3 か所で使う |
| SQL | session の数え方 | `state/execution/quotas.rs:190-200`、`state/execution/native_phase/quota.rs:509-516` | 機能 | パイプラインの件数を加える |
| SQL | 実行プロファイルによる除外 | `candidates.rs:49`、`claim.rs:365-369` | セキュリティ（経路の分離） | `NOT EXISTS goal_execution_profiles` |
| 関数 | `reconcile_runtime_attention` | `state/runtime/service.rs:25,45-48,70` | 表示 | `cli_pipeline` の Task を除く |
| 関数 | `accept_runtime_goal` | `state/runtime/goals.rs:28-207` | 機能 | プロファイルの行を挿入し、検査を加える |
| 関数 | `set_runtime_goal_lifecycle` | `goals.rs:361-516` | 機能 | pause / cancel をパイプラインへ通知する（行の更新は、パイプラインの tx が行う） |
| 型 | `GoalPlan.execution`（任意） | `runtime/goal.rs:57,166-235`、`runtime/control.rs:29-33,638` | 互換性（省略したときは従来どおり） | `#[serde(default)]` |
| 型 | `Config.pipeline`（任意） | `config.rs:226,376-428`、`runtime/control.rs:540-545`（policy digest） | 互換性 | 未設定なら digest が変わらないことをテストで確かめる |
| API | `ControlAction::TaskRecover` / `ProjectAdmitTests` | `runtime/control.rs:24-139,761-766`、`main.rs:142,487` | 機能・認可（OS UID） | 追加する。既存の `TaskRetry` は Unavailable のまま |
| 関数 | `ManagedVerifier::admit_tests` | `verification/mod.rs:226-247`、`state/execution/verification.rs:638,669-677` | 機能（初めて本番から呼ぶ） | `ProjectAdmitTests` から呼ぶ。1 Project に 1 プロファイルで不変（`verification.sql:1-10`） |
| loop | sweep | `runtime/service.rs:76-161`（`admit_ready_tasks` `:130` の後に `pipeline_tick`）、`:168-200`、`runtime/stop.rs:7-34` | 機能・停止 | tick と停止の site を加える |
| 環境変数 | `RRX_PROCESS_COOKIE` | 既存の消費者は `execution/resources.rs:75`、`execution/cleanup.rs:85` | なし（値の空間は UUID で別） | パイプラインの値も UUID から作る |
| 環境変数 | `CODEX_HOME`・`TMPDIR`・`PATH` | 子プロセスの中だけ（rrxd 自身の環境は変えない） | パスの長さ（TMPDIR は 60 byte 以下） | 起動時に検査する |
| 入力の一覧 | 必須 flag・許可する環境変数・固定の拒否パス・秘密の pattern | パイプラインだけ | 安全 | 広がる向きを検出するテストにする |
| 定数 | `global_max_sessions` の既定 12（`config.rs:64`）、`MVP_PROJECT_TASKS`=1（`config.rs:17`）、`PHASE_SLOTS_PER_PROJECT`=4（`runtime/phase_supervisor.rs:22`） | 変えない | | 両経路の合算だけを加える |

### 9.2 影響なしの根拠

- **native の `start_phase` 経路**: `cli_pipeline` の Goal を candidate と claim の段階で除くだけで、その他は変えない。既存の Goal には `goal_execution_profiles` の行がない。
- **`GenericCliAdapter` と `ProcessGroup`**: 変えない（§12 D5）。`cleanup_group`・`observe_exit`・`reap` は既存の `pub(crate)` をそのまま使う。
- **Docker・Grok・`ResultStore`・`AttemptManager`**: パイプラインから呼ばない。
- **M0 branch**: 今は影響しない。M0 を再開するときは、M0 の schema を v13 へ番号を振り直す（§12 D1）。

### 9.3 回帰の確認（各 unit の最後に行う）

`cargo test --workspace` の全件、`tests/runtime_cli.rs`・`tests/daemon.rs`・`tests/goal_cli.rs`・`tests/state.rs` の明示実行、v11 の DB を v12 に版上げするテスト、`managed_native` の Goal の既存のテストがすべて通ること。

## 10. テスト方針

- **偽の CLI**: `tests/support/fake_codex.py` と `fake_claude.py`（既存の `tests/support/grok_fake.py` と同じ流儀）。
  - 環境は空から組み立てるので、シナリオは同じディレクトリの `scenario.json` から読む。テストごとに別のディレクトリへ置く。
  - `--version`、`--help`、`login status` / `auth status`、`exec` / `-p` を実装する。
  - 書くもの、出力、終了コード、遅延、子プロセスの fork（cookie を持ったままのもの・消したもの）、フィールドの欠落・重複を、シナリオで指定する。
- **障害注入マトリクス（AC-8）**:

| ID | 注入 | 期待する結果 |
|---|---|---|
| F-a1 | Executor が非 0 で終了する | rejected、run held、他 Project は進む |
| F-a2 | 出力の上限を超える / timeout | group を kill し、死亡を確認して rejected |
| F-a3 | scope 外を変更する / symlink / 秘密の pattern / 自分で commit する | rejected または held。commit を採用しない |
| F-b1 | `request_changes`、指摘が Confirmed | fix に入り、round が +1 |
| F-b2 | 上限を超える / Unclear の blocking / 全指摘が Rejected | held / held / 修正せずに再レビュー |
| F-b3 | verdict の未知フィールド・重複キー・存在しない行・model の不一致・snapshot の変化 | rejected または held |
| F-c1 | spawn が Err を返す（存在しない実行ファイルではなく、hook で注入する） | unknown、run held、再実行しない |
| F-c2 | tx1 の後、spawn の前に crash する | 再起動時に unknown・held |
| F-c3 | tx2 が失敗する | group を kill し、確認できれば exited、できなければ launching のまま→unknown |
| F-d1 | running 中に正常停止する | interrupted、attempt+1 で再開する |
| F-d2 | running 中に SIGKILL で rrxd を止める | 再起動時に custody を失ったとして held。cookie の孤児がいれば pid を記録する |
| F-d3 | release の `update-ref` の前後で crash する | ref==old ならやり直し、ref==new なら done、それ以外は held |
| F-e1 | 古い epoch の tx1 / tx2 / run の更新 | CAS か trigger で拒否する。spawn しない |
| F-f1 | Scheduler と `task recover` が同じ step を同時に更新する | 片方だけが成功し、負けた側は spawn しない |
| F-g1 | Project A を held にする | Project B の run が完了まで進む。容量の合計は正しい |
| F-h1 | `abandon-step` を実行したとき、cookie を持つプロセスが残っている | 拒否する |

- **mutation の対象**（壊したら落ちることを確かめる）。
  - 配線:
    - occupied CTE のパイプラインの UNION を消す
    - native の除外を消す
    - tx1 の epoch 照合を消す
    - spawn の前に Store lock を解放しないようにする
    - `pipeline_tick` を呼ばないようにする
  - 部分一致と存在の検査:
    - scope の glob を前方一致にする
    - quote の照合を緩める
    - `--help` の flag 検査を部分文字列にする
    - 許可する環境変数を広げる
  - 「この検査が保証する」と書いた箇所:
    - step / run の trigger の各条件
    - 死亡確認の 5 条件の一つずつ
    - `unknown` から `abandoned` への EXISTS
    - ref journal の old / new の判定

## 11. Unit の分割

| Unit | 内容 | 進める AC | 更新するマスター設計書 |
|---|---|---|---|
| B1 | 基準の決定（§0）、main の CI を `gh` で確認、偽 CLI の基盤 | 前提 | なし（`doc/verification/issue-106-b1.md`） |
| B2 | schema v12 と Store API（spawn しない）、両経路の除外、容量の合算 | AC-2、AC-7、AC-8(e)(f) | `architecture.md` §13、`goal-runtime.md` §14、`multi-project-runtime.md` §7 |
| B3 | `PipelineConfig`、CLI の pin、Q | AC-1 | `agent-adapter.md` §10、`agent-execution.md` の「Process launch boundary」 |
| B4 | StepLauncher と StepCustody | AC-8(c)(d) | `agent-execution.md` §2・§4 |
| B5 | Executor と AdoptionChecker、`PipelineGit` | AC-3 | `agent-execution.md`、`architecture.md` §10・§14 |
| B6 | `PipelineVerificationPort`、Seatbelt、`ProjectAdmitTests` | AC-3 | `workflow-engine.md` §17、`agent-execution.md` §5 |
| B7 | Reviewer と VerdictDecoder、snapshot | AC-4 | `workflow-engine.md` §6、`agent-adapter.md` §7 |
| B8 | ふるい分け、Confirmer、fix のループ | AC-5 | `workflow-engine.md` §6・§7 |
| B9 | `pipeline_tick`、release の journal、次の Task、Goal の完了 | AC-6 | `goal-runtime.md` §6・§7、`multi-project-runtime.md` §7 |
| B10 | 復旧表、正常停止、pause / cancel、`task recover` | AC-7、AC-8(c)(d) | `goal-runtime.md` §14、`architecture.md` §9 |
| B11 | 障害注入マトリクスの完了、オーナーが実 Mac で実行するチェックリストの script、受入の記録 | AC-1〜AC-8 | `doc/verification/issue-106-acceptance.md` |

## 12. 承認済みの計画との相違

| ID | 計画 | この設計 | 根拠 |
|---|---|---|---|
| D1 | schema v13 | v12 | main は v11（`state/mod.rs:32`）。v12 は M0 branch（`feature/issue-5-m0:…/state/mod.rs:34`）にしかない。M0 を再開するときは v13 へ番号を振り直す |
| D2 | `goals.execution_profile` 列 | 不変の表 `goal_execution_profiles`（行がなければ `managed_native`） | 既存の `goals` 表を変えると、全 catalog の byte 比較（`managed_binding/schema.rs:249-257`）と、定義の trigger（`runtime/schema.sql:21-30`）に波及する。`goal_authority` と同じ形の不変の行のほうが、既存の Goal を変えずに済む |
| D3 | Q の記録先は `runtime_startup_probes`（kind cli） | `pipeline_cli_qualifications` | `runtime_startup_probes` は M0 にしかない（`startup.sql:1`）。その `step` の CHECK も native 用（version / schema / handshake） |
| D4 | テストは `verification_runs` に記録する | `verification_profiles` と `collector::collect` は再利用する。記録は `pipeline_steps` に残す | `verification_runs` は、`execution_units`・`workflow_verification_contracts`・`managed_effects` への FK を要求する（`verification.sql:26,36,52`）。偽の unit を作ると、native の epoch fence（`state/execution.rs:557-584`）に巻き込まれる |
| D5 | `ProcessGroup` を共有の module へ移す | 移さずに、`StepCustody` で包む | `ProcessGroup` は native の Codex（`codex/transport.rs:23,125`、`codex/preparation.rs:295`）と Grok が使っている custody（#19）で、別の担当の範囲に触れる。必要な `observe_exit`・`cleanup_group`・`reap` はすでに `pub(crate)` |
| D6 | 死亡の確認 = group の死亡と cookie の探索 | group が空になることを観測する手順を、条件として明示する | 既存の `ProcessGroup` は、SIGKILL の配送と leader の reap までしか確かめない（`adapter.rs:520-568`）。group が空になるまで待つのは `git.rs:653-662` だけ |
| D7 | strict_json で未知フィールドを拒否する | strict_json（上限と重複キー）＋ `deny_unknown_fields` の DTO | `strict_json.rs:65-66` に、型の検査は後で行うと明記されている |
| D8 | step の状態: planned…held | `interrupted` と `abandoned` を明示する。`unknown` からは `abandoned` だけ（held へは移さない） | B10 の正常停止と放棄の command を表すため。unknown を held へ移せる経路を作らないため |
| D9 | Claude で「project / user の setting sources を使わない」 | `--setting-sources ""` を Q で確かめる。受け付けない版は pin しない | 空の指定を受け付けるかは実機で未確認（この文書の作成中、`codex` / `claude` の `--help` の実行は hook で拒否された） |
| D10 | 計画にないもの | テストを Seatbelt で包む。`ProjectAdmitTests` の command | `admit_tests` は本番から呼ぶ経路がない（`verification/mod.rs:226`。呼ぶのはテストだけ）。テストは Agent のコードを、sandbox なしで実行する（§8）。承認事項 7 に含めて判断を求める |
| D11 | Goal の完了 = 型付きの条件 | 新しい evaluator を足さない。`cli_pipeline` では `RequiredTasksVerified` だけを受け付け、Goal branch の goal_check を完了の固定条件にする | evaluator を足すと、canonical digest（`runtime/goal.rs:149-157`）と既存の Goal に波及する |
| D12 | `ProcessGroup` の macOS での「孤児は Held のまま」 | 計画どおり。加えて、見つかった pid に signal を送らない | `rrx-process-tracker/src/macos.rs:12-23` は、探索で見つけたプロセスへの signal を無効にしている |
