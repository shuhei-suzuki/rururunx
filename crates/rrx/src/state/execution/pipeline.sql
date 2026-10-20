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
WHEN OLD.rowid<>NEW.rowid OR OLD.id<>NEW.id OR OLD.kind<>NEW.kind OR OLD.task_id IS NOT NEW.task_id OR OLD.goal_id<>NEW.goal_id
 OR OLD.project_id<>NEW.project_id OR OLD.base_commit<>NEW.base_commit
 OR OLD.worktree IS NOT NEW.worktree OR OLD.branch IS NOT NEW.branch
 OR NEW.version<>OLD.version+1
 OR NEW.owner_epoch<OLD.owner_epoch
 -- 引継ぎ（epoch を上げる更新）は owner だけを変える
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.round<>OLD.round OR NEW.head_commit<>OLD.head_commit
     OR json_remove(NEW.body,'$.version','$.epoch') IS NOT json_remove(OLD.body,'$.version','$.epoch')))
 OR (NEW.head_commit<>OLD.head_commit AND NOT EXISTS(SELECT 1 FROM pipeline_steps s
     WHERE s.run_id=NEW.id AND s.kind IN ('implement','fix') AND s.state='adopted' AND s.result_commit=NEW.head_commit))
 OR NEW.round<OLD.round OR NEW.round>OLD.round+1
 OR NOT (NEW.state=OLD.state
   OR (OLD.state='running' AND NEW.state IN ('held','completed','abandoned'))
   OR (OLD.state='held' AND NEW.state IN ('running','abandoned','completed')))
 -- held から running・completed へは、回復の記録があるときだけ（completed は release 済みの journal を決着させた abandon-run。§4.4）
 OR (OLD.state='held' AND NEW.state IN ('running','completed') AND NOT EXISTS(
     SELECT 1 FROM pipeline_recoveries r WHERE r.run_id=NEW.id AND r.consumed_version=NEW.version
      AND r.then_action=CASE NEW.state WHEN 'running' THEN 'retry' ELSE 'abandon_run' END))
 OR (OLD.state='held' AND NEW.state='abandoned'
     AND (SELECT json_extract(body,'$.state') FROM goals WHERE id=NEW.goal_id AND project_id=NEW.project_id) IS NOT 'CANCELLED'
     AND NOT EXISTS(SELECT 1 FROM pipeline_recoveries r
     WHERE r.run_id=NEW.id AND r.consumed_version=NEW.version AND r.then_action='abandon_run'))
 -- 放棄と held からの完了は、生きているかもしれない step が 1 つもないときだけ
 OR (((NEW.state='abandoned' AND OLD.state<>'abandoned') OR (OLD.state='held' AND NEW.state='completed')) AND EXISTS(
     SELECT 1 FROM pipeline_steps s WHERE s.run_id=NEW.id
      AND (s.state IN ('planned','launching','running','exited','committed','unknown')
       OR (s.state='held' AND s.kind NOT IN ('prepare','release') AND json_extract(s.body,'$.death_confirmed') IS NOT 1))))
 -- 放棄は、その run の開いた release の journal がないときだけ（先に決着させる。§4.4・§9）
 OR (NEW.state='abandoned' AND OLD.state<>'abandoned' AND EXISTS(
     SELECT 1 FROM pipeline_ref_journal j WHERE (j.run_id=NEW.id OR (j.goal_id=NEW.goal_id AND j.kind='goal_create')) AND j.state IN ('pending','held')))
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
 OR json_extract(NEW.body,'$.death_confirmed') IS NOT NULL
 OR (NEW.kind='test' AND (json_type(NEW.body,'$.command_id') IS NOT 'text' OR length(json_extract(NEW.body,'$.command_id'))=0))
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
WHEN OLD.rowid<>NEW.rowid OR OLD.run_id<>NEW.run_id OR OLD.seq<>NEW.seq OR OLD.attempt<>NEW.attempt OR OLD.kind<>NEW.kind
 OR OLD.provider IS NOT NEW.provider OR OLD.round<>NEW.round OR OLD.step_base<>NEW.step_base
 OR OLD.input_sha256<>NEW.input_sha256 OR NEW.version<>OLD.version+1
 OR json_extract(OLD.body,'$.command_id') IS NOT json_extract(NEW.body,'$.command_id')
 OR json_extract(OLD.body,'$.command_index') IS NOT json_extract(NEW.body,'$.command_index')
 OR (NEW.kind NOT IN ('prepare','release') AND OLD.state='planned' AND NEW.state='launching'
     AND json_type(NEW.body,'$.launch.wall') IS NOT 'text')
 OR (json_type(OLD.body,'$.launch') IS NOT NULL
     AND json_extract(OLD.body,'$.launch') IS NOT json_extract(NEW.body,'$.launch'))
 OR NEW.owner_epoch<OLD.owner_epoch
 -- 引継ぎは owner だけを変える
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.cookie IS NOT OLD.cookie
     OR NEW.pid IS NOT OLD.pid OR NEW.birth IS NOT OLD.birth
     OR NEW.result_commit IS NOT OLD.result_commit OR NEW.result_tree IS NOT OLD.result_tree
     OR json_remove(NEW.body,'$.version','$.epoch') IS NOT json_remove(OLD.body,'$.version','$.epoch')))
 -- Death facts change only on never-launched, observed-dead or audited recovery transitions.
 OR (json_extract(OLD.body,'$.death_confirmed') IS NOT 1 AND json_extract(NEW.body,'$.death_confirmed') IS 1
     AND NOT ((OLD.state='planned' AND NEW.state IN ('held','abandoned'))
       OR (NEW.state IN ('exited','interrupted') AND OLD.state IN ('launching','running'))
       OR (NEW.state='abandoned' AND EXISTS(SELECT 1 FROM pipeline_recoveries r
           WHERE r.run_id=NEW.run_id AND r.seq=NEW.seq AND r.attempt=NEW.attempt))))
 OR (NEW.kind NOT IN ('prepare','release') AND OLD.state IN ('launching','running') AND NEW.state IN ('exited','interrupted')
     AND json_extract(NEW.body,'$.death_confirmed') IS NOT 1)
 OR (NEW.kind NOT IN ('prepare','release')
     AND json_extract(NEW.body,'$.death_confirmed') IS 1 AND OLD.state IN ('launching','running')
     AND (json_extract(NEW.body,'$.death_evidence.leader_exit_observed') IS NOT 1
       OR json_extract(NEW.body,'$.death_evidence.launch_checked') IS NOT 1
       OR (OLD.birth IS NOT NULL AND json_extract(NEW.body,'$.death_evidence.leader_birth_matched') IS NOT 1)
       OR json_extract(NEW.body,'$.death_evidence.group_kill_delivered') IS NOT 1
       OR json_extract(NEW.body,'$.death_evidence.leader_reaped') IS NOT 1
       OR json_extract(NEW.body,'$.death_evidence.group_empty') IS NOT 1
       OR json_extract(NEW.body,'$.death_evidence.discovery_complete') IS NOT 1
       OR json_extract(NEW.body,'$.death_evidence.matching_processes') IS NOT 0))
 OR (OLD.state IN ('adopted','interrupted','limited','abandoned') AND NEW.owner_epoch=OLD.owner_epoch)
 OR (NEW.result_tree IS NOT NULL AND NEW.result_commit IS NULL)
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
      AND o.kind NOT IN ('prepare','release')
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

-- Independent maintenance receipts: never update a launch/monitor cursor version.
CREATE TABLE pipeline_test_cleanup (
 run_id TEXT NOT NULL, seq INTEGER NOT NULL, attempt INTEGER NOT NULL,
 state TEXT NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','done')),
 failures INTEGER NOT NULL DEFAULT 0 CHECK(failures BETWEEN 0 AND 32),
 not_before INTEGER NOT NULL DEFAULT 0,
 failure_class TEXT CHECK(failure_class IN ('checkout_remove','tmpdir_remove','cleanup_sweep')),
 path TEXT,
 PRIMARY KEY(run_id,seq,attempt),
 FOREIGN KEY(run_id,seq,attempt) REFERENCES pipeline_steps(run_id,seq,attempt)
);
CREATE TRIGGER pipeline_test_cleanup_insert BEFORE INSERT ON pipeline_test_cleanup
WHEN NEW.state<>'pending' OR NEW.failures<>0 OR NEW.not_before<>0 OR NEW.failure_class IS NOT NULL OR NEW.path IS NOT NULL
 OR NOT EXISTS(SELECT 1 FROM pipeline_steps s WHERE s.run_id=NEW.run_id AND s.seq=NEW.seq AND s.attempt=NEW.attempt
   AND s.kind='test' AND json_extract(s.body,'$.death_confirmed') IS 1)
BEGIN SELECT RAISE(ABORT,'invalid test cleanup obligation'); END;
CREATE TRIGGER pipeline_test_cleanup_update BEFORE UPDATE ON pipeline_test_cleanup
WHEN OLD.rowid<>NEW.rowid OR OLD.run_id<>NEW.run_id OR OLD.seq<>NEW.seq OR OLD.attempt<>NEW.attempt OR OLD.state='done'
BEGIN SELECT RAISE(ABORT,'invalid test cleanup update'); END;
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
 ref_name TEXT NOT NULL CHECK(ref_name='refs/heads/rrx/goal/' || goal_id),
 old_commit TEXT NOT NULL CHECK(length(old_commit) IN (40,64) AND old_commit NOT GLOB '*[^0-9a-f]*'),
 new_commit TEXT NOT NULL CHECK(length(new_commit) IN (40,64) AND new_commit NOT GLOB '*[^0-9a-f]*'),
 owner_epoch INTEGER NOT NULL CHECK(owner_epoch>0),
 state TEXT NOT NULL CHECK(state IN ('pending','done','held','aborted')),
 version INTEGER NOT NULL CHECK(version>0),
 body TEXT NOT NULL CHECK(json_valid(body) AND length(CAST(body AS BLOB))<=16384),
 CHECK((kind='goal_create' AND run_id IS NULL AND old_commit=hex(zeroblob(length(new_commit)/2))) OR (kind='task_release' AND run_id IS NOT NULL)),
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
WHEN OLD.rowid<>NEW.rowid OR NEW.version<>OLD.version+1 OR NEW.owner_epoch<OLD.owner_epoch
 -- 期限は短くならない
 OR NEW.not_before<OLD.not_before
 -- 引継ぎは owner だけを変える
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.not_before<>OLD.not_before
     OR NEW.backoff_ms<>OLD.backoff_ms OR NEW.probe_run IS NOT OLD.probe_run
     OR NEW.probe_seq IS NOT OLD.probe_seq OR NEW.probe_attempt IS NOT OLD.probe_attempt
     OR json_remove(NEW.body,'$.version','$.epoch') IS NOT json_remove(OLD.body,'$.version','$.epoch')))
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

CREATE TRIGGER pipeline_ref_journal_insert BEFORE INSERT ON pipeline_ref_journal
WHEN NEW.version<>1 OR NEW.state<>'pending'
BEGIN SELECT RAISE(ABORT,'invalid pipeline journal'); END;
CREATE TRIGGER pipeline_ref_journal_transition BEFORE UPDATE ON pipeline_ref_journal
WHEN OLD.rowid<>NEW.rowid OR OLD.id<>NEW.id OR OLD.goal_id<>NEW.goal_id OR OLD.project_id<>NEW.project_id
 OR OLD.run_id IS NOT NEW.run_id OR OLD.kind<>NEW.kind OR OLD.ref_name<>NEW.ref_name
 OR OLD.old_commit<>NEW.old_commit OR OLD.new_commit<>NEW.new_commit
 OR NEW.version<>OLD.version+1 OR NEW.owner_epoch<OLD.owner_epoch
 OR json_extract(OLD.body,'$.creator_run_id') IS NOT json_extract(NEW.body,'$.creator_run_id')
 OR (NEW.owner_epoch>OLD.owner_epoch AND (NEW.state<>OLD.state OR NEW.body IS NOT OLD.body))
 OR NOT (NEW.state=OLD.state
   OR (OLD.state='pending' AND NEW.state IN ('done','held','aborted'))
   OR (OLD.state='held' AND NEW.state IN ('pending','done','aborted')))
 OR (SELECT state FROM pipeline_runs WHERE id=NEW.run_id) IS 'abandoned'
BEGIN SELECT RAISE(ABORT,'invalid pipeline journal transition'); END;
