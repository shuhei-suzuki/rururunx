# macOS 最小一本道 M0: capability probe 設計案

状態: **DRAFT / A3 の旧 API 方針が未確定 / 修正差分の独立確認・オーナー設計承認前**。本番コード・テストは変更していない。

基準の公開 main: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。本修正の入力 HEAD: `d452675989df799d29931c7d4f19d73d6b387550`。

本文の修正執筆: Codex CLI に `gpt-6.1-sol` / high を固定 argv で要求。CLI の実使用モデルと effort は未報告（一致は未確認）。独立レビュアーの context は執筆へ流用していない。これは独立レビュー・実装・実機検証の証拠ではない。

前回の Sol/Opus 修正差分レビューはともに REQUEST CHANGES。今回の修正版は独立確認前であり、指摘を解消済み・close と認定しない。詳細レビューと指摘の突合は私的記録に保持する。[進捗](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6137767541)

オーナー確定事項（2026-10-13）: 番号 pin 撤廃と capability probe、R1（decoder selfcheck＋実 event strict、未観測は未証明）、M4 の runtime 自動 merge、public [rrx-e2e-sandbox](https://github.com/shuhei-suzuki/rrx-e2e-sandbox) の作成・使用。再承認を求める事項ではない。sandbox は初期 README のみで、fixture・CI・main 保護は未設定。

**macOS最小一本道M0：設計修正全文**

基準と状態は要件に同じ。本稿の型名は実装予定の契約名であり、存在する実装・取得済み証拠を意味しない。

### 1. Producerからconsumerまでの鎖

| producer／契約 | consumer |
|---|---|
| installationの純粋decoder selfcheckが生成する非公開`DecoderChecked` | Native graphのinstallation。単なる関数呼出しではなく成功値を必須引数にする。 |
| Sのfreshな全alias観測、既知settlement、宣言digest、program identity | メモリ上のstartup gate、Task admission、installed composition。journalから合格を再構成しない。 |
| 静的profile／Frame digest／既存hook宣言検査 | `NativeCompatStatic`。番号比較を含めない。 |
| SAME actorのVersion、CodexではSchema／Handshake、Git×13の閉じた履歴 | prefix qualification、command、no-dispatch、quota計画。 |
| ClaudeのSAME prefix・command・quota Admit・Probe settlement | `ClaudeProbeQualified`。`issue_prepared`が必須とする。 |
| provider別の最終inventory末尾 | Prepared、transport登録、quota closure。ClaudeはProbeを含む末尾、Codexはprefix末尾。 |
| 共通wire decoderが生成するvalidated frame | 所有フィルタ、RunState、quota consumer、answer collector、approval consumer。未検査`Value`を迂回入力しない。 |

Pの順序は次のとおりとする。

- Codex：準備commit→static→Version→Schema→Handshake→Git×13→command→no-dispatch→quota→Prepared→transport。
- Claude：準備commit→static→Version→Git×13→command→no-dispatch→quota Admit→Probe→Prepared→transport。

Claude Probeは専用custody slotに保持する。prefixの先頭Versionと13本のGit順序を変えない。no-dispatch／quota admissionは`Prefix`末尾を要求し、Probe intentがまだないことを確認する。Prepared／transportはClaudeの`Probed`末尾を要求する。closureはProbe intentなしならPrefix、既知settlementありならProbed、未確定intentありなら成立させない。

authorityは既存のowner／marker／epoch／actor原本から生成する。Task.versionやjournal DTOから再構成しない。

### 2. Sのcustody、path、journal

S専用custodyは既存`RetainedRawProcess`を保持し、intent commit後にだけspawnする。spawn後のadoptをpipe／identity検査より先に行う。#19のcustody poolを流用しない。

`RetainedRawProcess::pipes`は現在stdout／stderrだけを返すため、stdinを保持・一度だけcloseできる所有型へ拡張する。stdin、stdout、stderrの移動とclose責任をcustody内に一本化し、非同期taskがstdinを失って停止不能になる構造を作らない。

**path境界**

raw aliasはpath成分に使わない。directoryはowner root配下の固定`startup`、rrx発行epoch ID、alias identityのdomain-separated SHA-256を小文字64hexにした名前から構成する。digest対象にはinstallation IDとaliasのUTF-8長・bytesを含め、曖昧な文字列連結をしない。

rootから各成分をsymlink追跡なしで確認し、自uid所有、0700、通常directory、root配下を要求する。既存probe directoryは再利用せず拒否する。作成時のdirectory identityをcustodyが保持する。cleanupはrrxが当該runで作った同一directoryだけを、symlinkをたどらず有限件数で削除する。canonical path確認だけで途中のsymlink置換を安全と扱わない。境界を維持できなければ削除せず失敗を記録する。

`..`、slash、absolute形aliasはraw joinを復活させたmutantがroot外へ到達しないための負例とする。digest方式ではこれらのbytesから作られるpathもroot内に限定される。既存合法aliasのpositiveを維持する。

**journal**

新表`runtime_startup_probes`のキーは`(owner_epoch, installation, alias, step)`。stepはVersion／Schema／Handshake。provider、instance、宣言digest、設定model／effort、program identity、有限な観測要約をbodyに保持する。stateはpending／confirmed／unknown、versionは正整数、bodyは4096 bytes以下。

intentは自instance／epoch一致と未存在キーを同一transactionで検査する。settlementはpendingからconfirmedまたはunknownへの、versionとbody完全一致CASだけを許可する。confirmedは物理的結果が既知という意味であり、protocol合格とは別である。

aliasはinstallationあたり32、1 epochのstep行は96以下。全journal行は8192以下、unknown＋pendingの予約込みは4096以下とする。上限を超える新intent／spawnを拒否する。confirmedは64 epochより古いものだけ削除対象とし、unknownを自動削除しない。旧pendingはepoch fenceでunknownへ移す。保存失敗を成功へ反転しない。

再起動は新epochのfresh検査であり、旧unknownの復旧ではない。`oldEpochUnknown`件数をstatusに保持する。上限到達時は再起動だけでは解除されない。

### 3. Migrationと版別layout

現在の45表を`M11`、Binding10の8表を`B10`とする。版別manifestを明示的に選択する。

| layout | mutable table／guard集合 | read-index | startup表 |
|---|---|---|---|
| actual v9 | `M11 − B10`、writer contract 9 | なし | なし |
| v10 | `M11`、writer contract 10 | なし | なし |
| v11 | `M11`、writer contract 11 | `goals_by_project` | なし |
| v12/current | `M11 ∪ {runtime_startup_probes}`、writer contract 12 | v11と同じ | あり |

v1〜8の入力は既存の版別namespace検査と順序付きmigrationを維持する。v9は既存actual9のSQL構造を厳密検査する。旧layoutに存在しないstartup表のguardを作らない。

table作成とguard設置を分離し、旧reference生成がcurrent installerのguard副作用を受けないようにする。

- **new DB**：空DB／application ID検査→既存全table→startup表→retained guard／read-index→v12 writer guard→current exact layout検査→version 12公開。
- **v1〜9**：既存入力検査→既存ordered table／JSON migrationとhold処理→既存全tableが揃った後にstartup表→read-index／retained guard→v12 writer guard→exact検査→commit。途中のinstallerも実在tableの版別集合を使い、新表への早期guardを作らない。
- **v10**：exact v10検査→startup表→read-index→v12 writer guard→exact検査→commit。
- **v11**：exact v11専用枝→startup表→v12 writer guard→exact検査→commit。legacy一般枝へ落とさない。
- **current v12**：version／application／exact layoutをcoherent transactionで再確認。修復installをしない。
- 同時initializerが既にv12へ進めていた場合は再installせずcurrent検査する。

`BINDING_LAYOUT_VERSION=10`を維持する。layout referenceはv10／v11／v12の3種類を独立生成し、v9 actual検査は旧集合を固定する。旧バイナリのv12 open拒否を維持する。

`MUTABLE_TABLES`のconsumer inventoryは以下を閉じた対象とする。

- production：`state/execution.rs`のwriter guard、`state/managed_binding/schema.rs`のactual9件数・guard照合とlayout reference。
- tests：`state/read_index_tests.rs`、`state/runtime/tests.rs`、`state/execution/native_results_tests.rs`、`state/managed_binding/schema_tests.rs`、`state/execution/tests.rs`、`state/execution/source_recovery.rs`内tests、`state/execution/verification.rs`内tests。

各consumerはcurrent集合かhistorical集合かを明示する。migration用SQL fixtureは旧DB入力であり、実行成功authorityのseedには使わない。

### 4. Startup公開と停止

`start`は短いcontrol lock内でcustodyを登録し、probe本体をlock外で実行する。全alias合格の公開前はadmission 0。compositionは全aliasの合格、宣言digest、現在program statを再確認する。

各spawn直前にstop／revokeを確認する。shutdownはS task joinとleader reapも確認し、未完なら`ShutdownPending::StartupProbes`を返す。既存deadlineを延ばさない。

| 状況 | 期待 |
|---|---|
| 通常stop | 新spawnを止め、stdin close／group signal、drain、leader reap、task joinを確認。未完ならpending。 |
| custody Drop | revokeとhygieneを試みる。Dropだけでreap済み・Confirmedとしない。 |
| task panic | 合格を公開しない。未settled intentを保持し、次epoch fenceでunknown。保持されたcustodyのhygiene／reap事実を別に記録する。 |
| daemon forced death | 死んだdaemonがreapしたとは記録しない。再起動時のepoch fenceで旧pendingをunknownにする。子孫死亡の証明にしない。 |

`RuntimeMetadata`のwire producer／consumerにstartup全体状態、alias別状態、oldEpochUnknown件数を追加する。`deny_unknown_fields`の型、CLI表示・deserialize testsを同時更新する。

`ShutdownSite`の追加は`runtime/stop.rs`、`runtime/control.rs`、`cli/daemon.rs`、`cli/daemon/tests.rs`まで配線し、「six」の注記も更新する。installation拒否理由を固定文へ潰す現行consumerは型付き分類を保つ形へ変更する。

### 5. 行予算

実producerは全roleでGit×13を生成する。Reviewerを9本とする計算を廃止する。256行上限、Git本数、既存byte上限は緩めない。

baselineを`B`とすると、閉じた契約は次のとおり。

| provider／role | Version前B | Version後 | Git前 | Git後prefix | Claude Probe後／Prepared上限 |
|---|---:|---:|---:|---:|---:|
| Codex Executor／Reviewer | 232 | 233 | 235（Schema＋Handshake後） | 248 | 248 |
| Claude Executor／Reviewer | 237 | 238 | 238 | 251 | 252 |

Codexは`232+Version1+Schema1+Handshake1+Git13=248`。Claudeは`237+Version1+Git13+Probe1=252`。Claude Probe末尾はPrepared上限に含め、prefixだけを`check_prepared`へ渡さない。

最初の入力までの後続reserveはCodexがtransport1＋setup6＋input1、Claudeがtransport1＋setup1＋input1。したがってCodexは256、Claudeは255。terminalは既存input effectのsettlement／result publicationであり、新しいmanaged effect行を追加する計算にはしない。terminal consumerが追加行を必要とする変更はこの契約を再設計する。

同じbudget定数をVersion intent、metadata helper、Git reserve、Probe intent、Prepared、registration、setup/input consumerへ渡す。byte reserveは追加helperの最大imageも含む。

境界positiveとB+1 negativeは実installed producerが生成する履歴で検査する。`vec![row; N]`やSQL seedだけでは予算契約の正例にならない。Reviewerの実producer未接続は§11の未達に含む。

### 6. Claude Probeとstrict wire

Probeは本番`command_argv`を使い、専用session UUID、同じmodel／effort／overlay／worktreeを用いる。Executorにはplan modeを追加する。既存設定・認証・hookを変更・スキップしない。

canary入力は次の固定文面とする。

> Return exactly the following token as plain text, with no explanation and without using tools: `<nonce>`

nonceは`RRX-CANARY-`＋乱数32hex。成功は相関initialize応答→canary送信→identity／cwd一致のsystem-init→`subtype=success`、`is_error=false`、humanまたはorigin absent/nullのresult、前後空白除去後のnonce完全一致とする。包含比較をしない。

Probe中のtool・権限要求・tool result・background task・注入turnは拒否し、応答しない。既存hook書込みや自動許可toolは検出以前に起きる可能性がある。plan modeを隔離証明としない。

共通decoderはduplicate key、切れたJSON、型不正を拒否してから、次の有限構造を検査する。

| frame | 必須構造／有限分岐 |
|---|---|
| `system/init` | session ID、cwd文字列、tools配列、mcp_servers配列。各要素も所定の構造を検査する。 |
| `session_state_changed` | state=`idle/running/requires_action`。 |
| `task_started` | task_id、task_type=`local_agent/local_workflow`。 |
| `task_notification` | task_id、status=`completed/failed/stopped`。 |
| `task_updated` | task_id、patch object、patch.status=`completed/failed/stopped/killed`。 |
| `api_retry` | retry情報の既知整数・文字列・nullable field。未知statusをcapacity成功証拠へ変換しない。 |
| `assistant` | message object、content配列。textはtext文字列、thinkingはthinking文字列、tool_useはid/name文字列＋input object。 |
| `user` | message object、role、content文字列または既知block配列。tool_resultはtool_use_idと型付きcontent、is_errorがあればboolean。 |
| `result` | session ID、有限subtype、is_error boolean。成功resultは文字列。error subtypeは既存4種を保持し、未知subtypeは拒否。 |
| `control_request` | request_id、request object、`can_use_tool`のtool_use_id/tool_name/input。 |
| `control_response` | response object、相関request_id、success/error subtype、subtype別response／error構造。 |
| `stream_event` | event objectと下記入れ子構造。 |
| `tool_progress` | tool_use_id、tool_name、経過時間の有限非負数。 |
| `tool_use_summary` | summary文字列、preceding tool ID配列。 |
| `auth_status` | boolean状態、文字列配列、optional nullable error。 |
| `rate_limit_event` | session IDと§9のrate構造。 |

stream eventは`message_start/content_block_start/content_block_delta/content_block_stop/message_delta/message_stop/ping`だけを許可する。block開始には非負indexと型付きcontent_block、deltaには非負indexとdelta objectを要求する。delta.typeは`text_delta/thinking_delta/input_json_delta/signature_delta`で、それぞれtext／thinking／partial_json／signature文字列を要求する。message_startのmessage、message_deltaのdelta／usageも型付き検査し、単にevent.typeが既知という理由で通さない。

origin.kindは`human/task-notification/hook/mcp-channel/peer`の有限集合とする。既知構造でもProbe consumerはhuman以外を成功証拠に使わない。このClaude集合はrrxの受理契約であり、対象CLIのlive観測済み集合ではない。表外実eventはvariant名だけを安全に記録して不合格とする。

UUIDがあるframeはUUIDを解析して小文字hyphen形へ正規化する。JSON object keyを再帰的にsortし、array順序、文字列内容、absentとnullを保持してcanonical digestを計算する。未知fieldを削ってdigestを作らない。8192 UUID上限。構造検査後に重複照合し、同一UUID・同一内容だけをdedup、内容改変を拒否する。request IDなどopaque IDはUUIDへ変換しない。

Claude rate eventはforeign sessionでもstrict decodeを先に行う。正常foreign eventだけを所有フィルタで除外し、quota更新しない。

### 7. Codexの有限wire契約

rrx側に`CodexWireContractV1`を定義し、typed decoder、schema gate、S/P、本番consumerで共用する。CLIのruntime出力から型・required・enumを作り直さない。ignored通知も同じdecoderを通す。

以下の表の記法は、`s=string`、`b=boolean`、`i=int64`、`T=threadId:s`、`U=turnId:s`、`I=itemId:s`、`[]`は型付きarray。IDには既存の有界ID制約を追加する。表は**全83通知の必須payload**を定義する。同じ行にある複数methodは同一形を共有する。

| method | 必須payload |
|---|---|
| `error` | T,U,error:TurnError,willRetry:b |
| `thread/started` | thread:Thread |
| `thread/status/changed` | T,status:ThreadStatus |
| `thread/archived/deleted/unarchived/closed/reverted`（各独立method） | T |
| `skills/changed` | 空object |
| `thread/name/updated` | T |
| `thread/attachment/updated` | T,attachmentId:s,attachmentType:s,identityKey:s,operation:`created/deleted` |
| `thread/goal/updated` | T,goal:ThreadGoal |
| `thread/goal/cleared`、`thread/queue/changed` | T |
| `project/changed` | projectId:s,changeType:`created/updated/deleted` |
| `thread/project/updated` | T,projectId:string/null |
| `thread/environment/connected`、`disconnected` | T,environmentId:s |
| `thread/settings/updated` | T,threadSettings:ThreadSettings |
| `thread/tokenUsage/updated` | T,U,tokenUsage:ThreadTokenUsage |
| `turn/started`、`turn/completed` | T,turn:Turn |
| `hook/started`、`hook/completed` | T,run:HookRun |
| `turn/diff/updated` | T,U,diff:s |
| `turn/plan/updated` | T,U,plan:PlanStep[] |
| `item/started` | T,U,item:ThreadItem,startedAtMs:i |
| `item/completed` | T,U,item:ThreadItem,completedAtMs:i |
| `item/autoApprovalReview/started` | T,U,action:GuardianAction,review:GuardianReview,reviewId:s,startedAtMs:i |
| `item/autoApprovalReview/completed` | 上記＋completedAtMs:i,decisionSource:`agent` |
| `autoApprovalReview/strictReviewRequired` | T,U,startedAtMs:i |
| `item/agentMessage/delta`、`item/plan/delta` | T,U,I,delta:s |
| `command/exec/outputDelta` | processId:s,stream:`stdout/stderr`,deltaBase64:s,capReached:b |
| `process/outputDelta` | processHandle:s,stream:`stdout/stderr`,deltaBase64:s,capReached:b |
| `process/exited` | processHandle:s,exitCode:int32,stdout:s,stderr:s,stdoutCapReached:b,stderrCapReached:b |
| `item/commandExecution/outputDelta`、`item/fileChange/outputDelta` | T,U,I,delta:s |
| `item/commandExecution/terminalInteraction` | T,U,I,processId:s,stdin:s |
| `item/fileChange/patchUpdated` | T,U,I,changes:FileUpdateChange[] |
| `serverRequest/resolved` | T,requestId:string/int64 |
| `item/mcpToolCall/progress` | T,U,I,message:s |
| `mcpServer/oauthLogin/completed` | name:s,success:b |
| `mcpServer/startupStatus/updated` | name:s,status:`starting/ready/failed/cancelled` |
| `mcpServer/event/stream/notification` | subscriptionId:s,notification:{method:s,params:OpaqueJson} |
| `account/updated` | 空のrequired集合、optional authMode／planTypeは有限enum |
| `account/gatewayOAuth/changed` | providerId:s,status:`notReady/started/succeeded/failed` |
| `account/rateLimits/updated` | rateLimits:RateSnapshot |
| `app/list/updated` | data:AppInfo[] |
| `remoteControl/status/changed` | installationId:s,serverName:s,status:`disabled/connecting/connected/errored` |
| `externalAgentConfig/import/progress`、`completed` | importId:s,itemTypeResults:ImportResult[] |
| `fs/changed` | watchId:s,changedPaths:string[] |
| `item/reasoning/summaryTextDelta` | T,U,I,summaryIndex:i,delta:s |
| `item/reasoning/summaryPartAdded` | T,U,I,summaryIndex:i |
| `item/reasoning/textDelta` | T,U,I,contentIndex:i,delta:s |
| `thread/compacted` | T,U |
| `model/rerouted` | T,U,fromModel:s,toModel:s,reason:`highRiskCyberActivity` |
| `model/verification` | T,U,verifications:`trustedAccessForCyber`[] |
| `modelProvider/authRecoveryStarted`、`Completed` | T,U,message:s,provider:s |
| `turn/moderationMetadata` | T,U,metadata:OpaqueJson |
| `model/safetyBuffering/updated` | T,U,model:s,reasons:string[],useCases:string[],showBufferingUi:b |
| `warning` | message:s |
| `guardianWarning` | T,message:s |
| `deprecationNotice`、`configWarning` | summary:s |
| `fuzzyFileSearch/sessionUpdated` | sessionId:s,query:s,files:SearchResult[] |
| `fuzzyFileSearch/sessionCompleted` | sessionId:s |
| `thread/realtime/started` | T,version:`v1/v2/v3` |
| `thread/realtime/itemAdded` | T,item:OpaqueJson |
| `thread/realtime/item/started`、`item/completed` | T,item:RealtimeItem |
| `thread/realtime/item/transcript/delta` | T,itemId:s,delta:s |
| `thread/realtime/transcript/delta` | T,role:s,delta:s |
| `thread/realtime/transcript/done` | T,role:s,text:s |
| `thread/realtime/outputAudio/delta` | T,audio:{data:s,numChannels:uint16,sampleRate:uint32} |
| `thread/realtime/sdp` | T,sdp:s |
| `thread/realtime/error` | T,message:s |
| `thread/realtime/closed` | T |
| `windows/worldWritableWarning` | extraCount:uint,failedScan:b,samplePaths:string[] |
| `windowsSandbox/setupCompleted` | mode:`elevated/unelevated`,success:b |
| `account/login/completed` | success:b |

optional fieldもpresentなら型検査する。特にthreadName、projectId、turnId、error、reason、details、targetItemId、environmentIdなどのnullableとoptionalを混同しない。

補助型は、今回の固定公開structureの同名definitionを開発時根拠として、rrxの有限型へ個別に転記する。転記範囲は上表、11 server request、送信6 method、7 responseから到達する**宣言済みproperty／required／enum／型／数値域の全閉包**であり、requiredだけの投影に縮めない。出所definition→rrx型→decoder→consumer対応表を実装成果物に含める。runtime生成schemaをvalidatorとして実行する方式は禁止する。

内部discriminatorは少なくとも次を有限化する。

- ThreadStatus：notLoaded／idle／systemError／active。activeはactiveFlags配列を要求。
- GuardianAction：command／execve／writeStdin／applyPatchApproval。各枝のcommand、argv、cwd、processId、stdin、approvalId等を枝別検査。
- GuardianReview status：inProgress／approved／denied／timedOut／aborted。
- HookRun：eventName、executionMode、handlerType、scope、source、status、entries.kindの各有限enum。
- FileSystemPath／SpecialPath、AskForApproval、SandboxPolicy、SessionSource／SubAgentSource：各oneOfのtagと枝別必須構造。
- RealtimeItem：共通id／realtimeSessionIdに加え、realtimeSessionStarted／transcriptSegment／bemItemPromoted／realtimeSessionClosedの枝別構造。presentationのwholeItem／inlineMarkdown／inlineVisualizationも検査。
- ImportResult、AppInfo、SearchResult、ThreadGoal、ThreadSettings、token usage、permission profile：配列要素・map値・nullable参照を再帰検査し、objectだけでは通さない。

`OpaqueJson`は、固定契約が意図して任意JSONとするarguments、MCP params／meta／content要素／structuredContent、moderation metadata、realtime itemAdded.item、webSearch.results要素、outputSchema等の明示した位置だけに限定する。未知method・item.type・status・discriminatorをopaqueへ落とさない。opaqueもstrict JSONと既存size／depth上限の対象である。

**ThreadItem全19枝**

全枝でid／typeを要求する。追加必須は以下とする。

| type | 追加必須 |
|---|---|
| userMessage | content:UserInput[] |
| hookPrompt | fragments:{hookRunId:s,text:s}[] |
| agentMessage | text:s |
| functionCallOutput | name:s,output:FunctionOutput |
| plan | text:s |
| reasoning | なし。summary／contentがあればstring[] |
| commandExecution | command:s,commandActions:CommandAction[],cwd:s,status:`inProgress/completed/failed/declined` |
| fileChange | changes:FileUpdateChange[],status:`inProgress/completed/failed/declined` |
| mcpToolCall | arguments:OpaqueJson,server:s,tool:s,status:`inProgress/completed/failed` |
| dynamicToolCall | arguments:OpaqueJson,tool:s,status:`inProgress/completed/failed` |
| collabAgentToolCall | agentsStates:typed map,receiverThreadIds:string[],senderThreadId:s,status:`inProgress/completed/failed/interrupted`,tool:有限9種 |
| subAgentActivity | agentPath:s,agentThreadId:s,kind:`started/interacted/interrupted/completed` |
| webSearch | query:s |
| imageView | path:s |
| sleep | durationMs:uint64 |
| imageGeneration | result:s,status:s |
| enteredReviewMode／exitedReviewMode | review:s |
| contextCompaction | なし |

UserInputはtext／image／localImage／audio／localAudio／skill／mention。textはtext文字列、imageはurlまたはfileId、他は各path／url／nameを要求する。CommandAction、FunctionOutput content、WebSearchAction、ImageGenerationFailureも入れ子の有限枝を検査する。imageGeneration.statusは仕様上stringであり、rrxが根拠なくenumを発明しない。

agentMessage.phaseはoptional nullable、presentならcommentary／final_answer。欠落／nullはwire合法だが回答承認には使わない。delivery／questionsもwire構造を検査した後、collectorの業務条件で採用可否を決める。

本番処理順はstrict envelope→method→payload全構造→所有フィルタ→consumer。正常toolは構造検査後に回答から除外できる。malformed foreign frameはProtocolError。S/P握手では全server requestとthread系通知を拒否し、正常な全体通知とrate通知だけを数えて続行する。

server requestは既知11種すべてのparamsを先に検査する。本番で対応する承認2種だけ既存ledgerへ渡し、残り9種は既存の非対応拒否を維持する。M0で権限応答能力を増やさない。

### 8. Schema gateと実握手

Schema helperは`app-server generate-json-schema --experimental --out <new-dir>`。生hash一致を要求しない。

4 envelopeに加え、Initialize／Account／Rate／Environment／ThreadStart／TurnStart responseの生成ファイルを読み、§7のfinite contractと比較する。file名はfixture manifestで固定し、任意探索で同名definitionを拾わない。

出力は自uidの新規0700 directory、symlink禁止、1024項目、16 MiB、深さ4、各読取file1 MiBまで。JSONは深さ64、262144 nodes、文字列64 KiB、object／array4096。local refは同一fileの`#/definitions/`のみ、ref深さ16・訪問512。循環・未解決・外部refを拒否する。

**適合意味**

- method枝は単一enumの一意枝。入れ子oneOfもtagの単一enumまたは相互排他的構造で一意に選択する。
- `$ref`は解決してから比較する。allOfは全制約の積、anyOfはunion、oneOfはちょうど1枝。type配列はunion、nullableは明示null枝だけ。
- boolean schema／型なしschemaを任意型扱いするのは、rrx契約が明示OpaqueJsonとする位置だけ。
- enum、integer／number、minimum／maximum、minLength、items、additionalPropertiesを含めて意味比較する。int32／uint16／uint64等のwire域を保ち、曖昧なformatを無制約へ落とさない。
- 送信値は実producerが生成可能な値集合のすべてがCLI宣言に適合し、CLI requiredを満たす。
- 受信側はCLIが宣言する既知fieldの型・nullability・enum・枝がrrx decoderの受理範囲を越えないことを要求する。業務上の追加条件は別に検査する。
- 未知の宣言field／method／discriminatorを無条件受理しない。未使用ClientRequest methodは今回送らないため対象外だが、全notification／server requestは有限表に含まれることを要求する。

`initialized`はschema上methodしか宣言しない。S/P／本番producerを`{"method":"initialized"}`へ変更し、`params:{}`を送らない。これをschemaの例外で救済しない。

送信契約はinitializeのclientInfo／experimentalApi、account/readのrefreshToken=false、environment/statusのlocal ID、rate readの空params、thread/startのcwd／environments／任意model／Reviewer sandbox、turn/startのthreadId／text input／任意effortを、実producerと共通の型で表す。

responseは以下も対象にする。

- Initialize：4必須文字列と既存platform／absolute path業務条件。
- Account：requiresOpenaiAuth boolean、nullable accountのapiKey／chatgpt／amazonBedrock枝。資格付けはchatgptのみ。
- Environment：ready／pending／disconnected／unknown。資格付けはreadyのみ。
- ThreadStart：thread、cwd、modelProvider、model、approvalPolicy、approvalsReviewer、sandbox等の必須構造。
- TurnStart／Turn：id、items、statusとoptional error／timestamps。
- TurnStatus：completed／interrupted／failed／inProgress。
- CodexErrorInfo：既知15文字列、4 HTTP object枝、activeTurnNotSteerable枝。HTTP statusはnullable uint16、turnKindはreview／compact。
- RateLimitReachedType：既知5値とnullable。

握手はinitialize→initialized→account/read→environment/status→rate read。schema宣言だけではRPC成功を代替しない。schema適合でも`-32601`なら拒否する。S/P metadataをquota poolへ書かない。

### 9. Rate契約とquota closure

Claude decoderは`NotRateEvent／Classified／Unparseable`。bucketは既存5種、statusはallowed／allowed_warning／rejected。optional resetsAtはnullまたは秒int64でmilliseconds変換overflowを拒否し、utilizationはnullまたは有限0〜1。欠落・wrong type・未知enumはUnparseable。

Codexはresponse envelopeとsnapshotを分ける。rateLimits objectは必須。rateLimitsByLimitIdはoptional nullable typed map、最大32、keyは1〜128 bytes。両方presentなら両方を構造検査し、選択しない側のmalformedを隠さない。

limitIdはabsent／nullなら既存fallback、present non-stringは拒否。rateLimitReachedType未知値を未到達へ変換しない。windowのusedPercentはint32であり、0〜100のschema制約を発明しない。negative値はusage unknown、100超は割合値を捏造せず既存exhaustion判断を維持する。既知credit／spend到達値をAvailableへ変換しない。

`{"rateLimits":{}}`、全window absent/null、合法な空mapはwire合法で、usable-window件数0、availability／usage unknown。これらをmalformedやAvailableにしない。metadata構造合格とusable-window観測を別記録し、既存quota policyを緩めない。wrongtyped snapshotは拒否する。

selfcheckはClaudeのbucket×status×optional項目、Codex snapshot／map、合法empty、wrongtype／未知enum／overflowを対で検査する。成功tokenをinstallationへ渡す。selfcheckはCLI live互換の証明ではない。

`unclassified_limit`とそのcapacity／terminal分岐を廃止し、解析不能をProtocolErrorへ一本化する。既存terminal_capacityの意味をrate unknownで代用しない。

quota Waitはcanary入力0、Parked。Admit後Known不合格は既存no-dispatch／no-Session／inventory一致／revoke条件を満たしたclosureでlease・自probe ownershipを解放し、必要な期限／CASを更新する。UnitはRetired／Refused、Taskは`explicit retry required`のWaiting。UnknownはHeld、leaseを勝手に解放しない。

普通Admitとrecovery Admitを別controlsで検査する。観測非更新のassertionはwindow／availability／usage／backoffのcanary反映0であり、quota_pools全write 0ではない。poolがcanary exhaustionを学習しないため後続TaskもAdmitされうることを明記し、M0で自動復旧を追加しない。

### 10. Terminal、tail、identity、監査

helperはprotocol、stop方式、exit、pipe EOF、tail、signal、leader reapを別記録する。必須観測後にstdin close、5秒の自発終了猶予、必要ならgroup signal、2秒drain、10秒reap。期限を延ばして解決しない。

Confirmedは生成結果既知＋両pipe EOF＋leader reap既知。qualifiedはConfirmed＋protocol pass＋合法tail＋自発exit 0または猶予後signal。未知tail／partial JSON／自発nonzeroはKnown不合格。pipe／reap不明はUnknown。

Claude成功後の新規tail許容集合は、strict-validなrate event、auth_status、stream ping／message_stop、system state=idleのみとする。完全同一UUIDの再送はdedup可能。新しいassistant／user／result／task／tool／control要求はtail不合格。late rate rejected等はcanary成功を覆し、Known不合格へ分類する。Codex tailは握手で許す正常全体／rate通知だけである。

Native programはcanonical path、dev／ino／size／mtime／ctime、SHA-256をVersion intent前に記録し、Native helper／transport前にhash、spawn直前にstatを再検査する。Git helperはCLI identityの再hash対象外。execとの非原子的な窓、wrapper内部、escaped descendantsは未証明として残す。

監査は有限語彙、method件数、digest、nullable観測状態だけ。account本文、codexHome、nonce、canary本文、raw frame、stderrを保存しない。版数はexit 0、完全捕捉、256 bytes以下、単一行、制御文字なし。安全に記録できない文字列はdigestを記録し、版数値によって不合格にしない。

### 11. Legacyとcollectorの完了境界

L1の入口拒否予定はNativeSessions legacy start、NativeAdapter start、Codex legacy startであり、helper／Session生成前にCapabilityUnavailableを返す。しかしcoverage保存が成立するまでL1を完成・承認可能とは扱わない。

必須台帳は各test IDについて、旧producer／consumer／検出する副作用・権限・custody、移植／残置／削除、理由、移植先実producer、positive／negative assertionを記録する。87件はsession 60、custody 20、ownership 7として照合する。その他Native／adapter／workflow対象も別集計する。移植N＋残置M＋削除Kが対象件数と一致し、未分類0を要求する。

現行installedレーンには最初のExecutorを通るpositiveがある。一方、Reviewer実Gitキャンセル、immutable review、resume／checkpoint等にはM0のSC-Nまでの鎖では生成できない経路がある。例は`codex/session.rs`の`actual_registered_review_git_cancel_reaps_owned_hook_before_failed_without_native_spawn`と、Nativeの`readonly_review_profile_is_selected_in_the_actual_fixture_launch`。synthetic入力をauthority代替として移植したことにしない。

custody mechanical／parser単体の残置候補はあるが、public入口拒否で到達不能になる鎖を残しただけではproduction coverage維持にならない。**L1の全positive移植とReviewer境界producerは限定未達として残る。PR2〜4のない現行範囲で埋まったとは認定しない。**

collectorはvalidated itemを受ける。id／type／text欠落など前段でProtocolErrorになる枝は削除・再配置し、到達不能コードを隠さない。phase欠落／null、非final、非対応delivery、質問付きmessage、複数final、内容改変、overflowは回答usabilityの業務判定として残す。

既存synthetic childの私的wrapperが`--exact … --ignored`で実childを起動するpositiveを保持する。test本体をskipする許可にはしない。

### 12. Fixture、因果検証、owner-run

CI schema positiveは今回提供された公開生成structureから、使用6 request、initialized、全83通知、全11 server request、6種responseとその到達definition閉包を保った有限縮約fixtureとする。合成の最小schemaだけで代替しない。

sourceには入力HEAD d4526759のstatic資料のSHA-256
`458d41061922f54a84817b6c9af7fb448dd7ce4ecdff74a40e4b611730b9c621`、今回追加提供されたstructureの出所を記録する。fixture実体のdigestは作成後に計算・保存する。今はfixture未作成でありdigest／適合結果を捏造しない。static.jsonの古いbaseline、空notification branch位置、ClientNotification／response欠落も補完対象とし、live各項目はnot_runを保持する。

| control群 | positive／negativeと必要mutant |
|---|---|
| Startup | 全alias合格／1 alias拒否、gate除去、journalからpass復元、P省略。 |
| Strict | 全83通知・19itemの正常形、各required欠落・wrongtype・nested tag改変、自／foreign双方。正常ignored／tool維持。所有filter先行mutant。 |
| Schema | 実fileの枝複製・削除、required追加、型・nullable・enum・数値域・ref先改変、無関係同名property、未解決／循環ref。 |
| R1 | 合法empty／usable window／wrongtypeを区別。foreign rate先drop、unknownをAvailable／not_observed化、selfcheck呼出し除去mutant。vector mutationは試験用authority seamを作らずsource差分として実施。 |
| Quota | Wait、普通Admit、recovery Admitそれぞれのモデル入力数、lease／probe ownership／期限／CAS、観測非更新。 |
| Budget | 実producerによる全provider×roleの境界／+1、Git13→9仮定mutant、Probe末尾省略mutant。Reviewer未接続は未完了。 |
| IPC | Claude ProbeとCodex Schema／Handshakeそれぞれ、shim呼出し、Grant0、helper以外効果追加0。 |
| Custody | stop／Drop／panic／forced deathの別期待、stdin close、tail打切り、reap不明をConfirmed化するmutant。 |
| Migration/path | new、v1〜9、v10、v11、current、同時initializer、guard欠落・旧reference新表混入、alias traversal／symlink／ownership／境界cleanup。 |
| Consumer | RuntimeMetadata／ShutdownSite wire、型付きinstallation refusal、collector usability、synthetic child実呼出し。 |
| Legacy | 完全台帳と移植positive。入口拒否だけのtestへの置換では合格しない。 |

本番負例はbaseline差分とterminal disposition、登録前負例はAC-08、SはUnit0を検査する。

両OSで既存CIのfmt、clippy、workspace test、debug／release buildを実施する。まずローカルでまとめて検証し、試行pushや根拠のないrerunをしない。実CLI資格付けをCI fixture結果で代替しない。

owner-runは設計承認・実装後、新しいrun ID／状態directory、実設定のClaude Executor＋Codex Reviewer、1 Project／1 Active Task／QUICKで1runだけ行う。selfcheck→全S→Claude P→Worktree→Implement success→SC-Nを確認する。rateは`not_observed／empty_valid／usable`を明示し、未証明一覧を出力する。失敗runは保存・再利用拒否。既存のPASS時cleanup規則を維持し、追加cleanup工程を受入条件へ増やさない。

### 13. 一括有限承認表

| ID | 全体設計承認に含める決定 | 現状 |
|---|---|---|
| A1 | quota Admit後、準備ごとのClaude canary 1turn。費用・履歴・既存hook・quota消費、Known failureの明示retry帰結を含む。 | 未承認 |
| A2 | Probeの専用UUID、固定文面、Executor plan mode、tool／権限応答なし。既定modeや隔離を証明しない。 | 未承認 |
| A3 | legacy入口拒否とpositive coverage維持の一体変更。 | **限定争点未解決。現行M0でReviewer等のproducer不足。完成案ではない。** |
| A4 | S全alias gate、版12 migration、有限journal／unknown上限、安全なdigest path、停止wire変更。 | 未承認 |
| A5 | 本番strict decoder、finite item／notification／nested variant、UUID改変拒否、rate解析不能終端。 | 未承認 |
| A6 | schemaの未知method／member／discriminatorで起動前拒否。raw hash pinなし。CLI拡張時は契約見直しまでBLOCKED。 | 未承認 |

R1、M4 runtime merge、public sandboxの作成・使用は決定済み。執筆者は今後の実装者・レビュアーを承認しない。

### 14. pin 設計との差分一覧（orchestrator の転記）

| 項目 | pin 設計 | M0 |
| --- | --- | --- |
| 許可根拠 | 単一の厳密版数 | 実観測。番号は記録のみ |
| 版数検査の消費者 | phase 4、legacy 3 | すべて削除 |
| legacy 経路 | ピンの下で起動 | A3 未確定。旧 API の機能廃止か全経路への probe 配線かを確定する |
| 起動時 | 何もしない | 層 S（非モデル、拒否専用） |
| 準備の helper | Version＋Git×13 | Codex は＋Schema＋Handshake、Claude は admission 後に Probe |
| 登録前の観測 | `--version` | 握手、Claude の `system/init`／`result`、Codex の schema 構造 |
| 実行ファイル | canonical パス | stat と sha256 の束縛 |
| 解析 | top `type` のみ有限、未知通知を無視 | 内部 variant、method、payload まで有限 |
| rate decoder | 未知を `None` | 型分離、解析不能は Probe でも本番でも fail closed |
| 宣言 | `cli_version` 必須 | 新 profile、installation で旧宣言を拒否 |
| DB | 版 11 | 版 12（S の journal） |

### 15. この稿の完了境界

A3 の記述は未解決の旧案であり、旧 API の廃止・テスト削除・機能維持のいずれも承認済みではない。M0 の対象として現に提供する振る舞いと、廃止する振る舞い、M3 で新たに提供する Reviewer の振る舞いを区別して、必要な安全性検証を具体化する。Scope の訂正は独立確認とオーナー設計承認へ戻す。
