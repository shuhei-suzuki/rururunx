# macOS 最小一本道 M0: capability probe 設計案

基準: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。状態: **DRAFT / 独立レビュー・オーナー設計承認前**。本番コードは変更していない。

M0 は #5/#6 の互換性変更（STRICT）、M1 は #95、M2 は #12、M3 は #9、M4 は #13、M5 は #16 に対応する。M0 の実 Mac 受入まで M1 の実装へ進まない。

本文執筆: Claude Opus 5.5 (`claude-opus-5-5`)、要求 effort high。CLI 出力の modelUsage は同じ model を報告。effort の実測値は未報告。オーナー回答の転記と静的観測の追記は orchestrator が実施。これらは独立レビューの代替ではない。

オーナー確定事項（2026-10-13）:

- 版数の厳密固定を capability probe に置換する。auth/CAS/owner/marker/#19/process custody は維持する。
- rate_limit_event: 起動時 decoder selfcheck、実イベントが出た場合は厳密解析。未観測は記録し、実イベント確認済みとは扱わない。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/5#issuecomment-6136393618)
- M4 を rrx 自身の自動マージまで拡張する。Draft PR → 必須 CI/review/safety/repository gate → ready → PR merge → 結果照合。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/13#issuecomment-6136256684)
- 試験用リポジトリ候補は新規 private の `shuhei-suzuki/rrx-e2e-sandbox`。作成・使用は未確定。rururunx 本体で試運転しない。

実装時には両 OS の検証と causal controls/mutants、独立実装・安全性レビューが必要。設計本文に残る未確認事項は完了扱いしない。

## 設計

### 1. 二層構成

| | 層 S（起動時） | 層 P（準備ごと） |
| --- | --- | --- |
| 契機 | daemon 起動、設定された全 Native agent | 各 Native 準備 actor（Unit ごと） |
| custody | 独立した起動 probe custody（Unit も actor も持たない） | 既存 `NativePreparationCustody` と SAME actor |
| cwd | rrx が作る専用の空ディレクトリ（`<owner.root>/startup-probe/<epoch>/<alias>/`、0700） | Unit の worktree |
| 検査内容 | 層 P と同じ helper 契約（Version、Codex schema、握手、canary） | 同左 |
| 結果の使い道 | 拒否専用。`installed_driver_composition`（`runtime/installation.rs:371-414`）が roster の alias に同一 epoch・同一 installation の合格観測を要求 | Git intent、`compat.observe`、`PreparedNativePhase` の conjunct |
| 権限 | 何も与えない。層 P を省略させない | 既存鎖の一部。permission/ownership ではない |

層 S を置く理由は、コードで確認した現行契約にあります。

- 層 P の失敗は no-dispatch 発行前の失敗で、job は `Failed` になり `preparation.abandon()` が呼ばれ、Unit/Task は retire されません（`runtime/phase_jobs.rs:986-1008`）。
- 非成功 closure は no-dispatch witness を要求するため `Held` になり（`execution/native/preparation.rs:144-147`）、5 秒周期で `"original preparation Held"` を再評価し続けます（`phase_jobs.rs:647-661, 1079-1089`）。
- つまり認証切れのような失敗が Task を Held に残します。層 S で先に拒否すれば Unit 割当て前に止まります。

層 S の journal には新しい runtime スコープの表が必要です。`managed_effects` は `execution_units` への外部キーを持ち（`state/execution.sql:100-107`）、`audit` は `project_id` 必須（`state/schema.sql:64-66`）なので、既存表では記録できません。これはスキーマ変更です（A6）。

「source preparation ごとの fresh な qualification」は、Source seal から作られる各 Native 準備 actor ごと、と解釈しました。

### 2. 層 P の順序と producer / consumer

変更後の順序（`preparation.rs:1036-1066` への挿入）:

**準備 commit → 静的互換（版数比較なし）→ Version →（Codex のみ）Schema → Probe（握手と canary）→ Git×13 → `compat.observe` → 以降不変**

Git より前に置く理由: Status helper は空出力を要求する（`execution/native/readonly.rs:297-298`）ので、canary が worktree を汚せば既存検査が拒否します。Git バッチの 180 秒期限（`readonly.rs:10`）にも食い込みません。

| 役割 | 場所 | 契約 |
| --- | --- | --- |
| producer | 新規 `execution/native/probe.rs` | 閉じた SAME 前任 helper を前提に 1 回だけ起動 |
| 計画と settlement | 新規 `state/execution/native_phase/version/probe.rs`（`closure.rs` と同じ子 module。`Inventory` が private のため） | identity は allocation facts 由来（`state/…/version.rs:571-577`）。Task.version を使わない |
| 実行枠 | 既存 `run_phase_helper`（`execution/native/version.rs:574-656`） | admission 内 intent → `attempted` → spawn → adopt → 50ms fence → reconcile。capture 本体だけ差し替える |
| consumer | `plan_phase_git_intent`（`state/…/version.rs:621-632`） | SourceTop の前任は qualified な Probe |
| consumer | `confirm_phase_helper_history`（`:475-502`）、`no_dispatch_original`（`preparation.rs:559-565`） | 順序つき manifest: Claude 15 本、Codex 16 本 |
| consumer | `NativeCompatStatic::observe`（`compat.rs:128-151`）、`issue_prepared`（`prepared.rs:248-270`）、`retain_prepared`（`preparation.rs:382-427`） | SAME actor の閉じた qualified な観測を ptr-eq で要求 |

### 3. 確認済みの内部契約

- **効果 kind**: `managed_effects` に kind の CHECK はありません（state の CHECK のみ）。kind を見る trigger は admission 用の `native_input` 一致だけです（`state/managed_binding/schema.sql:194-196`）。公開の予約・reconcile API は `native_phase_` 接頭辞を拒否します（`state/execution/effects.rs:204, 309`）。新 kind を `native_phase_schema` と `native_phase_probe` にすれば、層 P はスキーマ変更なしで私的 producer 専用になります。
- **helper 鎖**: actor の ptr-eq（`preparation.rs:739-758`）、baseline 行は確定済み `git_helper` のみ（`state/…/version.rs:560-567`）、intent は inventory 完全一致の CAS（`:727-732`）、settlement は完全捕捉なら Confirmed、それ以外は Unknown（`:783-787`）、closure は Unit 像と効果像の CAS（`version/closure.rs:369-399`）。revoke 済み actor も事実の記録だけはできます（`closure.rs:332`）。
- **予算**: 行上限は 256（`state/…/version.rs:18`）。phase の Codex は setup 6 行（`initialize`、`initialized`、`account/read`、`environment/status`、`account/rateLimits/read`、`thread/start` がすべて `native_setup` として journal される。`execution/native.rs:1660-1664, 2134-2185`）で余裕 0、Claude は setup 1 行で余裕 1 です（`:853-856`）。Codex は +2 行なので baseline 閾値を 2 下げ、Claude は +1 行で余裕を使い切ります（`execution/native/budget.rs:23-29`）。baseline が上限近傍の Unit は拒否側へずれます。
- **捕捉量**: helper 上限 32 本、合計 8 MiB（`readonly.rs:6-7`）。Probe と Schema の捕捉上限は各 1 MiB にします。
- **Session の作成点**: `register_prepared_transport`（`execution/native/transport.rs:583-603`）、spawn は `:644`。helper 鎖はそれより前です。
- **receipt**: 私的経路の本体上限は 8192 バイト（`state/…/version.rs:19`）。

### 4. 固定 helper 契約（本番 argv との対応）

同一 argv とは主張しません。差分は次の有限集合に固定します。

| | 本番 | helper | 対応の説明 |
| --- | --- | --- | --- |
| Claude argv | `command_argv`（`prepared.rs:280-344`） | 同じ関数の出力。`--session-id` は helper 専用 UUID。Executor では `--permission-mode plan` の 2 語を追加 | plan は書込みを権限コールバックへ回す（`native.rs:516-518` の注記）。canary の副作用を減らすための差分で、既定モードでの挙動は証明しない |
| Codex argv | `app-server --listen stdio://` | 同一 | — |
| Codex `thread/start` | cwd、environments、runtimeWorkspaceRoots、（Reviewer のみ）`sandbox=read-only`、model | 同じ組み立て。Executor でも `sandbox=read-only` を付ける | 既定 sandbox での挙動は証明しない |
| 入力 | 作業 payload | 固定文面と一意 nonce（`RRX-CANARY-` と乱数 32 hex）。「この token だけを返し tool を使わない」 | 作業 NativeInput とは別物。`native_input` 効果も NativeInvocation も作らない |
| model／effort | allocation facts | 同一 | 設定 model の利用可否を確認 |
| 権限応答 | 既存規則 | 一切しない。要求が来たら不合格で停止 | `#10` の代替ではない |
| 環境 | `profile.environment` | 同一（`version.rs:361-365`）。追加キーなし | `check_overlay` は不変 |

終了は、必須観測がそろった時点で owner 側がプロセスグループを停止して reap を確認します。それより前の自発終了は不合格です。

### 5. Claude の検査

| ID | 検査 | 不合格の語彙 |
| --- | --- | --- |
| P-CL-1 | `claude_wire::control("initialize", helper 専用 id)` に対し `control_result`（`claude_wire.rs:46-58`）が成立 | `response_uncorrelated` / `response_error` / `response_malformed` / `timeout` |
| P-CL-2 | canary の user フレーム送信後、`system/init` を既存規則（`claude_wire.rs:203-225`）で検査。未着のまま `result` が来ても不合格 | `init_missing` / `init_mismatch` |
| P-CL-3 | `result`: `session_id` 一致、origin が null か human、`subtype=success`、`is_error=false`、`result` の前後空白を除いた値が nonce と完全一致、`RunState::complete()` | `result_missing` / `result_unsuccessful` / `canary_mismatch` |
| P-CL-4 | 全フレームの `type` が `RunState::observe` と同じ有限集合（`:201-427`）。分類器を共有関数にする | `frame_unknown` / `frame_unreadable` |
| P-CL-5 | `control_request`、assistant 内の `tool_use`、`task_started`、注入ターンは不合格。応答はしない | `tool_requested` / `unexpected_request` / `background_activity` |
| P-CL-6 | `rate_limit_event` は §7 | `metadata_malformed` |

正直に書くと、plan モードでも権限確認なしで自動許可される読み取り系 tool は、rrx が `tool_use` を見た時点で既に走り始めている可能性があります。rrx は検出して不合格にするだけで、防止はできません。

### 6. Codex の検査

RPC に method discovery は存在しません。次の 3 つを同じ canonical program と SAME actor の helper 鎖で行います。

**(a) Schema helper**（kind `native_phase_schema`、1 回起動、stdin は null）

- 起動: `<program> app-server generate-json-schema --experimental --out <dir>`。program は既存の canonical 絶対パス検査（`version.rs:376-383`）を通ったもの。
- 出力先: Unit の所有名前空間 `units/<unit>/tmp`（`version.rs:384-422` が検査済み）の直下に、rrx が operation id 名で 0700 の新規ディレクトリを作る。既存なら拒否。canonical パスが名前空間内であることを確認する。
- 読取: 固定の有限ファイル名のみ（`ClientRequest.json`、`ServerNotification.json`、`ServerRequest.json` と、params 定義を持つファイル）。symlink 不可、通常ファイル、所有者が自 uid、各 4 MiB 以下、ディレクトリ項目数と総バイトに上限、専用上限の strict JSON。
- 検査: 必須 client method（`initialize`、`account/read`、`environment/status`、`account/rateLimits/read`、`thread/start`、`turn/start`）が宣言されている。`ThreadStartParams` に `environments`、`runtimeWorkspaceRoots`、`sandbox` がある。`TurnStartParams` の `outputSchema` は有無を記録するだけ（本番の `turn/start` は送っていない。`native.rs:2193`）。宣言された notification と server request の method 集合が、rrx の有限表に含まれる（A5）。
- cleanup: rrx が作ったディレクトリだけを、項目数上限つきで symlink をたどらず削除する。失敗は `cleanup=failed` として残し、パスは Unit 名前空間内なので既存の Unit 資源片付けの対象になる。
- receipt: 各ファイルの sha256 とバイト数（schema は秘密でない）、宣言 method 数、検査別結果。

**(b) 握手**: `initialize` → `verify_initialize`（`codex/protocol.rs:36-67`）、`initialized`、`account/read`（type は 3 値分類のみ保存）、`environment/status=ready`、`account/rateLimits/read` → `codex_windows`。解析のみで、quota への書込みはしません。

**(c) canary**: `thread/start`（§4 の契約）→ cwd と `modelProvider` を既存規則（`native.rs:2186-2190`）で検査 → `turn/start` → 自 thread・自 turn の `turn/completed` が `status=completed` で、`final_answer` 相の agentMessage がちょうど 1 件あり本文が nonce と完全一致。回答の取得は本番と同じ `native_result::Collector`（`execution/native_result.rs:416-505`）を使います。承認 request が来たら不合格で、応答しません。

**通知の有限分類（probe と本番で共有）**

初稿の「未知 method を無視」は撤回します。

| 分類 | 内容 | probe | 本番 consumer |
| --- | --- | --- | --- |
| 消費 | `account/rateLimits/updated`、`item/started`、`item/completed`、`item/agentMessage/delta`、`error`、`turn/completed` | 既存規則で処理 | 不変 |
| 既知・無視 | 0.160.1 の `ServerNotification.json` が宣言するその他の method を有限列挙 | 数えて無視 | 明示的に無視 |
| 既知・他 thread／turn | 上 2 分類のうち `threadId`／`turnId` が自分でないもの | 無視 | 無視 |
| 未知 method | 表に無い | 不合格 | `ProtocolError` 終端 |
| 読めない | envelope 不正（`protocol.rs:105-165`）、strict JSON 違反 | 不合格 | 既存どおり |

本番への影響:

- 現在の本番（`native.rs:2218-2263`）は thread フィルタを先に行い、最後に `_=>{}` で未知 method を黙って捨てています。これを「method 分類が先、所有フィルタは既知 method にのみ適用」に変えます。
- `Core::codex` は legacy 経路と共有なので、legacy も未知通知で fail closed になります。
- 既存 fixture が出す method はすべて消費分類に入ることを確認しました（`native_fixture.py` は `item/completed`、`item/agentMessage/delta`、`error`、`turn/completed`、`item/commandExecution/requestApproval` のみ）。

### 7. rate limit

「rrx codec の完全性」と「実 CLI の event との適合」を分けます。

- **codec selfcheck**（起動時、層 S）: 埋め込みの既知有限ベクタで `claude_window`／`codex_windows` が受理と拒否を正しく行うことを確認する。rrx 側の実装検査であり、CLI 互換の証明ではない。
- **実 event 観測**: Codex は `account/rateLimits/read` の実応答が必須（毎回観測できる）。Claude の `rate_limit_event` は任意で、モデルターンでも出るとは限らない。

オーナーの技術判断（A2）:

| 案 | 内容 | 帰結 |
| --- | --- | --- |
| R1 | canary 中に event が出たら厳密 decoder で検査し、不正なら不合格。出なければ `rate_event=not_observed` と記録して合否に使わない。codec selfcheck は必須 | 起動前合格を出せる。ただし「実 event との適合は未証明」と receipt と出力に残る |
| R2 | 実 event の観測を必須にし、不在は不合格 | 厳密。event が出ないターンでは BLOCKED になり、M0 完了を保証できない |

握手のみで済ませる案と、実 Session 後の検査で代替する案は採りません。

### 8. legacy 経路のピン

追加で確認したピン: `execution/native.rs:439`（Codex）と `:441`（Claude）、`codex/protocol.rs:24-31`、`codex/session.rs:1591`。`start_legacy` は spawn 前に Session を登録します（`native.rs:541`）。呼び出し元は `adapter/native.rs:422` と、テストは `execution/native/tests.rs` に約 35 箇所、`tests/terminal_lock_tests.rs` に 3 箇所、`phase_fence_tests.rs:150` です。

ピンを外すだけの変更は不可です。

| 案 | 範囲 | 互換影響 | 「CLI を固定しない」との関係 |
| --- | --- | --- | --- |
| L1 | phase 経路だけ probe-ready。legacy の入口と Codex legacy session は、起動前に型付き capability-unavailable を返す | `tests.rs`（#93 の対象ファイル）等の legacy 正例約 40 件が拒否表明への書換えになる。未保護 scope（`native.rs:279-285`）の利用者は起動不能になる | 番号ピンは残らない |
| L2 | legacy にも同じ helper を配線する | legacy は helper journal が別系統（`effects.rs:52` の許可語彙）で登録順序も違うため、別設計が要る。fixture も全面改修 | 番号ピンは残らない。M0 の範囲を大きく超える |
| L3 | legacy のピンを当面残す | 変更なし | **番号ピンが残る。条件と矛盾しうるので、無条件の承認扱いにしない** |

推奨は L1 です。番号ピンを残さず、置換なしの緩和も作らず、L2 より範囲が小さいためです。ただし `tests.rs` の変更は #93 のマージ後になります。

### 9. config 移行

- 新 profile（例 `rrx-native-probed-v1`）は `cli_version` が無いことを要求する。
- 旧 profile、または `cli_version` ありは `qualify_compat_static` で型付き拒否する。
- フィールドを単純削除しない理由: `deny_unknown_fields`（`config.rs:104`）で設定読込全体が汎用エラーになり、拒否の型が失われる。
- 宣言 digest は roster と contract digest の要素なので（`runtime/installation.rs:166-183`）、profile 名の変更で自然に変わる。

### 10. 監査・limits・cleanup・失敗後の状態

- **receipt（有限語彙のみ）**: `observed_version`（64 バイト以下、`[A-Za-z0-9 ._()-]`）、検査別の `pass|fail|not_observed`、`rate_codec_selfcheck`、`rate_event`、フレーム数、総バイト、`creation`、`capture`、`end`、`cleanup`。nonce、canary 本文、account の中身、`codexHome`、フレーム原文は保存しない。
- **`source_version`**（`quota.rs:146, 227`、`native.rs:2227, 2248, 2254`）は番号なしの契約ラベルにする。
- **期限**: 応答 30 秒（本番と同値、`native.rs:940, 2295`）、canary ターン 120 秒、helper 全体 180 秒、reap 10 秒。新設値は失敗を隠す延長ではなく、モデルターンのための上限です。
- **cleanup**: 取消は既存の `CaptureOwner`／`CaptureKick`（`version.rs:319-352`）。停止や reap が確認できなければ効果は Unknown のまま保持され、cleanup 失敗でも Confirmed や Resolved に変えない。journal は消さない。
- **層 P 失敗後**: §1 のとおり Held。回復は既存の明示回復に従い、M0 では自動 retire を追加しない。

### 11. 実装順序と競合

producer の無い module を先に入れることはしません。設計レビュー後、producer から consumer まで一貫して繋いだ 1 つの変更として実装します。

| ファイル | 競合 | 扱い |
| --- | --- | --- |
| `execution/native/tests.rs` | #93 | L1 採用時に変更が必要。#93 マージ後 |
| `runtime/installation/tests/fm_d2.rs` | #93 | `cli_version` リテラル（`:64-67`）が config 変更でコンパイル不能になる。#93 マージ後 |
| `execution/results/tests/retained_tests.rs`、`workflow/committed_source_tests.rs` | #93 | M0 は触れない（後者の該当は `git_helper` のみ） |
| `runtime/phase_jobs.rs` | #94 | M0 は変更しない（失敗後の状態は現行のまま） |
| `runtime/installation/tests/epoch_fence.rs` | #94 | 版数リテラルは無い。効果件数の表明があるかは実装時に確認 |
| `native_fixture.py`、`runtime/installation/tests.rs`、`tests/pin.rs`、`tests/success.rs`、`tests/activation/composition.rs`、`crates/rrx/tests/daemon_native.rs` | どちらの集合にも無い | 変更可。fixture は `tests.rs` が使うので内容を調整 |
| F3／P4 の対象 | 未確認 | 協調待ち |

fixture の `BOOTSTRAP_HOLD`／`BOOTSTRAP_CASE`（`native_fixture.py:15-19, 57-76`）は、Session 作成後の boot 失敗を前提にしています。helper 導入後は、同じ欠陥が Session 作成前の拒否として現れます。

---

## pin設計との差分表

比較対象は `doc/design/issue-43-claude-cli-pin-2-1-294.md` です。

| 項目 | pin 設計 | M0 |
| --- | --- | --- |
| 許可根拠 | 単一の厳密版数 | SAME actor の実観測（握手、canary、Codex は schema も）。番号は記録のみ |
| `claude_wire.rs:26-37` | phase と legacy が使用 | phase は不使用。legacy は A3 |
| `version.rs:149-150` | 版数一致でラベル | 有界 1 行の検査のみ |
| `compat.rs:82-90, 138-146` | 宣言＝ピン、観測ラベル一致 | 削除。qualified な観測の ptr-eq |
| Frame digest、hook 宣言、overlay | 維持 | 維持 |
| helper 構成 | Version + Git×13 | + Probe（Codex は + Schema） |
| wire 処理 | 不変 | Claude の解析は不変。Codex は通知分類を厳格化 |
| 登録前に観測するもの | `--version` のみ | `initialize`、`system/init`、`result`、Codex の各 method 実応答 |
| モデル入力 | 作業入力のみ | helper 専用 canary が加わる（A1） |
| `quota.rs:227` 等 | 番号入り | 契約ラベル |
| C2／C3 | 番号で拒否 | 旧宣言は型付き拒否。各必須応答の欠落で拒否 |
| 新版の資格付け | 実機 run 後にピン更新 | 毎回の実観測。実機 run は M0 の証跡 |
| daemon 起動時 | 何もしない | 層 S（拒否専用） |

---

## 影響範囲分析

### 変更対象と消費者

| 変更 | 消費者・既存契約 |
| --- | --- |
| `NativePhaseHelperAction` に 2 変種追加（`readonly.rs:154-183`） | `version.rs:93-103, 107-114, 139-142, 435-468, 618-625`、`state/…/version.rs:486-501, 621-641, 687-698` |
| helper 数、行予算 | `preparation.rs:560`、`state/…/version.rs:476, 281-319, 811-866`、`budget.rs:23-29` |
| `qualified_profile()` の廃止 | `version.rs:94, 150, 156, 202`、`compat.rs:144`、`state/…/version.rs:625` |
| `cli_version` と profile | `config.rs:107, 143, 163, 542, 571`、`compat.rs:79-90, 138-142, 253`、`runtime/installation/tests.rs:71-74`、`tests/fm_d2.rs:64-67`、`tests/success.rs:1726`、`tests/activation/composition.rs:18`、`tests/pin.rs`、`crates/rrx/tests/daemon_native.rs:41, 82, 368, 451` |
| `PreparedNativePhase` の conjunct | `preparation.rs:336-427`、`prepared.rs:248-270` |
| stdin アクセサ（`execution/process.rs:139-215`、加算のみ） | `version.rs` の helper 全般 |
| Codex 通知分類 | `native.rs:2209-2264`（phase と legacy の共有）、`native.rs:939-952`（boot 中の queue）、fixture |
| `source_version` | `state/execution/quotas.rs:40`、`native_phase/live_quota.rs:216`（保存） |
| 層 S | `runtime/mod.rs:83`、`runtime/installation.rs:371-414`、`runtime/admission.rs:34`、`runtime/service.rs:58-71`、新規の表 |
| legacy（L1 の場合） | `native.rs:237-245, 270`、`adapter/native.rs:422`、`codex/session.rs:1591` |

### 修正手段の消費者

| 手段 | 意識する消費者 | 他の消費者 |
| --- | --- | --- |
| 登録前に CLI を本起動しモデルに入力する | helper の検査 | ユーザーの設定、hook、MCP。CLI 側の履歴。quota。worktree（後段 Git が検出）。helper に渡る runtime socket と cookie |
| helper の所要時間（最大 180 秒） | helper の期限 | same-Unit start gate の保持時間（`preparation.rs:994-1017`）、admission、テスト所要時間、起動時間（層 S） |
| 行予算の減少 | 256 行上限 | baseline 上限近傍の Unit、数値を固定するテスト |
| 通知分類の厳格化 | probe | 本番セッションが未知通知で終端する。legacy も同様 |
| profile 名変更 | 宣言 digest | roster と contract digest、保持済み Frame との不一致（`compat.rs:91-100`） |
| 新しい表 | 層 S の journal | スキーマ版の検査と移行、legacy DB の扱い |

### 影響なしと判断した箇所

marker、epoch、transport 登録、Core、terminal、非成功 closure、#19 custody プールは、入力型もコードも変えません。層 P に DB スキーマ変更はありません（§3 の根拠）。

---

## セキュリティ自己点検

### 攻撃面

| 面 | リスク | 対策 |
| --- | --- | --- |
| CLI 出力 | 巨大・深い・重複キーのフレーム、偽の相関、偽の成功 | strict JSON、専用 request id、有限分類、nonce 完全一致、期限、総量上限 |
| canary ターン | モデルが tool を使う、hook が書き込む | plan／read-only の固定契約、`tool_use`・承認要求で不合格、権限応答なし、後段の Git Status |
| schema 出力 | symlink、他者所有ファイル、巨大ファイル | rrx が作る新規 0700 ディレクトリ、固定ファイル名、symlink 不可、所有者検査、上限 |
| 観測の流用 | 別 Unit の観測や層 S の結果を根拠にする | ptr-eq と inventory CAS。層 S は拒否専用で conjunct にしない |
| 秘密の永続化 | account、パス、canary 本文 | 有限語彙のみ保存 |
| fail open | timeout や未着を合格にする | 全必須検査の論理積、未観測を証明として記録しない |
| legacy の緩和 | ピンだけ外れる | A3（L1 推奨） |

probe は強い隔離を与えません。helper は利用者の継承権限で動き、プロセスグループ外へ逃げた子孫の封じ込めは現行同様に保証外です。

### 認可行列（コード上の権限立場）

| 操作 | SAME actor | 同一 Runtime の別 actor | revoke 済み・旧 epoch | 層 S custody | CLI ピア・hook |
| --- | --- | --- | --- | --- | --- |
| helper intent の予約と spawn | 可（admission と CAS、各 1 回） | 不可 | 不可（`validate_open`、epoch 検査 `state/…/version.rs:443-450`） | 自分の表にのみ可 | 不可 |
| 観測の settlement 記録 | 可 | 不可 | 可（事実のみ。許可は再開しない） | 自分の表にのみ可 | 不可 |
| 観測の消費（Git intent、Prepared） | 可 | 不可 | 不可 | **不可** | 不可 |
| 起動の拒否 | — | — | — | 可（拒否のみ） | — |
| Session 登録、NativeInput | 既存鎖のみ | 不可 | 不可 | 不可 | 不可 |
| 権限応答 allow | 既存規則のみ。helper 中は誰も不可 | 不可 | 不可 | 不可 | 要求はできるが決定は不可 |

### 人の立場との対応

| 立場 | probe に関してできること | probe 成功で新たに得るもの |
| --- | --- | --- |
| owner（daemon と設定の所有者） | 設定の選択、A1〜A6 の承認、結果の閲覧 | なし |
| admin（制御面を操作できる運用者） | 既存の制御操作。probe の強制合格や再判定の手段は設けない | なし |
| another（別 Project／別 Task の担当） | 自分の scope の結果のみ既存経路で閲覧 | なし |
| unauth | なし | なし |

制御面の人の認証モデルそのものは本稿では読んでおらず、上表は「probe が新しい人向け操作を追加しない」ことだけを述べています。

### source→sink

| source | 経路 | sink | 制御 |
| --- | --- | --- | --- |
| `--version` の stdout | 有界捕捉 → 文字集合検査 | receipt | 64 バイト、許可文字 |
| helper のフレーム | strict 解析 → 有限語彙 | receipt | 原文は破棄 |
| 生成 schema | 固定名 → 上限 → strict 解析 | sha256 と件数 | 内容は保存しない |
| nonce | 乱数 → canary 入力 | 比較のみ | 保存しない |
| allocation facts | `command_argv` と固定差分 | 子プロセスの argv と環境 | 既存検査と固定契約 |
| 失敗理由 | 有限語彙 | refusal 文字列、状態表示 | プロバイダ本文を含めない |

---

## 検証

### 1. 因果コントロール

専用の模擬 CLI に欠陥を 1 つだけ入れます。各ケースで語彙と AC-05 のゼロ条件を確認し、対の正例（CL-0／CX-0、版数は任意文字列）は NativeInput に到達します。

| ID | 欠陥 | 期待 |
| --- | --- | --- |
| CL-1 | `control_response` なし／id 違い／error | `timeout` / `response_uncorrelated` / `response_error` |
| CL-2 | `system/init` を出さずに `result` | `init_missing` |
| CL-3 | init の `session_id` または cwd が違う | `init_mismatch` |
| CL-4 | `result` を出さない | `result_missing` |
| CL-5 | `result` の本文が nonce と 1 文字違う、または前後に余分な文 | `canary_mismatch` |
| CL-6 | `subtype` が error、または `is_error=true` | `result_unsuccessful` |
| CL-7 | 未知 `type`、重複キー、過大行 | `frame_unknown` / `frame_unreadable` |
| CL-8 | `can_use_tool` を送る、または `tool_use` を含む assistant | `tool_requested`。応答フレームが 0 であること |
| CL-9 | 不正な `rate_limit_event` | `metadata_malformed` |
| CL-10 | 正しい `rate_limit_event` あり／なし | R1 では両方合格で記録が異なる。R2 では「なし」が不合格 |
| CX-1 | schema から必須 method を 1 つ欠く（6 通り） | `schema_method_missing` |
| CX-2 | schema から `environments`、`runtimeWorkspaceRoots`、`sandbox` のいずれかを欠く | `schema_param_missing` |
| CX-3 | schema は完全だが実 RPC が `-32601`（method ごと） | `unsupported_method`（宣言は代替にならない） |
| CX-4 | schema 出力が symlink、過大、他者所有 | `schema_unsafe` |
| CX-5 | `account` が null または apiKey、environment が非 ready、rateLimits が `{}` | 各語彙 |
| CX-6 | 未知 notification method | `frame_unknown` |
| CX-7 | 既知 method だが他 thread | 合格（未知と混同しない） |
| CX-8 | 承認 request | `tool_requested`。応答なし |
| CX-9 | `turn/completed` なし／failed／nonce 不一致／final_answer が 2 件 | 各語彙 |
| S-1 | 層 S 不合格の alias を含む roster | Unit 割当て前に拒否、Held なし |
| S-2 | 層 S 合格、層 P 不合格 | 層 P で拒否（層 S は代替にならない） |
| G-1 | canary 中に worktree へ書込み | Git Status で拒否 |
| LG-1 | 本番 Codex セッションで未知通知 | `ProtocolError` 終端 |

### 2. ミュータント

| 変異 | 落ちるべきコントロール |
| --- | --- |
| request id の相関を外す | CL-1 |
| init の未着を許す | CL-2 |
| nonce 比較を前方一致や包含にする | CL-5 |
| `result` の成功条件を外す | CL-6 |
| tool 要求に allow／deny で応答する、または無視して続行 | CL-8、CX-8 |
| 未知 notification を無視に戻す | CX-6、LG-1 |
| thread フィルタを method 分類より先に戻す | CX-6（他 thread の未知 method） |
| schema 検査だけで実 RPC を省く | CX-3 |
| 実 RPC だけで schema 検査を省く | CX-1、CX-2 |
| timeout や EOF を合格にする | CL-4、CX-9 |
| `compat.observe`／`issue_prepared` が観測を要求しない | 全負例が NativeInput に到達 |
| actor の ptr-eq を外す、または層 S の結果を層 P の代わりに使う | 別 actor 流用テスト、S-2 |
| spawn を intent の前に移す | 順序テスト |
| 版数一致を許可条件に戻す | CL-0／CX-0 |
| `rate_event=not_observed` を `pass` と記録する | CL-10 |
| helper 契約の差分を消す（Executor で plan を外す） | 契約一致テスト |

結果は PR に表で残します。

### 3. 失敗時ゼロ

`pin.rs` と同じ本番読取り経路（`native_invocations` 件数、Session binding、効果一覧）で確認します。SQL seed は使いません。

### 4. Linux と macOS

`ci.yml` の両 OS で fmt、clippy、test、build。push 前にローカルで同じコマンドを通します。

### 5. 実 Mac 手順案

前提: A1 承認済み。スクリプトは CLI の設定・認証・hook ファイルを読みません。

1. `claude --version` と `codex --version` を表示（観測のみ）。
2. release build。
3. 一時ディレクトリに試験用 Git リポジトリと rrx 状態を作成し、パスを記録。
4. 新 profile で executor=claude、reviewer=codex を設定し、daemon を起動。
5. 層 S の結果を表示（両 CLI）。
6. 1 Project／1 Active Task（QUICK）を投入。
7. 層 P の receipt、Worktree gate、Implement の `work=success`、SC-N の Waiting を確認。
8. Pending／Unknown 効果が 0 であることを確認。
9. 停止し、一時ディレクトリを削除して残存なしを確認。

期待出力（固定行の例）:

- `M0 startup claude=pass version="2.1.296 (Claude Code)" init=pass result=pass rate_codec=pass rate_event=<observed|not_observed>`
- `M0 startup codex=pass version="codex-cli 0.160.1" schema=pass account=chatgpt environment=ready canary=pass`
- `M0 preparation executor=claude probe=pass`
- `M0 implement=success stop=SC-N session=1 unknown_effects=0`
- `M0 not_proven: can_use_tool round trip; claude rate_limit_event when not_observed`

rrx の公開 CLI のコマンド名は、既存の `crates/rrx/tests/daemon_native.rs` のレーンと同じ表面を使います（本稿では未読）。

---

## オーナー承認事項

A2 はオーナーが R1 を選択済み（2026-10-13）。その他の事項と設計全体は承認前です。

| ID | 事項 | 推奨 | 根拠 |
| --- | --- | --- | --- |
| A1 | 登録前 helper が、作業入力とは別の canary 用モデル入力を行う。費用と quota の消費、CLI 側の履歴生成、設定と hook の実行を伴う。層 P は準備ごとに 1 ターン、層 S は起動ごとに agent あたり 1 ターン | 承認 | `system/init` と `result` を Session／NativeInput 作成前に観測する方法が他に無い。不承認なら M0 は BLOCKED |
| A2 | Claude の rate limit: R1（出たら厳密検査、codec selfcheck 必須、未観測は未証明と記録）か R2（実観測必須、不在は不合格） | R1 | event は任意で、R2 は完了を保証できない。R1 は未証明を隠さず記録する。確実性を優先するなら R2 で、BLOCKED を受け入れる判断になる |
| A3 | legacy 経路: L1（起動前に capability-unavailable）、L2（同じ probe を配線）、L3（番号ピン温存） | L1 | 番号ピンを残さず、置換なしの緩和も作らない。L3 は「CLI を固定しない」条件に反しうる |
| A4 | 固定 helper 契約の差分（Claude Executor に plan、Codex Executor に read-only） | 承認 | canary の副作用を減らす。差分は有限で明記済み。代償は既定モードを canary で証明しないこと |
| A5 | schema が宣言する notification／server request に rrx の表に無い method がある場合、起動前に拒否する | 拒否 | セッション途中の未知フレーム終端（作業が Unknown になる）を、起動前の BLOCKED に前倒しできる。代償: CLI が通知を追加すると表の見直しまで使えない |
| A6 | 層 S の journal 用に runtime スコープの表を追加する（スキーマ変更） | 承認 | 既存表は Unit か Project を要求する。保存なしでは intent 先行と Unknown 保持を満たせない |

決定済みで再確認しないもの: rrx 自身の PR の自動 merge を M4 に含めること（2026-10-13）。

---

## 未確認事項

**外部の実 CLI（未実施）**

- Claude 2.1.296: plan モードで canary が tool や ExitPlanMode を使わずに nonce だけを返すか。`system/init` が出る時点。`rate_limit_event` の出現頻度。helper UUID の履歴の残り方。自動許可される tool の範囲。
- Codex 0.160.1: 認証済みの実応答（account、environment、`thread/start`、`turn/start`）。read-only sandbox での canary の挙動。canary thread の履歴。

**schema 生成物**

- ご提示の事実（`ClientRequest.json` の method、`TurnStartParams`、`ThreadStartParams`）は採用しましたが、生成物そのものは読んでいません。
- 未確認: method 名の JSON 上の位置（抽出規則）、`ServerRequest.json` のファイル名、0.160.1 の notification 全列挙、各ファイルのサイズ。0.160.0 の `ServerNotification.json` は 208,255 バイトと `doc/design/native-result-receipts-design.md:36-45` にあります。

**rrx 内部**

- スキーマ版の検査と移行の手順（`state/execution/migration.rs` は未読）。
- 層 S を `Runtime::start`（`runtime/service.rs:58-71`）のどこで実行し、停止要求でどう取り消すか。
- `installed_driver_composition` が拒否したときの Driver 側の再試行挙動。
- Held になった準備の具体的な回復操作。
- helper に渡る runtime socket と cookie で、Preparing 中の Unit が IPC から何をできるか。
- `epoch_fence.rs` など、helper 数や効果件数を固定するテストの総数。
- 制御面の人の認証モデル。

**協調**

- F3／P4 の対象ファイル。