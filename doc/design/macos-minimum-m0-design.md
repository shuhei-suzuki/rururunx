# macOS 最小一本道 M0: capability probe 設計案

基準: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。状態: **DRAFT / 修正差分の独立レビュー・オーナー設計承認前**。本番コードは変更していない。

M0 は #5/#6 の互換性変更（STRICT）、M1 は #95、M2 は #12、M3 は #9、M4 は #13、M5 は #16 に対応する。M0 の実 Mac 受入まで M1 の実装へ進まない。

本文執筆: Claude Opus 5.5 (`claude-opus-5-5`)、要求 effort high。CLI は同じ model を報告、effort の実測値は未報告。改訂4の本文を転記した。オーナー決定・静的事実の記録は orchestrator が実施した。いずれも独立レビューの代替ではない。

オーナー確定事項（2026-10-13）:

- 版数の厳密固定を capability probe に置換する。auth/CAS/owner/marker/#19/process custody は維持する。
- rate_limit_event: 起動時 decoder selfcheck、実イベントが出た場合は厳密解析。未観測は記録し、実イベント確認済みとは扱わない。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/5#issuecomment-6136393618)
- M4 を rrx 自身の自動マージまで拡張する。Draft PR → 必須 CI/review/safety/repository gate → ready → PR merge → 結果照合。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/13#issuecomment-6136256684)
- 試験用リポジトリは public の [shuhei-suzuki/rrx-e2e-sandbox](https://github.com/shuhei-suzuki/rrx-e2e-sandbox)。作成・使用承認済み。初期 README のみで、fixture・CI・main 保護は未設定。rururunx 本体で試運転しない。

初回の Sol・Opus 独立設計レビューは両方 REQUEST CHANGES。修正版の独立レビューは未完了。本文の未確認事項を承認済み・実証済みと扱わない。実装には設計承認、両 OS の検証、causal controls/mutants、独立実装・安全性レビューが必要。

### 1. 二層の役割

| | 層 S（起動時） | 層 P（準備ごと） |
| --- | --- | --- |
| 対象 | 設定された全 Native agent（alias ごと） | その準備の actor の provider |
| モデル | 呼ばない | Claude のみ canary 1 ターン |
| Codex | version、schema、握手 | 同じ検査を SAME actor で |
| Claude | version、`initialize` | version、admission 後に `initialize`＋canary |
| journal | 新表 `runtime_startup_probes` | 既存 `managed_effects`（kind 接頭辞 `native_phase_`） |
| 環境と cwd | daemon の継承環境、専用の空ディレクトリ | Unit の overlay、worktree |
| 使い道 | 拒否専用。Task admission と composition の必要条件 | Git intent、command、`PreparedNativePhase` の conjunct |

S は P の conjunct になりません。S の行から合格を再構成する経路も作りません（`runtime/installation.rs:125-126` の原則）。

### 2. 層 S の契約

**対応関係**: 1 行は（epoch、installation、alias、step）に対応し、provider、宣言 digest、設定上の model／effort、実行ファイルの同一性（§9）を body に持ちます。project overlay による model／effort の上書き（`config.rs:264-266`）、role 別 argv、Unit の環境 overlay は S では観測しません（P が担う）。

**プロセス**: 既存の本番型 `RetainedRawProcess`（`execution/process.rs:139-215`）を持つ専用 `StartupProbeCustody`。`process_group(0)`、`kill_on_drop`。cwd は `<owner.root>/startup/<epoch>/<alias>/`（0700、rrx が新規作成、終了後に項目数上限つきで削除）。Claude は `command_argv` の Executor 形に専用 UUID で起動し、`initialize` の応答後に停止する（user フレームは送らない）。#19 の custody プールは使いません。

**journal（新表）**

| 項目 | 契約 |
| --- | --- |
| キー | `(owner_epoch, installation, alias, step)`。step は `version`／`schema`／`handshake` |
| 列 | `instance_id`、`provider`、`state`（`pending`／`confirmed`／`unknown`）、`version`>0、`body`（JSON、4096 バイト以下） |
| intent | spawn の前に `pending` を INSERT。同一 tx で `runtime_epoch` が自分の instance と epoch に一致し、キーが未存在であること |
| settlement | `pending`→`confirmed` または `unknown` の CAS（version と body の完全一致）。他の遷移は trigger で拒否 |
| 旧 epoch | epoch fence の tx（`state/execution.rs:588-596`）で、残った `pending` を `unknown` にする。`unknown` は自動削除しない。`confirmed` は 64 epoch より古いものだけ同 tx で削除 |
| 数値 | alias 32 以下／installation（超過は installation 拒否）、alias 128 バイト以下、1 起動あたり 96 行以下 |

**移行（11→12）**

- 新規 DB: 専用 installer で表を作り、版 12。
- 既存 v11: 専用の枝を追加する。現行の一般枝は legacy namespace 検査と再 install を行うため使えない（`state/mod.rs:244-253`）。v11 の厳密レイアウトを確認 → 表を作成 → writer guard を 12 で再設置 → 版 12。
- 既存 v10: 現行の枝（`:233-243`）の後に同じ表作成を足す。
- レイアウト検査の参照（`state/managed_binding/schema.rs:257-281`）は現行の schema ファイルから作るので、新表の installer は版 12 の参照にだけ含める。v10／v11 の参照に入ると既存 DB が厳密検査で落ちる。
- 新表を `MUTABLE_TABLES` に加え、writer contract 12 の guard を付ける（`state/execution.rs:124-143`）。旧バイナリは版 12 を開けない（`state/mod.rs:190-193`）。`BINDING_LAYOUT_VERSION` は 10 のまま。

**起動・停止**

- `start`（`runtime/service.rs:58-71`）は `control_admission` の中で custody を登録して非同期 task を起動するだけで返る。probe 本体は lock の外で動くので、制御面は status と stop に応答する（`cli/service.rs:114` 以降）。
- 合格の公開まで `admit_ready_tasks`（`runtime/admission.rs:33-42`）は 0 を返す。公開はメモリ上の状態で、全 step の settlement が既知になった後にだけ行う。
- `installed_driver_composition`（`runtime/installation.rs:371-414`）は、roster の各 alias について「合格」「記録した実行ファイル同一性と現在の stat が一致」「宣言 digest が一致」を要求する。
- 各 spawn の直前に、停止要求と custody の revoke を短い lock の下で確認する。
- `shutdown`（`:168-200`）: `stopping` の後に custody へ停止要求（group signal）。完了確認の並び（`:196-198`）に S を加え、子が未 reap または task 未 join なら `ShutdownPending`（`runtime/stop.rs:7` に site を追加）を返す。待ち時間は延ばさない。
- Drop: custody が revoke と group signal を行う。task の panic: その alias は `unknown`、行は `pending` のまま残り次の epoch fence で `unknown`。どちらも合格を公開しない。
- 制御面の status に S の状態（`pending`／`pass`／`refused:<語彙>`／`unknown`）を加える（`runtime/control.rs:214, 389`）。

### 3. 層 P の順序と typed conjunct

現行（`execution/native/preparation.rs:1036-1066`）: 準備 commit → 静的互換 → Version → Git×13 → `compat.observe` → command → no-dispatch → quota → `issue_prepared` → transport 登録（`execution/native/transport.rs:583-603`）。

| provider | 変更後 |
| --- | --- |
| Codex | 準備 commit → 静的互換 → Version → **Schema → Handshake** → Git×13 → 以降は現行どおり |
| Claude | 準備 commit → 静的互換 → Version → Git×13 → command → no-dispatch → quota Admit → **Probe** → `issue_prepared` → transport |

循環を避けるため、互換の資格付けを 2 段に分けます。

| 型 | 生成条件 | 消費者 |
| --- | --- | --- |
| `NativeCompatStatic`（既存 `compat.rs:21-26`） | 宣言 profile、Frame digest、hook 宣言。版数比較は削除 | 下段 |
| prefix 資格（既存 `NativeCompatQualification` の意味を変更） | static ＋ SAME actor の閉じた qualified な Version（Codex は Schema と Handshake も） | `plan_native_command`、`check_command`、no-dispatch |
| Probe 資格（新規、Claude のみ） | SAME actor、SAME prefix 資格、SAME command、SAME `NativeQuotaAdmitted`、閉じた qualified な Probe（すべて ptr-eq）。Probe 計画の `before` が helper 完了末尾と同一 | `issue_prepared`（`prepared.rs:248-270`） |

**helper の保持**: no-dispatch 前の prefix は既存の `helpers` 配列（Codex は 16 本、Claude は 14 本。`preparation.rs:559-565` と `state/execution/native_phase/version.rs:475-502` の本数・順序検査を provider 別にする）。Probe は custody の専用 slot に置き、`helpers.first()` が Version である不変条件を保つ。

**inventory 末尾**: Unit の効果 inventory を検査する reader は、custody が持つ typed な末尾（`Prefix(履歴)` または `Probed(履歴, Probe settlement)`）を受け取ります。

| reader | 要求する末尾 |
| --- | --- |
| no-dispatch 検査（`state/…/preparation.rs:228`）、quota 計画と negative 検査（`state/…/quota.rs:287, 595`） | `Prefix`。Probe の intent が存在しないこと |
| transport の計画・登録（`state/…/transport.rs:138, 238, 336`） | Claude は `Probed`、Codex は `Prefix` |
| quota closure（`state/…/quota.rs:76, 1294`） | Probe の intent が無ければ `Prefix`、settlement が既知なら `Probed`。intent があり settlement 不明なら不成立 |

確認済みの内部契約:

- `managed_effects` に kind の CHECK は無い（`state/execution.sql:100-107`）。公開 API は `native_phase_` 接頭辞を拒否（`state/execution/effects.rs:204, 309`）。
- helper の環境は cookie、socket、shim PATH を含む（`execution/resources.rs:36-44`）。準備中の Unit は marker-bound Driver の下にあり、IPC は Grant 前に拒否される（`execution/ipc.rs:145-165` → `state/execution.rs:463-484` → `state/runtime/driver.rs:341-345`）。変更しない。
- 行予算（上限 256、`state/…/version.rs:18, 852-856`）: Codex は prefix +2 行で baseline 上限が 2 下がる（Executor 234→232、Reviewer 238→236）。Claude は Probe +1 行で余裕 1 を使い切り、上限は不変（`execution/native/budget.rs:23-29`）。

### 4. 失敗後の状態

| 失敗の位置 | 結果（コードで確認した既存挙動） |
| --- | --- |
| no-dispatch 前（Version、Schema、Handshake、Git） | job は `Failed`、`abandon()`（`runtime/phase_jobs.rs:986-1008`）。closure は witness 不在で `Held`（`preparation.rs:144-147`、`phase_jobs.rs:653-661`）。5 秒周期で再評価され続ける |
| Probe が Known の不合格 | 下の条件がそろえば、既存の quota closure が lease を解放し（`state/…/quota.rs:1273-1348`）、RN-1 が Unit を Retired・`Refused` にする（`state/…/version/closure.rs:18-44`） |
| Probe が Unknown | closure は不成立で `Held`。pre-Session lease は次の epoch fence（`state/execution.rs:594`）まで残る |

Known 不合格が既存の非成功経路を通る条件:

1. custody が abandoned、actor が revoke 済み、transport 無し、no-dispatch あり（`preparation.rs:137-147`）。
2. 登録が無く、inventory が末尾と一致（`state/…/quota.rs:68-79`）。ここが上表の `Probed` 追従を要する箇所。
3. Unit が Preparing、Session 無し、wait reason が無しか Quota／Capacity（`closure.rs:18-31`）。Admit は wait reason を消し Preparing のまま（`state/…/quota.rs:904`、`state/…/preparation.rs:102`）。

これはコードの条件を読んだ結果で、実行での確認は control Q-2（未実装）です。回復操作は新設しません。Retired 後の Task の見え方（Driver 側）は未読です。

quota・capacity の型（Known 不合格、再試行なし、pool 書込みなし）: `quota_exhausted`（自 session の classified な `rejected`）、`capacity`（既存 `terminal_capacity` 相当、`claude_wire.rs:373-390`）、`auth_unavailable`。限界として、canary の結果は pool にも backoff 履歴（`state/…/quota.rs:527-545`）にも反映されません。

### 5. Claude Probe と固定 helper 契約

| | 本番 | Probe |
| --- | --- | --- |
| argv | `command_argv`（`prepared.rs:280-344`） | 同じ関数の出力。`--session-id` は専用 UUID。Executor では `--permission-mode plan` の 2 語を追加 |
| 入力 | 作業 payload | 固定文面と一意 nonce（`RRX-CANARY-`＋乱数 32 hex） |
| model／effort、環境、cwd | allocation facts、overlay、worktree | 同一 |
| 権限応答 | 既存規則 | 一切しない |

必須: 相関する `control_result`（`claude_wire.rs:46-58`）→ canary 送信後の `system/init`（既存規則 `:203-225`。未着のまま `result` なら `init_missing`）→ `result` が `subtype=success`、`is_error=false`、origin が null か human、前後空白を除いた本文が nonce と完全一致。

拒否（応答しない）: `control_request`、`tool_use` を含む assistant、tool 結果の `user`、`tool_progress`、`tool_use_summary`、`task_started`、注入ターン。

Executor は `writes=worktree` の hook を宣言できます（`compat.rs:39-61`）。Probe は Git×13 の後なので、canary 中の hook の書込みは Held にならず、本番 Session 開始時の同じ hook と区別できません。rrx は再検査しません。plan でも自動許可される読み取り系 tool は、検出時点で既に走っている可能性があります。

### 6. 共通 strict 解析

**Claude**（Probe consumer と本番 `RunState` が同じ解析器を使う）

| 階層 | 有限集合 | 表外 |
| --- | --- | --- |
| top `type` | `system`、`assistant`、`user`、`result`、`control_request`、`control_response`、`stream_event`、`tool_progress`、`tool_use_summary`、`auth_status`、`rate_limit_event`（`claude_wire.rs:201-427`） | 拒否 |
| `system.subtype` | `init`、`session_state_changed`、`task_started`、`task_notification`、`task_updated`、`api_retry` | 拒否（現行 `:276` は通す） |
| content block | `text`、`thinking`、`tool_use` | 拒否 |
| stream の `event.type` | `message_start`、`content_block_start`、`content_block_delta`、`content_block_stop`、`message_delta`、`message_stop`、`ping` | 拒否 |
| uuid の重複 | 正規化 digest を保持（8192 件まで）。同一内容は無視 | 内容違いは拒否（本番も） |

Probe は tool を拒否し、本番は正当な tool と `can_use_tool` を許可します。本番で変わるのは、表外 variant と uuid 改変が終端エラーになる点です（fixture `answer-sameuuid-changing-held` の期待が変わる）。Claude の表には schema 相当の静的根拠が無く、表外に出会った run は variant 名だけを記録して不合格になります。

**Codex の method 表**（rrx 側の有限表。処理順は envelope → method → payload 構造 → 所有フィルタ → consumer）

| 分類 | payload の必須 | method |
| --- | --- | --- |
| A 消費（6） | 各 consumer の必須キー | `account/rateLimits/updated`、`item/started`、`item/completed`、`item/agentMessage/delta`、`error`、`turn/completed` |
| B thread＋turn（22） | `threadId`、`turnId` が有界文字列 | `thread/tokenUsage/updated`、`turn/diff/updated`、`turn/plan/updated`、`item/autoApprovalReview/started`、`item/autoApprovalReview/completed`、`autoApprovalReview/strictReviewRequired`、`item/plan/delta`、`item/commandExecution/outputDelta`、`item/commandExecution/terminalInteraction`、`item/fileChange/outputDelta`、`item/fileChange/patchUpdated`、`item/mcpToolCall/progress`、`item/reasoning/summaryTextDelta`、`item/reasoning/summaryPartAdded`、`item/reasoning/textDelta`、`thread/compacted`、`model/rerouted`、`model/verification`、`modelProvider/authRecoveryStarted`、`modelProvider/authRecoveryCompleted`、`turn/moderationMetadata`、`model/safetyBuffering/updated` |
| C thread（31） | `threadId` が有界文字列（`turnId` があれば文字列か null） | `thread/status/changed`、`thread/archived`、`thread/deleted`、`thread/unarchived`、`thread/closed`、`thread/reverted`、`thread/name/updated`、`thread/attachment/updated`、`thread/goal/updated`、`thread/goal/cleared`、`thread/queue/changed`、`thread/project/updated`、`thread/environment/connected`、`thread/environment/disconnected`、`thread/settings/updated`、`turn/started`、`hook/started`、`hook/completed`、`serverRequest/resolved`、`guardianWarning`、`thread/realtime/` の 11 種（`started`、`itemAdded`、`item/started`、`item/transcript/delta`、`item/completed`、`transcript/delta`、`transcript/done`、`outputAudio/delta`、`sdp`、`error`、`closed`） |
| D 全体（24） | params が object。`threadId` があれば文字列か null | `thread/started`、`skills/changed`、`project/changed`、`command/exec/outputDelta`、`process/outputDelta`、`process/exited`、`mcpServer/oauthLogin/completed`、`mcpServer/startupStatus/updated`、`mcpServer/event/stream/notification`、`account/updated`、`account/gatewayOAuth/changed`、`app/list/updated`、`remoteControl/status/changed`、`externalAgentConfig/import/progress`、`externalAgentConfig/import/completed`、`fs/changed`、`warning`、`deprecationNotice`、`configWarning`、`fuzzyFileSearch/sessionUpdated`、`fuzzyFileSearch/sessionCompleted`、`windows/worldWritableWarning`、`windowsSandbox/setupCompleted`、`account/login/completed` |
| server request（11） | 承認 2 種は既存 ledger（`codex/protocol.rs:498-572`） | 対応: `item/commandExecution/requestApproval`、`item/fileChange/requestApproval`。非対応: `item/tool/requestUserInput`、`mcpServer/elicitation/request`、`item/permissions/requestApproval`、`item/tool/call`、`account/chatgptAuthTokens/refresh`、`attestation/generate`、`currentTime/read`、`applyPatchApproval`、`execCommandApproval` |

| フレーム | S／P の握手 | 本番 Session |
| --- | --- | --- |
| D と `account/rateLimits/updated`（payload 正常） | 数えて続行 | 既存処理または無視 |
| A〜C の thread 系 | thread が無いので不合格 | payload 検査 → 所有フィルタ → 処理または無視 |
| 既知 method で payload 不正（他 thread 宛を含む） | 不合格 | `ProtocolError` 終端 |
| 表外 method | 不合格 | `ProtocolError` 終端 |
| server request | すべて不合格 | 承認 2 種は既存処理、他は既存の拒否 |

現行の本番（`execution/native.rs:2218-2263`）は所有フィルタが先で、未知 method を `_=>{}` で捨て、`params` 欠落を `{}` に置き換えます（`protocol.rs:137-143`）。これを上の順序に変えます。receipt には method 名と件数だけを残し、`account/*` の payload は記録しません。

item は「object、有界な `id` と `type`」、agentMessage は `text` と `phase` を検査します（`execution/native_result.rs:420-487` と同じ要求を解析段へ前倒し）。`ThreadItem` の変種名は投影に無いため、`type` の列挙は行いません。本番は tool を許可し、回答には agentMessage しか使いません。

**rate limit（R1）**

- Claude decoder の結果を `NotRateEvent`、`Classified`、`Unparseable` に分ける。`Unparseable` は `rate_limit_info` の欠落・非 object、`rateLimitType`／`status` の欠落・型違い・未知値、`resetsAt`／`utilization` の型違い・範囲外。現行は未知値を `Ok(None)` にする（`execution/quota.rs:180-197`）。
- Probe でも本番（`native.rs:2323-2346`）でも `Unparseable` は fail closed。本番は終端 `ProtocolError`。現行の「未分類として続行」（fixture `capacity-unknown-window`、`quota-unknown-held`、テスト `quota.rs:249`）は廃止し、期待を書き換える。
- 起動時 selfcheck は installation 内の純粋関数。正例は 5 bucket × 3 status × 任意項目の有無と Codex の 2 形。負例は上の `Unparseable` 全種と、Codex の両キー欠落、`usedPercent` の欠落・非整数・int32 外、bucket 33 個、bucket 名の空・129 バイト、`resetsAt` の型違い・桁あふれ。失敗は installation 拒否。CLI 互換の証明ではない。

### 7. Codex の schema gate と握手

**Schema helper**: `<program> app-server generate-json-schema --experimental --out <dir>`。method discovery の RPC は存在しません。CLI の出力で検証規則を定義し直さず、rrx の有限表と比較します。生成物の hash 一致で許可することはしません（hash は記録のみ）。

| 項目 | 契約 |
| --- | --- |
| 出力先 | P は `units/<unit>/tmp` 直下、S は §2 の専用ディレクトリ直下に、rrx が 0700 で新規作成。既存なら拒否 |
| 上限 | 項目 1024 以下（観測 440）、総量 16 MiB 以下（観測 4,286,533）、深さ 4 以下 |
| 読むファイル | `ClientRequest.json`、`ClientNotification.json`、`ServerNotification.json`、`ServerRequest.json`。各 1 MiB 以下（観測 293,760／208,255）。symlink 不可、通常ファイル、自 uid 所有 |
| strict JSON | 深さ 64、ノード 262,144、文字列 64 KiB、object／array 各 4,096 |
| `$ref` | 同一ファイルの `#/definitions/` のみ。深さ 16、訪問 512。非ローカル・未解決・循環は拒否 |
| cleanup | rrx が作ったディレクトリだけを、上限つきで symlink をたどらず削除。失敗は記録し、効果状態は変えない |

構造述語:

1. **枝**: `oneOf` のうち `properties.method.enum` が単一要素 `[m]` の枝が、rrx が送る各 method にちょうど 1 つ。枝の `required` は `id`・`method`・`params`（`account/rateLimits/read` は `params` 任意で `anyOf [参照, null]`、`initialized` は `method` のみ）。
2. **送る値**: 下表の各キーが参照先 `properties` に宣言され、型が適合する。参照先の `required` は送るキーの部分集合。
3. **消費する定義**: A の 6 method と承認 2 種の params が、§6 の必須キーを `required` に持つ。承認 params の宣言キーが本番の許可集合（`protocol.rs:524-552`）に収まる。`Turn`、`TurnError`、`RateLimitSnapshot`、`ThreadItem` は、本番 consumer が読むキー（`native.rs:2233-2262`、`quota.rs:87-172`、`native_result.rs:420-487`）を宣言する。
4. **包含**: 宣言された notification と server request の method が、すべて §6 の表にある。B・C の method は `required` に `threadId`（B は `turnId` も）を持つ。

| method | rrx が送る値（`native.rs`） | 投影との対応 |
| --- | --- | --- |
| `initialize` | `clientInfo{name,version}`、`capabilities{experimentalApi}`（`:2134`） | `clientInfo` 必須、`capabilities` は参照か null |
| `initialized` | `params:{}`（`:2137`） | 枝は `method` のみ必須 |
| `account/read` | `refreshToken:false`（`:2144`） | boolean、必須なし |
| `environment/status` | `environmentId:"local"`（`:2151`） | string、必須 |
| `account/rateLimits/read` | `{}`（`:2158`） | 必須なし |
| `thread/start` | `cwd`、`environments[]{environmentId,cwd,runtimeWorkspaceRoots[]}`、任意で `model`、`sandbox:"read-only"`（`:2175-2181`） | `cwd`／`model` は string\|null、`environments` は array\|null（items `TurnEnvironmentParams`）、`sandbox` は `SandboxMode`\|null、必須なし |
| `turn/start` | `threadId`、`input[]{type:"text",text,text_elements}`、任意で `effort`（`:2193-2196`） | `input`・`threadId` 必須、`effort` は `ReasoningEffort`\|null。`outputSchema` は送らない（有無を記録） |

**握手**（P は Git の前、S は schema の後）: `initialize` → `verify_initialize`（`protocol.rs:36-67`）→ `initialized` → `account/read`（type は 3 値分類のみ保存）→ `environment/status=ready` → `account/rateLimits/read` → `codex_windows`。quota への書込みはしません。P の握手が枯渇を示しても P の合否にはしません（枯渇の判断は admission と本番の既存経路）。

### 8. helper の事実と Confirmed・qualified

Claude Probe と Codex Handshake（S／P）は、`protocol`、`stop`（`stdin_closed`／`group_signal`）、`exit`、`pipes`、`tail`（`clean`／`late_frame_known`／`late_frame_unknown`／`partial`）、`group_signal`、`reap` を別項目で記録します。

停止手順: 必須観測の後に stdin を閉じ、5 秒まで自発終了を待ちながら読取りと分類を続ける。終了しなければ group signal。両 pipe を最大 2 秒 drain し、leader を reap（最大 10 秒）。

- **Confirmed** = 生成が既知 ∧ 両 pipe が EOF ∧ reap が既知。
- **qualified** = Confirmed ∧ `protocol=pass` ∧ tail が `clean` か `late_frame_known` ∧（自発終了 0、または猶予後の signal）。
- 成功後の未知フレーム、途中で切れた JSON、自発終了の非 0 は Known の不合格。reap 不明と pipe 未 drain は Unknown。

既存 Version の規則（`execution/native/version.rs:105-115, 696-699`）は変えません。custody は現行のまま（group signal と leader reap）です。完全な CLI を登録前に起動するので、group を離れる子孫の余地は従来より広がりますが、M0 では検証も強化もしません。

### 9. 実行ファイル同一性

- 記録（各層の Version の intent 前）: canonical パス、`(dev, ino, size, mtime_ns, ctime_ns)`、sha256。
- 再検査: 後続の全 helper と transport spawn の直前。sha256 は lock の外で再計算、stat の組は spawn 直前に比較。不一致は `executable_changed`、spawn しない。層 S の記録は composition でも現在の stat と比較する。
- in-band の値は使わない（`protocol.rs:406` の注記のとおり `userAgent` は版を保証しない）。
- 限界: 最後の比較と exec の間は原子的でない。ラッパー型 CLI では起動ファイルだけを束縛し、内側は検出しない。

### 10. `--permission-prompt-tool stdio`

登録前に証明するのは、この引数で起動し、stdio の制御チャネルで `initialize` が往復し（S と P）、canary が完走する（P）ことまでです。CLI が権限要求を stdio に出すこと、その形、allow／deny の扱いは証明しません。canary は tool 要求を不合格にし、権限応答をしないためです。本番では `Pending::parse`（`claude_wire.rs:68-101`）が要求を厳格に読み、未知の要求は終端になります。

### 11. legacy 経路（L1）

| 入口 | 変更 |
| --- | --- |
| `NativeSessions::start`／`start_inner`／`start_legacy`（`native.rs:229-245, 270`） | 先頭で型付き CapabilityUnavailable。`:439, :441` の版数検査、`claude_wire::verify_version`、`codex/managed.rs:8` を削除 |
| `NativeAdapter::start`（`adapter/native.rs:422`） | 同じ拒否を `UnsupportedCapability` として返す |
| Codex legacy session（`codex/session.rs:1591`、`codex/protocol.rs:24-31`） | `--version` helper の前で同じ拒否。版数関数を削除。公開 capability は既に EMPTY（`doc/design/master/agent-adapter.md` §16） |

**移植する正例**（拒否表明への置換だけでは不可。installed レーンの harness に移す）

| 元（テスト数） | シナリオ | 待ち |
| --- | --- | --- |
| `execution/native/tests.rs`（25） | wire（`native_raw_duplicate_and_depth_frames…`、`bootstrap_failure_categories…`）、quota（`structured_quota_terminals…`、`codex_two_window_recovery…`、`claude_live_quota_wait…`、`native_subscription_wait_reason…`）、capacity／auth（`authentication_and_unclassified_capacity…`、`configured_global_provider_alias_and_project_caps…`）、answer（`native_answers_are_owned…`、`native_claude_running_hold…`、`native_collector_overflow…`）、permission（`readonly_review_profile…`、`native_dispatch_admission…`）、lifecycle（cancel、competing start、epoch fence、receipt 再試行） | #93 |
| `execution/native/tests/terminal_lock_tests.rs`（3） | terminal lock | — |
| `execution/native/phase_fence_tests.rs`（1） | phase fence。拒否理由が変わるだけで主旨は維持 | — |
| `adapter/native.rs`（2）、`workflow/managed_tests.rs`（5）、`adapter.rs` の該当分 | adapter 経由の起動と cancel | — |
| `codex/session.rs`（60）、`codex/custody_tests.rs`（20）、`codex/ownership.rs`（7） | 別 transport の部品。protocol／approval／usage の単体は起動を伴わず残る。起動を伴うものは、phase 経路に対応する挙動があるものだけ移植する。この分類は未実施 | — |

source bootstrap の 2 件（`tests.rs:47, 205`）は Native 起動前の拒否を見るテストで、移植不要の見込みです。

比較として L2（legacy にも Probe を配線）は公開 API を保てますが、未合成経路に新しい起動能力を足すことになり、Codex legacy session は transport が違うため対象外で残ります。番号ピンを残す案は候補にしません。L1 は公開 API の互換を変えるので、§20 の A3 で全体承認の対象にします。

### 12. config 移行

- 新 profile（例 `rrx-native-probed-v1`）は `cli_version` の不在を要求。
- 拒否は `AgentRegistry::from_managed_config`（`adapter.rs:386-396`）。`InstallationRefusal` になり admission は 0（`runtime/admission.rs:34`）。`qualify_compat_static` の profile 検査は多層防御として残す。
- 保持済み Frame が旧宣言なら、既存の digest 検査（`compat.rs:91-100`）で fail closed。位置は準備 commit の後なので、その Unit は Held。移行の前提は「進行中の managed Task が無いこと」。
- フィールドの単純削除はしない（`config.rs:104` の `deny_unknown_fields` で設定全体が汎用エラーになる）。

### 13. 監査と数値

receipt は 16 項目以下、値 128 バイト以下（私的経路の上限は 8192 バイト、`state/…/version.rs:19`）。層 S の body も同じ語彙です。

- 版数: qualify は exit 0、完全捕捉、256 バイト以下、単一行、制御文字なし。記録は 96 バイト以下で `[A-Za-z0-9 ._()+/:-]` なら原文、外れれば sha256 と `nonrecordable`（合否に影響しない）。
- 保存しない: account の中身、`codexHome`、nonce、canary 本文、フレーム原文、stderr。
- `source_version`（`quota.rs:146, 227`、`native.rs:2227, 2248, 2254`）は番号なしの契約ラベル。
- 期限: 応答 30 秒（本番と同値）、canary 120 秒、Probe 全体 180 秒、Handshake 90 秒、Schema 60 秒、S 全体 10 分。
- 捕捉: Probe 4 MiB、Handshake 1 MiB、Schema／Version 64 KiB（合計 8 MiB の枠内、`execution/native/readonly.rs:6-7`）。

### 14. 競合と実装順序

producer の無い module を先に入れず、設計承認後に producer から consumer までを繋いで実装します。

| ファイル | 競合 | 扱い |
| --- | --- | --- |
| `execution/native/tests.rs`、`runtime/installation/tests/fm_d2.rs` | #93 | 編集が必要（移植、`cli_version` リテラル `:64-67`）。#93 マージ後 |
| `execution/results/tests/retained_tests.rs`、`workflow/committed_source_tests.rs` | #93 | 触れない |
| `runtime/phase_jobs.rs`、`runtime/installation/tests/epoch_fence.rs` | #94 | 触れない見込み。S の gate で前提が変わるテストの有無を実装時に確認 |
| `adapter/inspection.rs`、`inspection/diagnostics.rs` | F3 | 触れない |
| `crates/rrx/tests/project_api.rs` | P4（production 側は未確定） | 触れない |
| `native_fixture.py`、`runtime/installation/tests.rs`、`tests/pin.rs`、`tests/success.rs`、`tests/activation/composition.rs`、`crates/rrx/tests/daemon_native.rs` | 提示集合に無い | 編集する |

fixture は S、P の helper、本番の起動を区別して応答する必要があります。`BOOTSTRAP_CASE`（`native_fixture.py:57-76`）の握手欠陥は、Session 作成前に現れるようになります。

### 15. pin 設計との差分

| 項目 | pin 設計 | M0 |
| --- | --- | --- |
| 許可根拠 | 単一の厳密版数 | 実観測。番号は記録のみ |
| 版数検査の消費者 | phase 4、legacy 3 | すべて削除 |
| legacy 経路 | ピンの下で起動 | 起動前に CapabilityUnavailable |
| 起動時 | 何もしない | 層 S（非モデル、拒否専用） |
| 準備の helper | Version＋Git×13 | Codex は＋Schema＋Handshake、Claude は admission 後に Probe |
| 登録前の観測 | `--version` | 握手、Claude の `system/init`／`result`、Codex の schema 構造 |
| 実行ファイル | canonical パス | stat と sha256 の束縛 |
| 解析 | top `type` のみ有限、未知通知を無視 | 内部 variant、method、payload まで有限 |
| rate decoder | 未知を `None` | 型分離、解析不能は Probe でも本番でも fail closed |
| 宣言 | `cli_version` 必須 | 新 profile、installation で旧宣言を拒否 |
| DB | 版 11 | 版 12（S の journal） |

### 16. 影響範囲分析

| 変更 | 消費者・既存契約 |
| --- | --- |
| helper 変種（`readonly.rs:154-183`） | `version.rs:93-103, 139-142, 435-468, 618-625`、`state/…/version.rs:486-501, 621-641, 687-698` |
| inventory 末尾 | §3 の 8 箇所 |
| 行予算 | `budget.rs:23-29`、`state/…/version.rs:281-319, 811-866`、`prepared.rs:266-268` |
| `qualified_profile()` の廃止 | `version.rs:94, 150, 156, 202`、`compat.rs:144`、`state/…/version.rs:625` |
| profile と `cli_version` | `config.rs:107, 143, 163, 542, 571`、`compat.rs:79-90, 138-142, 253`、`adapter.rs:386-396`、`runtime/installation/tests.rs:71-74`、`tests/fm_d2.rs:64-67`、`tests/success.rs:1726`、`tests/activation/composition.rs:18`、`tests/pin.rs`、`crates/rrx/tests/daemon_native.rs:41, 82, 368, 451` |
| Claude 解析器 | `claude_wire.rs:176-429, 467-481`、`native.rs:2320-2392`、fixture |
| Codex 分類 | `native.rs:939-952, 2209-2264`、`protocol.rs:105-165, 1085-1093`、fixture |
| rate decoder | `quota.rs:175-230, 236-274`、`native.rs:2323-2346` |
| legacy 入口 | §11 の全テスト、`prepared.rs:440-457` |
| S の gate | `runtime/installation.rs:371-414`、`runtime/admission.rs:33-42`、`runtime/service.rs:58-71, 168-200`、`runtime/stop.rs:7`、`runtime/control.rs:214, 389`、composition を通る全テスト |
| 版 12 | `state/mod.rs:32-34, 189-299`、`state/execution.rs:124-143, 588-596`、`state/managed_binding/schema.rs:250-281`、版を検査する各所 |

修正手段の消費者:

| 手段 | 意識する消費者 | 他の消費者 |
| --- | --- | --- |
| 登録前に CLI を本起動 | helper | ユーザーの設定・hook・MCP、CLI 側の履歴、quota の実消費（Claude）、helper に渡る socket と cookie |
| Probe を admission 後に置く | quota 境界 | pre-Session lease と same-Unit gate の保持が最大約 3 分延びる |
| 起動 gate | S | 起動から最初の Task admission までの時間。全統合テストの前提 |
| 解析の厳格化 | Probe | 本番 Session が表外の variant、解析不能な rate event、uuid 改変で終端する |
| 版 12 | S の journal | 旧バイナリとの併用不可、並行作業との版番号の調整 |
| 行予算の減少 | Codex の 256 行 | baseline 上限近傍の Unit |

影響なしと判断: marker、transport 登録の手順、Core の起動、terminal、#19 の custody プール、`runtime/phase_jobs.rs`。

### 17. セキュリティ自己点検

| 面 | 対策と限界 |
| --- | --- |
| CLI 出力の偽装 | strict JSON、専用 id、有限分類、nonce 完全一致、成功後のフレームも検査 |
| canary の副作用 | plan の固定契約、tool と承認要求で不合格、権限応答なし。自動許可 tool の開始は防げない |
| helper からの IPC | marker-bound Driver の拒否が Grant 前にある。control で行 0・Grant 0 を検査 |
| S の合格の流用 | 公開はメモリ上のみ。行から合格を再構成しない。P を省略させない |
| 実行ファイルの差替え | §9。exec 直前の窓とラッパーの内側は限界 |
| quota の迂回 | モデル入力は Admit の後だけ。S はモデルを呼ばない |
| 秘密の永続化 | 有限語彙のみ保存 |
| legacy の緩和 | ピン削除と同時に入口を閉じる |

probe は強い隔離を与えません。helper は利用者の継承権限で動きます。

**認可行列（コード上の立場）**

| 操作 | SAME actor（P） | S custody | 別 actor | revoke 済み・旧 epoch | CLI ピア・子孫 |
| --- | --- | --- | --- | --- | --- |
| helper intent と spawn | 可（admission と CAS） | 自分の表にのみ可 | 不可 | 不可 | 不可 |
| settlement の記録 | 可 | 可 | 不可 | 事実の記録のみ可 | 不可 |
| 観測の消費（Git intent、Prepared） | 可 | **不可** | 不可 | 不可 | 不可 |
| Task admission の拒否 | — | 可（拒否のみ） | — | — | — |
| モデル入力 | 自分の Admit 後のみ | 不可 | 不可 | 不可 | — |
| IPC の Grant、Session 登録、権限応答 | 既存鎖のみ（Probe 中は権限応答不可） | 不可 | 不可 | 不可 | 不可 |

**人の立場**

| 立場 | できること | probe 成功で得るもの |
| --- | --- | --- |
| owner | 設定の選択、§20 の承認、結果の閲覧 | なし |
| admin（制御面の運用者） | status と stop。強制合格や再判定の手段は無い | なし |
| another（別 Project／Task、同一 uid の別プロセス） | 自分の scope の既存閲覧 | なし |
| unauth | なし | なし |

**source → sink**

| source | sink | 制御 |
| --- | --- | --- |
| `--version` の stdout | receipt／S の body | §13 |
| helper のフレーム | 有限語彙、method 名と件数 | 原文は破棄 |
| 生成 schema | sha256 と件数 | 内容は保存しない |
| 実行ファイル | stat と sha256 | 秘密でない |
| nonce | 比較のみ | 保存しない |
| allocation facts／設定 | 子プロセスの argv と環境 | 既存検査と固定契約 |
| 失敗理由 | refusal、status | プロバイダ本文を含めない |

### 18. 検証

各負例で AC-08 のゼロ条件を、`tests/pin.rs` と同じ本番読取り経路で確認します。対の正例は任意の版数文字列で NativeInput に到達させます。

| ID | 欠陥（1 つだけ） | 期待 |
| --- | --- | --- |
| S-1 | S の完了前に Task を投入 | admission 0、status は `pending` |
| S-2 | Codex S: 握手の各応答欠落／`-32601`／account 非 chatgpt／environment 非 ready／rateLimits `{}` | その alias は `refused`、admission 0 |
| S-3 | Claude S: `control_response` なし／id 違い／error | 同上 |
| S-4 | S の途中で stop／custody の Drop／task の panic／daemon 強制終了後の再起動 | 子は reap、shutdown は pending を正しく返す。行は `unknown`、合格は非公開 |
| S-5 | S 合格後に実行ファイルを差替え | composition 拒否 |
| S-6 | v10／v11 の DB を開く／旧 contract で新表へ書く／版 12 を旧版で開く | 移行成功／writer guard で拒否／open 拒否 |
| S-7 | S 合格、P の検査を欠落させる | P で拒否 |
| CL-1 | P: `control_response` 欠陥 | 各語彙 |
| CL-2 | init なしで `result`／init の id・cwd 違い | `init_missing`／`init_mismatch` |
| CL-3 | `result` なし／非成功／nonce と 1 文字違い／余分な文 | 各語彙 |
| CL-4 | 未知 top type／system subtype／content block／stream event／必須項目の欠落 | `frame_unknown`／`frame_unreadable` |
| CL-5 | 同一 uuid で内容違い／同一内容 | 不合格／合格 |
| CL-6 | `can_use_tool`／`tool_use`／tool 結果／`tool_progress` | 不合格、応答フレーム 0 |
| CL-7 | rate event: なし／分類可能／未知 bucket／未知 status／欠落／型違い | 合格（`not_observed`）／合格（`classified`）／以降は不合格 |
| CL-8 | 成功直後に未知フレーム／途中で切れた JSON／自発終了が非 0 | Known の不合格 |
| CL-9 | pipe を閉じない子孫／reap 不能 | Unknown、Held、lease 保持 |
| CX-1 | 枝なし／枝 2 つ／`method.enum` が 2 要素 | `schema_branch` |
| CX-2 | 非ローカル・未解決・循環の `$ref` | `schema_ref` |
| CX-3 | 送るキーの未宣言／型不適合／`SandboxMode` に `read-only` なし／必須キーの追加 | `schema_param` |
| CX-4 | 無関係な定義に同名 property だけ | `schema_param` |
| CX-5 | 消費定義の必須キー欠落 | `schema_consumed` |
| CX-6 | 表外の notification／server request の宣言 | `schema_unknown_method` |
| CX-7 | 出力が symlink／過大／他者所有 | `schema_unsafe` |
| CX-8 | schema は完全だが実 RPC が `-32601` | `unsupported_method` |
| CX-9 | 握手中に thread 系通知／server request／表外 method／D の payload 不正 | 不合格 |
| CX-10 | 握手中に D の正常通知 | 合格 |
| Q-1 | admission が Wait | canary を送らず Parked、モデル入力 0 |
| Q-2 | canary で `rejected`／capacity／auth | 各型。lease 解放、Unit Retired、pool 書込み 0 |
| X-1 | Version 後に差替え（各 helper 前、transport 前） | `executable_changed`、spawn 0 |
| I-1 | canary 中に shim の git を呼ぶ | IPC 拒否、helper 以外の効果 0、Grant 0 |
| M-1 | 旧 profile／`cli_version` あり | installation 拒否、Unit 0 |
| M-2 | 保持 Frame が旧宣言 | digest 検査で不合格、Session 0 |
| L-1 | legacy の各入口 | CapabilityUnavailable、helper 0、Session 0 |
| R-1 | selfcheck のベクタを 1 つ壊す | installation 拒否 |
| LG-1 | 本番 Session: 表外通知／payload 不正の他 thread 通知／未知 system subtype／uuid 改変／解析不能な rate event | 終端エラー |
| LG-2 | 本番 Session: 正常な他 thread 通知、正当な tool 使用 | 継続 |

| ミュータント | 落ちるべきコントロール |
| --- | --- |
| S の gate を外す／S の行から合格を復元する | S-1／S-4 |
| S の合格を P の代わりに使う | S-7 |
| shutdown の確認から S を外す | S-4 |
| 新表を writer guard から外す／v10・v11 の参照に入れる | S-6 |
| request id の相関を外す／init 未着を許す／nonce を包含比較にする | CL-1／CL-2／CL-3 |
| subtype の既定枝を「通す」に戻す／uuid 重複を先に捨てる | CL-4・LG-1／CL-5 |
| Probe consumer を本番 consumer に差し替える／tool 要求に応答する | CL-6 |
| 未知 bucket を分類済みにする／解析不能を未着にする／本番で未分類に戻す | CL-7／LG-1 |
| 成功フレームで読取りを打ち切る／reap 不明を Confirmed にする | CL-8／CL-9 |
| 枝検査を文字列一致にする／`$ref` を解決しない | CX-1・CX-4／CX-3 |
| schema だけで握手を省く／握手だけで schema を省く | CX-8／CX-1 |
| 所有フィルタを payload 検査より先に戻す | LG-1 |
| Probe を admission の前に移す | Q-1 |
| 末尾の追従を closure だけ外す／Unknown を Retired にする | Q-2／CL-9 |
| 束縛の比較を 1 点外す | X-1 |
| marker-bound の拒否を外す | I-1 |
| legacy の拒否を helper の後に移す／版数一致を戻す | L-1／正例 |
| actor の ptr-eq を外す／spawn を intent の前に移す | 流用テスト／順序テスト |

OS: `ci.yml` の ubuntu と macos で fmt、clippy、test、build。push 前にローカル（macOS）で同じコマンドを通します。

### 19. 実 Mac の手順と期待出力

前提: §20 の全体承認済み。スクリプトは CLI の設定・認証・hook ファイルを読みません。

1. run id を決め、状態ディレクトリと一時ディレクトリを新規作成する。既に存在すれば拒否して終了。
2. `claude --version`、`codex --version`、各実行ファイルの sha256 を表示（観測のみ）。
3. release build。
4. 試験用 Git リポジトリを作り、新 profile で executor=claude、reviewer=codex を設定して daemon を起動。
5. status が S の合格を示すまで待つ（S の上限 10 分）。
6. 1 Project／1 Active Task（QUICK）を投入。
7. P の receipt、Worktree gate、Implement の `work=success`、SC-N の Waiting、Unknown 効果 0 を確認。
8. daemon を停止。PASS のときだけスクリプトが作ったディレクトリを削除し、残存なしを確認。

合格時の出力:

- `M0 selfcheck=pass`
- `M0 startup claude=pass version="2.1.296 (Claude Code)" exe=<sha256> initialize=pass`
- `M0 startup codex=pass version="codex-cli 0.160.1" exe=<sha256> schema=pass initialize=pass account=chatgpt environment=ready rate_limits=parsed`
- `M0 preparation executor=claude probe=pass init=pass result=pass rate_event=<classified|not_observed> tool=none`
- `M0 implement=success stop=SC-N sessions=1 unknown_effects=0`
- `M0 NOT_PROVEN codex_model_methods; codex_review_not_run; can_use_tool_roundtrip; default_permission_mode; codex_approval; codex_error_classification; sandbox_effect; escaped_descendants; wrapper_inner_identity`（`rate_event=not_observed` なら `claude_rate_limit_event` を加える）
- `M0 RESULT=PASS`

失敗時:

- 即座に止まり、何も削除しない。出力は `M0 RESULT=FAIL stage=<startup|version|git|quota|probe|implement> reason=<語彙> unit=<held|retired|none> retained=<状態ディレクトリ> tmp=<一時ディレクトリ>`。
- その run は不合格で確定。同じ状態ディレクトリの再実行は手順 1 が拒否する。Held の Unit は回復しない。
- 再挑戦は、原因を確認したうえで新しい run id と状態ディレクトリで行う。保持したディレクトリの削除はオーナーが判断する。

### 20. オーナー承認（M0 全体の設計承認と一括）

決定済み（再確認しない）: R1、M4 の runtime 自動 merge、試験 repo（public、作成済み）、#95、設計資料の送信。

| ID | 判断が必要な scope | 内容と推奨の根拠 |
| --- | --- | --- |
| A1 | Claude の canary | 準備ごとに helper 専用のモデル入力を 1 ターン行う。費用と quota の消費、CLI 側の履歴、設定と hook の実行を伴う。`system/init` と `result` を登録前に観測する方法が他に無い |
| A2 | 固定 helper argv の差分 | Claude Executor の canary に `--permission-mode plan` を付ける。副作用を減らす。代償は既定モードを証明しないこと |
| A3 | legacy 入口の変更 | `NativeSessions::start`、managed native の `AgentAdapter::start`、Codex legacy session が CapabilityUnavailable を返すようになる。番号ピンを残さず、置換なしの緩和も作らない。正例は §11 のとおり移植する |
| A4 | 層 S と版 12 | 新表、移行、起動から S 合格までの Task admission 停止（最大 10 分、通常は数秒の見込みだが未計測） |
| A5 | 本番挙動の厳格化 | 表外の Claude variant、Codex の表外 method・不正 payload、uuid 改変、解析不能な rate event で本番 Session が終端する。既存の「未分類として続行」は無くなる |
| A6 | schema が表外 method を宣言した場合 | 起動前に拒否する。Session 途中の終端を前倒しできる。代償は CLI が method を足すと表の見直しまで使えないこと |

「証明しないこと」の記載は、条件緩和への同意を求めるものではありません。

### 21. 未確認事項

**静的（今回の投影に無い参照先。gate の述語 2・3 が依存）**

転記時の補足（orchestrator の静的観測）: 以下の参照先は生成済みの公開 CLI schema で確認し、`doc/verification/macos-minimum-m0-static.json` の `additional_static_definitions` に保存した。`TurnEnvironmentParams` は cwd/environmentId が必須で runtimeWorkspaceRoots は任意、UserInput の text 枝は text/type が必須で text_elements は array、SandboxMode は read-only を含む。ReasoningEffort は enum ではなく minLength=1 の string。Turn/TurnError/RateLimitSnapshot/ThreadItem と全83 notification の params required/properties も同記録にある。著者の未読と、静的な定義の不在を区別する。これらの事実は gate の設計承認や live protocol qualification を示さない。特に ThreadItem は oneOf の有限 variant を持ち、本文 §6 の未列挙と各 payload 検査の十分性は修正差分レビューの対象とする。

- `TurnEnvironmentParams` が `environmentId`、`cwd`、`runtimeWorkspaceRoots` を宣言するか。rrx は後者を各 environment の内側に送る（`native.rs:2175`）。
- `UserInput` に `type:"text"`、`text`、`text_elements` の枝があるか。
- `SandboxMode` に `read-only`、`ReasoningEffort` に rrx が送る値があるか。
- `Turn`、`TurnError`、`RateLimitSnapshot`、`ThreadItem` の構造。

**外部の実 CLI（未実施）**

- Claude 2.1.296: plan で canary が tool を使わず nonce だけを返すか。`initialize` だけの起動でモデル呼出しや hook 実行が起きないか（S の前提）。stdin の EOF での自発終了と終了コード。`system/init` の時点。内部フレームの実 variant。
- Codex 0.160.1: 認証済みの握手の実応答。握手中に届く通知の種類。Git でない空ディレクトリでの起動（S）。stdin の EOF での終了。

**rrx 内部（未読）**

- 非成功 closure 後の Task と Workflow の見え方。
- `codex/session.rs` などの起動を伴うテストの移植要否の分類。
- 版を検査する箇所の全列挙。
- rrx の公開 CLI 表面（Mac スクリプト用）。
