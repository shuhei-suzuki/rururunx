# macOS 最小一本道 M0: capability probe 設計案

状態: **役割入れ替え改訂（2026-10-15）はオーナー承認済み（固定HEAD `2a9114bc`、B1〜B4・B2a許可）。U6のB2a実現はBLOCKED（P契約変更待ち）。B2bは提案・オーナー承認待ち（2026-10-18、§13）** / `c6413082`の全体設計承認とA1〜A6は維持する。継承設定で証拠1を満たすには対象feature全件の明示false、web_search="disabled"、全MCP serverのenabled=falseが必要で、既定設定は拒否となる見込み（config/readによる既定値の実体化は未確認）。M0-U1（Claude strict wire decoder／metrics、O03、I01〜I04、共有queue予算補正）は実装・限定検証・独立修正確認済み（`doc/verification/macos-minimum-m0-u1.md`）で、役割入れ替え後はReviewer側のdecoderとして使う。それ以外のM0本番実装・検証は未着手。

オーナー全体設計承認（2026-10-13）: 固定HEAD `c64130828453cb38b742c9b31d08d9e488e85ee1` の3文書を承認し、必須検証を維持してM0実装へ進むとの明示回答を取得した。A1／A2／A4／A5／A6を含む設計承認であり、実装・実CLI適合・controls／mutants・両OS／実Mac受入・実装レビュー・マージ条件の免除ではない。

役割入れ替え改訂（オーナー決定 2026-10-15）: rururunxの役割をオーナーの日常開発に合わせ、**CodexをExecutor、ClaudeをReviewer**とする。`c6413082`の承認はClaude Executor＋Codex Reviewerを前提としていた。A1〜A6の他の決定、R1、M4のruntime自動merge、public `rrx-e2e-sandbox`、安全規則、M0→M5対応、M0実Mac受入前にM1へ進まない順序は変えない。再承認の対象は§0の差分一覧と§13のB1〜B4・B2aだけである。本稿は執筆成果であり、レビュー票・承認・実装・検証結果ではない。

Codex契約改訂（2026-10-16、**オーナー承認済み**）: インストール済みCLIがcodex-cli 0.160.1から0.162.1へ上がり、オーナーは契約をインストール版へ合わせること、版数は今後も上がることを示した。本改訂はCodex有限契約を、通常のCLI更新で毎回止まらない形（strict／toleratedの位置二分類）へ改め、契約basisを0.162.1へ移し、strict位置に触れる更新の再生成手順を定める。差分は§0a、決定は§13のC1〜C4であり、A5／A6と§7免責表を広げる項目を明示する。未決だった点は、Q1（rrxが送らないoptional propertyの送信params追加）とQ2（Turnへのnullable property追加）をstrictのまま、Q3（inventoryは既存述語のまま）・Q4（許容追加の上限はThreadItem type 32・通知64）・Q5（CodexErrorInfo Unclassifiedのfailedは Unknown→Held）を提案どおりとする。本改訂に依存するU4実装（Schema gate、Codex wire decoder、ConfigRead／McpServerStatusList）はこの承認後に進める。

公開sourceのbaseline: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。過去の修正入力: `d452675989df799d29931c7d4f19d73d6b387550`。前回の局所修正はclean固定HEAD `2c9fb68f6f6544ef9c8b3b53c7f936aec2e354e2`から設計草案へ取込済み。今回のR5後補修本文案の入力はclean固定HEAD `0fadf545c45558a05d89377bcb073009875bdb01`であり、設計草案へ取込済み・固定HEAD b746dfd4の補修diffはOpus承認済みである。baselineと各執筆入力HEADを混同しない。

R5結果の事実転記: SolはAPPROVE（確認済required 0）。固定argvでの要求は記録されているがserved model／effortは未報告。OpusはREQUEST CHANGES（確認済required 2: R5-O-01／R5-O-02）、runner ok／model_match=match、報告モデルは`claude-opus-5-5`のみ、effortは未報告。CLI完了と本文の判定は別であり、両レビュー承認済みとはしない。今回の補修はSolが執筆し、指摘側Opusが修正diffだけを独立確認する。無変更範囲のSol R5全文レビューを重複させず、両系の独立レビュー条件は維持する。適用状態はorchestratorが事実転記した。全体オーナー設計承認・実装・台帳・fixture・Linux／macOS／live検証は未完了。

R6局所補修の執筆・取込記録: 専用read-only Sol contextの14差分をorchestratorがsourceと照合して取込済み。要求モデルは`gpt-6.1-sol` / high、served model／effortは未報告。修正diffのOpus独立確認と全体オーナー承認は完了。[R5結果の公開記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142182746)、[R6開始記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142322693)。

R6独立確認結果の事実転記: 固定HEAD `b746dfd4c1c0de49fe07f173f40b2c236d2c97dd` の補修diffにOpusがAPPROVE（確認済み必須欠落0）。runner ok、model matchで`claude-opus-5-5`のみ報告、effort未報告。R5-O-01／R5-O-02は解消し、Sol R5の既承認範囲と今回の補修diff確認を合わせて扱えるとの判定。任意の検証補足は実装台帳で扱い、追加の必須設計条件やレビュー反復にはしない。レビュー後、#94のmain `b8344ab951fa24f545174917eee7be067284c625`へrebaseし、取込後HEAD `deb9909c661e98c12a7e7a8207c9eb6a80bad0d0`でも3文書のbyte不変を確認した。#94はマージ済み・同SHAの両OS CI成功であり、固定依頼文のOPEN記述は過去の状態。M0本番実装・M0検証は未完了。全体オーナー設計承認は下記の固定HEADで完了。[R6結果記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142873096)。

既存本文の修正執筆履歴: Codex CLI に `gpt-6.1-sol` / high を固定 argv で要求。CLI の実使用モデルと effort は未報告（一致は未確認）。独立レビュアーの context は執筆へ流用していない。これは独立レビュー・実装・実機検証の証拠ではない。 今回のSol R5局所修正は専用read-only contextの出力42差分をorchestratorが照合して取り込んだ。本番rrxのCLI資格付け証拠ではない。

R3後の修正執筆は新規の専用2 context。契約補正と欠落節の復元は Sol、owner root 条件は Claude Code に `claude-opus-5-5` / high を要求した Opus が担当した。Opus は同モデルを報告し、effort は未報告。両出力の原文・hash・適用箇所を私的記録に保存し、orchestrator が全終了後に結合した。追加のモデル・承認情報は事実の転記であり、レビュー票ではない。

履歴転記: Sol／Opusのround 3修正差分レビューはともにREQUEST CHANGES。

Sol R4は正式なREQUEST CHANGES。Opus R4はCLI child exit 0でもmodelUsageにOpus＋Haikuがあり、runner exit 9／model_mismatchで必須レビュー未完了。参考本文は承認・票に使わず、orchestratorのsource照合で確認した差分だけを局所補修する。修正後はmodel一致のOpus独立確認と全体オーナー承認が必要である。[R4結果の公開記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6140066238)。

オーナー確定事項（2026-10-13）: 番号 pin 撤廃と capability probe、R1（decoder selfcheck＋実 event strict、未観測は未証明）、M4 の runtime 自動 merge、public [rrx-e2e-sandbox](https://github.com/shuhei-suzuki/rrx-e2e-sandbox) の作成・使用。再承認を求める事項ではない。sandbox は初期 README のみで、fixture・CI・main 保護は未設定。

**macOS最小一本道M0：設計修正全文**

基準と状態は要件に同じ。本稿の型名は実装予定の契約名であり、存在する実装・取得済み証拠を意味しない。

### 0. 役割入れ替えの差分一覧（`c6413082`比）

オーナーが再承認するのは本表の差分だけである。表にない決定は`c6413082`の承認内容のまま変えない。

| ID | 項目 | `c6413082` | 本改訂 | 承認 |
|---|---|---|---|---|
| D-01 | 役割 | Claude Executor＋Codex Reviewer | Codex Executor＋Claude Reviewer | B1 |
| D-02 | M0で提供するExecutor | Claude（最初のExecutorまで） | Codex（最初のExecutorまで、§8a）。aliasはproviderごとに登録し、Claude aliasはReviewer用に受理する。TaskのExecutorがClaude provider aliasへ解決される場合は§8a(0)の拒否点で型付き拒否（helper・Unit・Held・quota Admit 0） | B1 |
| D-03 | Codexモデル実行 | M0範囲外。CodexはS／Pの握手だけ | Executorのthread/turn実行をM0へ入れる（§8a） | B4 |
| D-04 | Codex approval方針・sandbox・ツール面 | 規定なし。本番Executorのthread/startはどちらも送らずCLIの継承設定に従う | 承認要求を出さない固定approval方針、Worktree＋Unit temp／output／cacheだけのworkspace-write、network無効、config上書きなし、応答一致確認、turn/start前のMCP・web search・dynamic tool・collab無効／不在の肯定証拠（得られなければBLOCKED）、許可item集合外はUnknown→Held、rrxの自動付与0、turn/start応答待ちを含む全受信経路でserver request着信はUnknown→Held、work軸と物理custody軸の分離（§8a） | B2（制限方向の上書きの可否はB2a） |
| D-05 | Claude P（A1のcanary、A2のProbe条件） | Executor準備としてM0で本番化 | Reviewer準備の検査として定義を維持し、本番producer・実境界・受入はM3。M0はProbe producerを追加しない | B3 |
| D-06 | Probe argvのplan mode | Executor argvへplan modeを追加 | Reviewerの本番argvが既に`--permission-mode plan`を含み、Probeはそれをそのまま使う | B3 |
| D-07 | SのClaude argv | `command_argv(claude, Executor, …)` | `command_argv(claude, Reviewer, …)`。plan mode付きの実argvをSで確認 | B1 |
| D-08 | AC-13 owner-run | 全aliasのS、Claude P、Claude Implement、SC-N | 全aliasのS、Codex P、Codex Implement＋retained capture、SC-N。ClaudeはSとReviewer argv | B4 |
| D-09 | 実producerの予算境界 | Claude Executor（B237、Prepared252、初回入力255） | Codex Executor（B230、prefix246、初回入力256。§8a(3a)のsetup2行を含む）。Claude Reviewerは純粋計算で、実境界はM3 | B3（B230への引下げはB2） |
| D-10 | controls | Claude Probe系をM0で実施 | CL-1／CL-3／CL-6／CL-8／CL-9、Q-1のProbe配置、Q-2、I-PのProbe分をM3へ移す。Codex Executor群EX-1〜EX-7（EX-3bを含む）を追加 | B4 |
| D-11 | spawn経路 | 既存spawn | P4（`79e4d273`）の`rrx-spawn-hygiene`消費関数だけ。U0でrebaseする（§12a） | 事実反映（承認不要） |
| D-12 | M0-U1差分 | Executor側のClaude decoder | 同じ差分をReviewer側（S initialize、M3のSession）のdecoderとして保持 | 事実反映（承認不要） |
| D-13 | Codex送信method | 6 method＋initialized | Executorだけconfig/read・mcpServerStatus/listを加えた8 method＋initialized、応答8種（§7・§8・§8a(3a)） | B2 |

変えないもの: capability probeによるpin撤廃、R1、S全alias gate、migration v12とjournal、private path、Codex finite wire・schema gate（A5／A6。§0aのC1〜C4で改訂済み）、rate契約、旧API機能廃止（A3）と#19、auth／CAS／owner／marker／fencing／process custody、禁止事項（Task.version authority、test専用authority constructor、SQL成功seed、`allow(dead_code)`、skip／ignore／直列化、timeoutだけの修正、warning抑止）、M0→M5対応とM1開始条件。

### 0a. Codex契約改訂の差分一覧（承認済みA5／A6・`c44baad3`比、2026-10-16 承認）

オーナーが承認を判断するのは本表の差分だけである。「広げる」はrrxが受理する範囲またはA6／§7免責表を承認済みより広げる項目、「狭める」は安全側への追加である。表にないA5／A6の内容（strict JSON、所有フィルタ先行禁止、closed object／key-tag／RPC判別key、raw hash pinなし、static catalogをruntime validatorにしない）は変えない。

| ID | 項目 | 承認済み（A6、0.160.1 catalog `c44baad3`） | 本改訂 | 広げる／狭める | 承認 |
|---|---|---|---|---|---|
| E-01 | Schema gateの比較 | 未知の宣言property／method／discriminatorは全位置で起動前拒否 | strict位置は完全一致、tolerated位置は§8の規則内で増減を許容（§7・§8） | 広げる | C1 |
| E-02 | 新しい通知method | 拒否 | params閉包が構造をもつstrict到達definitionへ到達しなければgate許容追加（上限64。スカラー葉への到達は可、§8規則2）。実行時は内容不使用 | 広げる | C1 |
| E-03 | 新しいThreadItem type | 拒否 | gate許容追加（上限32）。所有turnでUnknown→Held（成功0）、foreignはid／type検査後に除外 | 広げる（結果はHeld） | C1 |
| E-04 | 新しいserver request method、strict位置の新property、used enumの新値 | 拒否 | 拒否のまま | 変更なし | C1 |
| E-05 | 通知payloadの実行時検査 | 全83通知の全payloadを型検査 | 21通知（S-notify）は全payload、4通知（S-held）はmethodと所有キー、他59通知は所有キーだけ型検査し残りは有界strict JSON（§7） | 広げる | C3 |
| E-06 | ThreadItem内容 | 19枝すべての内容を型検査 | agentMessage・commandExecution・fileChangeだけ内容を型検査。他16枝はid／typeだけ（所有turnの許可集合外は従来どおりHeld）。既存のterminal原本digest（`terminal_sha256`）はtolerated item内容を含むTurn全体のdigestのまま維持し、既存の原本同一性照合（receipt全体の構造等価・本文の完全一致）には含めたまま、意味上の分類・判断には使わない例外として明記する（§7） | 広げる | C3 |
| E-07 | Thread／TurnError | Thread全体、ThreadExtra明示map、TurnError全体を型検査 | Thread core（id、cwd、modelProvider、status、turns）とTurnError（misalignment内容を除く）だけ型検査 | 広げる | C3 |
| E-08 | ConfigRead `config` | 未確定（catalog不在でBLOCKED） | 判定path（web_search、features.<name>、mcp_servers.<name>.enabled）だけを型検査。`Config`の開放領域（additionalProperties=true。`permissions`等）を§7免責表へ追加し、他の宣言property（sandbox_mode、approval_policy等）はtolerated（§7・§8）。M0のconsumerは判定pathだけを読む共有述語で、`permissions`や他の宣言propertyを読む既存`DecisionPolicy`（構築と`verify_configuration`のprofile比較）はM0 Executor経路で使わない（§8a(3a)） | 広げる（免責表に1行追加） | C3 |
| E-09 | McpServerStatusList | 未確定（catalog不在でBLOCKED） | data、nextCursor、各serverのname・tools（空であること）・toolsErrorだけ型検査。他とTool値はtolerated | 広げる | C3 |
| E-10 | §7免責表 | 15行 | 9行（strict領域内の型なし位置だけ）。外れた7行はtolerated領域に含まれ、Config開放領域1行を追加 | 広げる | C3 |
| E-11 | CodexErrorInfo | 既知15文字列＋5 object枝で閉じる（oneOf） | basisの宣言（anyOf＋`string/object`の開放枝）に合わせ、開放枝だけに合う値は`Unclassified`。failedの所有turnではUnknown→Held、quota／auth／capacityの根拠にしない | 広げる（結果はHeld） | C2 |
| E-12 | MessagePhase | commentary／final_answer | partial_answerを追加。回答に採用しない（commentaryと同じ扱い）。final_answerだけ採用 | 広げる（回答authorityなし） | C2 |
| E-13 | thread/goal/updated | params.threadId＝goal.threadIdをrrx refinementで検査 | tolerated通知のため内容を検査しない（refinement廃止） | 広げる | C3 |
| E-14 | thread/settings/updated | 型検査のみ | 所有threadのapprovalPolicy・sandboxPolicy・cwdがThreadStart応答と異なればturn/start前は拒否、後はUnknown→Held | 狭める | C2 |
| E-15 | 契約basis | 0.160.1 catalog（10 source、268共有definition、`c44baad3…`）。再生成しない | 0.162.1 catalog（12 source、303共有definition、`bbdede32…`）。0.160.1 catalogは履歴として書換えずに保持（§12） | basis変更 | C2 |
| E-16 | 契約の更新 | 「未適合CLI拡張は契約見直しまでBLOCKED」（手順なし） | §8bの再生成・再現確認・出所記録・レビュー手順。strict位置を広げる、または免責／tolerated集合を広げる更新だけオーナー承認 | 手順の追加 | C4 |

0.160.1→0.162.1の実差分（§8bの分類）: toleratedで吸収できるのは`thread/prediction/updated`の追加、`McpServerOauthLoginCompletedNotification.loginId`、`MisalignmentErrorDetails.reviewTarget`、subAgentActivity枝のoptional `model`／`reasoningEffort`。strict位置に触れるため本規則でも再生成が要るのは`TurnStartParams.parentTurnId／rootTurnId`、`Turn.rootTurnId`、`MessagePhase.partial_answer`、`CodexErrorInfo`のoneOf→anyOfと開放枝である。今回のbasis更新でこれらを契約へ取り込む。二分類は更新ごとの再承認を減らすが、strict位置に触れる更新を無承認で通すものではない。

**B2bによる追加差分（提案・オーナー承認待ち、2026-10-18）**: E-08の同じ判定pathを本番だけでなくP Handshakeの不足集合選択にも使う（§8a(3a)、§12a-3）。C1のstrict／tolerated集合・C2の8 method／8 response・C3の免責範囲は広げない。C4の更新規則も維持する。変更するのはPのmethod使用箇所、メモリ上の決定原本、Preparedのcommand導出・同一性契約と有限receiptである。Sにはconfig/readを足さない。このP契約変更はC1〜C4の既承認に含まれず、B2bの別承認が必要。

### 1. Producerからconsumerまでの鎖

| producer／契約 | consumer |
|---|---|
| installationの純粋decoder selfcheckが生成する非公開`DecoderChecked` | Native graphのinstallation。単なる関数呼出しではなく成功値を必須引数にする。 |
| Sのfreshな全alias観測、既知settlement、宣言digest、program identity | メモリ上のstartup gate、Task admission、installed composition。journalから合格を再構成しない。 |
| 静的profile／Frame digest／既存hook宣言検査 | `NativeCompatStatic`。番号比較を含めない。 |
| SAME actorのVersion、CodexではSchema／Handshake、Git×13の閉じた履歴 | prefix qualification、command、no-dispatch、quota計画。 |
| ClaudeのSAME prefix・command・quota Admit・Probe settlement | `ClaudeProbeQualified`。`issue_prepared`が必須とする。Claude Reviewer Pに属し、producerと配線はM3で実装する。 |
| provider別の最終inventory末尾 | Prepared、transport登録、quota closure。ClaudeはProbeを含む末尾、Codexはprefix末尾。 |
| 共通wire decoderが生成するvalidated frame | 所有フィルタ、RunState、quota consumer、answer collector、approval consumer。未検査`Value`を迂回入力しない。 |

Pの順序は次のとおりとする。

- Codex（Executor、M0で本番化）：準備commit→static→Version→Schema→Handshake→Git×13→command→no-dispatch→quota→Prepared→transport→§8aの本番Session。
- Claude（Reviewer、M3で本番化）：準備commit→static→Version→Git×13→command→no-dispatch→quota Admit→Probe→Prepared→transport。M0はこの順序を契約として保持するが、Probe producerを追加しない。

Claude Probeは専用custody slotに保持する。prefixの先頭Versionと13本のGit順序を変えない。no-dispatch／quota admissionは`Prefix`末尾を要求し、Probe intentがまだないことを確認する。Prepared／transportはClaudeの`Probed`末尾を要求する。closureはProbe intentなしならPrefix、既知settlementありならProbed、未確定intentありなら成立させない。

authorityは既存のowner／marker／epoch／actor原本から生成する。Task.versionやjournal DTOから再構成しない。

### 2. Sのcustody、path、journal

S専用custodyは既存`RetainedRawProcess`を保持し、intent commit後にだけspawnする。spawn後のadoptをpipe／identity検査より先に行う。#19のcustody poolを流用しない。spawnはP4の`rrx-spawn-hygiene`消費関数（`spawn_tokio`等）だけを通し、`clippy.toml`の直接spawn禁止と`OwnedProcess::spawn(Command)`の所有移動を維持する。macOSでpre_execにより全spawnがforkする点は#98の残余リスクであり、M0で解消したとは扱わない。

`RetainedRawProcess::pipes`は現在stdout／stderrだけを返すため、stdinを保持・一度だけcloseできる所有型へ拡張する。stdin、stdout、stderrの移動とclose責任をcustody内に一本化し、非同期taskがstdinを失って停止不能になる構造を作らない。

**provider別S手順**

Sはdaemon継承環境と専用の空cwdを使い、既存の設定・認証・hookを変更・省略しない。設定上のmodel／effortを記録するが、projectによる上書き、Unit環境overlay、実role別argvのP観測をSで済ませたことにしない。各stepは専用custody、intent先行、program identity再確認、§10の期限・捕捉・停止契約を使う。

| provider | stepとargv／送信 | 合否・停止点 |
|---|---|---|
| Codex | Versionは`--version`。Schemaは`app-server generate-json-schema --experimental --out <new-dir>`。Handshakeは`command_argv(codex, Executor, model, effort, None)`の`app-server --listen stdio://`。 | Schema gate後、initialize→paramsなしinitialized→account/read（refreshToken=false）→environment/status（local ID）→account/rateLimits/read。chatgpt／ready／strict-valid rateを確認し、thread/start／turn/startを送らず停止。 |
| Claude（Reviewer） | Versionは`--version`。Handshake stepにinitialize観測を記録。`command_argv(claude, Reviewer, 設定model, 設定effort, fresh UUID)`の`-p --input-format stream-json --output-format stream-json --verbose --session-id <UUID> --permission-prompt-tool stdio --settings {"forceLoginMethod":"claudeai"}`に、存在するmodel／effortを既存の有界argv検査で付加し、末尾に`--permission-mode plan`を含む。実argvにplan modeがなければ拒否する。 | 専用request IDのinitialize（既存hooks=null）だけ送信し、相関したsuccess control_responseを検査して停止。user／canary／権限応答は0。Sでsystem-init／resultを必須にも観測済みにもしない。 |

Claudeはepoch・aliasごとのfresh session UUIDを用い、opaque request IDと区別する。SでReviewer用authorityを作らない。Codexは3step、ClaudeはVersion／Handshakeの2stepで、journalの96行上限を維持する。実argv／空cwd／停止が成立しなければBLOCKEDとし、モデル入力や設定迂回で成立させない。

**path境界**

raw aliasはpath成分に使わない。directoryはowner root配下の固定`startup`、実発行済み`RuntimeOwner.instance`と`owner.epoch`から作るepoch成分、alias identityのdomain-separated SHA-256を小文字64hexにした名前から構成する。epoch成分は`e-`＋domain-separated SHA-256（instanceのUTF-8長・bytesと正整数epochの固定幅表現）とし、DB再作成で数値epochが戻っても別instanceの旧領域を採用しない。同名衝突は拒否し、再利用して解消しない。alias digest対象にはinstallation IDとaliasのUTF-8長・bytesを含め、曖昧な文字列連結をしない。

0700の保証は、Sがowner root直下に作る`startup`から始め、owner root自体には0700を要求しない。rootは`execution/owner.rs`の`lock_state_root`が通常の`create_dir_all`と`canonicalize`で用意するため、modeはumaskに従い（022なら0755）、既存stateのrootも同じである。rootに要求するのは、現行`cli/endpoint.rs`の`owned_execution_root`が使う実uid `getuid()`所有、通常directory、`mode & 0o022 == 0`とし、Sはpathではなく開いたdescriptorの`fstat`で確認する。満たさないrootは拒否し、rrxが`chmod`／`chown`で直さない。`startup`、epoch、alias digestの3成分は、symlink追跡なしで自uid所有、`mode & 0o777 == 0o700`、通常directory、root descriptor配下であることを要求する。既存probe directoryは再利用せず拒否する。作成時のdirectory identityをcustodyが保持する。cleanupはrrxが当該runで作った同一directoryだけを、symlinkをたどらず有限件数で削除する。canonical path確認だけで途中のsymlink置換を安全と扱わない。境界を維持できなければ削除せず失敗を記録する。

現行`RuntimeOwner::open`のproducerはrootの通常作成・canonical化、`owner.lock`のsymlink拒否・`O_NOFOLLOW`・排他lock・close-on-exec、state DBの単一link・close-on-execを維持する。lockの`.mode(0o600)`は新規作成時指定であり、既存lockのmode検査ではない。rootの実uid／group・other書込み検査は`ControlEndpoint::bind`／`open_log`側のcheckerにあり、RuntimeOwner producerの保証と混同しない。M0 Sはendpointを束縛しない起動でも§2のdescriptor検査を行う。既存`daemon`／`control`の0700作成・group／other権限検査も維持し、root／lockのchmod・chownを追加しない。root 0755を受理するproducerは維持する。rootがgroup／other書込み不可なら他uidは直下成分を作成・改名・置換できず、startup 0700なら配下を走査・読取りできないというmode上の境界であり、ACL等の限界は後段に残す。

作成と確認はdescriptor相対で行い、root以降をpath文字列から再解決しない。開く際は`context.rs`の既存走査と同じ`O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC`を使う。

1. `owner.root`を開いて`fstat`でroot条件を確認する。root descriptor相対の`owner.lock`が、`RuntimeOwner`の保持するlock descriptorと同じ`(dev, ino)`の通常fileであることも確認し、開いたdirectoryを排他lock保持中のrootに束縛する。
2. `startup`は`mkdirat`（mode 0700）で作る。既に存在する場合は、開いたdescriptorが自uid・通常directory・0700のときだけ、epoch間で共有する親として使う。symlink、通常file、他uid、0700以外は拒否する。
3. epoch成分は当該runが`mkdirat`（0700）で排他的に新規作成し、そのdescriptorを同じrunの全aliasで使う。既存は拒否する。
4. alias成分は`mkdirat`（0700）で排他的に新規作成する。既存は所有者・mode・中身にかかわらず拒否する。同じrunの後続stepは保持descriptorを通じて同じalias成分を使い、これは再利用拒否の対象にしない。
5. 新規成分は作成直後に開いて`fstat`し、自uid・通常directory・0700を確認して`(dev, ino)`を保持する。

`mkdirat`のmodeは作成と同時に適用され、umaskはbitを削るだけなので、group／other bitは作成の瞬間から0であり、後追いの`chmod`を使わない。umaskが所有者bitを削った場合は0700の確認で拒否する。`mkdirat`から`openat`までの置換は、親descriptorがroot条件または0700を満たすことで他uidに対して閉じる。rootは実uid、新規成分は作成物の所有者となる実効uid `geteuid()`で比較する。S/Pはpath作成／helper spawn前に実uidと実効uidの一致を要求し、不一致は型付き拒否する。private directoryのpermission比較は`mode & 0o777 == 0o700`、special bitsを0700比較へ混算しない（継承setgidでもこの比較自体は拒否しない）。§8のSchema出力はalias直下の専用新規child `schema-out`として同じ作成・所有・mode検査を行い、親とchildの(dev, ino)をcustodyへ保持する。Version／Handshake spawn直前には保持alias descriptorから中身を列挙して空cwdを確認する。Schema stepは専用出力childを持つため空cwd検査と分け、出力child以外がalias cwdにあれば拒否する。

workspaceは`unsafe_code = "forbid"`で、descriptorをchildのcwdとして渡す`fchdir`を使えない。childのcwdと§8の`--out`はpath文字列で渡すため、各spawn直前にroot descriptorからstartup／epoch／aliasの3成分を`fstatat(AT_SYMLINK_NOFOLLOW)`でたどり、型・uid・permissionと`(dev, ino)`を再確認する。Schema spawnと生成file読取前はalias相対のschema-out childも同様に再確認する。後続Handshake前はSchema出力childを§2／§8の有界cleanupで除去し空cwdを確認する。cleanup不能ならHandshakeをspawnしない。不一致ならspawnも置換物の削除もしない。

cleanupは保持descriptor相対で行う。alias directoryの中身は保持descriptorから列挙して`unlinkat`で除去し、symlinkはlink自体だけを除去する。下位directoryは`O_NOFOLLOW`で開き、自uidかつ同一deviceのときだけ降りる。上限は§8の出力上限と同じ1024項目・深さ4とし、超過分は残して失敗を記録する。alias成分とepoch成分の`rmdir`は、当該runが作成したもので、親descriptor相対の名前が保持した`(dev, ino)`と一致するときだけ行う。root／startupは削除・mode変更しない。検査済みstartupは共有親として使える。旧epoch／aliasや他runのprobe領域は削除・再利用・mode変更しない。当該runの後続stepが保持descriptorを使うこととは区別する。

この検査が保証しない範囲は、同一uidの別process、superuser、mode bitに現れない拡張ACL（継承ACLを含む）、およびchildへpathで渡すcwd／`--out`について再確認からchild側の解決までにroot祖先が置換される場合である。最後の一つは他uidがroot祖先へ書込める配置でだけ成立し、state DBのpathも同じ前提に立つ既存の限界である。M0はこれらを安全と表明しない。

positive／negativeは次を母集団とし、§12 Migration/pathのsymlink／ownership／境界cleanupに対応させる。fixtureが自分で作ったdirectoryのmodeを設定することは、本番codeがrootを改変しないことと区別する。

| 区分 | 入力 | 期待 |
|---|---|---|
| positive | 新規state。rootは`lock_state_root`が作成（umask 022で0755） | S続行。3成分は自uid・0700。root modeは不変。 |
| positive | 既存rootが0755または0700 | 同上。 |
| positive | 前epochが残した自uid・0700の`startup`と旧epoch成分 | `startup`を親として新epoch成分を作成。旧epoch成分は不変。 |
| positive | 事前作成したrootに対し、umask 000の別processで起動 | 3成分は0700。通常の`create_dir_all`へ戻すmutantを落とす。 |
| negative | rootがgroupまたはother書込み可（0775、0757、0777） | 型付き拒否。`startup`未作成、root modeは不変。 |
| negative | root相対の`owner.lock`が保持lockと別inode | 拒否。 |
| negative | 他uid所有のroot、または他uid所有の`startup` | 拒否。 |
| negative | `startup`がroot外、またはroot内（例：`units`）を指すsymlink | 拒否、参照先は不変。root内向きはcanonical化＋prefix比較へ戻すmutantを落とす。 |
| negative | `startup`が通常file、または自uidの0755 directory | 拒否、modeは不変。 |
| negative | 同名のepoch成分またはalias成分が既存（directory、symlink） | 拒否、既存物は不変。 |
| negative | 先行stepの後、後続stepのspawn前に、alias成分をsymlinkか別directoryへ置換 | 後続stepをspawnしない。置換物を削除しない。 |
| negative | run中にalias成分を外部directoryへのsymlinkへ置換、または内部へ外向きsymlinkを置く | 置換先・参照先のfileは残る。alias内のlinkだけを除去し、不一致は失敗として記録。 |

umaskはprocess全体の属性なので、test process内で変更せず別processの起動で与え、直列化を足さない。run中の置換は、test専用のseamやauthorityを足さず、同じaliasの先行step（Version）のsynthetic childが行う。他uid所有は非特権の実行では作れないため、root条件と0700条件を実descriptorの`fstat`に対する単一の述語とし、他uid所有の実在directoryのdescriptorで拒否を確認する（特権実行ではfixtureが`chown`して作る）。これはroot producerを通る負例には数えず、skipにもしない。uid≠euidも同じ限界と計数に従う。非特権CIでは本番が使う単一のuid一致述語へ一致／不一致の値を与えて拒否条件を確認し、実processのgetuid()／geteuid()不一致を作った証拠、S/P producerの実拒否到達、権限境界の実証には数えない。実producerの不一致負例はその権限条件を実際に持つ実行で確認できた場合だけ別計数し、未実施を明記する。skip／ignore、test専用authority constructor、直列化、後付timeoutは追加しない。Sのroot検査の因果は、control endpointを束縛しないRuntime起動で確認し、daemon経路で先に働く既存`ControlEndpoint::bind`の拒否と区別して記録する。

`..`、slash、absolute形aliasはraw joinを復活させたmutantがroot外へ到達しないための負例とする。digest方式ではこれらのbytesから作られるpathもroot内に限定される。既存合法aliasのpositiveを維持する。

**journal**

新表`runtime_startup_probes`のキーは`(owner_epoch, installation, alias, step)`。stepはVersion／Schema／Handshake。provider、instance、宣言digest、設定model／effort、program identity、有限な観測要約をbodyに保持する。stateはpending／confirmed／unknown、versionは正整数、bodyは4096 bytes以下。

intentは自instance／epoch一致と未存在キーを同一transactionで検査する。settlementはpendingからconfirmedまたはunknownへの、versionとbody完全一致CASだけを許可する。confirmedは物理的結果が既知という意味であり、protocol合格とは別である。

aliasはinstallationあたり32、1 epochのstep行は96以下。全journal行8192とunknown＋pending（予約込み）4096は独立upper boundsであり、両方の容量を使い切れる保証ではない。最大96行×64 epochのconfirmedだけで6144行となり、unknown等が4096未満でもtotal上限が先に効く。両上限をintent transactionで予約込み検査し、超過するintent／spawnを拒否する。既存epoch fence transaction内で旧pendingをunknownに移し、64 epochより古いconfirmedだけをpruneする。その結果に対して次epochの予約を評価する。unknownは自動削除しない。保存失敗を成功へ反転しない。prune後も上限ならBLOCKEDであり、上限増加・unknown破棄・再起動による解除を約束しない。

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

`RuntimeMetadata`のwire producer／consumerにstartup全体状態、alias別状態、oldEpochUnknown件数を追加する。producer／型／dispatchは`runtime/control.rs:337,753`、消費者は`main.rs:583`、`cli/daemon.rs:238`、`runtime/tests.rs:1304,1316`、`crates/rrx/tests/runtime_cli.rs:464`、`crates/rrx/tests/daemon_native.rs:182`。`ControlResponse`の`deny_unknown_fields`を維持し、CLI表示・deserialize testsを同時更新する。新旧wireの不一致は拒否し、新旧バイナリの混在互換を保証しない。DB v12と合わせて同じ契約のdaemon／CLIへ切り替える。

`ShutdownSite`の追加は`runtime/stop.rs`、`runtime/control.rs`、`cli/daemon.rs`、`cli/daemon/tests.rs`まで配線し、「six」の注記も更新する。installation拒否理由を固定文へ潰す現行consumerは型付き分類を保つ形へ変更する。

### 5. 行予算

実producerは全roleでGit×13を生成する。Reviewerを9本とする計算を廃止する。256行上限、Git本数、既存byte上限は緩めない。

baselineを`B`とすると、閉じた契約は次のとおり。

| provider／role | Version前B | Version後 | Git前 | Git後prefix | Claude Probe後／Prepared上限 |
|---|---:|---:|---:|---:|---:|
| Codex Executor | 230 | 231 | 233（Schema＋Handshake後） | 246 | 246 |
| Codex Reviewer | 232 | 233 | 235（Schema＋Handshake後） | 248 | 248 |
| Claude Executor／Reviewer | 237 | 238 | 238 | 251 | 252 |

Codex Executorは`230+Version1+Schema1+Handshake1+Git13=246`、Codex Reviewerは`232+Version1+Schema1+Handshake1+Git13=248`。Claudeは`237+Version1+Git13+Probe1=252`。Claude Probe末尾はPrepared上限に含め、prefixだけを`check_prepared`へ渡さない。

B2b提案でもPのconfig/readは同じHandshake helper内の追加RPCであり、Handshakeは1行のまま（§8a(3a) B2b-5）。本番setup8行は維持する。B230→246→256、inventory2 MiB、body8192、receipt16／128、捕捉・aggregate上限は変更しない。追加RPC／決定の有限記録が残量に入らなければPを拒否する。

最初の入力までの後続reserveはCodex Executorがtransport1＋setup8（§8a(3a)のconfig/read1とmcpServerStatus/list1を含む）＋input1、Codex Reviewerがtransport1＋setup6＋input1、Claudeがtransport1＋setup1＋input1。したがってCodexはどちらも256、Claudeは255。Executorの追加2行は256上限を緩めずBの引下げで吸収する。terminalは既存input effectのsettlement／result publicationであり、新しいmanaged effect行を追加する計算にはしない。terminal consumerが追加行を必要とする変更はこの契約を再設計する。

同じbudget定数をVersion intent、metadata helper、Git reserve、Probe intent、Prepared、registration、setup/input consumerへ渡す。byte reserveは追加helperの最大imageも含む。

境界positiveとB+1 negativeは実installed producerが生成する履歴で検査する。`vec![row; N]`やSQL seedだけでは予算契約の正例にならない。M0 は現に提供する Codex Executor の実 producer（B230→prefix246、Prepared上限246、初回入力256）で境界を検証する。Claude Reviewer の同一予算政策（Prepared上限252、初回入力255）は純粋計算として照合するが、実 Reviewer helper 鎖の境界合格は M3 で検証する。純粋計算を実 Reviewer 起動の成功証明としない。

### 6. Claude Reviewer Probeとstrict wire

適用範囲: ClaudeはReviewerである。本節のProbe（canary文面、nonce、Probe中の拒否、成功後tail、§10のcanary期限、§9のcanary Known closure）はClaude Reviewer Pの契約であり、本番producer・実境界・受入はReviewer helper鎖を提供するM3で行う。M0はProbe producerを追加せず、Probe合格を記録しない。共通strict decoder、rate decoder selfcheck、UUID内容digest、telemetry契約はM0の範囲に残し、SのClaude initialize応答と、M3のReviewer Sessionが使う本番Claude Session経路のdecoderとして維持する（M0-U1の実装済み部分を含む）。M0の設定ではClaudeの本番Sessionを起動しないため、本番Session向け契約はM0-U1の実child検査とM3受入で扱い、owner-runの観測には数えない。

Probeは本番`command_argv(claude, Reviewer, …)`を使い、専用session UUID、同じmodel／effort／overlay／worktreeを用いる。Reviewer argvは既に`--permission-mode plan`を含む（`native/prepared.rs::command_argv`）ため、Probeはplan modeを追加・除去せず、実argvにplan modeがなければ拒否する。既存設定・認証・hookを変更・スキップしない。

canary入力は次の固定文面とする。

> Return exactly the following token as plain text, with no explanation and without using tools: `<nonce>`

nonceは`RRX-CANARY-`＋乱数32hex。成功は相関initialize応答→canary送信→identity／cwd一致のsystem-init→`subtype=success`、`is_error=false`、humanまたはorigin absent/nullのresult、前後空白除去後のnonce完全一致とする。包含比較をしない。

Probe中のtool・権限要求・tool result・background task・注入turnは拒否し、応答しない。既存hook書込みや自動許可toolは検出以前に起きる可能性がある。plan modeを隔離証明としない。

共通decoderはduplicate key、切れたJSON、型不正を拒否してから、次の有限構造を検査する。以下はrrxの受理契約であり、consumerの条件分岐をCLIの合法wire domainとみなさない。必須は欠落を拒否し、optionalは欠落を許し、nullableと書いたものだけnullを許す。文字列・ID・配列・JSONは既存上限内、数値は有限、計数／時間は非負で対象整数域への変換とoverflowを検査する。present wrong typeを`as_str/as_u64`失敗による欠落へ丸めず、分類／所有判断前にProtocolErrorとする。

公開型の根拠は依頼に固定された[Claude Agent SDK TypeScript資料](https://code.claude.com/docs/en/agent-sdk/typescript.md)の2026-10-13観測と、現`claude_wire.rs`の消費fieldを分けて用いる。公開型に掲載されたoptional stringをnullableへ拡大しない。SDKMessageOriginは保存済み公開型で確認済みとして下段に分離し、公開固定factsで確認できないcontrol／session-state等はrrx限定契約として明記し、資料不在をCLI仕様の断定で埋めない。SDKMessage全variantの追加をM0 scopeへ自動拡大しない。

| frame | field・型・必須性と消費条件 |
|---|---|
| `system/init` | 必須session_id:string、cwd:string、tools:string[]、mcp_servers:{name:string,status:string}[]。statusはstringであり未根拠のenumを作らない。identity／cwd一致とdecision時の空inventoryは業務条件。 |
| `system/session_state_changed` | 必須state=`idle/running/requires_action`。 |
| `system/task_started` | 必須task_id:string、description:string。optional task_type:string（null不可）。公開資料のlocal_bash／local_agent／remote_agentは例でありenumではない。現consumerのlocal_agent／local_workflowは追跡する条件でありwire domainではない。Probeはtask_started自体を拒否。 |
| `system/task_notification` | 必須task_id:string、status=`completed/failed/stopped`、output_file:string、summary:string。optional usage:{total_tokens:number,tool_uses:number,duration_ms:number}（3field必須、有限非負、null不可）。本番は追跡済taskの終了だけを消費。 |
| `system/task_updated` | 必須task_id:string、patch:object。patch.statusはoptional `pending/running/completed/failed/killed`（null不可）。optional description:string、end_time:number、total_paused_ms:number、error:string、is_backgrounded:booleanもnull不可。status欠落／pending／runningはwire合法で終了扱いしない。completed／failed／killedだけ終了消費し、stoppedをpatch.statusへ流用しない。 |
| `system/api_retry` | rrx限定契約は必須attempt:uint64、max_retries:uint64、retry_delay_ms:uint64、error_status:nullまたはuint16。現quota fixtureのstatus_codeは追加fieldであり成功・capacity証拠にしない。公開固定factsはこのrowを資格付けておらず、実CLI適合は未証明。 |
| `assistant` | 必須session_id:string、parent_tool_use_id:string/null、message:{role:`assistant`,content:typed block[]}。optional errorは下記12値、null不可。textは必須text:string、thinkingはthinking:string（signatureがあればstring）、tool_useはid/name:string＋input:object。parent／originがowned human条件に一致した場合だけ既存error hintを消費。 |
| `user` | 必須message:{role:`user`,content:stringまたはtyped block[]}。配列要素はtext／tool_result。textはtext:string、tool_resultはtool_use_id:string、content:stringまたはtext block[]、optional is_error:boolean（null不可）。parent_tool_use_idがあればstring/null。これはrrx限定content集合であり公開SDKの全content domainを主張しない。parent／originのinjected判定は検査後。 |
| `result` | 必須session_id:string、subtype=`success/error_during_execution/error_max_turns/error_max_budget_usd/error_max_structured_output_retries`、is_error:boolean。success枝はresult:string、optional api_error_status:number/null（rrxでは非負整数uint16へrefine）。error枝はerrors:string[]必須、resultがあればstring、api_error_statusがあれば同じrefinement。error枝のapi_error_status受理はrrx契約であり公開固定factsの保証としない。errorsは全要素を検査し、分類のtake(64)以前のmalformedを隠さない。success/error両枝のusage容器・実消費6項目・structured_outputは下段の契約を共用する。成功・auth・capacityは検査後の業務条件。 |
| `control_request` | 必須request_id:有界string、request:object、request.subtype=`can_use_tool`、tool_use_id/tool_name:string、input:object。未知subtype拒否。Probeは正常形でも拒否・応答0。 |
| `control_response` | 必須response:object、response.request_id:有界string、response.subtype=`success/error`。success枝はresponse.response:object、error枝はresponse.error:string。相関IDは型検査後に完全一致。 |
| `stream_event` | 必須event:object。parent_tool_use_idがあればstring/null。入れ子fieldは次段落の有限構造。 |
| `tool_progress` | 必須tool_use_id:string、tool_name:string、parent_tool_use_id:string/null、elapsed_time_seconds:有限非負number。rrx受理契約であり今回の公開固定factsによる実CLI適合証明ではない。 |
| `tool_use_summary` | 必須summary:string、preceding_tool_use_ids:string[]。 |
| `auth_status` | 必須isAuthenticating:boolean、output:string[]、optional error:string（null不可）。 |
| `rate_limit_event` | 必須session_id:string、rate_limit_info:§9の型付きobject。 |

全rowのtypeは表のtop type、system rowのsubtypeは表の名前を要求する。共通optional uuidはUUID、session_idはpresentなら有界string（init／assistant／result／rateでは必須）。optional originはnullまたはobjectで、非nullはkindを要求する。assistant.errorの12値は`authentication_failed/oauth_org_not_allowed/account_on_hold/billing_error/rate_limit/overloaded/invalid_request/model_not_found/server_error/max_output_tokens/cloud_credential_error/unknown`。現consumerがhint分類に使う4値とこのwire集合を混同しない。api_error_statusの公開numberに対するuint16 refinement、usageの非負性、配列上限はrrx契約でありlive適合は未証明。欠落とnullはdecoder上で別状態を保持する。今回の局所実装ではapi_error_statusをPresence<uint16>としてValidatedClaudeResultに保持し、RunStateのauth／capacity分類はこの検査済み値を消費する。整数値、整数decimal、整数exponentの業務分類を揃え、分類結果からquota／retry authorityを作らない。本番Sessionのcorrelated initialize responseはbootstrap専用consumerでoptional session_idのpresent時一致を検査し、optional UUIDの内容履歴をnative_input後の同じRunStateへ引き継ぐ。responseはinit／input ACKではなく、system/initの既存所有・cwd検査を代替しない。O03のnormal-loop control_response拒否はUUID再送判定より先に維持する。

今回の局所実装の本番Claude byte経路はLinesの元bytes→有限strict検査→元number lexemeの有界な保持→検査済み整数／UUID内容digestとする。共有strict_json::decodeのValue契約は変更しない。Claude専用の数値補助は元decimal／exponentの十進係数と指数から整数性・uint64域を判定し、整数float受理、u64::MAX、absent／null／zeroを維持する。UUID digestは未知fieldを含む元数値の正確な十進内容、key順序非依存、UUID正規化を消費する。従来の整数Numberとfloat Numberの表現区別、float signed zeroの区別は保持する。同じfloat数値の等価なdecimal／exponent表記は正規化し、f64への丸めでしか同一にならない異なる元数値は同一内容にしない。非整数用の有限非負f64 fieldとrate resetsAtの既存整数表現契約は変更しない。Value-only入口は供給されたValueだけを検査でき、上流で失われた精度を復元したとは扱わない。startup queueはValueと元bytesを対で保持する。既存128件上限とencoded Valueの8MiB上限を保持し、元bytesにも独立した8MiB上限を適用する。短い整数exponentがValue serializationで長くなる場合に旧encoded上限を緩めず、長い元表記も別予算で有界化する。dequeueは保存したencoded長と元bytes長から双方を回復する。これは本番Claudeと同じLinesを使うCodex queueにも及ぶ。元bytesだけで旧encoded計数を置き換えた9937案はこの後続局所補正で訂正し、元案と未検証の履歴は保存する。S/P資格付けは引き続き未実装で、将来のbyte producerも元bytes入口を独立に検査する必要がある。以上はM0-U1として実装した局所差分であり、固定source `a61d038d`でfmt・clippy・対象tests、7件のsource operator kill／復元、Sol修正確認（APPROVE）を`doc/verification/macos-minimum-m0-u1.md`に記録した。full workspace test、両OS CI、installed S/P、owner-runは未完了であり、full M0の未完了項目を閉じない。役割入れ替え後もこの差分はClaude Reviewer側のdecoderとして保持する。

result telemetryの公開型はsuccess/error両枝で必須duration_api_ms:number、total_cost_usd:number、usage:NonNullableUsageである。NonNullableUsageではここで消費するinput_tokens／output_tokens／cache_read_input_tokens／cache_creation_input_tokensはnumberでnull不可。rrxは既存のnullable観測を維持するため、usage容器と6項目をoptional nullableとして受理する公開型との差を明記する。usage非nullはobject、各4カウンタおよびduration_api_msの非null値は有限・非負・整数・uint64域、total_cost_usdの非null値は有限・非負f64域へのchecked変換を要求する。absent／null／0は別状態、present wrongtype、負数、fractional counter、overflowはProtocolErrorであり欠落へ丸めない。未消費のSDK metrics全体を新しいrequired契約へ拡大しない。

strict decode→ValidatedClaudeResult（presence/null状態と検証済数値）→claude_wire::Metrics→native.rs:2387-2390のstatus.metrics→adapter/native.rs:516-525のUsageという経路へ揃える。Metrics::parseの未検査Value／as_u64による再解釈は廃止し、内部status画像もvalidated型からだけ生成する。optional telemetryは終端成功、capacity、quota Admit、再実行権限の根拠にしない。

本番Sessionで、先行する完全validatedなowned終端原本がなく、単一のsuccess-looking result内のtelemetryが不正なら、frame全体のstrict decode不合格として`WorkOutcome::Unknown`／`Disposition::ProtocolError`／`Some(NativeFailure::ProtocolFailure)`を固定する。result本文・metricsを採用せず、`Collector::claude_terminal`への成功入力と`Collector::confirm_complete`の呼出しを行わず、回答は不採用（成功回答の`AcquisitionStatus::Complete`を発行しない）。他field・所有を独立検証できた部分は有界なnongrant観測だけに保持し、`RunState::complete()`、成功tuple、`Core::observed_terminal=(WorkOutcome::Success, Disposition::Completed, None)`へ変換しない。native.rs:1918-1929のobserved_terminal上書きやCompletedかつ非Unknownによるconfirm_completeを、nongrant観測が通る抜け道にしない。現Claude成功経路:2385-2390はstate.complete→metrics→Success／Completedを返し、observed_terminalを立てないというsource事実を保持し、decoderエラーからこの成功経路へ合流させない。

この未知結果は既存の本番terminal原本と原子的Session closureへ接続する。所有・CASが成立して保存できた場合は`UnitState::WorkUnknown`、`SessionState::Lost`であり、owned success settlementは発行しない。Task側のM0 consumer契約は、この非成功terminalを既存variantの`SettledLookup::Held`→`WorkflowEngine::step_driven_initial`の`StepResult::Waiting`へ接続し、有界な固定reason `native result unknown; explicit action required; Held`で明示操作待ちにすることとする。この分類・配線は実装時の検証対象であり、現sourceの`runtime/phase_jobs/success.rs::settlement_turn`は非成功terminalを成功未観測として保持するpollで、Session未知結果の明示待ちが既に完成しているとは扱わない。登録前closureの`ClosedPhaseAck::NonSuccess`／explicit retry requiredを本番Sessionへ流用しない。所有・CAS・保存の照合不能は既存Held／pending原本保持に従い、closure済みともsuccessとも記録しない。Taskへの明示操作待ちはreplay許可ではなく、どちらの場合もtelemetry欠陥だけから自動再送・新しい入力・未知結果のretry authorityを発行しない。Task.versionによる許可や#10／#14の新機能は追加しない。

ここで「既に証明された終端成功」は、欠陥frameより前の別の完全validated resultについて、owned human受理・`RunState::complete()`・validated metricsを通り、collectorの既存回答採否を確定し、そのvalidated終端だけにconfirm_completeを行ったうえで`Core::capture(WorkOutcome::Success, Disposition::Completed, None)`の`NativeTerminal`原本を`captured_terminal`／既存frozen terminal保持経路へ確保した時点以降を指す。同じinvalid frame内のsuccess fieldやnongrant観測はこの時点に含めない。その後に別の欠陥frameが観測された場合は、保持済み原本のwork=`WorkOutcome::Success`／disposition=`Disposition::Completed`／failure=`None`と、その先行原本だけの回答・confirm事実を維持する。欠陥frameから回答・metricsを追加採用せず、confirm_completeを新たに呼ばず、有界なprotocol不合格観測だけを分離して保持する。既存published／confirmed事実を巻き戻さず、既存原本のCAS照合・保存だけを継続し、新しい成功資格付けやreplay権限を出さない。現本番Claudeは成功tupleを返して読取りを終えるため、この契約は後続frameを新たに読む機能を要求しない。

登録前Pは本番Sessionの上記結果と区別する。canaryの完全validated成功resultだけではProbe合格ではなく、§10の合法tail・両pipe EOF・leader reapまで必要であり、後続欠陥frameはKnown不合格→§9の既存Known closure、物理観測不明ならUnknown→Heldへ接続する。NativeInput／Session登録0、成功qualification0、Taskの明示操作待ち・自動再送拒否を維持し、helperの`EffectState::Confirmed`（物理結果既知）を本番SessionのSuccessへ読み替えない。

structured_outputはClaude resultの明示optional bounded OpaqueJson位置でnullも受理する。strict JSON／byte・depth・nodes・容器上限とUUID canonical digestに含める。native_result.rs:555,582-590のCollector::claude_terminalは非nullならanswer_structured_profile_unsupportedとして回答不採用を維持し、内容を回答やauthorityにしない。absent/nullの通常text結果と非null値の不採用を別controlにする。再開時のtotal_cost_usdは復元totalsを含みうる推定累積値であり、Metricsのcumulative_cost／cumulative_api_msは観測値を保持し、resumed時のcost／api_ms deltaはNone、previousや単調性からphase deltaを発明しない。claude_wire.rs:595付近の不正telemetryを欠落へ丸める既存testは意図したstrict契約変更として台帳に記録し、正しい欠落・null・ゼロ、再開累積値、replay safetyの共有ケースを削除／skipせず移植する。

stream eventは`message_start/content_block_start/content_block_delta/content_block_stop/message_delta/message_stop/ping`だけを許可する。全枝にevent.typeを要求する。block_startはindex:uint64とcontent_block（assistantと同じ3枝）、block_deltaはindex:uint64とdelta.type=`text_delta/thinking_delta/input_json_delta/signature_delta`、それぞれtext／thinking／partial_json／signature:string。block_stopはindex:uint64。message_start.messageは必須id:string、type=`message`、role=`assistant`、model:string、content:typed block[]、stop_reason:string/null、stop_sequence:string/null、usage:{input_tokens:uint64,output_tokens:uint64}。message_deltaは必須delta:{stop_reason:string/null,stop_sequence:string/null}とusage:{output_tokens:uint64}。usageのcache_read_input_tokens／cache_creation_input_tokensがあればuint64。message_stop／pingはtype以外の必須fieldなし。これはrrxの有限受理構造であり、今回の公開固定factsによる全stream variantの資格付けではない。構造が既知でもProbeのtool拒否を省略しない。

SDKMessageOriginの公開wire集合はkind必須で`human/channel/peer/task-notification/coordinator/auto-continuation/unclassified`。human／coordinator／auto-continuation／unclassifiedはkind以外の必須fieldなし。channelはserver:string必須。peerはfrom:string必須、fromModeはoptional `bypass/prompting`、name／fromSession／senderTaskId／bodyはoptional string、verifiedPeerPidはoptional number。task-notificationはsubkindがoptional `scheduled-trigger/peer-send-message`、fireReason／runIdがoptional string。これらのoptional fieldはpresent nullを許さない。rrx refinementは全string／IDの既存上限、verifiedPeerPidの有限・非負整数uint64変換であり、公開numberと混同しない。外側originのabsent/null受理はrrxの互換契約として保持する。

RunState::observe（claude_wire.rs）のassistant hint、非human resultのbackground／injected処理、userのinjected判定は全frame構造・session／parent等の所有検査後にだけ行う。非human originはwire正常でもhuman結果／canary成功／回答authorityにしない。Probeでは非human turnを拒否し成功に使わず、本番の正常非human処理と分ける。既存合成fixtureのhook／mcp-channelは公開origin値ではなく、新契約では拒否negativeへ分類する。正常fixtureはchannel（server必須）等の根拠ある値へ移し、peer／task-notificationのrequired・optional/null対、coordinator／auto-continuation／unclassified、malformed foreign／非humanも構造検査を通すCL-4／LG-1に束ねる。

UUIDがあるframeはUUIDを解析して小文字hyphen形へ正規化する。JSON object keyを再帰的にsortし、array順序、文字列内容、absentとnullを保持してcanonical digestを計算する。未知fieldを削ってdigestを作らない。8192 UUID上限。構造検査後に重複照合し、同一UUID・同一内容だけをdedup、内容改変を拒否する。request IDなどopaque IDはUUIDへ変換しない。

既知Claude frame内の未宣言fieldは有界strict JSONとして保持受理する。canonical digestに含め、authority／分類／回答には使用しない。既知fieldのwrong typeや未知type／subtype／block／delta／origin.kind／error enumを追加fieldとして逃がさない。将来追加fieldをconsumerで読むときは契約へ明示追加して検査する。追加fieldの正常形／過大・不正形／同UUIDでの改変をcontrolに含める。

有限表の意図した拒否とAC-05母集団: 本番RunState::observeのsystem未知subtypeの黙認、user/tool_result.contentの未検査を新decoderで閉じる。SDKAssistantMessage／SDKUserMessage／SDKResultMessage、SDKSystemMessageのinit、SDKToolProgressMessage／SDKAuthStatusMessage／SDKTaskNotificationMessage／SDKTaskStartedMessage／SDKTaskUpdatedMessage／SDKToolUseSummaryMessage／SDKRateLimitEventはそれぞれ表の構造内だけが正常母集団である。SDKSessionStateChangedMessage／SDKAPIRetryMessageについては表のrrx限定rowを維持し、公開wire資格付けとは扱わない。

SDKCompactBoundaryMessage（固定参考本文にwire名があるsystem/compact_boundary）、SDKInformationalMessage（system/informational）、SDKPermissionDeniedMessage（system/permission_denied）は表外として意図して拒否する。他の表外候補はSDKStatusMessage、SDKLocalCommandOutputMessage、SDKHookStartedMessage、SDKHookProgressMessage、SDKHookResponseMessage、SDKPluginInstallMessage、SDKTaskProgressMessage、SDKBackgroundTasksChangedMessage、SDKThinkingTokensMessage、SDKWorkerShuttingDownMessage、SDKCommandsChangedMessage、SDKNotificationMessage、SDKFilesPersistedEvent、SDKMemoryRecallMessage、SDKElicitationCompleteMessage、SDKPromptSuggestionMessage、SDKMirrorErrorMessage、SDKConversationResetMessage。これらは固定本文だけではwireタグ／枝構造が資料未確定であり型名から推定しない。SDKUserMessageReplay／SDKPartialAssistantMessageも型名だけでuser／stream_eventへの対応を認定せず、個別frameが既存rowに適合すると確認できる範囲だけ受理し、表外形は意図した拒否範囲へ入れる。全36variant対応は新しいM0条件にしない。

assistant/content_block_startのtext／thinking／tool_use以外、user配列のtext／tool_result以外、tool_result.content配列のtext以外（imageを含む）は意図して拒否する。image tool_resultの実CLI発生は未実測。公開合法でもこの有限表外なら副作用後の本番Sessionを終端させうるため、AC-05の正常ignored／合法toolはこの有限集合内を指し、旧黙認全体を維持したとは扱わない。LG-1で型付き正常ignored／toolと新拒否の対を保存し、A5の全体オーナー判断材料にする。live表外観測はBLOCKEDであり、実発生や互換性を断定しない。表外の実eventは安全に記録可能な有界variant名だけを既存監査枠内に記録して不合格とし、raw bodyや秘密は保存せず、未知frameを無視・正常化しない（安全に記録できない名前は有限分類unknownとする）。

Claude rate eventはforeign sessionでもstrict decodeを先に行う。正常foreign eventだけを所有フィルタで除外し、quota更新しない。

### 7. Codexの有限wire契約

rrx側に`CodexWireContract`を1つだけ定義し、typed decoder、schema gate、S/P、本番consumerで共用する。契約はbasis catalogのsubtree digestで識別し、CLI版数で契約を選ばない。CLIのruntime出力から型・required・enumを作り直さない。ignored通知も同じdecoderを通す。

適用範囲: 本節の契約は、全Codex aliasのS、Codex Executor P（Schema／Handshake）、§8aのCodex Executor本番Sessionで同じdecoderとして使う。Claude Reviewerには適用しない。

**契約basisと位置の二分類（C1〜C3）**

契約basisはcodex-cli 0.162.1の公開tag export（§12）とする。各位置（method、definitionのproperty path、enum値集合、枝集合、required集合）をstrictかtoleratedのどちらかに割り当てる。分類は到達経路ごとに行う。Schema gateでは、あるsource file内でstrict位置から到達するdefinition（strict到達集合。§8規則2で定義）の宣言は、同じdefinitionがtolerated位置からも到達していてもstrictとして比較する（共有definitionのstrict比較。例: AbsolutePathBufは`thread/started`→Thread.cwdでstrict到達し、`hook/started`→HookRunSummary.sourcePathからも到達する。UserInputはServerNotification内ではuserMessage item内容（tolerated）からだけ到達するが、ClientRequest_usedのturn/start送信の閉包でstrict到達するため、その宣言の変化はClientRequest_usedの比較で拒否される。CommandActionはcommandExecution枝とserver requestでstrict）。これは比較の規則であり、tolerated位置にある値を実行時に型検査することではない。tolerated位置からstrict到達definitionへ届くこと自体は拒否理由ではなく、拒否するのは§8規則2の「構造をもつstrict到達definitionへの新しい到達」だけである。下表にない位置はstrictとする。

| strict区分 | 位置 | 根拠 |
|---|---|---|
| S-send | `initialized`と送信8 methodのparams閉包の全宣言。rrxが送らないoptional property、default値、approval方針・sandbox・roots・network・environment・cwd・dynamicToolsを含む | rrxが送る値と、送らない値のCLI既定がrrxの安全前提を決める |
| S-handshake | Initialize／Account／Environment／Rate responseの全宣言（rateLimitUpsellは免責表） | account mode、ready、rate証拠は資格付けの判断値 |
| S-config | ConfigReadResponseのroot required（config、origins）とlayersの宣言、`config`がConfig objectであること、Configの`additionalProperties: true`、`web_search`の宣言（nullable WebSearchMode）、`features`と`mcp_servers`が宣言propertyでないこと | §8a(3a)の判定path。B2b承認後はPの不足選択にも同じ位置を使う。features／mcp_serversはsourceでは開放領域にあり、rrx refinementで型付けする（§8） |
| S-inventory | ListMcpServerStatusResponseのdata（required array）とnextCursor（nullable string）、McpServerStatusのname（required string）・tools（required object map）・toolsError（nullable string） | §8a(3a)の判定値 |
| S-thread | ThreadStartResponse top-levelの全宣言、Thread core（id、cwd、modelProvider、status、turns）、TurnStartResponse、Turnの全宣言、TurnStatus、TurnError（misalignmentの内容を除く）、CodexErrorInfoの全枝、ThreadStatus、ThreadSettings | owned thread／turnの同一性・状態・終端と応答echo |
| S-notify | 次の21通知の全payload: error、thread/started、thread/status/changed、thread/archived、thread/deleted、thread/unarchived、thread/closed、thread/reverted、thread/settings/updated、turn/started、turn/completed、item/started、item/completed、item/agentMessage/delta、item/commandExecution/terminalInteraction、item/fileChange/patchUpdated、serverRequest/resolved、mcpServer/startupStatus/updated、account/updated、account/rateLimits/updated、account/login/completed | 決定値、効果、終端、回答、資格付けの相反観測を運ぶ |
| S-held | item/autoApprovalReview/started、item/autoApprovalReview/completed、autoApprovalReview/strictReviewRequired、item/mcpToolCall/progressのmethodと所有キー | 所有turnでの着信だけでUnknown→Held。内容は判断に使わない |
| S-request | server request 11 methodとparams閉包の全宣言 | rrxへ許可・入力を求めうる |
| S-item | ThreadItemの全枝のid／type、agentMessage（MessagePhaseを含む）・commandExecution（CommandAction、cwd、status）・fileChangeの内容の全宣言 | 回答・効果・path検査の対象 |
| 所有キー | 84通知すべての所有キー（threadId、turnId、thread.id、turn.id）の宣言 | 所有フィルタと握手扱い |

| tolerated区分 | 位置 | 実行時 |
|---|---|---|
| T-notify | S-notify以外の63通知のpayloadのうち所有キー以外（S-heldの4通知を含む） | envelope・method・所有キーだけ型検査し、残りは有界strict JSONとして保持 |
| T-item | userMessage・reasoning・plan・contextCompactionの内容、§8a(3a)許可集合外12枝の内容 | id／typeだけ検査。所有turnの許可集合外はUnknown→Held、foreignは除外 |
| T-thread | Thread core以外のproperty（extra、gitInfo、source等）とTurnError.misalignmentの内容 | 保持のみ |
| T-config | Configの宣言propertyのうちweb_search以外、origins、layersの内容 | 保持のみ。layersは実行時absent／nullを要求する（includeLayers=falseを送るため） |
| T-inventory | McpServerStatusのcore以外（authStatus、resources、resourceTemplates、runtimeStatus、serverInfo、serverCapabilities、httpOrigin、pluginId）とtools mapの値（Tool） | 保持のみ。tools mapは空であることだけを使う |

tolerated位置の実行時規則: 既存のbyte／depth／node／文字列／容器上限とstrict JSON（duplicate key、切れたJSON等の拒否）をそのまま適用する。frame処理中のメモリ保持だけとし、journal・receipt・log・statusへ本文を出さない。分類・回答・quota・auth・authority・所有判断に使わず、tolerated値でstrict位置の判定を変えない。consumerがtolerated位置を読む必要が出た場合は、§8bの手順でstrictへ移してから読む。黙って再分類しない。

例外は1つだけとする: 既存collectorのterminal原本digest（`execution/native_result.rs::Collector::codex_terminal`の`serde_json::to_vec(turn)`のSHA-256。`Content.terminal_sha256`→`execution/native.rs`の`NativeResultReceipt.terminal_sha256`）は、所有`turn/completed`で受けた検査済みTurnのValue画像全体（items内のtolerated item内容を含む）のdigestとして維持する。これは原本の完全性値であり、strict部分への投影に置き換えない（既存の原本照合を弱めないため）。digestの使われ方は次の(a)(b)だけであり、(c)の意味上の使用とは区別する。

- (a) 形式・存在の検査: receiptの`validate`（`native_result.rs:239`の64hex形式、`:290`のComplete時の存在）。
- (b) 原本同一性の照合: digestは`NativeResultReceipt`の1 fieldとして、receipt全体の構造等価（derive `PartialEq`）またはserialized本文の完全一致に含まれる。field名で読む箇所ではないため、名前の検索では見つからない。現sourceでは次の4か所で、いずれも「同じ保存済みterminal・同じreceiptか」を決める照合であり、digestだけを取り出して読まない。(1) `execution/native/phase_protocol.rs:563-567`の`OwnedPhaseSettlement::completed`は、受けたreceiptが保存済みterminalのreceipt（authorityを`OwnedTerminal`にしたもの）と構造等価であることを成功settlementの条件にする。(2) 同`:273-279`の`settled`は、replay時に先行settlementのreceiptとの構造等価を要求する。(3) `state/execution/native_phase/terminal.rs:183-191`の`receipt_committed_tx`は、`native_results.body`と保存予定本文（`:543`の`serde_json::to_string(&receipt)`）の完全一致を要求する（`SettledTerminalImages::validate_tx`:162と`NativeTerminalPlan::committed_tx`:717から呼ばれる）。(4) `state/execution/native_results.rs:476-483`の`finish_native_result_tx`は、既存本文と再保存本文の完全一致を要求する（不一致は`changed native result replay refused`）。これらの照合は維持し、digestを比較対象から外さない（外すと、同じinvocationで別のterminal原本を持つreceiptを同一とみなし、原本照合を弱める）。不一致は各照合の既存の拒否になるだけで、成功・分類・回答・authorityを新しく作らない。
- (c) 意味上の使用はしない: digestの値で分岐・分類する、値を別の値（strict投影のdigest等）と照合して受理する、値からquota・auth・回答・authority・所有を導くconsumerはない。`projection()`（`native_result.rs:209-213`）はdigestを含まずstatus／resultへ出さず、`native_results`の索引列（`check_indexed`）にも含まない。

存在はturn/completedの受理で決まり、tolerated内容に依存しない。1つの実行ではterminalを1つだけ作り（`retain_saved_terminal`の`Arc::ptr_eq`）、(b)の各照合は同じterminalから作ったreceiptどうしを比べるため、tolerated内容の違いは(b)の結果を変えない。したがってtolerated内容だけを変えるとこのdigestは変わり、それ以外の記録と判断は変わらない。digestだけが違うreceiptは(b)の照合で拒否される（どちらもCX-8）。digestを(a)(b)以外に使うconsumerの追加、(b)の照合からdigestを外す変更、(b)をstrict部分の比較へ限る変更は、この例外を見直す設計変更とする。

新しい宣言の扱いは次表で閉じる。

| 新しい宣言 | Schema gate | 実行時 |
|---|---|---|
| server request method | 拒否 | 着信はS/Pで拒否、SessionでUnknown→Held（従来どおり） |
| ThreadItem type | 許容追加（上限32、名前1〜64 bytesの`[A-Za-z0-9_]`） | 所有turnでUnknown→Held、成功0。foreignはid／type検査後に除外 |
| 通知method | params閉包が構造をもつstrict到達definition（§8規則2）へ到達せず（スカラー葉への到達は可）、所有キーが宣言されていればstring／nullable stringのときだけ許容追加（上限64、名前1〜128 bytesの`[A-Za-z0-9/_.-]`、既知method名と非衝突）。それ以外は拒否 | T-notifyと同じ。所有区分は宣言から決める（threadId requiredならThread、optional／nullableならOptional Thread、なければGlobal。turnIdも同様） |
| strict位置の新property（optional／nullableを含む） | 拒否 | - |
| used enumの新値（MessagePhase、TurnStatus、ThreadStatus、CodexErrorInfoの既知値集合、WebSearchMode、AskForApproval、SandboxMode、SandboxPolicy type、startup status、rateLimitReachedType、authMode、environment status等） | 拒否 | - |
| tolerated位置の追加・削除・変更 | 許容。ただしtolerated anchorから構造をもつstrict到達definitionへの新しい到達（§8規則2）、strict到達definition自体の変化（規則1）、所有キーの型変更は拒否 | 保持のみ |
| strict位置・strict通知の削除 | 拒否 | - |
| tolerated既知通知の削除 | 許容 | - |

ThreadItem typeと通知methodのgate許容追加は、そのgateを実行したSのHandshake、またはPと同じUnitの本番Session decoderへ、メモリ上の有界集合（名前と所有区分）として渡す。Session前のprogram identity再検査（X-1）でgateを実行したprogramとの一致を確認する。この集合はauthorityではなく、journalには件数とdigestだけを記録し、journalから復元しない。契約にもgate許容追加にもないmethod／typeの実着信は従来どおりProtocolErrorとする。

以下の表の記法は、`s=string`、`b=boolean`、`i=int64`、`T=threadId:s`、`U=turnId:s`、`I=itemId:s`、`[]`は型付きarray。IDには既存の有界ID制約を追加する。表は**basisの全84通知の必須payload**を記す。実行時に全payloadを型検査するのはS-notifyの21通知だけで、他は所有キーだけを型検査する（上表）。同じ行にある複数methodは同一形を共有する。

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
| `thread/prediction/updated` | T,sourceTurnId:s,result:ThreadPredictionResult（type=`completed`（optional text:string/null）／`failed`） |
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

optional fieldもpresentなら型検査する。特にthreadName、projectId、turnId、error、reason、details、targetItemId、environmentIdなどのnullableとoptionalを混同しない。以下の所有列を上のpayload表へmethodで自然結合し、全84methodがちょうど1区分に属することを検査する。列挙中のmethodはそれぞれ独立methodであり、prefixによる未知methodの受理ではない。

| method集合（件数） | 所有区分／キーと型 | S/P握手中 |
|---|---|---|
| error、thread/tokenUsage/updated、turn/diff/updated、turn/plan/updated、item/started、item/autoApprovalReview/started、item/autoApprovalReview/completed、autoApprovalReview/strictReviewRequired、item/completed、item/agentMessage/delta、item/plan/delta、item/commandExecution/outputDelta、item/commandExecution/terminalInteraction、item/fileChange/outputDelta、item/fileChange/patchUpdated、item/mcpToolCall/progress、item/reasoning/summaryTextDelta、item/reasoning/summaryPartAdded、item/reasoning/textDelta、thread/compacted、model/rerouted、model/verification、modelProvider/authRecoveryStarted、modelProvider/authRecoveryCompleted、turn/moderationMetadata、model/safetyBuffering/updated（26） | Thread＋Turn。params.threadId:string、params.turnId:stringとも必須。 | strict検査後に拒否 |
| thread/status/changed、thread/archived、thread/deleted、thread/unarchived、thread/closed、thread/reverted、thread/name/updated、thread/attachment/updated、thread/prediction/updated、thread/goal/cleared、thread/queue/changed、thread/project/updated、thread/environment/connected、thread/environment/disconnected、thread/settings/updated、serverRequest/resolved、guardianWarning、thread/realtime/started、thread/realtime/itemAdded、thread/realtime/item/started、thread/realtime/item/transcript/delta、thread/realtime/item/completed、thread/realtime/transcript/delta、thread/realtime/transcript/done、thread/realtime/outputAudio/delta、thread/realtime/sdp、thread/realtime/error、thread/realtime/closed（28） | Thread。params.threadId:string必須。 | strict検査後に拒否 |
| thread/goal/updated、hook/started、hook/completed（3） | Thread＋任意Turn。params.threadId:string必須、params.turnIdはoptional string/null。非nullだけ追加の所有比較。 | turnId欠落／nullでも拒否 |
| thread/started（1） | Nested Thread。params.thread.id:string必須。Thread全体も検査。top-level threadId不在を全体通知としない。 | 拒否 |
| turn/started、turn/completed（2） | Thread＋Nested Turn。params.threadId:string、params.turn.id:string必須。Turn全体も検査。 | 拒否 |
| mcpServer/oauthLogin/completed、mcpServer/startupStatus/updated、warning（3） | Optional Thread。params.threadIdはoptional string/null。欠落／nullなら全体、非nullならThread。 | 欠落／nullなら正常全体通知として続行、非nullなら拒否 |
| skills/changed、project/changed、command/exec/outputDelta、process/outputDelta、process/exited、mcpServer/event/stream/notification、account/updated、account/gatewayOAuth/changed、account/rateLimits/updated、app/list/updated、remoteControl/status/changed、externalAgentConfig/import/progress、externalAgentConfig/import/completed、fs/changed、deprecationNotice、configWarning、fuzzyFileSearch/sessionUpdated、fuzzyFileSearch/sessionCompleted、windows/worldWritableWarning、windowsSandbox/setupCompleted、account/login/completed（21） | Global。Thread／Turn所有キーなし。processHandle、processId、watchId、subscriptionId、sessionId等をNative Thread／Turn authorityへ変換しない。 | strict-validなら件数に算入して続行。rateは§9も適用 |

本番では存在する所有キーをすべて比較し、foreignはpayload全体の検査後にだけ除外する。任意Turnが欠落／nullでもThread比較は省略しない。Global通知はwire合法性だけで入力・回答・権限authorityを発行しない。`thread/goal/updated`はT-notifyであり、goalの内容を検査・使用しない（params.threadIdの所有比較だけを行う）。S/P握手ではaccount/updated.authModeがpresentでchatgpt以外（null含む）、またはaccount/login/completed.success=falseなら相反観測として拒否し、後の正常account/readで打ち消して資格付けない。authMode欠落は肯定証拠にせず、成功通知自体もchatgpt資格付けの代替にしない。

補助型は、basis catalog（§12）でsource schemaごとの`local_definition_digests`から`shared_definitions`へ解決したdefinitionを開発時根拠にする。rrx予定型名とsource definition名は同名とは限らない。strict位置で型付けする補助型の対応は次のとおり。GuardianApprovalReviewAction、GuardianApprovalReview、HookRunSummary、ThreadRealtimeItem、ExternalAgentConfigImportTypeResult、FuzzyFileSearchResult、TurnPlanStep、AppInfo、ThreadGoal、token usageはT-notifyの内容であり、rrx型を作らない。

| source definition | rrx予定型 |
|---|---|
| FunctionCallOutputBody | FunctionOutput |
| RateLimitSnapshot | RateSnapshot |
| FileSystemSpecialPath | SpecialPath |

その他の同名型もsource schema名とdefinition名を明記する。転記範囲はstrict位置（上の二分類表）から到達する**宣言済みproperty／required／enum／型／数値域／defaultの全閉包**であり、optional／nullable、配列要素、map値も含めてrequiredだけの投影に縮めない。tolerated位置は閉包の転記対象にしない。出所definition→rrx型→decoder→consumer対応表を実装成果物に含める。static catalogは開発時の観測資料であり、runtime validatorの仕様生成器ではない。全JSON dump、runtime生成schemaからの型生成、schemaをvalidatorとして実行する方式でこの固定rrx契約を代替しない。

strict位置の内部discriminatorは少なくとも次を有限化する。

- ThreadStatus：notLoaded／idle／systemError／active。activeはactiveFlags配列を要求。
- FileSystemPath／SpecialPath、AskForApproval（`untrusted/on-request/never`と閉じたgranular枝）、SandboxPolicy（dangerFullAccess／readOnly／externalSandbox／workspaceWrite）、SandboxMode（`read-only/workspace-write/danger-full-access`）：各tagと枝別構造。
- ThreadSettings、permission profile（RequestPermissionProfileを含む）、server request params：配列要素・map値・nullable参照を再帰検査し、objectだけでは通さない。LegacyAppPathStringとAbsolutePathBufはsourceのstring形を保ち、後者には既存absolute-path業務条件を適用する。

GuardianAction（7枝）、GuardianReview、HookRun、RealtimeItem、ImportResult、AppInfo、SearchResult、ThreadGoal、token usage、SessionSource／SubAgentSource（Thread.source）はtolerated位置にだけ現れるため型付き検査しない（E-05／E-07）。server request methodの`applyPatchApproval`等をtolerated内容から読み取らない。

`OpaqueJson`の免責集合は、strict領域内にsourceが型なし（`true`、`{}`、`additionalProperties: true`）で宣言した位置だけを次表で閉じる。位置はsource definitionのproperty pathであり、枝をまたいで拡大しない。任意JSON自身にはnullを含められるが、外側のtyped容器の型・required／optional／nullableは維持する。tolerated領域内の型なし位置（McpServerEventNotification.params、McpToolCallResultの3位置、ThreadItemのmcpToolCall／dynamicToolCall arguments、webSearch results[]、ThreadRealtimeItemAddedNotification.item、TurnModerationMetadataNotification.metadata、Tool／Resource／serverCapabilities等）はtolerated区分に含まれ、本表に載せない。

| source位置 | 外側の契約 |
|---|---|
| DynamicToolCallParams.arguments | 必須任意JSON |
| McpServerElicitationRequestParams[mode=openai/userVerification・form・openai/form・openaiForm・url]._meta | 5枝それぞれoptional任意JSON |
| McpServerElicitationRequestParams[mode=openai/form・openaiForm].requestedSchema | この2枝だけ必須任意JSON |
| DynamicToolNamespaceTool[type=function].inputSchema、DynamicToolSpec[type=function].inputSchema | それぞれoneOf/0の必須任意JSON |
| InitializeCapabilities.extensionsのmap値 | extensionsはoptional object/null、値だけ任意JSON |
| ThreadStartParams.configのmap値 | configはoptional object/null、値だけ任意JSON |
| TurnStartParams.outputSchema | optional任意JSON（sourceは型なし`{}`） |
| GetAccountRateLimitsResponse.rateLimitUpsell | optional任意JSON（sourceは型なし`{}`）。quota／可用性判断に使用しない |
| ConfigReadResponse.config（Config）の`additionalProperties: true`領域のうち`features`と`mcp_servers`以外のkey（`permissions`、`model_providers`等）、および`mcp_servers`各entryの`enabled`以外のkey | 値だけ任意JSON。`features`と`mcp_servers.<name>.enabled`は§8のrrx refinementで型付けし、本免責に含めない。M0のconsumerは免責位置を読まない（§8a(3a)）。内容は保存・監査しない（E-08） |

McpServerElicitationRequestParamsのmode=formではrequestedSchemaは必須`McpElicitationSchema`であり、型付きproperties／required／primitive schema閉包を検査する。mode=openai/userVerification／urlへrequestedSchemaの免責を追加しない。共通必須serverName／threadIdとoptional nullable turnIdも検査する。

選択した12source schemaのstrict位置と到達definition閉包に明示された`true`／型なしschema位置について、この表との双方向集合一致を検査する。typed枝を免責へ広げず、欠落免責を暗黙追加しない。未知method・status・discriminatorをopaqueへ落とさない（ThreadItem typeと通知methodのgate許容追加は上の規則だけ）。opaqueもstrict JSONと既存byte／depth／nodes／文字列／容器上限の対象であり、構造外のauthority・業務成功証拠にしない。Thread.extraはT-threadであり、明示typed mapとしての照合はしない。

**ThreadItem全19枝**

全枝でid／type（string）を要求する。内容を型検査するのはagentMessage・commandExecution・fileChangeだけ（S-item）で、他の16枝の追加fieldはbasisの宣言を記録するがT-itemとして保持だけ行う。追加必須は以下とする。

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
| subAgentActivity | agentPath:s,agentThreadId:s,kind:`started/interacted/interrupted/completed`（optional model:string/null、reasoningEffort/null） |
| webSearch | query:s |
| imageView | path:s |
| sleep | durationMs:uint64 |
| imageGeneration | result:s,status:s |
| enteredReviewMode／exitedReviewMode | review:s |
| contextCompaction | なし |

UserInputはtext／image／localImage／audio／localAudio／skill／mention。textはtext文字列、imageはurlまたはfileId、他は各path／url／nameを要求する。UserInputとFunctionOutput contentはturn/start送信の閉包（S-send）で、CommandActionはcommandExecution枝（S-item）で入れ子の有限枝を検査する。WebSearchAction、ImageGenerationFailureはT-itemの内容である。

agentMessage.phaseはoptional nullable、presentならcommentary／partial_answer／final_answer。回答に採用するのはfinal_answerだけで、partial_answerはcommentaryと同じく回答に採用しない（E-12）。欠落／nullはwire合法だが回答承認には使わない。delivery／questionsもwire構造を検査した後、collectorの業務条件で採用可否を決める。

本番処理順はstrict envelope→method→payload（strict位置の全構造、tolerated位置は有界strict JSON）→所有フィルタ→consumer。正常toolは構造検査後に回答から除外できる。malformed foreign frameはProtocolError。S/P握手では全server requestをparams検査後に拒否し、通知は所有列の握手扱いを使う。正常なGlobal通知と、Optional ThreadでthreadId欠落／nullの通知だけを数えて続行する。Sの受理規則は変えず、B2b承認後のPに限り、共通の`Frame::handshake_notification`を通した後、P専用関数でconfig/readとMCP起動通知の相反を検査する（§8a(3a) B2b-2）。Codex Executorの本番Sessionでは、構造検査を通った正常なforeign通知・itemは除外して続行し、所有turn内で作業を別thread／turn／agentへ広げるitemと§8a(3a)の許可集合外itemだけを§8a(4)のUnknown→Heldとする。

既知Codex wire frame内の未宣言fieldは、Claudeと同じく有界strict JSONとして保持受理し、業務・authorityには使わない。strict位置の既知fieldの不正型、未知method／tag／enum、branch不適合は拒否する。このwire追加field規則と、Schema gateのstrict／tolerated比較規則（§8）は別である。schemaのadditionalProperties未指定を、未知methodやtyped field不正を受理する根拠にしない。明示typed mapとOpaque位置以外に、新しい宣言propertyを自動追加しない。 例外としてsourceでadditionalProperties=falseのclosed objectは、strict位置から到達する場合に宣言key以外を拒否する。basis catalogのshared definitions全体では29位置（8 method閉包。0.160.1の8 method閉包でも29で、6 method閉包の27からの増分2はConfigRead閉包のWebSearchLocation／WebSearchToolConfigであり、T-config内）である。数から契約を生成せず、strict位置のref解決後閉包にある各位置のproperties／required／additionalPropertiesを保持する。tolerated位置だけから到達するclosed objectは有界strict JSONとして保持する。externally-tagged／key-tag unionはtag keyの存在もdiscriminatorであり、既知tag keyはちょうど1つ、0または複数は拒否する。CodexErrorInfoの5 object枝（4 HTTP＋activeTurnNotSteerable）にも適用する。

CodexErrorInfo（S-thread）はbasisでanyOfになり、既知15文字列・5 object枝に加えて`{"type":["string","object"]}`の開放枝を宣言する。rrx decoderは既知値と既知枝を先に照合し、既知tag keyを1つだけ持つobjectが既知枝に適合しなければProtocolError、既知tag keyを複数持てばProtocolErrorとする（開放枝で不正な既知枝を救済しない）。既知tag keyを持たないobjectと未知文字列（既存の文字列上限内）だけを`CodexErrorKind::Unclassified`とする。Unclassifiedはsubscription exhaustion・auth・capacityのどれの根拠にもせず、Availableにもしない。所有turnのfailedでUnclassifiedならKnown FailureではなくUnknown→Held（有限reason `codex error kind unclassified`）とし、`error`通知のUnclassifiedはquota観測を作らずturn終端の分類へ委ねる（E-11）。

JSON-RPC envelopeの判別keyはid／method／params／result／errorであり未宣言追加field扱いしない。受信95 methods（server request 11／通知84）とgate許容追加の通知methodは、requestならid＋method＋params、notificationならmethod＋paramsかつidなしを要求する。success responseはid＋resultかつmethod／params／errorなし、error responseはid＋errorかつmethod／params／resultなしを要求する。送信requestのparams presenceは各source branchのrequiredに従い、`account/rateLimits/read`はid／method required・params optional nullableで、現rrxのnative.rs:2158は`{}`を送る。送信ClientNotificationの`initialized`はparamsなしであり、optional nullableなrequest paramsとは別契約である。送信使用8 methodsの既存wire検査とCX-3／CX-4を緩めない。idは有界stringまたはsource整数域でnull不可、methodは有限method。result＋error、method＋result/error、id:null、initialized＋paramsを拒否し、result:null等のpayload nullabilityは選択済response型で別検査する。判別key以外の追加fieldはclosed objectでない既知位置だけ既存のbounded保持規則を適用する。

server requestは既知11種すべてのparamsを先に検査する。S/P握手では全種を拒否する。本番turn/start送信前の他のboot_callでは既存の`-32601`返信・待機継続を維持し、証拠不成立は`ExecutorCapabilityUnverified`／NativeInput 0とする（§8a(3a)）。M0のCodex Executor Sessionは§8aの固定approval方針で承認要求を生じさせない契約とし、turn/start送信から所有turnの終了まで（turn/start応答待ちを含む全受信経路）に着信した11種はparams検査後に既存ledgerへ渡さず、応答（`-32601`を含む）も送らずにUnknown→Heldへ接続する（§8a(3)）。既存ledgerの承認2種対応と`submit_approval`経路は#10の範囲として存置し、M0で呼出し元・権限応答能力を増やさない。

本番のturn/start送信直前のqueue処理は§8a(3a)の「送信前queue処理」に従う。

### 8. Schema gateと実握手

Schema helperは`app-server generate-json-schema --experimental --out <new-dir>`。生hash一致を要求しない。

4 envelopeに加え、Initialize／Account／Rate／Environment／ConfigRead／McpServerStatusList／ThreadStart／TurnStart responseの生成ファイルを読み、§7のfinite contractと比較する。file名はfixture manifestで固定し、任意探索で同名definitionを拾わない。

生成schema filesのsinkは、自uidの新規0700 directory、symlink禁止、1024項目、総16 MiB、directory深さ4、各読取file1 MiBまで。JSONは深さ64、262144 nodes、文字列64 KiB、object／array4096。16 MiBは生成file総量の上限であり、process stdout＋stderrの64 KiB捕捉や準備aggregate8 MiBとは別sinkである。既存上限を増やさず、生成fileをprocess捕捉枠に混算しない。

local refは同一fileの`#/definitions/`のみ。512は各source fileで解決・memo化する異なるdefinition target数の上限であり、再参照回数や非メモ化展開node数ではない。keyは検査済みfile identity＋definition名、memoは最大512 entry、現在の解決stack／ref深さは16までとする。stack内target再訪を循環として拒否し、completed memoへの再参照は512を再消費しない。比較側も既処理のschema-node／rrx-type対をmemo化し、全再参照・分岐の処理を含むwork-node数を各file262144以下に制限する。raw JSON node数とwork-node数を別々に検査し、memoによる無制限な反復処理を許さない。循環・未解決・外部refを拒否する。

PのSchema出力先はSAME Unitの既存tmp直下にrrxが新規作成する専用childとする。既存pathを再利用せず、custodyは作成したdirectory identityとUnit／actorの対応を保持する。S/Pともcleanupは当該runで作成した同一directoryだけに限定する。Unknownや強制死では旧epoch／Unitの生成fileが残りうる。後続runで再利用・自動復旧・旧残骸の自動削除をせず、残存を未清掃として記録する。M0に追加cleanup工程を要求しない。

**適合意味**

- method枝は単一enumの一意枝。入れ子oneOfもtagの単一enumまたは相互排他的構造で一意に選択する。
- `$ref`は解決してから比較する。allOfは全制約の積、anyOfはunion、oneOfはちょうど1枝。type配列はunion、nullableは明示null枝だけ。
- boolean schema／型なしschemaを任意型扱いするのは、strict位置ではrrx契約が明示OpaqueJsonとする位置だけ。closed object／key-tagの制約もstrict位置のref閉包ごとに比較する。
- enum、integer／number、minimum／maximum、minLength、items、additionalProperties、defaultを含めて意味比較する。int32／uint16／uint64等のwire域を保ち、曖昧なformatを無制約へ落とさない。description／title／examples／`$schema`は、schema位置のkeywordであるときだけ注記として比較しない（basis catalogの生成規則と同じ、§8b）。default／enum／const等の値位置の中にある同名keyや`$ref`はJSON値の一部として比較し、`$ref`として解決しない。
- 送信値は実producerが生成可能な値集合のすべてがCLI宣言に適合し、CLI requiredを満たす。
- 受信側はCLIが宣言する既知fieldの型・nullability・enum・枝がrrx decoderの受理範囲を越えないことを要求する。業務上の追加条件は別に検査する。
- 未知の宣言field／method／discriminatorは、§7の新しい宣言の表が許容追加とするもの以外を受理しない。未使用ClientRequest methodは今回送らないため対象外。

**新しいCLIの判定（C1）**: gateは生成schemaの各位置を§7の二分類で照合し、次の順で判定する。番号・raw hashは使わない。

1. strict位置の一致: 各strict位置について、ref解決後の宣言が契約と意味的に等しいことを要求する。等しいとは、property名集合（optional／nullableを含む）、required集合、type集合（null枝を含む）、enum値集合、枝集合と各枝の構造（oneOf／anyOf／allOfの区別を含む）、数値・文字列・配列の制約、additionalProperties、defaultがすべて一致することである。default／enum／constはJSON値として完全一致を比較する。両側のrefをたどるときはdefinition名も一致することを要求し、構造が同じでもrenameはstrict_mismatchで拒否して§8bのrefresh手順へ回す。property・枝・enum値の追加、削除、変更はどれも不一致とする（optional／nullableの追加を含む）。strict到達集合（規則2）の各definitionは、tolerated anchorからも到達していても本規則で比較する。
2. strict到達と新しい到達: 次の手順をsource fileごとに、basisと生成schemaの両方へ同じ関数で適用する。
   - strict到達集合: strict位置からref解決をたどって到達するdefinitionの集合。strictとtoleratedが混在するdefinitionでは、tolerated部分位置へは降りない。混在definitionと部分位置は、Thread（core以外のproperty）、ThreadItem（agentMessage・commandExecution・fileChange以外の枝のid／type以外）、TurnError（misalignment）、Config（web_search以外の宣言property）、McpServerStatus（name・tools・toolsError以外のpropertyとtools mapの値）、ConfigReadResponse（originsとlayersの内容）である。
   - tolerated anchor: T-notifyの各method（所有キーを除くparams）、T-itemの各枝の内容、上記の各tolerated部分位置。名前はmethod名、`ThreadItem[<type>]`、`<definition>.<property>`とする。definition由来のanchorは比較rootから比較が実際に到達したdefinitionだけから作り、未参照definitionは寄与させない。同じanchor keyに異なる内容が届けばstrict_mismatchで拒否する。
   - 到達組: 各anchorから閉包全体をたどって届くstrict到達集合のdefinitionとの組（anchor, definition）。definitionは両側で一致を確認した名前で表す。
   - 構造の判定: ref解決後のschemaに`$ref`・`properties`・`patternProperties`・schemaとしての`additionalProperties`・`items`・`prefixItems`、またはtypeとして`object`／`array`を含むdefinitionを「構造をもつ」とし、それ以外（type・enum・const・format・数値／文字列制約だけで表せるもの）を「スカラー葉」とする。
   - 判定: 生成schemaの到達組のうち、definitionが構造をもち、かつbasisの到達組集合にない組があれば`strict_reach`で拒否する。新しいanchor（新通知、新ThreadItem type、tolerated位置の新property）はbasisの組を持たないため、構造をもつstrict到達definitionへ届けば拒否となる。スカラー葉への組は新旧とも拒否理由にしない。スカラー値は効果や結果の構造を運べず、同じ値をinlineの`string`で宣言されれば本規則では検出できないため、ここで止めても防げる範囲は増えない。スカラー葉の宣言自体は規則1でstrictに比較される。
   - basisの値: 0.162.1 basisでは、tolerated anchorからstrict到達definitionへの組はServerNotificationに13組（`hook/started`・`hook/completed`・`fs/changed`・`item/autoApprovalReview/started`・`completed`→AbsolutePathBuf、`item/autoApprovalReview/*`・Thread.environments・`ThreadItem[imageView]`→LegacyAppPathString、Thread.reasoningEffort・`ThreadItem[collabAgentToolCall]`・`ThreadItem[subAgentActivity]`→ReasoningEffort、`ThreadItem[imageGeneration]`→AbsolutePathBuf）、ThreadStartResponseに6組、TurnStartResponseに1組あり、すべてスカラー葉である。構造をもつdefinitionへの組は0である（執筆時に§7の分類で求めた補助計算。実装ではgateの同じ関数をbasisへ適用して組集合をfixture manifestに記録し、この件数と照合する）。したがって無変更のbasisは本規則を通り、既存hook通知も通る。
   - 意図: tolerated位置を経由して、効果・結果・判断を運ぶ構造（Turn、ThreadItem、CommandAction、FileUpdateChange、CodexErrorInfo等）が型検査されない経路に新しく現れることを、起動前に止める。basisの組集合へ構造をもつ組を加える更新は§8bの「tolerated集合を広げる変更」としてオーナー承認の対象とする。
3. tolerated位置: 追加・削除・変更を許容する。所有キーの宣言は型（string／nullable string）とrequired性が契約と一致することを要求する。
4. 新しい宣言: §7の表に従う。server request methodの追加、strict通知・送信methodの削除、used enumの新値は拒否する。許容追加はThreadItem type 32件・通知method 64件まで、名前は§7の文字種と長さに限る。超えたら拒否する。
5. 既存の生成file sink、JSON、ref、memo、work-node上限は変えず、tolerated位置の読取りにも同じ上限を適用する。

決定記録（SEC-R4-01）: 同型のgate不備が反復したため、definition名変換と全live definitionからのanchor収集を廃止する。これは受理範囲の縮小であり、§8b手順8により追加のオーナー承認は不要。

結果は`Pass{ tolerated_changes, accepted_item_types, accepted_notifications }`か`Refused{ reason }`の有限値であり、journalには結果、有限reason、件数、strict不一致の最初の8件の位置だけを記録する。reasonは`strict_mismatch`（kind: added／removed／type／required／enum／branch／constraint／closed／default）、`strict_reach`、`ownership_key`、`server_request_added`、`strict_removed`、`addition_limit`、`addition_name`、`ref`、`limit`、`file`のいずれか。位置はsource名（固定manifestの名前）と、definition名・property path（`strict_reach`ではanchor名とdefinition名）があれば§10の版数と同じ規則（96 bytes以下かつ`[A-Za-z0-9 ._()+/:-]`なら原文、それ以外はSHA-256とnonrecordable）で記録する。生schema、schema本文の抜粋、tolerated位置の内容は保存しない。S bodyの4096 bytesとreceiptの既存上限を増やさず、超える分は件数だけにする。Passはcapability・authorityを発行せず、許容追加の集合は§7のとおりメモリ上でだけdecoderへ渡す。

`initialized`はschema上methodしか宣言しない。S/P／本番producerを`{"method":"initialized"}`へ変更し、`params:{}`を送らない。これをschemaの例外で救済しない。

送信契約はinitializeのclientInfo／experimentalApi、account/readのrefreshToken=false、environment/statusのlocal ID、rate readの空params、config/readのcwd（Unit Worktree）／includeLayers=false、mcpServerStatus/listのthreadId／limit=100／cursor=null（この2 methodはExecutorだけ、§8a(3a)）、thread/startのcwd／environments／任意model／Reviewer sandbox／Executorの固定approval方針・sandbox（§8a）とdynamicTools等の不送信、turn/startのthreadId／text input／任意effort／Executorのapproval方針・sandboxPolicy（§8a(3)）を、実producerと共通の型で表す。

responseは以下も対象にする。

- Initialize：4必須文字列と既存platform／absolute path業務条件。
- Account：requiresOpenaiAuth boolean、nullable accountのapiKey／chatgpt／amazonBedrock枝。資格付けはchatgptのみ。
- Environment：ready／pending／disconnected／unknown。資格付けはreadyのみ。
- ThreadStart：thread、cwd、modelProvider、model、approvalPolicy（AskForApproval）、approvalsReviewer、sandbox（SandboxPolicy）等の必須構造とtop-levelの全宣言。Executorでは応答のapprovalPolicy・sandboxが§8aの送信値と一致することも要求する。threadはThread coreだけを型検査する（§7）。
- TurnStart／Turn：id、items、statusとoptional error／timestamps／rootTurnId（string/null、業務非使用）を含むTurnの全宣言。
- TurnStatus：completed／interrupted／failed／inProgress。
- CodexErrorInfo：既知15文字列、4 HTTP object枝、activeTurnNotSteerable枝、`string/object`開放枝（anyOf）。HTTP statusはnullable uint16、turnKindはreview／compact。開放枝の扱いは§7（E-11）。
- RateLimitReachedType：既知5値とnullable。
- ConfigRead（Executorだけ）：rootはconfig（Config）とorigins（ConfigLayerMetadataのmap）がrequired、layersはnullable array。rrxはincludeLayers=falseを送るため、layersはabsent／nullだけを受理し、非nullは`ExecutorCapabilityUnverified`とする。Configはrequiredなし・`additionalProperties: true`のobjectであり、判定pathのうち`web_search`だけが宣言property（nullable WebSearchMode `disabled/cached/indexed/live`）である。`features`と`mcp_servers`はsourceでは開放領域のkeyなので、rrx refinementとして次の型で読む: `features`はabsent／null／object、objectならmap値はboolean（§8a(3a)の対象名はpresentかつfalseを要求）。`mcp_servers`はabsent／null／object（最大4096 entry）、各値はobjectで`enabled`はpresentならboolean。refinementに合わない値は欠落へ丸めず`ExecutorCapabilityUnverified`とする。Configの他の宣言property（sandbox_mode、approval_policy、approvals_reviewer、model_provider等）とoriginsはT-config、開放領域の他のkey（`permissions`、`model_providers`等）と`mcp_servers`各entryの`enabled`以外のkeyは§7免責表の1行であり、判断に使わない。本文は保存・監査しない。
- McpServerStatusList（Executorだけ）：rootはdata（McpServerStatus[]、required）とnextCursor（string/null）。McpServerStatusはname（string）・tools（Toolのobject map）がrequired、toolsErrorはstring/null。rrxはnameとtoolsのkey数とtoolsErrorとnextCursorだけを読み、Toolの値・resources・resourceTemplates・authStatus・runtimeStatus・serverInfo・serverCapabilities等はT-inventoryとして保持だけ行う。

現契約のS/P握手はinitialize→initialized→account/read→environment/status→rate read。B2b承認後だけP Executorは末尾にconfig/readを加える（§8a(3a) B2b-1）。PのConfigReadも既存S-send（ConfigReadParams全宣言）／S-config（ConfigReadResponse判定pathとrefinement）を使い、T-config／免責位置へ読み取りを広げない。schema宣言だけではRPC成功を代替しない。schema適合でも`-32601`なら拒否する。S/P metadataをquota poolへ書かない。

### 8b. 契約の更新手順（C4）

Schema gateがstrict位置の不一致または許容追加の規則外で拒否した場合だけ、次の手順で契約を更新する。tolerated位置だけの変化はgateが通すため、この手順を要しない。

1. 拒否のjournal（有限reasonと位置）から対象のCLI版数を確認する。版数は観測記録であり、許可根拠にしない。
2. 公開source `openai/codex`の対応tag（`rust-v<版数>`）のtag object・commitを記録し、`codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`のSHA-256とfile数を記録する。tagが無い、またはexportが無い版はBLOCKEDとし、CLI出力からの推定で代替しない。
3. 記録済みの生成規則（§12、`scripts/build-codex-catalog.py`。source rootの選び方、使用methodだけのClientRequest枝、schema位置だけをたどる走査、schema位置のkeywordとしての注記（description／title／examples／`$schema`）の除去、値位置（default／enum／const／required／type／format／数値・文字列・配列の制約）の原文保持、`definitions`の除去、schema位置の`$ref`だけをたどるrootからの推移的ref閉包、compact sorted-key JSONのSHA-256によるdefinition digest）で新catalogを作る。schema位置はroot、schema map（properties、patternProperties、definitions、$defs、dependentSchemas）の各member、単一subschema（additionalProperties、additionalItems、items、contains、propertyNames、not、if、then、else、unevaluatedProperties、unevaluatedItems）、subschema列（allOf、anyOf、oneOf、prefixItems、配列形のitems）である。どちらの集合にもないkeywordが現れたら生成器は失敗し、規則の明示的な変更（と次の再現確認）を要する。
4. 再現確認: 同じ生成器と規則で直前basisのtag exportから直前basis catalogを作り、subtree digestが記録値と完全一致することを要求する。一致しなければ生成器か規則が変わったとみなし、そのままでは新catalogを採用しない。意図した規則変更が直前basis catalogの内容を変える場合は、直前basis catalogの新旧差分を位置ごとに列挙して理由を設計に記録し、手順7のレビュー（差分が直前basisの受理を広げる向きなら手順8のオーナー承認も）を経て、新しい規則での直前basisの値を新catalogのfileへ追記する（旧値・旧fileは書換えない）。以後の再現確認はその記録値と比べる。生成器か規則を変更した場合は、再現確認に加えて次の(a)を通し、結果を記録する。
   - (a) 値位置のcontrol（全版の合格条件）: schema位置の注記だけが違う対はdigest一致、default／enum／const内の同名keyや`$ref`だけが違う対はdigest不一致、値内の`$ref`は閉包へ入らない、未知keywordで失敗。変更後の生成器はこれを全件満たすこと。変更前の生成器も正しければ同じく全件満たすのが正常であり、変更前の生成器の結果は合否に使わない（正しい直前生成器がcontrolを通すことは更新を止めない）。
   - (b) controlの検出力確認（固定mutant）: 値位置を区別しない当初の欠陥を再現する固定mutant（commit `6741d530`の`scripts/build-codex-catalog.py`。注記keywordの除去と`$ref`の追跡を値位置にも適用し、未知keywordで失敗しない）を(a)へ与え、値位置の4項目（default内の同名key、enum内の同名key、const内の`$ref`、default内`$ref`の非追跡）と未知keywordの失敗のそれぞれで(a)が不合格になることを確認する。この確認はcontrolを定めた時点で1回行い（結果は§12）、以後はcontrolの対や期待値を変えた場合だけ同じ固定mutantで再実行する。生成器の通常の変更では再実行を要求しない。
5. 旧basisと新catalogの差分を位置ごとに列挙し、§7の二分類で分類する。strict位置の差分ごとに、型付き契約・decoder・consumer・fixture・テスト・影響範囲を更新する。tolerated位置の差分は件数と一覧を記録だけする。
6. 新catalogは別のverification fileとして追加し、旧basis catalogを書換えない。出所（tag、tag object、commit、export SHA-256、source数、共有definition数、subtree digest、再現確認の結果）を同じfileに記録する。
7. 設計差分を非執筆側1系でレビューし、実装は通常の実装レビュー（初回2系）と必須検証を通す。
8. オーナー承認は、strict位置を広げる変更（rrxが受理する値・枝・enum値・item type・server requestを判断位置で増やす、型を緩める、strictからtolerated／Opaqueへ移す、送信値・approval方針・sandboxを変える）と、§7の免責表またはtolerated区分を広げる変更（basisの到達組集合へ構造をもつstrict到達definitionへの組を加える変更を含む、§8規則2）に限って求める。strict位置の一致だけを取り戻す変更（consumerが読まない宣言の取込み、型を狭める、tolerated通知のstrict化）はレビューで閉じ、オーナー承認を要しない。どちらに当たるかはレビューで確定し、判断がつかなければオーナーへ回す。
9. 新basisの実CLI確認（インストール版の`generate-json-schema --experimental`出力がgateを通ること、握手とExecutor Sessionが成立すること）はowner-runで行い、static catalogやfixtureの合格をlive合格としない。

### 8a. Codex Executor の実行経路

適用範囲: M0で本番提供するExecutorはCodexであり、本節はPの後にtransportで起動する本番Sessionの契約である。§7のfinite decoder、§8の握手と送信契約、§9のrate契約、§10のidentity・停止契約をそのまま使い、緩めない。

**現sourceの事実**: `execution/native.rs`のCodex Sessionは、initialize→initialized→account/read→environment/status→account/rateLimits/read→thread/start（cwd＝Unit Worktree、local environmentのruntimeWorkspaceRoots＝Worktree／Unit temp／output／cache、任意model）→ACK→turn/start（text input、任意effort）→通知loopの順に進む。Reviewerだけ`sandbox:"read-only"`を送り、Executorはapproval方針もsandboxも送らないため、実効値はCLIの継承設定で決まる。server requestは`codex::managed::Approvals`（`codex/protocol.rs::ApprovalLedger`）のpendingへ入り、応答経路は`NativeAdapter::submit_approval`→`NativeSessions::approve`だけである。runtime／workflow／CLIにその呼出し元はなく、Executor turnで承認要求が来るとturnはcancel／stopまで進まない。turn/completedはcompleted→Success、failedは既存のquota／auth／capacity／Failure分類、それ以外はUnknown／Lostになる。Codexモデル経路のlive実行は未証明である。

現sourceの事実（役割選択・受信経路・終端）: aliasはproviderごとに登録され、`config.rs:92`の`AgentConfig`はroleを持たない。ExecutorはTaskの`executor`（`domain.rs:310`）で後から選ばれ、`adapter.rs:332`の`from_managed_config`はroleを判定しない。`runtime/installation.rs:383`の`installed_driver_composition`（`runtime/admission.rs:27`から呼ばれる）、`adapter/native.rs:152`の`allocate`、`execution/workflow_source/native_handoff.rs:504`の最初のExecutor handoffは、いずれもClaude providerのExecutorを拒否しない。応答待ちの`Lines::response`は待機中に着信したserver requestへ`-32601`を返して待機を続け（`execution/native.rs:1039-1041`）、`boot_call`（:1743-1782）はturn/start（:2293-2308）にもこの経路を使う。通知loopではserver requestがledgerへ入り、別thread／turnのframeは除外される（:2313-2314）。終端では成功原本を`stop_and_reap`より前に捕捉・保存し（:2033-2052）、後のhygieneはknown workを置き換えない（`doc/design/master/agent-execution.md:45`）。

**0. Executor providerの選択と拒否点**: alias登録とTaskのrole選択を分ける。`AgentRegistry::from_managed_config`はprovider別のalias登録を変えず、Claude aliasをReviewer用に受理する（installationでroleを判定しない）。TaskのExecutorがClaude provider aliasへ解決されることは、次の主拒否点で型付き拒否`ExecutorProviderRefused`（有限分類。§4のとおり固定文へ潰さない）とする。(1) `runtime/goal.rs:190`のGoal計画検証は、Task.executorのaliasが設定済みであることに加え、その`AgentConfig.provider`がcodexであることを要求し、Task作成前に拒否する。(2) `runtime/installation.rs:383`の`installed_driver_composition`は、保存済みTask（設定変更前に作られたものを含む）について`native_phase_port(&task.executor)`の`provider()`がcodexであることを、`runtime/admission.rs:27`のadmissionでDriver起動前に要求する。どちらもUnit・helper・Held生成とquota Admitより前である。多層防御として`execution/workflow_source/native_handoff.rs:504`の最初のExecutor handoffと`adapter/native.rs:152`の`allocate`（role=Executor）にも同じ述語を置くが、Unit生成後に到達しうるため主拒否点に数えない。Task.reviewersのClaude aliasは拒否しない。

**1. 起動とcustody**: transportはPrepared・登録後に既存の登録手順で起動する。現契約のargvは`command_argv(codex, Executor, …)`の固定3要素`app-server --listen stdio://`で、追加argvは0。`compat.rs::check_command`はこれとの完全一致を要求するため、最初のtransportにも追加できない。B2bの承認後だけ、(3a)のP決定原本から導出した制限argvを最初から付ける契約へ変更する（完全一致検査は維持）。環境変数による方針変更はしない。spawnは`rrx-spawn-hygiene`の消費関数（`spawn_tokio`等）だけを通し、`OwnedProcess::spawn(Command)`へ所有を移す。`clippy.toml`の直接spawn禁止を維持する。spawn直前に§10のprogram identityを再検査する。停止はstdin close、group signal、drain、leader reapを§10の手順で行い、両pipe EOFとreapが既知の場合だけ物理custody軸をKnownとする（work軸との分離は(5)〜(6a)）。macOSでpre_execによりforkする点は#98の残余リスクとして記録し、M0で解消しない。

**2. Worktree束縛**: thread/startのcwdとenvironmentのcwdは、Unitが保持する既存Worktree（authority原本由来の絶対path）だけとする。ThreadStart応答のcwdがWorktreeと一致し、modelProviderが`openai`であることの既存検査（native.rs:2281-2285）を維持する。Task本文やモデル出力からcwd・rootを作らない。

**3. approval方針とsandbox（既定はfail-closed）**: Executorのthread/startで、approval方針（AskForApproval）を「承認要求を出さない」固定値（catalog上の`never`相当）に、sandbox（SandboxMode）を`workspace-write`相当に明示する。書込み可能rootは既存runtimeWorkspaceRootsと同じWorktree／Unit temp／output／cacheの4つだけとし、network accessは無効、`/tmp`と`TMPDIR`の暗黙書込み許可は除外する。これらを表すfield（thread/startのapprovalPolicy／sandbox、turn/startの`sandboxPolicy`のworkspaceWrite枝など）と値名は固定catalogの宣言で確定し、§8の送信値検査（CX-3）を通す。宣言がない、または同じ意味を表せない場合はBLOCKEDとして設計へ戻り、`config` mapや環境変数で代替しない。ThreadStartParams.configは送らない。ThreadStart応答のapproval方針とsandboxは送信値との完全一致を要求する。より広いsandbox、別のapproval方針、欠落はturn/start前に拒否する（NativeInput 0、既存の応答不一致と同じ失敗経路）。

basis catalog（0.162.1）での宣言: ThreadStartParamsはapprovalPolicy（AskForApproval: `untrusted/on-request/never`または閉じたgranular枝、nullable）とsandbox（SandboxMode: `read-only/workspace-write/danger-full-access`、nullable）とruntimeWorkspaceRootsを持ち、書込みrootやnetworkを表すfieldは持たない。TurnStartParamsはapprovalPolicyとsandboxPolicy（SandboxPolicyのworkspaceWrite枝: writableRoots、networkAccess、excludeSlashTmp、excludeTmpdirEnvVar）を持つ。ThreadStartResponseのsandboxはSandboxPolicy（object）である。したがってExecutorはthread/startでapprovalPolicy=`never`とsandbox=`workspace-write`を、turn/startでapprovalPolicy=`never`とsandboxPolicy=`{type:workspaceWrite, writableRoots:[4 root], networkAccess:false, excludeSlashTmp:true, excludeTmpdirEnvVar:true}`を送る（defaultに頼らず明示する）。宣言の存在はcatalogで確認済みだが、ThreadStart応答のSandboxPolicyにwritableRootsがどう現れるか（4 rootとの完全一致で照合できるか）は実CLIで未確認であり、owner-runで確認する。一致を確認できなければ上のとおりturn/start前に拒否する。

所有threadの`thread/settings/updated`（S-notify）は、approvalPolicy・sandboxPolicy・cwdがrrxの送信した方針（approvalPolicy=`never`、上のworkspaceWrite方針、Worktree cwd）と一致する場合だけ続行する。異なる場合はturn/start前なら拒否、turn/start送信後はUnknown→Heldとする（E-14）。

rrxが自動で付与できるapprovalは0件とする。承認済みWorktree範囲の内側でも、commandExecution・fileChange・権限追加・network・MCP elicitation等へ自動accept／自動declineを返さない。この方針ではCodexが承認要求を出さない前提なので、turn/start要求の送信から所有turnの終了までに着信した11種のserver requestは、受信経路（turn/start応答待ちと通知loop）を問わず、§7でparamsをstrict検査した後、応答を送らず（`-32601`を含む）、ledgerへ登録せず、成功原本を捕捉せずにSessionを停止し、`WorkOutcome::Unknown`／`Disposition::ProtocolError`→Heldとし、自動再送しない。turn/start応答待ちには既存`Lines::response`の`-32601`返信と待機継続（`execution/native.rs:1039-1041`）を使わず、この停止規則を持つ受信経路で待つ。turn/start送信前の他のboot_callは既存の`-32601`返信・待機継続を変えず、証拠不成立は`ExecutorCapabilityUnverified`／NativeInput 0とする。`item/autoApprovalReview/*`と`autoApprovalReview/strictReviewRequired`も、turn/start送信から所有turnの終了までの同じ窓だけUnknown→Heldとする。既存ledgerと`submit_approval`経路は#10の範囲として存置し、M0で呼出し元を追加しない。

本番のturn/start送信直前のqueue処理は(3a)の「送信前queue処理」に従う。

新しい本番権限: この方針により、Codexは承認済みWorktree範囲のsandbox内で、1 turnの間、操作ごとの承認なしにコマンド実行とファイル変更を行う。MCP・web search・dynamic tool・collabは(3a)の肯定証拠で無効／不在を確認した場合だけturnを始め、これらの権限は含まない。rrxが権限応答を出すわけではないが、従来の本番Executor（Claude、`--permission-prompt-tool stdio`で権限要求をrrxのpending承認へ回す形）にない権限形であり、オーナー判断B2（§13、承認済み）による。承認要求を許してrrxが範囲内のfileChangeだけを自動acceptする代替案は、rrxの自動付与権限と#10の資格付けを要するため採らない。

**3a. ツール面の制限と事前の肯定証拠**: M0のExecutorが所有turnで受理するThreadItemは、userMessage、agentMessage、reasoning、plan、commandExecution、fileChange、contextCompactionの7枝だけとする（userMessage・reasoning・plan・contextCompactionは回答・authorityに使わない）。mcpToolCall、dynamicToolCall、webSearch、collabAgentToolCall、subAgentActivityは禁止した能力の行使、hookPrompt、functionCallOutput、imageView、sleep、imageGeneration、enteredReviewMode、exitedReviewModeはM0の許可集合外とし、所有turnで観測したらid／type検査後にUnknown→Heldとする（内容はT-item）。gate許容追加のThreadItem type（§7）も許可集合外として同じ扱いとし、成功にしない。所有turnの`item/mcpToolCall/progress`も同じ。これは事後の検出であり防止ではないので、モデル入力（turn/start）より前に、同じ本番Sessionのapp-serverから次の肯定証拠を得る。

1. environment/status・rate readの後、thread/start前に`config/read`（cwd=Unit Worktree、includeLayers=false。`codex/session.rs:1611`の既存形）を送る。実効設定で`web_search`が`"disabled"`、`codex/policy.rs`の`DISABLED_FEATURES`（Executorは`restricted_features`のとおりshell_tool／unified_exec／code_mode_hostを除く。apps、multi_agent／multi_agent_v2、browser_use、computer_use等を含む）が全てfalse、`mcp_servers`の全entryが`enabled=false`であることを確認する。確認に使う述語は、既存`DecisionPolicy::verify_configuration`の前半（web_search・features・mcp_serversの比較）と同じ条件を、判定pathの検査済み値だけを入力とする共有関数として切り出したものである。対象feature名はExecutorの固定集合、対象server名はrefinement済み`mcp_servers`のkey全体とする。`verify_configuration`の後半（`permissions.<profile>.filesystem`と`network.enabled`の比較）と`DecisionPolicy`の構築（`from_native`／`for_executor`。宣言propertyのsandbox_mode・approval_policy・approvals_reviewerや開放領域のdefault_permissionsを読む）、`provider_environment`（model_provider・model_providersを読む）はM0 Executor経路で呼ばない。M0のExecutorは`permissions` profileを送らず、sandboxとapproval方針はthread/startの送信値とThreadStart応答・`thread/settings/updated`（strict）で照合するため、`permissions`はM0の判断対象ではない。これらの既存呼出し元はstandaloneの`codex/session.rs`（§11の台帳対象）だけであり、M0で変更しない。`mcp_servers`のabsent／null／空表は設定server 0件として合法だが、それだけを能力不在の証明にせず2のinventoryを必須とする。`features`／`mcp_servers`はbasisのConfigで宣言propertyでないため、§8のrrx refinementの型で読み、型不適合・layers非nullも`ExecutorCapabilityUnverified`とする。
2. ThreadStart応答の一致確認後、turn/start前に`mcpServerStatus/list`（threadId、limit=100、cursor=null）を1 pageだけ送り、`verify_inventory_page`と同じ述語で全serverのtoolsが空かつtoolsErrorなし、nextCursorなしを確認する。nextCursor非nullは証拠不成立とし、追加pageを送らない（§5の行予算を固定するため）。
3. dynamic toolは、rrxがthread/startで`dynamicTools`等を送らないことをCX-3の送信値検査で確認し、受信側は上の許可集合で閉じる。
4. 本番Sessionで`mcpServer/startupStatus/updated`のstarting／readyを観測したら証拠と相反する観測とし、turn/start前なら拒否、turn/start送信後ならUnknown→Heldとする。

いずれかが得られない、相反する、または応答の欠落・error・`-32601`・malformedの場合は、型付き拒否`ExecutorCapabilityUnverified`としてturn/startを送らない（NativeInput 0、既存の応答不一致と同じ失敗経路）。P HandshakeやSの観測は別processであり、本番Sessionの証拠に代えない。2 methodの送信は§5のsetup行に数え、§7・§8の有限契約へ加える。`config/read`応答の`config`は利用者設定全体（MCP serverの環境変数等の秘密を含みうる）なので、判定pathだけを検査してメモリ上で比較し、本文を保存・監査・receipt・logへ出さない。

`codex/policy.rs::DecisionPolicy`には継承設定に`--disable`・`-c`を加える既存の仕組みがあるが、その全体（permissions等を含む）はM0へ流用しない。B2aは制限方向の狭い上書きの許可であり、現在のP／command契約を変更した承認ではない。**現契約では追加argvは0**で、本番Sessionで1〜4が得られなければ`ExecutorCapabilityUnverified`／NativeInput 0とする。ThreadStartParams.configは送らない。以下のB2bだけがPで事前決定する変更案であり、承認前に実装しない。approval方針・sandbox・permissionは一切上書きせず、`permissions.*`／`network.enabled`の`-c`も送らない。

**U6追補改訂（2026-10-18）: 現B2aはP契約待ちでBLOCKED。第二transport案は不採用。** `prepared.rs::command_argv`のCodex argvは固定3要素で、`compat.rs::check_command`は完全一致を要求する。最初の起動への追加にもP契約変更が必要である。さらに`native/preparation.rs::retain_transport`は同じPreparedの一度限りの消費と`state.transport.is_none()`を要求し、no-dispatch原本を消費する。`native/registration.rs::registration_material`はargvをcommand digestへ束縛し、`state/execution/native_phase/transport.rs`はno_registrationと元のreadiness／inventoryを要求する。Known停止でも原本は復活しない。第二transportなら最短でもPrepared246＋第一transport/setup7＋第二transport/setup/input10＝263行（第一証拠を完走すれば265行）となり、256を超える。停止は既存effectのsettlement更新で追加行0としても超過するため、この案と専用の再起動tests／追加byte予約は削除する。B／Prepared境界を引き下げる案も採らない。

**失敗の時間窓（§7・(3)・本(3a)共通）**: 本番のturn/start送信前は他のboot_callの既存処理を維持する。strict検査後のserver requestには既存`Lines::response`が`-32601`を返して待機を続ける。これをoverride理由にも新たなHeld理由にもしない。turn前のautoApprovalReview通知にも既存boot_callの通知処理を維持し、Heldを新設しない。証拠の欠落・相反・error・malformedは`ExecutorCapabilityUnverified`／NativeInput 0であり、turn/startを送らない。server request／autoApprovalReviewによるwork軸のUnknown→Heldは、turn/start送信から所有turnの終了まで（応答待ちを含む）だけである。相反するsettings／MCP通知も前は型付き拒否、同じ窓内はUnknown→Held。turn前のpipe／reap／保存不明をこの窓へ繰り上げず、既存のcustody／CAS不明の保持・停止に従う。S/P握手は別契約で、server requestをstrict検査後に拒否し、応答・ledger・Grant 0。P拒否はAC-08の登録前拒否であり、本番turnのUnknownを作らない。

**送信前queue処理（§7・(3)から参照する唯一の定義）**: 現sourceの`Lines::response`はturn前に受けた通知をqueueへ積み、`receive`はturn送信後にもその先頭から取り出す。本番のturn/start送信直前に、送信前に着信してqueueにある通知を全件queueから取り除き、strict検査・所有検査を通したうえで、次の順に種類ごとに各通知を1回だけ処理する。

- 証拠4のMCP starting／ready、または所有threadのsettings相反は、型付き`ExecutorCapabilityUnverified`としてturn/startを送らない。
- `account/rateLimits/updated`は既存と同じ`observe_quota`へちょうど1回渡す。現sourceでturn/start後に行う反映を送信前へ移すだけとし、欠落も二重反映も許さない。
- autoApprovalReview、正常なforeign通知、残るGlobal通知は、既存boot_callと同様に内容を使わず破棄する。
- 残る所有thread通知は送信前の規則で処理する（整合するsettingsは続行）。

処理済み通知はqueueに残さず、`receive`で送信後の通知として再判定しない。turn/start応答待ちは空のqueueから開始する。queueの件数・byte上限は増やさない。ここでいう着信はrrxがpipeからframeを読み取った時点であり、送信後に初めて読み取るframeは送信後の規則に従う。

**継承設定の成立条件**: 証拠1の各feature比較は`Some(&false)`である。対象15 featureすべての明示false、`web_search="disabled"`、設定されたMCP全entryの明示`enabled=false`が要る。features absent／nullや対象key欠落は無効の証拠にならない。CLIのconfig/readが既定値を実体化するか未確認なので、現契約では通常の既定設定は拒否になると見込む。`mcp_servers` absent／null／空表はserver 0件として合法で、web／featureの不足判定も通常どおり行う（欠陥ではない）。この場合も本番のinventory（2）は必須であり、省略しない。B2b承認後は、対象feature全件とwebが未指定ならk=16／35要素を許容し、普段の設定fileを変更せず本番processだけを制限する。不足MCP serverは最大32件までとし、これを超える場合や他の上限・本番証拠の不成立では拒否するため、一般のowner-run成功を保証するものではない。

**B2b提案（Pで狭い上書きを決定。提案・オーナー承認待ち、§13）**

**B2b-1. P Handshakeの追加観測と寿命**: 同じ準備actorのSchema合格後、継承argvのP helper内でinitialize→paramsなしinitialized→account/read（refreshToken=false）→environment/status（local）→account/rateLimits/read→**config/read（cwd=当該Unitの原本Worktree、includeLayers=false）**の順とする。Sは従来のrate readまでで止め、config/readを追加しない。Pは5 request＋initialized、Sは4 request＋initialized。共通Handshakeには用途を明示し、Sが暗黙にPへ昇格する既定値を置かない。1 outstanding request、stepごとの専用id（既存1〜4の後に5）とsuccess responseを厳密に相関させ、重複・先行・stale id・error・`-32601`・欠落を拒否する。全必須RPCの合格、通知の検査、合法tail、両pipe EOF・reap既知と既存qualified停止条件を満たしてからだけ決定原本を発行する。config/read成功直後の候補は未確定で、tail相反やcustody／settlement不明なら破棄する。thread/start・inventory・turn/start・モデル入力はPでは送らない。

strict JSONをraw bytesから検査し、duplicate key・部分JSON・RPC判別keyの曖昧さを拒否する。既存wire ceiling（frame4 MiB、depth32、65536 nodes、単一string1 MiB、総string4 MiB、object／array4096）を維持し、P Handshake全体のstdout＋stderr捕捉1 MiBと準備aggregate8 MiBも別に適用する（従って4 MiB frameを実際に受けられる保証はない）。通知1024件、応答30秒／helper90秒・親期限、stdin close／5秒猶予／drain2秒／reap10秒の既存上限を増やさない。新RPCも残り時間・残りbyte内だけで送る。schema-file用のdepth64等をwireへ流用しない。

config/readは§8のConfigReadResponse全体のstrict／tolerated規則とrefinementを通す。判断するのはweb_search、固定対象featureの値、全MCP keyとenabledだけで、origins・layers内容・permissions・他の宣言property・entryのenv／path等を使わない。raw frameとconfig本文はdecode中だけ保持して捨て、候補には固定feature集合の不足bit、web不足bool、有界な選択MCP名だけを残す。全server名とenabledの検査済み投影、およびconfig応答前の起動通知の名・状態は、B2b-2の相反検査用の一時メモリに限り既存の容器／通知／byte上限内で保持し、tail検査終了後に選択外名も解放する。tolerated値は残さない。決定原本と実argvは準備／登録／transport custodyのメモリに限り保持し、寿命終了で解放する。journalやreceiptから再構築しない。

config/readを含むHandshakeでは、`capture_version`の`BoundedOutput.stdout`へstdout bytesを蓄積せず、byte数・EOF・上限の検査だけを保持する。wire decodeに必要な`pending`・現在frameは一時bufferだけとし、処理済みframeを蓄積せず、成功・拒否・timeout・custody不明の全終了経路で破棄し、observationへ移さない。custody→completion→Preparedが保持するHandshake observationは本文fieldのない専用型とし、`stdout()`等のbody accessorも共通body格納先への経路も持たせない。Version／Git等で必要な本文accessorはそれらの型に限定する。空のVecを入れるだけの慣例やreceiptからdigestを消すだけで非保持としない。

**B2b-2. 選択と拒否**: まず相関・全型・上限・通知の矛盾を検査し、その後だけ次表で不足クラスを集める。欠陥を不足へ変換しない。

| 観測 | Pの決定 |
|---|---|
| 対象featureが明示false | そのfeatureの追加なし |
| 対象featureがtrue／absent、またはfeatures全体がabsent／null | 不足featureごとに`--disable <固定名>`を1組。対象外feature名からargvを作らない |
| web_searchがdisabled | web追加なし |
| web_searchがcached／indexed／live／absent／null | `-c web_search="disabled"`を1組 |
| mcp_servers absent／null／空object | server 0件、MCP追加なし。他クラスを通常判定。本番inventoryは必須 |
| MCP entryのenabledがfalse | そのserverの追加なし |
| MCP entryのenabledがtrue／absent | そのserverに`-c mcp_servers.<name>.enabled=false`を1組 |
| feature値がbooleanでない、enabled presentで非boolean（nullを含む）、表／entryの型不適合、web未知enum／wrong type、layers非null | P拒否（`config_invalid`）。合法な不足として扱わない |
| 応答error／不相関／欠落、initialize・account・environment・rate不合格、server request、相反account（合法tail期間も含む） | 既存P失敗分類／有限reasonで拒否。矛盾を後続正常応答で相殺しない |
| MCP starting／ready（合法tail期間も含む）がconfig/readのenabled=falseのserverを指す | P専用検査で相反として拒否（`contradiction`） |
| MCP starting／readyがconfig/readでenabled=true／absentのserver、またはconfigに無い名を指す | 相反ではなく、決定の入力にも使わない。上書き選択はconfig/readの不足だけによる。本番の再検査で判断する |
| 名や容器の上限違反、不足serverがM=32を超える、生成argvの要素数／要素長／command byte上限超過 | P拒否。切詰め・部分適用・再起動なし |

MCP名の出所はそのPの検査済みconfig/readだけとする。全key（disabled entryを含む）を1〜128 ASCII bytes、`[A-Za-z0-9_-]+`で検査し、表4096 entry以下、duplicate key拒否。不正名を補正・引用・切詰めず`name_invalid`で拒否する。`.`、引用符、空白、非ASCII、pathを許さず、inventory／設定fileから補完しない。名の合法性とargv長は別条件であり、選択名では`mcp_servers.`の12 bytes＋名＋`.enabled=false`の14 bytesが128以下、つまり名102 bytesまでである。合法な103〜128 byte名が不足なら`argv_limit`で拒否する（falseならargv化しない）。

MCP起動通知の相反判定はP専用関数に置き、Sと共通の`Frame::handshake_notification`の受理規則を変えない。config/readより前の通知も一時投影を保って応答後に照合し、応答後・tail中も同じ判定を行う。enabledのserverやconfigに無い名の起動通知を理由に拒否したり、通知から上書き対象を増減したりしない。P helperは継承設定のままなのでMCP起動通知やMCP process起動が起こりうる。この挙動は未証明一覧に残し、無通知から内部process不在を推論しない。

不足集合だけを上書きする。固定15 featureの宣言順→web→MCP名ASCII昇順の決定的順序で、重複0、既に制限済みのクラス0。選択できる不足MCP serverの上限を**M=32**とする。対象feature15件＋web1件を全て補えることに加え、日常設定のMCPを有界な32件まで補える枠であり、選択名の保持量は最大3264 bytes（32×102）、argvも小さく収まる。32はCLIの能力上限ではなく本提案の運用上限で、超過時は切詰めず`override_limit`で拒否する。

k＝不足feature数＋web不足0/1＋不足server数とし、**本番Codex Executor経路が型付きP決定を受け取る場合だけ**、要素数上限を18から`3＋2×(15＋1＋M)=99`へ変更する（k≤48、最大99要素）。provider／roleだけでは上限を切り替えない。他用途へ決定を渡した場合は無視せず拒否し、決定なしの19要素vectorも拒否する。18は`prepared.rs`のClaude Reviewer固定vector長（`reviewer.len()==18`）を囲う値であり、安全性から導いた限界ではない。S、P helper、Codex Reviewer、Claudeのvectorは18のまま。各要素128 bytes、選択名102 bytes以下・`[A-Za-z0-9_-]+`、encoded command64 KiBは不変。argvは既存aggregate command digestとメモリで扱い、journal body・receipt・行予算には入らない。追加語彙はちょうど3形式、`--disable <固定feature名>`、`-c web_search="disabled"`、`-c mcp_servers.<name>.enabled=false`だけとし、要素数枠の拡大から任意argvを許可しない。全feature absent＋web absent・MCP 0件ならk=16／35要素を許容する。k=0はmode=inheritedで固定3要素、1≤k≤48だけmode=override。不足server33件以上は他クラスの不足が少なくても`override_limit`、argv100要素以上は`argv_limit`で拒否する。productionでdriftがあれば再検査で拒否するため、既にfalseの値を全件重ねて送らない。§13 B2bはこの限定的なargv上限変更もオーナー承認事項に含む。

`-c features={…}`／`-c mcp_servers={…}`の表形式は採らない。既存表を置換すると対象外featureの明示falseやserver定義を消し、権限を広げうるうえ、feature一括表は1要素128 bytesにも収まらない。`--disable a,b`等のカンマ列も0.162.1で未検証なので採らない。通常の設定file、profile、CODEX_HOMEを変更して成立させない。

**B2b-3. 原本からPrepared・registrationへ**: 決定原本は同じP helperの実観測／qualified closureからだけ生成するprivateな非serialize型とし、Sやcallerが作るDTO／任意argv／保存receiptを入力にしない。SAME actorのArc identity、当該Handshake custody／plan／closed settlement、Unit・owner instance／epoch・generation、元のFrame／Worktree束縛・program identity・Schema contractに対応付ける。別actor、旧epoch／再準備の古い決定、閉じていないhelper、revoke、receiptからの再生はcommand計画前に拒否し、registration／spawn 0。configの永続digestは新設しない。

本文を持たないHandshake observation→`NativeReadonlyHelperCompletion`→`plan_native_command`→`PreparedNativePhase`へ同じ決定原本を保持し、`issue_prepared`・`validate_original`・`registration_material`・transport登録の各境界で同一性／freshnessを検査する。`command_argv`はこの原本を参照して期待argvを導出し、`check_command`も同じ原本から独立に再導出した期待argvとの**完全一致**を要求する。`check_command`は`matches_actor`等で当該actor／closed prefixとの一致を確認したcompletionが保持する決定をcommandとは別引数で受け取り、command自身のargvや埋込み値を期待値の出所にしない。`NativeCompatStatic::observe(version)`が持つのはVersion custodyであってHandshake決定ではない。`preparation.rs`ではstatic compat→identity確認→Version→Schema／Handshake metadata→Git completion→compat.observe(version)→command計画の順序を維持し、completion由来の決定を各callerで明示的に渡す。S、P helper自身、Codex Reviewer、Claudeでは型付きP決定を渡されたら拒否し、決定なしで従来のargvを導出する。program／cwd／overlayの照合も維持する。

`registration_material`の既存command digestは導出済みargv全体（上書きの順序と値を含む）を引き続き覆う。この既存aggregate digestは原本同一性照合用の例外であり、設定本文／MCP名別digestを追加記録したり、digestだけで決定を復元したりしない。Preparedは一度だけ消費、no-dispatch失効・no_registration・元のreadiness／inventoryの完全照合はそのまま。本番transportは1回だけspawnし、必要なら最初からそのargvを付ける。P helper spawnと本番transport spawnを混同せず、Pに既存Handshake helper 1本、本番にtransport 1本である。第二transport／第三attemptはない。

**B2b-4. 本番再検査**: production Sessionはinitializeから開始し、上書き済みでも(3a)の**1〜4を全て同じSessionで取り直す**。Pのconfig判定、証拠、thread ID、queue、rate結果を本番証拠へ持ち越さない。inventoryはPで代替せず、server 0件でも必須。設定drift、CLIが上書きを反映しない、inventory不合格、相反通知等で証拠がなお失敗したら`ExecutorCapabilityUnverified`／NativeInput 0、retry・argv差替え・二度目spawn 0。既存のturn/start前boot_call処理と、送信から所有turn終了までだけのUnknown→Heldを上記の時間窓のまま維持する。

**B2b-5. 行・byte・記録**: Pのconfig/readは同じHandshake helperのwire RPCで、native setup行ではない。intent／settlementは既存Handshakeの**1行**の更新のまま。Version1＋Schema1＋Handshake1＋Git13＝16行で、B230→Prepared246を維持する。本番はtransport1＋setup8＋input1＝10行、初回入力256を維持し、config/readのsetup1を削らない。第二transportの7／9行や61019／78453 bytesを予約する案は廃止する。

inventoryは2 MiB、private body8192 bytes、receipt16項目／key・値128 bytesのまま。最大1行image8717 bytesのsettlement込み予約を維持する。追加RPCはP Handshake捕捉1 MiB／準備aggregate8 MiBの残量を消費し、stdout非保持でもbyte計数・超過拒否は省かない。決定の一時名表もobject4096／wire byte上限内、保持する選択名は32以下かつ各102 bytes以下。encoded command64 KiBは原本計画時に検査し、行imageへargv本文を保存しない。transport／本番setupのqueue・capture上限を増やさず、各段の残量を別管理する。

追加記録先は**P Handshakeのsettlement receiptだけ**とし、既に保存済みのtransport receiptを後書きしない。既存`safe_receipt`はHandshakeでunique key12（Schema用4項目は別action）。configを含むstdoutの`stdout_sha256`はHandshakeでは出さず11項目とし、`b2_mode`（inherited／override／refused）、`b2_counts`（feature不足0〜15、web0/1、server不足0〜4096、k0〜4112の固定順の数）、`b2_reason`を加えて最大14項目とする。`b2_reason`は`none/config_invalid/rpc_refused/contradiction/name_invalid/override_limit/argv_limit/capture_limit/custody_unqualified`の有限値、計数前の拒否はcounts=`unavailable`。既存捕捉byte・exit・custody／IPCの有限記録を残し、値128 bytes、body8192を最大画像で検査してからintentを保存する。parserの本文付きerror文字列は出さない。B2b未承認の経路は既存の拒否分類だけを使い、本番config/read setupにも新しいreason／receipt項目を追加しない。

config本文・origins・layers・stdout本文・server名／名別digest・argv本文・設定／program／Worktree pathは追加記録しない。決定本文の永続化も禁止。既存registrationのaggregate command digest以外へ内容由来digestを追加しない。receiptの生成はclosure原本から決定的に行い、`Inventory::read`／`EffectImage::decode`、settlement／closure／historyの完全画像CAS照合を弱めない。receiptは観測であり、decision原本の代用でも新authorityでもない。

**B2b-6. 最小tests／mutation（承認後の実装時、未実施）**: §12のEX-7とB-1を共用し、以下は同じ性質を重ねて固定しない。

| 性質 | 最小の正負対／killする変異 |
|---|---|
| 不足集合だけを選ぶ | 全明示false／disabled・MCP absent/null/空表はk0／固定3要素。feature1件trueまたはabsent、web合法非disabled、MCP enabled trueまたはabsentを1クラスずつ差替え、その組だけ追加。全件一式化・absentをfalse扱い・mcp absentを不正扱いする変異をkill |
| 欠陥を修復候補にしない | 同じ入力のwrong type／layers非null、RPC stale id／error／欠落でP拒否、registration／transport spawn／NativeInput 0。全失敗をoverride候補にする変異、config RPC相関・完了条件を省く変異をkill。Sはconfig/read不送信の対照 |
| Pだけの起動通知判定 | config/readでenabled=true／absentの名、またはconfig外の名のstarting/readyは通知なしと同じ決定でpass、同じ通知の名をenabled=falseに替えるとP拒否（config応答前／後・tailを含む）。Sでは従来の受理を維持。全起動通知を拒否・相反を見逃す・通知を選択入力にする・Sの共通判定を変える変異をkill |
| 名とargvが有限 | 対象15 feature＋web＋MCP32不足のk48／99要素passと、不足MCP33件のk49／101要素はoverride_limit拒否。要素数validatorは本番Codex Executorの型付きP決定ありで99／100（+1）の拒否境界を確認し、決定なしの19要素を拒否、S／P helper／Codex Reviewer／Claudeへの決定渡しも拒否する負例を加える。未承認語彙は枠内でも拒否。feature／web未指定・MCP0のk16／35要素pass、MCP102 byte pass／103 byte選択拒否、合法128 byte非選択pass／129 byte名拒否、不正文字名拒否、4096 entry／+1の既存JSON境界を共有。S／P helper／Codex Reviewer／Claudeは上限18のまま。切詰め・不正名補正・上限省略・全用途への99上限適用・他用途の決定を無視・禁止`permissions.*`／`network.enabled`付加の変異をkill |
| P決定＝本番argv、1 transport | 実installed経路で不足1件→P原本どおりのargvを登録してproductionを1回spawn、独立したproduction証拠で初回input1。k0なら固定3要素、拒否後の第二spawn0。決定の無視・caller argv採用・check_command完全一致除去・第二spawnの変異をkill |
| 決定のfreshness | genuineな別actor／別epoch・旧準備の決定との置換でPrepared／registration拒否・spawn0。原本束縛をreceiptのmode/countやdigest一致だけへ置換する変異をkill |
| Pは本番証拠を代替しない | 上と同じ成功fixtureのproduction側だけ証拠1〜4の欠落・相反を各最小差替えし`ExecutorCapabilityUnverified`／input0／spawn計1。P観測で代替・inventory省略・失敗後再起動の変異をkill。turn前server requestはboot_callの-32601継続、turn/start応答待ちはEX-3bのHeldで時間窓を対にする |
| 予算と非漏洩 | B-1の実producer230→246→256／+1拒否をoverride正例でも実行。既存byte境界を共有し、最大receipt14項目・有限値・秘密sentinel非出力を検査。config/readを含むHandshakeの成功／拒否／timeout／custody不明でstdout累積0、pending／frame非残留、body accessorのない型境界を確認。stdoutを保持する（BoundedOutputからobservationへの蓄積復活）変異を必ずkillし、raw config保持／hash記録・receipt後書き・P RPCをsetup行へ追加・本番setupを省く変異もkill |

**4. 実行中のdecodeと所有**: 本番処理順は§7のstrict envelope→method→payload全構造→所有フィルタ→consumerを維持し、次の3区分を分ける。(a) 全構造検査を通った正常なforeign thread／turnの通知・item（現sourceの`execution/native.rs:2313-2314`の除外）は§7のとおり除外して続行し、成功・Heldの根拠にしない。(b) malformed frame（foreignを含む。strict位置の不適合とtolerated位置の有界strict JSON違反）、契約にもgate許容追加にもないmethod・item.type、未知discriminatorはProtocolErrorとし、turn/start送信後はWorktreeに効果が生じうるためUnknown→Heldとする。gate許容追加の通知とtolerated通知は所有キー検査後に内容を使わず続行し、foreignのgate許容追加itemはid／type検査後に除外する。(c) 所有turnのitemのうち、作業を別thread／turn／agentへ広げるcollabAgentToolCall／subAgentActivity、(3a)の許可集合外item、gate許容追加のitem typeは、効果の帰属と範囲を確認できないためUnknown→Heldとする。所有turnのcommandExecution.cwdとfileChange.changesの各pathは、二段で4つの書込み可能root（Worktree／Unit temp／output／cache）内かを検査する。字句検査は`codex/protocol.rs::operation_paths`（:715-752）と同じ条件（非空・4096 bytes以下・NULなし、`..`成分の拒否、相対pathはWorktreeへjoin、結果がいずれかのrootにcomponent単位で前方一致）とする。実filesystem検査は`codex/session.rs:2162-2217`と同じ条件（存在する最も近い祖先を`symlink_metadata`でたどり、その`canonicalize`がcanonical化したrootの配下であること、既存の通常fileで`nlink>1`なら拒否、metadata取得・解決不能は拒否）とする。commandExecution.cwdはdirectoryなので、hard link条件は通常fileにだけ当てる。どちらかに外れればUnknown→Heldとする。この検査は観測時点の事後検査であり、効果の発生を防がない（防止はsandboxの強制に依存する）。同じ述語を共有し、ledger経路の呼出し元は増やさない。他の許可itemは構造検査後に回答・authorityへ使わず、既存collector（agentMessageの業務条件）だけが回答を採否する。

**5. turnの完了と結果（work軸）**: 成功はowned threadIdとturn.idが一致する`turn/completed`のstatus=completedだけとし、検査済みturnを既存collectorへ渡して`(Success, Completed)`とする。既存coreのとおり、この成功原本（`NativeTerminal`）は`stop_and_reap`より前に`captured_terminal`へ捕捉し保存する（`execution/native.rs:2033-2052`）。捕捉後のEOF・reap・hygieneの結果はknown workを置き換えない（`doc/design/master/agent-execution.md:45`、(6a)）。Worktreeの結果はCodexにcommitさせず、既存のprovider非依存経路（`workflow/driven_settled.rs::continue_settled_executor`→`execution/workflow_source.rs::capture_settled`→`execution/workflow_gates.rs`→`execution/results.rs::capture_settled`、`retained_io::RetainedGit`によるexact commitの保持）でcaptureする。この経路はM0で変更せず、Codexの成功tupleが既存の成功と同じ形で入ることを回帰検証する。その後`post_success_closure`がSC-Nで停止する。failedは既存のquota／auth／capacity／Failure分類を§9と§7の検査済み値で行う。CodexErrorInfoが`Unclassified`（§7、E-11）のfailedはKnown FailureにせずUnknown→Heldとする。

**6. Unknown／Held（work軸）**: interrupted、inProgressのままの完了通知、未知status、成功原本の捕捉前に生じたturn/completed前のEOF・exit・pipe EOF不明・leader reap不明、collector overflow、§3／§4の拒否、(3)のserver request、(3a)／(4)のUnknown区分は、成功にもKnown Failureにもせず`WorkOutcome::Unknown`とする。所有・CASが成立して保存できた場合は§6と同じ`UnitState::WorkUnknown`／`SessionState::Lost`→`SettledLookup::Held`→`StepResult::Waiting`の明示操作待ちへ接続し、Worktreeを消去・reset・captureしない。自動再送、新しい入力、retry authorityは発行しない。照合不能は既存Held／pending原本保持に従う。

**6a. 物理custody軸**: stdin close・group signal・drain・leader reapの結果は、protocolで既知のwork軸と別に記録する（stderr drainは`execution/native/transport.rs:716-725`）。成功原本の捕捉後に両pipe EOFまたはleader reapが不明になった場合、work軸はKnown successのまま既存retained captureへ進み、custody軸だけをUnknownとして記録する。custody Unknownは、そのUnitのNative側の新しいspawn・入力・Grant・authority発行を止め、statusではHeld custodyとして表示する。既存のRuntime result-finalization（retained Git capture）はmaster契約のとおり継続し、custody Unknownを理由にwork軸をUnknownへ反転・成功を取消し・自動再送・cleanupで解除しない。

**7. 期限とquota**: モデルturnに新しい期限を追加しない。停止は既存のcancel／stop、authority fence（100 ms）、§4のshutdownで行い、期限延長で問題を隠さない。turn/startは既存quota Admit後だけ送る。`account/rateLimits/updated`とusageLimitExceededの観測は§9と§10のラベル契約に従う。

### 9. Rate契約とquota closure

適用範囲: Claude rate decoderとselfcheck（R1）はM0で実装する。下記のcanary Known closure（quota Admit後のProbe失敗）はClaude Reviewer Pに属し、M3で実producerとともに実施する。Codex rate契約は全Codex aliasのS、Codex Executor P、§8aの本番Executor Sessionの`account/rateLimits/updated`／usageLimitExceeded観測に共通する。

Claude decoderは`NotRateEvent／Classified／Unparseable`。bucketは既存5種、statusはallowed／allowed_warning／rejected。optional resetsAtはnullまたは秒int64でmilliseconds変換overflowを拒否し、utilizationはnullまたは有限0〜1。欠落・wrong type・未知enumはUnparseable。

Codexはresponse envelopeとsnapshotを分ける。rateLimits objectは必須。rateLimitsByLimitIdはoptional nullable typed map、最大32、keyは1〜128 bytes。両方presentなら両方を構造検査し、選択しない側のmalformedを隠さない。 GetAccountRateLimitsResponse.ordinaryUsageAllowedはoptional boolean/null（absent・null・false・trueを区別）、present wrongtypeは拒否する。業務上のquota分類／Available判定／Admitには使用しない。

limitIdはabsent／nullなら既存fallback、present non-stringは拒否。rateLimitReachedTypeのwire集合は下表の5値とoptional null。spendControlReachedはoptional boolean/null。未知enum／wrong typeを未到達へ変換しない。

| rateLimitReachedType／到達条件 | quota分類 |
|---|---|
| rate_limit_reached | Exhausted、confirmed_subscription=true |
| workspace_owner_credits_depleted | Unknown。これだけからsubscription exhaustionを発行せず、Availableにしない |
| workspace_member_credits_depleted | Unknown。同上 |
| workspace_owner_usage_limit_reached | Unknown。同上 |
| workspace_member_usage_limit_reached | Unknown。同上 |
| spendControlReached=true | Unknown。同上 |
| enum欠落／null、spendControlReached欠落／null／false | それだけでは到達証拠なし。型付きwindowの既存分類へ進む |

独立したusedPercent>=100のwindow証拠があれば、既存のExhausted／confirmed_subscription=trueを優先する。それ以外でcredit／spend到達があればUnknownを優先し、割合が小さいことからAvailableへ落とさない。上記分類は`state/execution/quotas.rs`と`native_phase/live_quota.rs`がExhaustedにconfirmed_subscriptionを要求する既存契約を保つ。windowが存在しない場合は観測windowを創作せず、metadataとして到達種別を記録してavailability unknownとする。

windowのusedPercentはint32であり、0〜100のschema制約を発明しない。negative値はusage unknown、100超は割合値を捏造せず既存exhaustion判断を維持する。S/Pのこの分類結果はpoolへ反映せず、本番validated観測のconsumerと区別する。

`{"rateLimits":{}}`、全window absent/null、合法な空mapはwire合法で、usable-window件数0、availability／usage unknown。これらをmalformedやAvailableにしない。metadata構造合格とusable-window観測を別記録し、既存quota policyを緩めない。wrongtyped snapshotは拒否する。

selfcheckはClaudeのbucket×status×optional項目、Codex snapshot／map、合法empty、wrongtype／未知enum／overflowを対で検査する。成功tokenをinstallationへ渡す。selfcheckはCLI live互換の証明ではない。

`unclassified_limit`とそのcapacity／terminal分岐を廃止し、解析不能をProtocolErrorへ一本化する。既存terminal_capacityの意味をrate unknownで代用しない。

quota Waitはcanary入力0、Parked。Admit後Known不合格は既存`NativeQuotaClosurePlan`へ接続する。`state/execution/native_phase/quota.rs:1273-1439`の原本actor／owner一致、revoke、no-dispatch、no_registration、末尾inventory、readiness lineage、latest factual Unitの検査を維持し、Claudeでは既知Probe settlementを含むProbed末尾へ接続する。latest Unitはnongrant照合画像であり、Task.versionやDTOからauthorityを発行しない。

| 項目 | before→after／CAS |
|---|---|
| own quota_waiters | 7列原本画像とUnit／provider／account=unknown／resume_state=preparingを確認し、存在すれば削除。absenceも原本に含める |
| own quota_leases | 6列原本画像、Unit／provider／account／epoch一致を確認し、存在すればactiveを0。他列保持。未存在なら作成しない |
| quota_pools.probe_unit | 自Unitを保持している場合だけnullへ。普通Admitで他Unit／nullなら保持 |
| quota_pools.next_probe_at | 自probeを解放する場合だけ`max(before.next_probe_at, at.saturating_add(before.backoff))`へ。その他は保持 |
| poolのprovider／account_key／backoff／last_role | 原本保持。canaryのrate／auth／capacity結果から書換えない |
| readiness | 原本version+1、state=closed、既存`transition_readiness("closed", version, None, true)`の画像へ。overflow拒否 |
| transaction | pool6列・waiter7列・lease6列の全before画像、Unit、readiness／inventoryを同一transactionで再確認し、既存binding permitで更新。planの既存5秒freshness検査を延長しない |
| uncertain commit | 全after画像＋readinessならKnown、全before＋旧lineageならRolledBack、混在／照合不能ならHeld。部分成功をclosure完了にしない |

UnitのRetired／RefusedとTaskの`explicit retry required` Waitingは、このnongrant closure結果から既存terminal処理で行う。普通Admitとrecovery Admitを別々に検査し、recoveryで自probeを保持する場合の機械的更新を含める。UnknownはHeld、leaseを勝手に解放しない。観測非反映はwindow／availability／usage／canary由来backoffの更新0であり、pool全write0ではない。

普通Admitとrecovery Admitを別controlsで検査する。観測非更新のassertionはwindow／availability／usage／backoffのcanary反映0であり、quota_pools全write 0ではない。poolがcanary exhaustionを学習しないため後続TaskもAdmitされうることを明記し、M0で自動復旧を追加しない。

### 10. Terminal、tail、identity、監査

helperはprotocol、stop方式、exit、pipe EOF、tail、signal、leader reapを別記録する。観測前にも有限deadlineを持ち、合法通知や再送で時計を更新しない。

| 対象 | 観測開始からの期限 | stdout＋stderrの累積捕捉上限 |
|---|---:|---:|
| Version | spawnから30秒 | 64 KiB |
| Schema process | spawnから60秒 | 64 KiB。生成filesの16 MiB枠とは別 |
| Handshake | spawnから90秒、各相関応答はrequest送信から30秒 | 1 MiB |
| Claude canary（M3、Reviewer P） | user送信から120秒 | Probe枠に含める |
| Claude Probe全体（M3、Reviewer P） | intent／custody開始から180秒、initialize応答は送信から30秒 | 4 MiB |
| Git helper／batch | 現行各応答30秒／batch180秒 | 現行各action上限を維持 |
| Codex Executor turn（§8a） | 新しい期限を追加しない。既存cancel／stop／authority fenceと§4のshutdownで停止 | 既存Lines／collector上限 |
| S全体 | custody登録から10分 | 全alias合計8 MiB。各helper上限も独立に適用 |

入れ子deadlineは常に最も早い期限を使う。準備PではVersion／Schema／Handshake／Git／Probeのstdout＋stderrの全受信bytesを、既存`AGGREGATE_BYTES=8 MiB`へ一度だけ累積算入する。stderrを保存しなくても算入する。UUIDなし、同一UUID再送、foreign、ignored、合法tailもdedup／所有filter前に算入する。各helperは自身の上限とaggregate残量の小さい方で制限し、capture stateと固定size read bufferも有界にする。Sは全aliasで同じ8 MiB counterを共有し、予約・加算を競合時にも超過させない。PのHELPER_LIMIT=32、Sのstep上限96、既存line／JSON上限を維持する。生成schema filesは別sinkであり、この8 MiB捕捉counterへ混算しない。

期限／byte超過では新しい送信・spawn・モデル入力を止め、合格を出さず既存停止手順へ進む。必須観測後もstdin close、5秒の自発終了猶予、必要ならgroup signal、2秒drain、10秒reapを適用する。親deadlineが先ならそこで切り、猶予でS／Probe上限を延長しない。両pipe EOFとleader reapが既知ならKnown不合格としてsettleし、どちらか不明ならUnknown。Sはadmission 0、Pは既存Known closureまたはUnknown Heldへ接続する。停止要求／shutdownの既存deadlineを延長しない。

Confirmedは生成結果既知＋両pipe EOF＋leader reap既知。qualifiedはConfirmed＋protocol pass＋合法tail＋自発exit 0または猶予後signal。未知tail／partial JSON／自発nonzeroはKnown不合格。pipe／reap不明はUnknown。

Claude成功後の新規tail許容集合は、strict-validなrate event、auth_status、stream ping／message_stop、system state=idleのみとする。完全同一UUIDの再送はdedup可能。新しいassistant／user／result／task／tool／control要求はtail不合格。late rate rejected等はcanary成功を覆し、Known不合格へ分類する。Codex tailは握手で許す正常全体／rate通知だけである。

Native programはcanonical path、dev／ino／size／mtime／ctime、SHA-256をVersion intent前に記録し、Native helper／transport前にhash、spawn直前にstatを再検査する。Git helperはCLI identityの再hash対象外。execとの非原子的な窓、wrapper内部、escaped descendantsは未証明として残す。

Sのidentity観測は通常fileかつ512 MiB以下に限定する（大きなnative CLI配布物を許容する有限上限）。canonical化後は`O_NONBLOCK|O_NOFOLLOW|O_CLOEXEC`で開き、read前にdescriptorの型とsizeを検査する。hash中に初期sizeを越えた場合も拒否する。path解決・open・hash・観測後statはblocking poolで実行し、各read間にrevoke／S親deadlineを確認する。async側も10 ms間隔で停止と親deadlineを観測し、Stopped／Deadlineで有限に拒否する。OS内で停滞した同期I/Oを強制中断する保証はなく、開始済みblocking処理は戻り次第停止する。その処理はowner／child custodyを保持せず、新spawnやauthorityを発行しない。

監査は有限語彙、method件数、digest、nullable観測状態だけ。account本文、codexHome、nonce、canary本文、raw frame、stderrを保存しない。receiptは16項目以下、値128 bytes以下、既存private imageの8192 bytes上限を維持する。S bodyは4096 bytes以下。版数はexit 0、完全捕捉、256 bytes以下、単一行、制御文字なし。記録は96 bytes以下かつ`[A-Za-z0-9 ._()+/:-]`なら原文、それ以外はSHA-256とnonrecordableを記録する。記録可否を版数一致による合否へ変換しない。

S intent保存前に最大のobservation（96 bytesの合法version、最長の拒否理由、exit／byte数の型上限を含む）を加えたsettlementも4096 bytes以内と検査する。計画途中で後続aliasのpath／identity／journalが拒否された場合、既にcommitした未spawnのintentは元のpending imageへのexact CASで`confirmed`＋not-spawned観測へ精算する。精算失敗を成功へ変換せずJournal／Unknownで停止する。

`source_version`はCLI番号を含まない契約ラベルへ統一する。`execution/quota.rs:146`は`codex/account.rateLimits`、同:227は`claude/rate_limit_event`、`execution/native.rs:2227`のerror観測は`codex/usageLimitExceeded`、`:2248`のterminal観測は`codex/terminal usageLimitExceeded`、`:2254`のcorrelated recovery completed観測は`codex/correlated recovery turn completed`とする。番号だけを除き、Exhausted error／terminalとAvailable recoveryの意味を保持する。これらのラベルもauthorityではない。実際のCLI版数はVersion観測receiptへ別に記録し、quota DTOのsource_versionや保存済み旧ラベルからqualifyを再構成しない。

### 10a. Profile／config移行

新profile名を`rrx-native-probed-v1`へ固定し、settings=inheritedと既存hook宣言検査を維持する。新宣言はcli_versionキーの不在を要求する。`NativeCompatConfig`のdeny_unknown_fieldsを残したまま、廃止キーcli_versionだけはpresenceを識別できる移行用入力として認識し、TOML installation境界では存在すれば値にかかわらず型付き拒否する（TOMLにnullはない）。Frame JSON境界ではnullを含むpresenceを型付き拒否する。単純なfield削除で汎用unknown-fieldエラーへ落とさない。NativeCompatConfig::canonicalはserialization／digest生成前にもpresence検査を必須とし、cli_versionありを省略して新宣言digestへ変換せず型付きエラーとする。受理済み新宣言だけがcli_versionを含まないcanonical画像を生成できる。新Frame producerは旧keyを一切出さず、serialize→deserialize roundtripでも旧key不在と新宣言digest一致を保持する。新profile＋旧keyありFrameも旧宣言として拒否する。

拒否位置は`AgentRegistry::from_managed_config`（adapter.rsの現managed registry組立て前）。旧profile／cli_versionありはInstallationRefusalの有限分類を保持し、helper／Task admission／新規Unit／Heldは0。`qualify_compat_static`のprofile検査は多層防御として残し、`qualified_profile()`と番号付きprofile比較は全consumerから撤廃する。

切替の前提は進行中のmanaged Taskがないこと。既存Frameを新宣言へ書換えてauthorityを作らない。保持Frameが旧宣言なら、準備commit後の既存`compat.rs:91-100`の原本Frame／installed宣言digest照合で拒否し、Session／NativeInputを作らずPreparing UnitをHeldとする。installation拒否のUnit0とこのHeldを分けて検査する。

### 11. Legacy と collector の提供範囲（オーナー方向決定の転記）

2026-10-13、オーナーは「旧 API を廃止扱いで整理」を選択した。これは M0 の設計全体承認・実装承認・レビュー完了ではない。

旧起動経路は helper／Session／NativeInput の生成前に CapabilityUnavailable を返す。NativeSessions の legacy start、NativeAdapter の旧 start_managed 経路、Codex legacy transport を対象とする。resume/checkpoint 等も、旧 transport の新規起動・新しいモデル入力を生む consumer を列挙して同じ提供範囲を守る。型名や start だけを閉じ、別 consumer から旧モデル実行に届く抜け道を残さない。

共有する auth／owner／CAS／marker／quota／permission／process custody 機構は、現在提供する本番の管理経路で維持・検証する。status/cancel/cleanup 等の consumer を調べ、現在提供する経路の停止や custody 保持まで削除しない。旧機能固有で到達先が廃止されるコード・テストは、機能廃止の差分として明示して整理する。テストを skip/ignore して CI を緑にする方法ではない。

台帳は各test ID／example IDの旧producer／consumer、検出する安全性・機能、移植／残置／機能廃止に伴う削除、理由、現在の代替producerとassertionを記録する。N＋M＋Kが凍結した母集団と一致し、未分類0を要求する。87件（session60／custody20／ownership7）は既知subsetであり、whole standalone件数ではない。

母集団は固定入力HEADの追跡sourceから、(1) standalone Codex全test module、(2) 現管理経路と共用するprotocol、(3) integration testsとexamples、(4) 旧入口または共有機構を消費するNative／adapter／workflow／runtime testsを列挙する規則で確定する。既知追加対象はtransport8、attempt4、preparation2、availability1、`crates/rrx/tests/codex_unavailable.rs`2、examples2、shared protocol16、codex/policy.rsの全7 tests（auth readinessは1件。verify_auth_readinessの本番consumerはcodex/session.rs:1667、codex/transport.rs:19が消費するのはDecisionPolicy）。これらを87件へ混ぜて移植済みとせず、ID重複を除いたunionと区分別件数を実装台帳で確定する。`execution/native/tests.rs`のstart_inner経由25件も分類対象にする。新規追加testは旧母集団と別列で数える。

#19は`doc/design/master/agent-adapter.md:502-529`のprivate standalone selected componentとして、既存custody pool・ordinary選択None・cfg(test) factory・public availability EMPTYと既存検査を保持する。現Git／Native管理経路へ接続済みとは扱わず、起動しない既存部品のpositive／unknown保持／Drop検査として残置する。旧起動機能の廃止を#19維持条件の緩和へ拡張しない。共有安全性の管理経路への移植と、このselected componentの残置を別区分で記録する。実装分類で維持不能が確認された場合だけ、具体的根拠と影響を示してowner判断へ戻す。

旧入口inventoryはNativeSessions.start→start_inner→start_legacy、NativeAdapter.start_managed→start_inner、CodexAdapter.start／start_structured→launch、resume→register_existing→spawn_launch、checkpoint→register_existing→spawn_checkpointを含む。Workflowの既存protected拒否を緩めない。全入口でhelper／Session／新しいNativeInput生成前に拒否する。launchだけの拒否ではresume／checkpointを閉じたことにならない。

NativeSessionsのentries／starts／preparationsの3保持領域は、現installed transport／preparationも消費するため保持する。`native/transport.rs:532,696,748`、`native/preparation.rs:996,1024`、`native.rs:629,654,670,723,769`のstatus／照会／停止を台帳と影響範囲に含める。共有ケースは拒否テストだけで代替せず、旧機能固有の削除を元の正常系の実行証拠に数えない。

M0の実Actor経路は最初のExecutorまでで、旧Reviewer／resume等の本番実行機能を提供しない。根拠は`execution/workflow_source/native_handoff.rs:467-505`のgeneration=1、最初のRunning Executor、先行Succeeded Issue／Worktreeのみというhard checkと、同:560のartifact=None、`runtime/phase_handoffs.rs:532`のoffer_first_executorである。`marker_plan.rs`のReviewer variantやprivate adapterの型対応はsource-owned Reviewer producerの発行証拠ではない。M0を通すためにReviewer authorityや新しい拒否入口を創作しない。

ReviewerのSource-owned snapshot／lease、offer、保持と正常系はM3の実装・受入条件に残す。全role Git13の予算政策は純粋計算として照合し、M0のpure parser／予算政策／metadata検証をReviewer helper鎖の実境界合格やレビュー成功に数えない。M3で機能を提供する前に、その実producerを通る正常系と攻撃系を検証する。

collector は validated item を受ける。id/type/text の malformed は前段で ProtocolError とし、到達不能となる枝を隠さず整理する。phase absent/null、非final、非対応delivery、質問付きmessage、複数final、内容改変、overflowは回答 usability の業務判定に残す。維持する synthetic child wrapper が必要な実 child を起動する positive も確認し、test 本体の skip と混同しない。

### 12. Fixture、因果検証、owner-run

B2bのphase別method表（承認後）: Sはinitialize／account/read／environment/status／account/rateLimits/readの4 request＋initialized、P Executorはその4＋config/readの5 request＋initialized、本番Executorは既存8 request＋initialized（上記5＋thread/start／mcpServerStatus/list／turn/start）。全体のunionは8 method／8 responseのままで、9 methodへ増やさない。§0aのC1〜C4、catalogのClientRequest_used／response roots・共有definitions・subtree digestは不変。更新対象はphase別使用表、Handshake state／fixture／call siteとPの決定consumerである。既存0.162.1 catalogを書換え・再生成せず、この設計作業ではcatalog／コードを編集しない。

CI schema positiveは公開生成structureから、使用8 request（§8a(3a)の2 methodを含む）、initialized、全84通知、全11 server request、8種responseとその到達definition閉包を保った有限縮約fixtureとする。合成の最小schemaだけで代替しない。

**契約basis（C2）**: fixture sourceと§7の契約basisは、codex-cli 0.162.1の公開tag `rust-v0.162.1`（tag object `7a078f6bb4ce0dc2a81408b334ce52979d92c603`、commit `092d3acd6bec3e3a14bdc7e7a2810ab628ab759d`）の`codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`（SHA-256 `41d34da4227eadccf971272b9a02dfc6679e23a9992cce1d265ac01a20ab38e3`、447 file）から§8bの規則で作ったcatalogとする。sourceは承認済みcatalogの10 sourceにConfigReadResponseとListMcpServerStatusResponseを加えた12 source、ClientRequest_usedは送信8 method、共有definitionは303、subtree digestは`bbdede32736d0d954683bef90861c902a09aa46d33cf492f9f1cc3766005b38e`。digestの規則は従来と同じ（JSON読取り後のPython `json.dumps(catalog, sort_keys=True, separators=(",", ":"), ensure_ascii=False)`のUTF-8 bytesのSHA-256）であり、出所照合であってruntime hash pinではない。このcatalogは別のverification file `doc/verification/macos-minimum-m0-catalog-0162.json`として追加済みで、出所（tag、tag object、commit、export SHA-256・file数、source数、共有definition数、subtree digest）と再現確認の結果を同じfileに持つ。追加済みなのはこの静的catalogだけであり、縮約fixture、gate、decoder、インストール版でのlive確認は未作成・未実施である。

生成規則の再現確認: 同じ生成器を0.160.1のtag `rust-v0.160.1`（commit `d27764b82f7118f674371e6d6e76271d9d606edb`、export SHA-256 `63028565f715c584c2dad98163de85dcb03801f9a7c096f5ef07d4769a900984`、440 file）のexportへ承認済みの6 method／6 responseで適用すると、承認済みcatalog（10 source、268共有definition、`c44baad39a81231b2a1bd567b21ede7e7d155b08bf705df1ff8b087728944c7e`）と完全一致した。比較用に同じexportから8 method／8 responseで作ったcatalogのdigestは`4c4f7493…`であり、0.162.1との差分は27項目（4 source: ClientRequest_used、ServerNotification、ThreadStartResponse、TurnStartResponse）。分類は§0aの末尾のとおり。

生成器の値位置の修正: 当初の生成器は注記keywordの除去と`$ref`の追跡をschema位置と値位置で区別していなかった（例: `default: {"title":"A"}`と`{"title":"B"}`がともに`{}`になり、`default`内の`$ref`も閉包へ入る）。生成器をschema位置だけをたどる規則（§8b手順3）へ直し、同じ再現確認を再実行した。結果は0.160.1の6 method／6 response catalogが`c44baad39a81231b2a1bd567b21ede7e7d155b08bf705df1ff8b087728944c7e`と完全一致、0.162.1 basisが`bbdede32…`（303共有definition）、比較用8 method catalogが`4c4f7493…`で、いずれも修正前と同じdigestである。両exportで生成器が読む12 source fileに現れるkeywordは18種（`$ref`、`$schema`、additionalProperties、allOf、anyOf、default、definitions、description、enum、format、items、minLength、minimum、oneOf、properties、required、title、type）だけで、値位置（default、enum等）の中に注記keyや`$ref`を持つobjectは0件だった。したがって当初の除去は承認済みcatalog（`c44baad3…`）とbasis catalogの内容を変えておらず、A6承認時の根拠もbasisも影響を受けない。digestの規則（compact sorted-key JSONのSHA-256）は変えない。値位置のcontrol（§8b手順4(a)）は修正後の生成器で全件合格した。controlの検出力確認（§8b手順4(b)、1回）として、固定mutant（commit `6741d530`の修正前の生成器）を与えると、値位置の4項目（default内の同名key、enum内の同名key、const内の`$ref`、default内`$ref`の非追跡）と未知keywordの失敗のそれぞれでcontrolが不合格になることを確認した（schema位置の注記だけが違う対は両者とも一致）。verification fileは修正後の生成器で再生成し、byte不変だった。

履歴: 固定HEAD `b22f22a521424eefc3e61fb1a109bfc85965e5ef`のstatic資料`/schema_contract_catalog`（0.160.1、`c44baad3…`）はA6承認時の根拠として書換え・再生成せずに保持する。そのClientNotificationのshape文字列訂正だけを行った経緯もそのまま残す。static.baselineは公開source baselineを指し、fixtureのauthor入力HEADとは別である。旧static全体digestを今回fixtureの出所として流用しない。

fixture実体・縮約manifest・decoder対応表・適合結果は未作成／未検証。実体digestは作成後に計算・保存し、今は値や結果を捏造しない。live各項目はnot_runを保持する。

| control群 | positive／negativeと必要mutant |
|---|---|
| Startup | 全alias合格／1 alias拒否、gate除去、journalからpass復元、P省略。 |
| Strict | S-notify 21通知・S-item 3枝の全payload、84通知の所有キー、19itemのid／typeの正常形、各required欠落・wrongtype・nested tag改変、自／foreign双方。T-notify／T-itemは有界JSON違反だけ拒否。正常ignored／tool維持。所有filter先行mutant。 |
| Schema | 実fileの枝複製・削除、required追加、型・nullable・enum・数値域・default・ref先改変、無関係同名property、未解決／循環ref。strict位置の各変化kindの拒否とtolerated位置の変化の通過、許容追加の規則と上限、記録の有限性。 |
| R1 | 合法empty／usable window／wrongtypeを区別。foreign rate先drop、unknownをAvailable／not_observed化、selfcheck呼出し除去mutant。vector mutationは試験用authority seamを作らずsource差分として実施。 |
| Quota | Codex ExecutorのWait、普通Admit、recovery Admitそれぞれのモデル入力数、lease／probe ownership／期限／CAS。Claude canaryの観測非更新とKnown closureはM3。 |
| Budget | M0 が提供する Codex Executor の実 producer の境界／+1、Git13→9仮定mutant。Claude Reviewer のProbe末尾を含む純粋政策は照合だけとし、本番 helper 鎖とProbe末尾省略mutantは M3 で検証。 |
| IPC | Codex Schema／Handshake（M0）とClaude Probe（M3）それぞれ、shim呼出し、Grant0、helper以外効果追加0。 |
| Custody | stop／Drop／panic／forced deathの別期待、stdin close、tail打切り、reap不明をConfirmed化するmutant。 |
| Migration/path | new、v1〜9、v10、v11、current、同時initializer、guard欠落・旧reference新表混入、alias traversal／symlink／ownership／境界cleanup。 |
| Consumer | RuntimeMetadata／ShutdownSite wire、型付きinstallation refusal、collector usability、synthetic child実呼出し。 |
| Legacy | 機能廃止・残置・移植の完全台帳、旧入口と他 consumer の起動／入力0、共有する安全性ケースの実管理経路での positive。拒否だけを共有ケースの検証としない。 |
| Executor | Codex ExecutorのTask provider選択の拒否点、固定approval方針・sandbox・Worktree束縛、応答一致、turn/start前の能力の肯定証拠、turn/start応答待ちを含むserver request／許可集合外・範囲外item／未知variantのUnknown→Held、正常foreignの除外継続、turn結果分類、work軸と物理custody軸の分離、retained capture接続、spawn経路。 |

上の12群を次の個別control／対応mutantで具体化する。各negativeは正常な対を同じproducerで通し、指定欠陥を1つだけ変える。mutantで別の拒否が残っても、対象検査へ実到達してその検査の欠落を検出することを要求する。

| ID | positive／negativeと期待 | 落ちるべきmutant |
|---|---|---|
| S-1 | 全alias fresh合格／完了前Task投入・1alias拒否。未公開時admission0、status pending／refused | S gate除去 |
| S-2 | Codexの相関した全応答、chatgpt、ready、合法empty rate／各応答欠落、ID違い、error、-32601、非chatgpt、非ready。欠陥alias拒否 | RPCをschemaだけで代替、相関比較除去、account／ready比較除去 |
| S-3 | Claude Reviewer argv（plan mode付き）の相関initialize／未着、ID違い、error、plan mode欠落argv。user入力0、system-init／resultを観測済みにしない | initialize相関除去、未着を合格化、SのargvをExecutor role（plan modeなし）へ戻す |
| S-4 | stop／Drop／panic／forced death後の新epochを別検査。確認できたreap／joinだけ記録し、未知はpending／unknown、合格非公開 | Sをshutdown確認から除去、journalからpass復元、Drop／forced deathをreap済みにする |
| S-5 | S合格でもP fresh検査必須。P欠落は拒否 | S合格によるP省略 |
| CL-1 | P initialize成功／control_response欠落、相関ID違い、error。AC-08とterminal disposition | request ID相関除去 |
| CL-2 | init後の結果／init未着でresult、session／cwd違い。init_missing／mismatch | init未着許可、identity比較除去 |
| CL-3 | 成功・nonce完全一致／result未着、非成功、nonce1文字違い、包含だけ一致、余分な文 | nonceを包含比較、結果前の合格公開 |
| CL-4 | 有限全row／未知top・system subtype・block・delta・origin・error値、required欠落、wrong type。§6公開origin全枝の正常対、server／from欠落、optional null・wrongtype、fromMode／subkind未知、hook／mcp-channel拒否。非humanは検査後だけ本番処理しcanary成功0 | 未知枝を通す、wrong typeを欠落化、fixtureのhook／mcp-channelを公開wireへ流用、非humanを成功化 |
| CL-5 | 正常な同UUID同内容再送／同UUID内容改変。構造検査と累積bytesは再送にも適用 | UUID先drop、未知fieldをdigestから削除 |
| CL-10 | result success/error両枝でusage容器＋6項目の正常欠落／null／ゼロ／域内値、wrongtype／negative／fractional counter／overflowの対。structured_output absent/null回答可否と非null bounded値の回答不採用、過大値拒否、同UUID変更拒否。再開累積値はraw保持・delta None。本番Sessionの2ケースを同じ実consumerで固定する: (1) 先行原本なしの単一success-looking invalid resultはwork=`WorkOutcome::Unknown`／disposition=`Disposition::ProtocolError`／failure=`Some(NativeFailure::ProtocolFailure)`、そのresult回答・metrics不採用、confirm_complete 0、replay許可0。所有・CAS成立時は`UnitState::WorkUnknown`／`SessionState::Lost`と既存`SettledLookup::Held`→`StepResult::Waiting`の明示操作待ち（固定reasonは§6、配線は実装時検証対象）、照合不能時は既存Held／pending保持。(2) 別の完全validated owned resultから§6のcapture／frozen原本確保を実producerで通した後の欠陥frameは、先行work=`WorkOutcome::Success`／disposition=`Disposition::Completed`／failure=`None`、先行原本だけの回答・confirm事実維持、欠陥frameの回答・metrics採用0／追加confirm 0／新成功・replay許可0。published／confirmed事実を保持。ケース(1)のTaskは明示操作待ち、ケース(2)のTask進行は先行成功の既存consumerに従う。どちらも欠陥frameから自動再送・新retryの許可を作らない。P成功result後の欠陥tailはCL-8／CL-9のKnown不合格closure／物理不明Held・登録0として別検査 | telemetry wrongtypeをNone化、validated型迂回、nongrant観測をobserved_terminalのSuccess／Completedへ入れて回答・confirmを作る、先行完全validated原本をUnknownへ反転、resumed totalsからdelta発明、structured_outputを未宣言扱い、telemetry欠陥からUnknown replay化 |
| CL-6 | 本番合法tool継続／Probe中can_use_tool・tool_use・tool_result・tool_progress・task。Probe拒否、応答0 | 本番consumerでProbeを代替、tool要求へ応答 |
| CL-7 | rate未着、合法windowなし、usable／未知bucket・status、required欠落、wrong type、overflow。自／foreign双方 | foreign先drop、unknownをAvailable／not_observedへ変換 |
| CL-8 | 合法tail／成功直後の未知frame、partial JSON、自発nonzero、late rejected。Known不合格またはpipe不明Unknown | 成功frameで読取り打切り |
| CL-9 | EOF＋leader reap既知／pipe保持子孫、reap不明。後者Unknown／Held／lease保持 | reap不明をConfirmed、UnknownをRetired |
| CX-1 | basis source縮約／枝欠落・複製、method.enum複数、無関係な同名property | branch検査を文字列存在検査へ変更 |
| CX-2 | local ref閉包／未解決・外部・循環、深さ・memo・work-node上限+1 | ref解決省略、memo／stack／work counter除去 |
| CX-3 | 実送信8method（config/read・mcpServerStatus/listを含む）＋initialized／送信キー未宣言、型不適合、required追加、read-only欠落、Executor thread/startのdynamicTools等の送信 | schema省略、required／型比較除去 |
| CX-4 | strict受信閉包／nested required・nullable・enum・数値域・ref先改変、未知method／strict位置の宣言property、Opaque枝拡大。closed object各位置（strict到達）の正常対／追加key、key-tag 1個／0個・複数（httpConnectionFailed＋activeTurnNotSteerable等）、CodexErrorInfo開放枝の未知文字列・既知tagなしobject（Unclassified）／既知tag 1個の不正枝（ProtocolError）、RPC正常4形／result＋error・method＋result/error・id:null。ordinaryUsageAllowedの各状態／wrongtype。ConfigReadのweb_search各値、features／mcp_servers refinementの正常／型不適合、layers非null。MessagePhase partial_answerは回答不採用 | 消費requiredだけの投影、typed枝をopaque化、additionalProperties=false無視、複数tagを最初のkeyで選択、開放枝で不正な既知枝を救済、UnclassifiedをFailure／Available／capacity化、RPC判別keyを追加field化、refinement不適合を欠落化、partial_answerを回答採用 |
| CX-5 | 全所有区分の正常形／thread.startedのnested ID、turn.id、nullable turnId、任意threadId各状態、malformed foreign、gate許容追加通知の宣言由来の所有区分 | top-levelキー有無だけで分類、所有filter先行、許容追加通知の所有キー検査除去 |
| CX-6 | 握手Global通知／Thread通知・server request。両者ともpayload検査後の固有扱い。chatgptに相反しないaccount通知／authMode非chatgpt・null、login success=falseはS/P資格付け拒否。B2b承認後はPだけconfig/readを最後の相関RPCとして必須にし、Sは従来のmethod集合を維持（§8a(3a) B2b-6と共用） | 握手所有区分除去、server request params検査省略、相反account観測を後続正常応答で相殺、Pのconfig/read省略／Sへの誤追加 |
| CX-7 | CLI更新の模擬: 無変更のbasis生成schema（0.162.1 tag export）はPass（strict不一致0、`strict_reach`0。basisの到達組20組はすべてスカラー葉で、`hook/started`／`hook/completed`→HookRunSummary.sourcePath→AbsolutePathBufを含む。gateがbasisから導出した組集合と件数をfixture manifestと照合）。basis生成schemaに対し、(a) tolerated位置の変化（T-notifyへのnullable property追加、T-item枝の内容変更、Thread非coreの追加、tolerated既知通知の削除、スカラー葉（AbsolutePathBuf）だけへ届く新通知1件、新ThreadItem type 1件）はPassと件数記録、(b) strict位置の変化（送信paramsのoptional property追加、Turnのnullable property追加、used enumの値追加、CodexErrorInfo枝変更、strict通知の削除、server request追加、所有キー型変更、許容追加の上限+1・名前違反）と構造をもつstrict到達definitionへの新しい到達（新通知のparamsがThreadItemへ到達、既存tolerated通知`hook/started`のparamsがTurnへの参照を得る、Thread非coreの新propertyがTurnを参照、`ThreadItem[userMessage]`の内容がCommandActionを参照）はkind別の有限reason（後者は`strict_reach`とanchor・definition名）でRefused。共有スカラー葉（AbsolutePathBuf）の宣言変更はtolerated通知だけが使う形でも`strict_mismatch`。default／enum／const内の同名key（`title`等）だけの変更も`strict_mismatch`（kindはdefault／enum）。journalに生schemaなし、位置は記録規則どおり、body上限内。0.160.1 basisの生成schemaを0.162.1契約へ与えるとstrict不一致（TurnStartParams等）で拒否 | strict位置をtolerated扱い、default比較除去、値位置の同名keyを注記として除去、strict到達検査除去、basisの組集合を参照せず全strict到達で拒否（無変更basisのpositiveが落ちる）、構造の判定をスカラー葉側へ倒す（`hook/started`→Turnが通る）、上限検査除去、reasonを固定文へ潰す、生schemaの記録 |
| CX-8 | tolerated位置の非使用: 同じstrict内容でT-notify／T-item／T-thread／T-config／T-inventoryの内容、およびConfig開放領域の免責位置（`permissions`、`model_providers`等）の内容だけを上限内の別JSONへ差し替えても、所有判定・回答（acquisition、text、answer_sha256、diagnostics）・quota観測・work／disposition／failure・§8a(3a)判定・journal／status／receiptの各項目が不変。唯一の例外としてterminal原本digest（`terminal_sha256`）は、所有turnのTurn.itemsにあるtolerated item内容を変えた場合に変わること（原本digestが投影へ置き換わっていないこと）を確認し、値の違いが他の項目と判断に影響しないこと、各実行の中では§7(b)の照合が一致して成功settlementまで進むことを同じ対で確認する。原本照合の維持: 保存済みterminalのreceiptから`terminal_sha256`だけを別の64hexへ変えたreceiptを、`OwnedPhaseSettlement::completed`（構造等価）と`receipt_committed_tx`（保存本文の完全一致）へそれぞれ与えると両方が拒否し、成功settlement・confirm・回答採用が0であること。`settled`のreplayと`finish_native_result_tx`の再保存でも同じ1 field差のreceiptが拒否されること。gate許容追加itemが所有turnに来たらUnknown→Held、foreignなら除外、許容追加にないtypeはProtocolError。thread/settings/updatedの方針不一致はturn/start前拒否／後Held | tolerated値を判断に使う差分、`permissions`を§8a(3a)判定へ読む差分（`verify_configuration`全体の流用）、terminal digestをstrict部分への投影へ置き換える差分、terminal digestの値を判断へ使う差分、`completed`の構造等価や`receipt_committed_tx`の本文一致からterminal digestを除く差分（各々）、許容追加itemを成功扱い、許容集合をjournalから復元、settings不一致の無視 |
| Q-1 | Wait／Admitの実producer。Waitはモデル入力0、Parked | Probeをquota admission前へ移動 |
| Q-2 | 普通Admit／recovery AdmitのKnown canary失敗。§9の全before／after、pool自probe解放、期限、readiness CASと観測非反映 | closureからProbe末尾を除外、自probe／期限更新省略、before画像比較を1列除去 |
| X-1 | 同一Native program／S合格後、各Native helper前、transport前にpath・stat・hash差替え。拒否後spawn0 | 束縛比較を各1点除去 |
| X-2 | 現Git sealの正例と差替え負例。GitはCLI hash再検査の対象外 | CLI identityとGit sealを混同、Git seal比較除去 |
| I-S | SのSchema／Handshakeからshim要求を実serverへ到達させる。SにはUnitなしなので有効managed contextなしの拒否、Grant0、helper外効果0。marker拒否とは記録しない | 到達確認／S Unit0検査除去 |
| I-P | genuine SAME actor、現Unit／epoch／正しいcookieでshim要求をserverへ到達させ、cookie検査後にmarker-bound Driver拒否へ達した有限証跡を確認。Grant0、helper外効果0 | state/runtime/driver.rsのmarker-bound拒否除去。元の拒否境界消失を検出し、別の拒否や接続失敗でpassにしない |
| M-1 | TOML新profile・cli_version不在／旧profile、旧キーpresence（表現可能な各値）。canonical前とinstallationで型付き拒否、helper／Unit0。TOML null負例は作らない | legacy宣言検査をhelper後へ移動、canonical前／installationのpresence検査を各々除去 |
| M-2 | 新Frameの旧key不在serialize→deserialize roundtripとdigest一致／旧保持Frame、新profile＋cli_version旧keyあり（JSON null含む）。型付き拒否、Preparing Unit Held、Session0。新旧profile混在不可 | Frame presence検査除去、旧keyを省略してcanonical化、Frame digest比較除去、新Frameへ旧key再出力 |
| M-3 | 任意の合法な単一行版数で全検査通過→Codex ExecutorのNativeInput、ClaudeはS合格。番号外だけで拒否しない | 旧版数一致・番号付きqualified_profile復活 |
| MP-1 | new、v1〜9、v10、v11、current、同時initializer。旧版別layout、guard、旧binary open拒否 | 旧referenceへstartup混入、新表guard省略、v11をlegacy一般枝へ落とす |
| MP-2 | §2の既存正負例を同じproducerで束ねる。0755／0700 root・検査済みstartup共有親・umask000 private child0700はpositive。root g/o書込み、別inode lock、他uid、startup symlink／file／0755、既存epoch／alias、spawn前置換、run外cleanupはnegative。追加対はuid=euid／不一致（非特権CIの不一致は§2の単一述語確認として別計数し、実producer／権限証明に数えない）、special bits付きでも0777 maskが0700のprivate child、空Version／Handshake cwd／残存物、Schema child identity一致／置換。実instance＋epoch成分の新規作成と同名拒否を確認 | root書込み／uid検査除去、lock inode束縛除去、startupの0700検査除去、旧epoch／alias採用、通常create_dir_allへ戻す、raw alias join、空cwd／Schema child再確認除去、identity確認省略、symlink追跡、旧残骸まで削除 |
| B-1 | Codex Executor実producerのB境界（230）／B+1とprefix246・初回入力256（§8a(3a)のconfig/read・mcpServerStatus/list各1行を含む）。Claude Reviewer（237／252／255）は純粋政策照合だけ | Git13を9として計算、Codex Schema／Handshake行の省略、§8a(3a)の2行を予算から除外（Probe末尾省略はM3） |
| R-1 | selfcheck全vector／source差分で1vector改変。installation拒否 | 成功token不要化、selfcheck呼出し除去 |
| LG-1 | 本番の正常ignored／foreign／tool／追加field／task_type任意string／patch.status欠落／pending／running／各malformed対。公開origin正常非human／不正、§6の旧黙認variant／blockの新拒否、合法text tool_result／image等表外拒否を分離し、型名だけのwire対応をpositiveにしない | ignored検査省略、consumer条件をwire enumへ戻す、未知fieldの改変をdigestから除外、表外黙認復活 |
| C-1 | Metadata／ShutdownSite／型付きrefusalのwireとcollector usability、台帳で維持するsynthetic child実wrapper | consumer配線欠落、正常child起動省略 |
| L-1 | 全旧入口のhelper／Session／新入力0、#19残置検査、共有ケースの現管理producer positive、完全台帳 | 拒否をhelper後へ移動、resume／checkpoint抜け、共有保持領域削除 |
| T-1 | 各deadlineの無応答、UUIDなし通知洪水、同UUID再送、stderrのみ、各byte境界／+1、残aggregate不足。有限停止とKnown／Unknownを別検査 | 通知でdeadline更新、dedup後だけ加算、stderr除外、捕捉枠をfile枠と混同 |
| EX-1 | Codex Executor＋Claude Reviewer設定のinstallation正例。Claude aliasを（Reviewer用に）登録した正しい設定で、(a) Goal計画のTaskがそのaliasをExecutorに選ぶ、(b) 保存済みTaskのExecutorがそのaliasに解決される、の各負例は`ExecutorProviderRefused`でhelper・Unit・Held・quota Admit 0。同じClaude aliasをReviewerに選ぶTaskは受理。thread/startの固定approval方針・sandbox・書込み可能root・network無効・config不在・dynamicTools不在の送信値をCX-3と同じ宣言比較で検査 | 主拒否点（Goal計画検証／Driver composition）の各除去（多層防御で止まってもUnit／Heldが生じて検出）、provider判定をalias名比較へ置換、Reviewer側のClaude aliasまで拒否、approval方針またはsandboxの送信省略（CLI設定継承）、network有効化、書込み可能root追加、config map送信 |
| EX-2 | ThreadStart応答のcwd・modelProvider・approval方針・sandboxが送信値と一致／各1項目不一致（広いsandbox、別approval方針、別cwd）。不一致ではturn/start・NativeInput 0 | 応答一致比較の各1項目除去、不一致後のturn/start送信 |
| EX-3 | Executor turn中（通知loop）の11種server requestの正常params／malformed。いずれもledger登録0、応答送信0、Grant0、success capture 0、Unknown→Held、自動再送0。autoApprovalReview通知も同じ | ledgerへ転送、accept／declineの自動応答、server requestを無視して続行、malformedを欠落扱い |
| EX-3b | fixtureがturn/start要求の受信後、TurnStart応答より前にserver request（11種の各正常params／malformed）を送る。応答0（`-32601`を含む）、ledger登録0、Grant0、success capture 0、Unknown→Held、自動再送0。対照として空のqueueから始めた応答待ちで、送信後に正常通知だけが来る場合は既存queueで続行 | 応答待ちを既存`Lines::response`の`-32601`返信・待機継続へ戻す、応答待ちだけ停止規則から除外、TurnStart応答到着後に続行 |
| EX-4 | 所有turnの許可item（§8a(3a)）の正常形と、commandExecution.cwd／fileChange pathが書込み可能root内／外（`..`、root外の絶対path、component境界を越える前方一致（`<root>x`）、symlink祖先経由、既存hard link）。所有turnのmcpToolCall／dynamicToolCall／webSearch／collabAgentToolCall／subAgentActivity／許可集合外item／未知item.type、item/mcpToolCall/progress。外・許可外・未知はUnknown→Held、成功0。構造正常なforeign thread／turnのitem・通知は全検査後に除外して続行（成功・Heldへ影響0）、malformed foreignはProtocolError | 字句検査（`..`拒否・component前方一致）除去、文字列prefix比較へ戻す、実祖先canonical化省略、hard link検査除去、許可集合を許可外itemへ拡大、正常foreignをHeld化、所有フィルタを構造検査より前に置く |
| EX-5 | owned `turn/completed` completed→成功原本の捕捉・保存→Success→既存retained Git capture→SC-N／failedの各分類／interrupted・inProgress・未知status・成功原本の捕捉前のturn/completed前EOF・exit・reap不明。後者はUnknown→Held、capture成功0、自動再送0。成功原本の捕捉後にpipe EOFまたはleader reapが不明になる場合はwork=Success・retained captureを維持し、custody軸Unknownを記録して当該UnitのNative新spawn・新authority 0、Held custodyとして表示 | interruptedをFailure／Success化、EOFをSuccess化、foreign turnのcompletedを採用、capture経路の迂回、捕捉後のreap不明でworkをUnknownへ反転、custody Unknownでも新spawnを許す、捕捉前のreap不明をSuccess化 |
| EX-6 | Executor transportのspawnが`rrx-spawn-hygiene`の消費関数経由、spawn前のprogram identity再検査、stop／Drop／forced deathの物理custody軸Known／Unknown分類（work軸と別記録） | 直接`Command::spawn`へ戻す差分のclippy拒否確認、identity再検査除去、reap不明をConfirmed化、custody Unknownでwork軸を書換え |
| EX-7 | 本番Sessionのconfig/read（web_search=disabled、対象features全件明示false、mcp_servers全entry enabled=false。absent／nullはserver0件で合法）とmcpServerStatus/list 1 page（全server tools空・toolsErrorなし・nextCursorなし）の正例→turn/start。負例: web_search有効／欠落、features 1件true／欠落、MCP server enabled、inventoryにtool 1件／toolsError／nextCursor非null、応答欠落・error・-32601・malformed、mcpServer/startupStatus/updated（starting／ready）。いずれも`ExecutorCapabilityUnverified`、turn/start・NativeInput 0。通知なし／整合settingsならturn/startへ進む正例に対し、turn前のresponse待ちでqueueに積んだstarting／ready、またはsettings相反は送信直前の全queue検査で`ExecutorCapabilityUnverified`、turn/start 0。turn前のautoApprovalReviewは送信後のHeldへ繰り越さず、送信後着信のEX-3／EX-3bと対にする。turn前のrate通知1件あり／通知なしの対で、前者のquota反映がちょうど1回（0回・2回は不可）、後者は0回であることを確認する。現契約は上書き送信0、B2b承認後も本番失敗を追加上書き・再起動で補わず、P決定済みargvのまま1回で停止する。config本文の保存・監査出力0。B2bのP選択／freshness／実argv照合は§8a(3a) B2b-6を共用 | 証拠取得をturn/start後へ移動、P Handshakeの観測で代替、1項目の比較除去、空mcp_servers表を不在証明としてinventory省略、nextCursorを無視、事前queue検査省略／先頭だけ検査／送信後に検査、queueを空にする際にrateを捨てる、処理済み通知を残して再処理する、本番失敗を追加上書き／第二spawnで補う |

Claude Reviewer Pに属するCL-1、CL-3、CL-6、CL-8、CL-9、Q-1のProbe配置、Q-2、I-PのClaude Probe分、BudgetのProbe末尾mutantは、M3で実producerを通して実施する。M0ではこれらをsyntheticだけで合格扱いにせず、producerも追加しない。CL-2、CL-4、CL-5、CL-7、CL-10、LG-1、R-1はM0の共通decoder・selfcheck検査として実施する（M0-U1で実施済みの範囲は検証記録のとおり）。

I-Pの最小本番観測は、既存のgenuine経路を変更せず非authorityの有限拒否分類を通す方式を選ぶ。state/runtime/driver.rs::validateの実marker検査が返す拒否だけにtyped `MarkerBoundDriverRefused`を付け、state/execution.rs::validate_authority→Store::validate_execution→ipc.rs::currentへ失敗型を保持する。文字列matchingで推定しない。ipc.rs::serveはpeer／request decode／現Unit・epoch・cookie検査の実通過後に返ったこの型だけを`CookieAcceptedMarkerRefused`とする。cookie失敗、その他検査の拒否、接続失敗は別の有限分類でありmarker到達証明にしない。現行accept loopはserve Errを捨て、shimには固定Refusedだけを返すため、この観測は未実装である。

sinkは現RuntimeOwner instance／epochに束縛し、Unitを持つPの既存helper custodyだけへ追加する有界メモリ観測とする。Pの本番custody登録時にownerへ観測先の弱参照を登録し、Sには登録しない。ipc.rs::currentの現Unit取得とepoch／cookie一致検査を実通過した要求について、server自身が現instance／epoch／Unitと登録済みP custodyの実行中helper原本を一意照合できた場合だけ、そのhelperのsinkへ有限拒否分類を書き込む。shim指定IDだけで紐付けず、同時aliasも各Unitとhelper原本で分離する。同一Unitに複数の実行中helper候補がある場合やhelper照合不能、cookieなし／decode不正／cookie不正、foreign／retired UnitはSや別Pへ振り替えず、`CookieAcceptedMarkerRefused`の観測成立にしない。genuine試験では対象helperから1要求だけを送る。helperごとに最大32要求の有限連番・分類を保持し、超過／対応不能は観測不成立としてqualificationを拒否する。raw request、cookie、cwd、拒否詳細は保存せず、shimへの現行固定`Frame::Refused("managed tool refused or observation incomplete")`を維持する。

このP拒否観測はRuntimeMetadata／status wireへ追加しない。成果物は§10の既存managed helper receiptに集約する非authorityの有限分類・件数とし、`CookieAcceptedMarkerRefused`件数および観測不成立の有無を上限内で保持する。実装時producerは既存`execution/native/version.rs::NativeVersionObservation::safe_receipt`から`Store::plan_phase_version_settlement`の`ManagedEffect.receipt`へ渡す鎖を新P helperにも配線し、sinkの原本観測からだけ集約する。既存本番のreceipt読取りconsumerは`state/execution/native_phase/version.rs::Inventory::read`→`EffectImage::decode`（bounded strict JSONからManagedEffectをdeserialize）、`Store::plan_phase_version_closure`／`Store::close_phase_version_observation`（原本pending／after全inventoryのCAS照合）、`Store::confirm_phase_helper_history`（原本effectを含む完全inventory照合）である。現sourceのVersion／Git鎖が新Schema／Handshake／Probe拒否観測を既に生成・解釈するとは扱わず、この拡張とP qualificationへの原本照合を実装・検証する。receiptの読取り・DTOからauthorityを発行せず、分類を失った画像や他helperの画像で観測合格を作らない。receipt16項目／値128bytes／private image8192bytes、S body4096bytes、既存行・捕捉・helper・journal上限は増やさず、集約を越えると拒否する。観測登録はP helper期間に限定し、通常の本番tool経路のGrant条件は既存のまま維持する。custody lifetimeの終わりでメモリ観測を解放する。

genuine SAME actor／現Unit・epoch／正cookieのP helperのshim要求が実serverへ到達し、同じhelperの原本sink→safe_receipt→ManagedEffect.receipt→上記本番consumerのdeserialize／全inventory照合まで同一要求の`CookieAcceptedMarkerRefused`分類と件数1が届くことをI-P positiveにする。同時aliasの正例は各Unitの分類が別Pへ混入しないことを確認する。無cookie／不正cookie／foreign・retired Unit／同一Unitのhelper候補複数・照合不能の負例は観測成立0・別Pへの書込み0であり、S Unit0もこのsinkへ登録・帰属させない。marker拒否mutantは対象拒否のtyped returnも失わせる差分として、同一要求のCookieAcceptedMarkerRefusedが消えることを検出する。他の安全検査が後で拒否しても合格にせず、Grant成功を作らない。分類のproducer欠落／receiptへの集約欠落／読取り時の分類消失もpositiveを不合格にする。authority constructor、SQL成功seed、偽cookie／偽actorによるpositive、残る検査の解除、shimへの詳細漏えいは禁止する。I-Sは有効managed contextなし・Grant0・helper外効果0の別境界を維持し、marker到達証明へ昇格しない。これは拒否の因果を観測する設計補修でありIPC脆弱性の認定ではない。

本番負例はbaseline差分とterminal disposition、登録前負例はAC-08、SはUnit0を検査する。

両OSで既存CIのfmt、clippy、workspace test、debug／release buildを実施する。まずローカルでまとめて検証し、試行pushや根拠のないrerunをしない。実CLI資格付けをCI fixture結果で代替しない。

owner-runは設計全体承認・実装・必要な検証後に実Macで1run行う。スクリプトは設定・認証・hookファイルを読まない。

1. 新しいrun ID、状態directory、一時directoryを作る。既存なら起動前拒否し、前回状態を再利用しない。
2. Claude／CodexのVersion観測と実行ファイルSHA-256を表示する。特定番号との一致を条件にしない。
3. release buildと試験用Git repositoryを用意する。
4. 新profile、実設定のCodex Executor＋Claude Reviewerでdaemonを起動する。
5. selfcheckと全aliasのS合格を10分上限内で確認する。Sでモデル実行を確認済みにしない。
6. 1 Project／1 Active Task／QUICKを投入する。
7. 設定の充足・不足について、現契約では既存の拒否分類だけを記録し、証拠不足ならFAIL／BLOCKEDとして保持する。B2b承認・実装後はP Handshakeのsettlement receiptの`b2_mode`／`b2_counts`／`b2_reason`だけを既存の読取り経路で確認し、不足MCP33件以上や他の上限・証拠不成立なら同様に保持する。許容内ではP決定と本番argvの原本一致、本番transport spawn=1を確認する。本文・名・pathを表示／保存せず、設定fileの読み取りや自動変更で補わない。Codex P receipt（Version／Schema／Handshake／Git×13）、ThreadStart応答のapproval方針・sandbox一致、turn/start前の能力の肯定証拠（config/read・mcpServerStatus/list）pass、Worktree gate、Codex Implement work=success、retained Git artifactのcapture、server request 0・許可集合外item 0・範囲外item 0、SC-N Waiting、sessions=1、Unknown effects=0、custody=knownを確認する。Claude ReviewerはSとReviewer argvのplan mode確認だけで、Claude P・review成功とは記録しない。
8. daemonを停止し、PASS時だけ当該scriptが作成したdirectoryを既存規則で削除し、残存なしを確認する。

出力は`M0 selfcheck=pass`、provider別startup passと観測版数／exe digest／実際に確認したstep（ClaudeはReviewer argvの`permission_mode=plan`を含む）、`codex preparation schema/handshake pass`とSchema gateの`strict_mismatch=0 tolerated_changes=<件数> accepted_additions=<件数>`（S／Pそれぞれ。インストール版の生成schemaが0.162.1契約を通ることのlive確認を兼ねる）、`executor approval_policy=<固定値> sandbox=<固定値> capability_evidence=pass server_requests=0 scope_violations=0`、codex rate=`empty_valid/usable`、claude rate=`not_observed`（M0ではClaude Sessionを起動しない）、`M0 implement=success provider=codex artifact=retained stop=SC-N sessions=1 unknown_effects=0 custody=known`、`M0 RESULT=PASS`を持つ。未証明一覧にはconfig/readによる既定値の実体化（defaults）、MCP server数・不足件数（B2b承認・実装後の`b2_counts`で得られる不足件数を除く）、P helperのMCP起動通知の有無（`p_mcp_startup`）と内部MCP process起動の有無、Codexのapproval往復／error分類／sandbox実効／network遮断の実効／MCP・web search・dynamic tool・collab無効化の実効（設定・inventoryの観測を越えるCLI内部の不使用）、Claude Reviewer P（canary／init／result）／review／permission roundtrip／既定mode／claude_rate_limit_event、escaped descendants、wrapper内部identityを残す。1runの成功をCodex Executor経路全般の資格付けへ広げず、static／synthetic合格をこれらのlive証明へ昇格しない。

失敗時は有限stage／reason／unit=held|retired|noneと保持した状態・一時directoryを表示し、`M0 RESULT=FAIL`として保存する。失敗runは削除・再利用せず、原因確認後の再挑戦は新run IDで行う。保持directoryの削除はowner判断に残す。追加cleanup工程や2runをM0受入条件へ増やさない。

### 12a. 影響範囲と競合順序

実装着手前に固定sourceからconsumer inventoryを再照合し、並行担当の進捗を確認する。現在の担当完了を推測しない。producerのないmoduleを先に完成扱いにせず、承認後に実producerからconsumerまで接続する。

| 変更／手段 | 影響consumerと対応 |
|---|---|
| helper変種・capture budget | `execution/native/readonly.rs`、`version.rs`、`preparation.rs:778`、`state/execution/native_phase/version.rs`。32本／8 MiB／Git13／各action上限、期限、settlement、capture境界fixtureを同時更新。既存上限は増やさない |
| prefix／Probe末尾と行予算 | `native/budget.rs`、`prepared.rs`、`native/preparation.rs`、`state/execution/native_phase/version.rs:281-319,486-501,621-641,687-698,804-866`、registration／setup／input。Codex Executor B230（§8a(3a)のsetup2行を含む）、Codex Reviewer B232、Claude B237へ統一し、Reviewer9本の旧閾値・純粋testsも変更 |
| profile／cli_version／qualified_profile廃止 | 公開source `config.rs`のNativeCompatConfig／canonical／tests、`adapter.rs`、`native/compat.rs:79-100,138-144,253`、`native/version.rs:94,138,150,156,202`、`state/execution/native_phase/version.rs:625`、installation宣言fixture。設定・Frame digest消費者も照合 |
| Claude strict／rate／分類 | `execution/claude_wire.rs:176-429,467以降tests`、`execution/quota.rs:175-274`、`execution/native.rs:2320-2392`、collector、permission consumer、native_fixture.py。現as_str／as_u64／配列抽出前にvalidated型を要求 |
| Codex finite契約・所有区分 | `execution/native.rs:939-952,2135,2204,2209-2264`、`codex/protocol.rs:105-165,1085-1093`、approval／collector、Schema／Handshake fixture。shared protocol16のcoverage保持 |
| quota Known closure | `state/execution/native_phase/quota.rs`、preparation／readiness／no-dispatchの末尾consumer、`state/execution/quotas.rs`、`native_phase/live_quota.rs`。ordinary／recoveryの全画像CASと観測非反映を別assertionにする |
| S gate・停止wire | `runtime/service.rs`、`runtime/admission.rs`、`runtime/installation.rs`、`runtime/stop.rs`、§4のRuntimeMetadata／ShutdownSite全consumer。start lock外実行、admission前提、停止／表示／deserializeを更新 |
| migration v12 | `state/mod.rs`、`state/execution.rs`、`state/managed_binding/schema.rs`、§3のMUTABLE_TABLES全consumer。historical集合とcurrent集合を分離し、旧binary混在不可を確認 |
| 旧入口廃止 | §11の全入口、standalone／Native／adapter／workflow tests、examples、既存protected拒否。entries／starts／preparationsと現status／cancelは保持 |
| 共通fixture | `execution/native/native_fixture.py`、installation tests／pin／success／activation/composition、`crates/rrx/tests/daemon_native.rs`。S・P helper・本番Sessionを区別し、BOOTSTRAP_CASEの失敗がSession前へ移る期待を更新。旧最小payloadやUUIDを新strict正例の証拠として流用しない |
| spawn経路（P4） | S custody、P helper、Codex Executor transportの新規spawnを`rrx-spawn-hygiene`の6消費関数（`spawn_std`／`output_std`／`status_std`／`spawn_tokio`／`output_tokio`／`status_tokio`）へ限定する。`clippy.toml`の直接`Command::spawn/output/status`禁止を緩めない。`OwnedProcess::spawn`は`Command`を値で受けるため、identity再検査後のCommand構築とspawnの順序を保つ |
| Codex Executor実行経路 | `execution/native.rs`のCodex Session（thread/start送信値と応答検査、config/read・mcpServerStatus/listの能力証拠、server request処理、item・turn/completed分類、work軸／custody軸の記録）、turn/start応答待ちの受信経路（`Lines::response` :1030-1042と`boot_call` :1743-1782。他のboot_callの既存挙動は変えない）、`codex/managed.rs`、`codex/protocol.rs::ApprovalLedger`（存置、M0で呼出し元追加なし）、`operation_paths`の字句検査と`codex/session.rs:2162-2217`の実filesystem検査（同じ述語を共有し、ledger経路の呼出し元は増やさない）、`codex/policy.rs`の`DISABLED_FEATURES`／`verify_inventory_page`と、`verify_configuration`の前半（web_search・features・mcp_servers）を切り出した共有述語（§8a(3a)。`verify_configuration`は切り出した述語＋permissions比較の形で既存のstandalone呼出し元の挙動を保ち、`DecisionPolicy`の構築とpermissions比較はM0 Executor経路から呼ばない。§11台帳のpolicy 7 testsの分類へ反映）、`adapter/native.rs::submit_approval`（存置）、collector。capture経路の`workflow/driven_settled.rs`、`execution/workflow_source.rs`、`execution/workflow_gates.rs`、`execution/results.rs`は無変更で回帰検証する |
| 役割入れ替え（設定・fixture） | §8a(0)のExecutor provider拒否点である`runtime/goal.rs:190`、`runtime/installation.rs:383`（`runtime/admission.rs:27`）、多層防御の`execution/workflow_source/native_handoff.rs:504`・`adapter/native.rs:152`。`AgentRegistry::from_managed_config`のprovider別alias登録は変えない。installation fixture／`native_fixture.py`、`tests/daemon_native.rs`、`tests/runtime_cli.rs`、`runtime/installation/tests/success.rs`等のClaude Executor＋Codex Reviewer前提をCodex Executor＋Claude Reviewerへ移し、Claude aliasをExecutorに選ぶTaskは型付き拒否の負例に移す。thread/start送信値・setup行数・argv本数を数えるfixtureも更新する。既存正例を削除で置き換えない |

| 競合所有集合 | 編集順序 |
|---|---|
| #93: `execution/native/tests.rs`、`runtime/installation/tests/fm_d2.rs` | 移植／旧宣言変更が必要。担当進捗と保存済み変更を確認し、#93の取り込み後に編集 |
| #93: `execution/results/tests/retained_tests.rs`、`workflow/committed_source_tests.rs` | この修正では編集予定なし。参照／coverage影響が確認されたら担当と調整 |
| #94（マージ済み）: `runtime/phase_jobs.rs`、`runtime/installation/tests/epoch_fence.rs` | phase_jobs自体の契約変更は予定しない。S gateでepoch_fence fixture前提が変わるため、main上の現sourceへ差分を載せる |
| F3: `adapter/inspection.rs`、`inspection/diagnostics.rs` | 編集予定なし。共有refusal型の参照は確認 |
| P4（#89、PR #99、`79e4d273`マージ済み）: `crates/rrx-spawn-hygiene`、`clippy.toml`、`OwnedProcess::spawn(Command)`の署名、P4が書換えたtests（`tests/daemon_native.rs`、`tests/runtime_cli.rs`、`runtime/installation/tests/{success,fm_d2}.rs`、`cli/daemon{,/tests}.rs`、`runtime/tests.rs`）とproduction（`codex/{session,transport,preparation}.rs`、`native/{version,transport}.rs`）、`tests/project_api.rs` | U0でM0ブランチを`79e4d273`へrebaseしてから編集する。P4の書換えを戻さず、その上に§4・§10a・役割入れ替えの差分を載せる。新規spawnは6消費関数だけを使う。`tests/project_api.rs`は編集予定なし |
| 上記外のfixture／installation／daemon tests | sourceと消費者を再照合して編集。並行worktreeの未保存変更を上書きしない |

修正手段自体の影響も検査する。登録前CLI本起動は継承設定・hook・MCP・履歴・IPC環境を消費し、Claude canaryは実quotaと費用を消費する。ProbeをAdmit後に置くためpre-Session lease／same-Unit gateの保持時間が延びるが、§10の有限上限内とする。strict化は本番Sessionにも作用し、未適合eventを終端させる。schema免責集合、行・byte閾値、argv本数、UUID／path形はそれを数えるfixtureも更新する。marker authority、既存transport登録手順、Core／terminalの契約を緩める変更は予定しないが、共有consumerの回帰検証を省略しない。

役割入れ替え後、M0のowner-runでモデル入力を消費するのはCodex Executorのturnだけであり、Claude canaryの実quota・費用消費とpre-Session lease保持時間の延長はM3で生じる。Codex Executorの固定sandboxはCodexのツール実行（build・testを含む）からnetworkとWorktree外書込みを奪うため、Implementの成否に影響しうる。これは安全側の制約であり、広げる場合はowner判断へ戻す。

### 12a-2. 影響範囲分析（Codex契約改訂 C1〜C4）

1. **変更対象**: `CodexWireContract`（basis 0.162.1、二分類、許容追加の受け渡し）、Schema gateの比較器（規則2の到達組を含む）と有限結果、通知payload・ThreadItem内容の実行時検査範囲、CodexErrorInfoのUnclassified、MessagePhase partial_answer、thread/settings/updatedの方針照合、ConfigRead／McpServerStatusListの型と判定path共有述語、§7免責表、basis catalog生成器（`scripts/build-codex-catalog.py`、値位置の修正）とverification file（追加のみ）。terminal原本digestと、それを含むreceipt全体の原本同一性照合（§7(b)の4か所）は変更しない（§7の例外として明記）。DB schema、環境変数、RuntimeMetadata wire、行予算、auth／CAS／owner／epoch／marker／#19／process custody／quotaの原本は変更しない。
2. **影響を受ける箇所**:

| 箇所 | 影響と対応 |
|---|---|
| U4予定の`execution/codex_wire.rs`（Codex decoder）と`execution/native/schema.rs`（Schema gate） | 未作成。二分類・新しい宣言の表・有限結果をここで実装する。gate許容追加はメモリ上の有界集合としてS Handshake／同じUnitのSession decoderへ渡す |
| `runtime/startup.rs:238-240`、`runtime/startup/probe.rs` | U3でCodex Schema／Handshakeを`codex_schema_unavailable`／`codex_handshake_unavailable`で拒否中。U4でgateへ置換し、journal bodyへ有限reason・件数・位置を記録（4096 bytes以内） |
| `execution/native.rs:2209-2264`（通知loop、thread/start送信値・応答検査）、`:2313-2360`（所有filter、turn/completed分類） | validated frameへ置換する既存計画（§12a）の上で、T-notifyの内容不使用、許容追加itemのHeld、Unclassified→Held、thread/settings/updatedの照合、turn/startのapprovalPolicy／sandboxPolicy送信を加える |
| `execution/quota.rs:257-275`（codex_subscription_error／codex_capacity_error） | validated CodexErrorKindを受ける形へ変更。Unclassifiedはどちらにもfalse。既知値の分類は不変 |
| `execution/native_result.rs:473-484`（agentMessage phase） | partial_answerをcommentaryと同じ非採用に追加。他phaseのpoisonは現行どおり |
| `codex/managed.rs:16-19`（`frame`→`Event::parse`）、`codex/protocol.rs:105-165,973-974,1085-1093` | Executor Sessionの検査はvalidated frameへ移す。`thread/tokenUsage/updated`のTokenCountersはlegacy standalone経路の消費者であり、T-notify化はNative Session decoderだけに適用する。§11台帳のshared protocol 16 testsの分類へ反映 |
| `codex/policy.rs:213-298`（verify_configuration、verify_inventory_page） | `verify_configuration`の前半（web_search・features・mcp_servers）を判定pathの検査済み値だけを受ける共有述語へ切り出し、M0 Executor（継承設定の経路とB2aの上書き後の経路の両方）はこれと`verify_inventory_page`（S-inventoryの値だけを読む）だけを使う。後半のpermissions profile比較（`permissions.<profile>.filesystem`・`network.enabled`、Config開放領域＝免責位置）、`DecisionPolicy::from_native`／`for_executor`（T-configのsandbox_mode・approval_policy・approvals_reviewerと開放領域のdefault_permissionsを読む）、`arguments`のpermissions上書き、`provider_environment`（model_provider・model_providers）はM0 Executor経路で呼ばない。現在の呼出し元はstandaloneの`codex/session.rs:1611-1707`だけで（`execution/`配下からの参照なし）、§11の台帳で扱い挙動を変えない。M0のsandbox照合はThreadStart応答と`thread/settings/updated`（strict）で行う |
| `execution/native_result.rs:506-507`（`Collector::codex_terminal`のTurn全体digest）→`content()`:635の`terminal_sha256`→`execution/native.rs:1877,1915`（`NativeResultReceipt.terminal_sha256`）→`native_result.rs:239,290`（`validate`の形式・Complete時存在検査）。receipt全体の照合: `execution/native/phase_protocol.rs:563-567`（`OwnedPhaseSettlement::completed`の構造等価）、`:273-279`（`settled` replayの構造等価）、`state/execution/native_phase/terminal.rs:183-191`（`receipt_committed_tx`の本文完全一致。呼出し元`:162`、`:717`、本文の作成`:543`）、`state/execution/native_results.rs:476-483`（`finish_native_result_tx`の再保存本文の完全一致）。保存: `native_results.body`（`terminal.rs:799-800`、`native_results.rs:494`） | digestは検査済みTurnのValue画像全体（tolerated item内容を含む）のまま維持し、strict部分への投影に置き換えない。§7(b)の4つの照合はdigestを含めたまま維持する（変更なし）。field名で読むのは`validate`だけで、`state/`・`runtime/`・`workflow/`・`cli/`にfield名の読取りはなく、意味上の分類・判断に使うconsumerはない（§7(c)）。全体照合の点検: receiptを含む構造の等価・本文比較を`NativeResultReceipt`の利用箇所と`native_results`の参照から全列挙し、上の4つ以外にないことを確認した。digestを含まないもの: `projection()`（`native_result.rs:209-213`、status／resultへの出力）、`native_results`索引列の`check_indexed`、`NativeInvocation`（digest fieldなし。`native_phase.rs:576`の照合対象）、`retain_saved_terminal`（`native.rs:1343-1359`、`Arc::ptr_eq`の同一性で値比較なし）、transport effectのreceipt（`native/transport.rs:59`の`BTreeMap`で別型）。receiptの`wire`（`WireEvidence`、`native.rs:920-977`）は拒否されたframe（strict JSON・上限違反、EOF）のbytes digestで、同じ(b)の照合に含まれるが、上限内の正常frameでは作られないためCX-8の対では不変。§7の例外としてCX-8でdigestが変わること、他項目が不変であること、digestだけが違うreceiptを(b)の照合が拒否することを確認する。`codex_item`／`codex_terminal`のagentMessage署名と`ApprovalLedger::observe_item`（fileChangeだけ）はS-itemだけを読み、`codex_item`の256件上限はid／typeの件数でありtolerated内容に依存しない |
| `execution/native/native_fixture.py`、Schema／Handshake fixture、`tests/daemon_native.rs` | 0.162.1 basisの縮約fixtureへ更新（84通知、8 request／8 response、CodexErrorInfo開放枝、partial_answer、Turn.rootTurnId、TurnStartParamsの追加field）。CX-7／CX-8を追加 |
| `doc/verification/macos-minimum-m0-static.json` | 書換えない（履歴）。新catalogは別file |
| 受入条件 | AC-05の適用範囲をstrict位置へ明確化し、AC-18を追加（要件） |

3. **影響種別**: 機能影響（tolerated位置の変化でgateが止まらない、許容追加itemはHeld）。セキュリティ影響（tolerated位置の内容を判断に使わない規則、共有definitionのstrict比較、構造をもつstrict到達definitionへの新しい到達の拒否で、判断値の黙った緩和を防ぐ。E-05〜E-10は受理範囲を広げるためオーナー承認対象）。互換性影響（0.160.1のCLIはstrict位置不一致でgate拒否となる。旧版の同時サポートはしない）。パフォーマンス影響（比較は既存のnode／work上限内）。
4. **対応方針**: 上表のとおり修正し、tolerated化で不要になる型付き検査（GuardianAction等）は実装しない。旧版CLIの受理は追加しない。
5. **未影響の根拠**: Claude Reviewerのdecoder（§6、M0-U1）はCodex契約を参照しない。§5の行予算はmethod数・setup行数が変わらないため不変。§9のrate契約はGetAccountRateLimitsResponseとRateLimitSnapshotが0.160.1と同一宣言（compare結果）のため不変。§3のmigrationとjournal schemaは列を追加しない（bodyの中身だけ）。retained capture経路（`workflow/driven_settled.rs`等）はwork軸の成功tupleの形が変わらないため無変更。

### 12a-3. B2bの影響範囲（提案・オーナー承認待ち）

固定入力HEAD `55e16eed`のsourceを読み取って列挙し、U6設計レビューround 2修正ではHEAD `de290344`で該当consumerを再確認した。下表のpathは`crates/rrx/src/`相対（scripts／docを除く）。この執筆でコード・catalog・定数は変更しない。承認後の実装では同じconsumer集合を再照合する。

| 変更する契約／手段 | consumer全体と対応 |
|---|---|
| `command_argv`の本番呼出し | 定義`execution/native/prepared.rs:280`、直接callerは`plan_native_command`（:362）と`execution/native/compat.rs::check_command`（:172）。同じP決定原本を受けて期待argvを導出。`prepared.rs::issue_prepared`の原本保持／再検査も更新 |
| `command_argv`のprobe呼出し | `execution/native/version.rs:566`（P Handshake spawn）、`runtime/startup/probe.rs:363,374`（S Codex／Claude）。用途を明示し、probe自身には上書きを付けない。共通signature変更の全call siteで固定の継承argvを保つ |
| `command_argv`の直接test consumer | `execution/native/prepared.rs::primitive_tests::nongrant_native_command_keeps_exact_permission_vector_and_bounded_values`の:380,388,410,415,419,429。Claude／Codex Reviewerの既存argvと上限18を保ち、Executorのk0／k16／k48（99要素）／不足MCP33件拒否と要素数validatorの型付きP決定あり99／100境界、決定なし19要素拒否、他用途への決定渡し拒否を追加。固定3要素のcontrolを維持 |
| `check_command`の全caller | `execution/native/prepared.rs::issue_prepared`（:264）、同`plan_native_command`（:370）、`execution/native/registration.rs::registration_material`（:51）。3 callerとも`matches_actor`等で確認済みのcompletionが保持する決定をcommandとは別引数で渡す。`compat.rs::NativeCompatQualification`は元のactor／Frameと同じP決定の対応を検査し、期待値の出所をcommand自身にしない。`compat.rs::NativeCompatStatic::observe`（:128）はVersion custodyだけを受けるため、そこからHandshake決定を推定しない。`execution/native/preparation.rs:1114-1149`のstatic compat→identity→Version→metadata（Schema／Handshake）→Git completion→observe(version)→commandの順序と決定の受渡しを更新対象として確認する |
| 決定原本のproducer／custody | `execution/codex_wire.rs::Handshake::{new,request,initial,observe,complete}`に用途別stepとConfigJudgementによる不足投影。`ConfigJudgement::{decode,restricted}`、`codex_wire/types.rs`のConfigRead／Config／McpConfigEntry refinement、`codex/policy.rs::DISABLED_FEATURES`／`restricted_features`の固定対象を共用。全DecisionPolicy・permissions検査を流用しない |
| PだけのHandshake通知判定 | `execution/codex_wire.rs::Frame::handshake_notification`（:527）の共通受理規則はSを含め不変。HandshakeのP用途だけが呼ぶ専用相反判定関数を追加し、config/read前の一時通知投影と応答後／tailの検査済みserver表を照合する。enabled=falseの名だけ相反、enabled=true／absentやconfig外の名は相反でも選択入力でもなく、本番再検査に委ねる。Sの通知受理・method数の対照testを維持 |
| P Handshake method集合を消費する箇所 | `execution/native/version.rs`のHandshake構築（:844）、`capture_version`の送信・相関・完了・tail・`NativeVersionObservation::{qualify_once,safe_receipt}`、`NativeReadonlyHelperCompletion`と`prepare_phase_git`の構築、`execution/native/metadata.rs::prepare_phase_metadata`／`Store::plan_phase_metadata_intent`。`state/execution/native_phase/version.rs::NativeVersionHelperPlan`の同じactor／contract束縛を通す。`runtime/startup/probe.rs:346`のHandshake構築はS用途に固定。`execution/codex_wire/tests.rs`、`runtime/installation/tests/u5.rs`と`execution/native/native_fixture.py`のmethod順／完了条件fixtureをPだけ更新 |
| Prepared／registration／spawnの間接consumer | `execution/native/preparation.rs`の`metadata_contract`・`retain_completion`・`retain_compatible_command`・`prepared_original`・`retain_prepared`へ原本を保持し、`retain_transport`の単消費を維持。`prepared.rs::PreparedNativePhase`、`registration.rs::{validate_original,validate_open,registration_material}`、`state/execution/native_phase/transport.rs`の登録計画／`transport_effect`と原本照合、`execution/native/transport.rs::{validate_original,physical_command,start_prepared_transport}`が同じ決定・command digest・argvを消費する。argv本文の永続化は追加しない |
| 本番証拠consumer | `execution/native.rs`のCodex Session／`Lines::response`／`Lines::queue`（:1005）／`Lines::receive`（:994）／`boot_call`と`codex/managed.rs`、`codex_wire::ConfigJudgement`、inventory共有述語。本番は従来8 setupで証拠1〜4を再取得し、Pの不足計数をpass根拠にしない。送信直前のqueue処理は§8a(3a)の唯一の定義に従う。`execution/native.rs:2319-2320`（queueから取り出したrate通知→`observe_quota`）の既存反映を送信前へ移してちょうど1回を保ち、queueを空にする際のrate欠落／処理済み通知の再処理をEX-7の対とmutationで検出する。server requestの停止窓はEX-3b、turn前失敗はEX-7で維持 |
| catalog／method／Schema gateの表 | §0aのE-08・C1〜C4の適用説明、§7のS-send／S-config・§8の握手順、§12のphase別表／CX-6／EX-7を更新。`codex_wire.rs::{reserved_method,sent,response}`、`codex_wire/types.rs`のConfigReadParams／Response、`execution/native/schema.rs::{FILES,project,compare}`、`scripts/build-codex-catalog.py`のmethods／responses引数、`doc/verification/macos-minimum-m0-catalog-0162.json`のClientRequest_usedとConfigReadResponse閉包は既にconfig/readを含むため新method／新root追加0。12 source／303 definition／8 request・8 response／digestは変更不要。`doc/verification/macos-minimum-m0-u4-contract.md`のconsumer対応表とschema fixture manifestはPの使用箇所を更新する（旧catalogは不変） |
| receiptのproducer／全原本照合consumer | `execution/native/version.rs::NativeVersionObservation::safe_receipt`でHandshake stdout digestを除外し有限3項目を追加。`state/execution/native_phase/version.rs::{Inventory::read,EffectImage::decode,plan_phase_version_settlement,confirm_phase_helper_history}`と`version/closure.rs::{plan_phase_version_closure,close_phase_version_observation}`、同ファイルの原本receipt比較とhelper settlement／history画像、`preparation.rs::qualified_history`・completionが更新された完全画像を照合。receipt16／128とJSON decode（body8192、depth12、nodes256、object64、array16）は不変、最大画像で確認。transport receiptは変更不要 |
| Handshake stdout非保持のproducer／寿命 | `execution/native/version.rs::BoundedOutput::{observe,read_window}`（:33-58）と`capture_version`（:834以降）のstdout累積（:960）、pending decode・observation構築（:1034-1055）を変更し、config/readを含むHandshakeはbyte数・EOF・上限だけを保持、全終了経路でpending／frameを破棄。`NativeVersionObservation.stdout`／`stdout()`（:155）から分離した本文field／accessorのないHandshake型をcustody→completion→Preparedへ通す。Version／Gitの本文利用consumerはそれぞれの型に限定し維持する（同`version.rs`のGit seal照合:134、Version解析:186,198、receipt digest:219,269、`execution/native/readonly.rs::qualify_ownership`のstdout利用:551-558）。stdout保持復活の変異をkill |
| P／transport／setup予算定数 | `execution/native/budget.rs::native_effect_budget`のCodex Executor `(230,233,246)`、`readonly.rs`のHELPER_LIMIT32／AGGREGATE_BYTES8 MiB／GIT_ACTIONS13、Handshake `output_limit`1 MiB、`version.rs`のresponse30秒／Handshake90秒、`state/execution/native_phase/version.rs`のROWS256／BODY_BYTES8192／INVENTORY_BYTES2 MiB、`shape`・reserve計算は数値変更0。`prepared.rs`の要素数上限だけ本番Codex Executor経路で型付きP決定を受け取る場合に99（M=32）を新設し、他用途への決定渡しと決定なし19要素は拒否する。S／P helper／Codex Reviewer／Claudeは18、要素128／encoded64 KiBは不変。不足server32件／+1とargv99／100の事前検査、receipt最大画像とP capture残量の消費、`version.rs`の予算tests、B-1実producer、setup8＋transport1＋input1の検証fixtureを更新。追加RPCを新helper／setup行にしない |

承認後もBLOCKEDのままにする場合: 不足MCP33件以上、argv99要素超過、選択MCP名のargv要素長超過、捕捉・body・inventory残量不足、実CLIが制限argvを受理／反映しない、本番証拠や固定approval／sandbox応答一致が得られない場合。approval・sandbox・permissionの設定上書き、`permissions.*`／`network.enabled`送信が必要になるなら**BLOCKED（B2／B2aの承認範囲外）**とし、迂回・承認された上限を超える拡大・再起動で成立させない。

### 12b. セキュリティ自己点検

| 面 | 維持する対策・検証と限界 |
|---|---|
| CLI偽装／入力 | strict JSON、有限method／variant、分類前のfield型検査、相関ID、nonce完全一致、UUID内容digest、成功後tail、deadline／累積bytes。追加fieldからauthorityを作らない |
| canary副作用（M3） | Admit後だけ送信、Reviewer argvのplan mode、tool／権限要求で拒否、応答0。既存hookや自動許可toolの開始を隔離したとは保証しない |
| helper IPC | SはUnitなし、Pはgenuine contextからmarker-bound拒否へ実到達。cookie失敗／接続失敗をmarker検査の証明にしない |
| S流用／権限 | freshメモリ公開だけ、journalからpass復元しない。Sは拒否専用でP／Prepared／permission／Session authorityを発行しない |
| filesystem／process | raw aliasを使わず、private child identityと限定cleanup、intent先行custody、stdin保持、EOF／leader reap／joinの実観測。root境界は§2とMP-2の同じ正負例・mutantで独立確認する。canonical pathだけ、Drop、forced deathから全子孫死亡を推定しない |
| quota／CAS | SAME actor原本、epoch、revoke、no_registration、末尾inventory、全before／after画像、既存freshness。canary観測非反映と機械的closure更新を分離 |
| 秘密のsink | frame／account／codexHome／nonce／stderr／config/read本文（MCP serverの環境変数等を含みうる）を永続化せず、有限receipt・件数・digestのみ。file／capture／JSONの既存上限維持 |
| legacy／#19 | 起動・新入力前拒否、全consumer台帳、共有3保持領域と停止保持、#19既存部品・検査保持。削除や拒否だけで共有安全性のpositiveを代替しない |
| Codex Executor実行（新規面） | rrxはapprovalを自動付与しない（自動付与できる種類は0）。Executor providerはTask選択時の拒否点で限定する。承認要求を出さない固定approval方針、Worktree＋Unit temp／output／cacheだけを書込み可とするworkspace-write sandbox、network無効、config上書きなしを送信し、ThreadStart応答で一致を確認する。turn/start前に同じSessionの実効設定とinventoryでMCP・web search・dynamic tool・collabの無効／不在を肯定証拠で確認し、得られなければturn/startしない。turn/start応答待ちを含む全受信経路のserver request／autoApprovalReview着信は応答0でUnknown→Held。許可集合外・範囲外item・未知variantもUnknown→Held。モデルが出したcommand・path・patchは信頼しない入力として扱い、成功はowned turnのcompletedで成功原本を捕捉した場合だけ。限界: sandboxの強制はCodex CLIとOSの実装に依存し、rrxは実効を証明しない。能力の無効化証拠はturn/start時点の設定・inventoryの観測であり、CLI内部の不使用の証明ではない。所有turnでの禁止itemやpath範囲外の検出は事後の検出であり、既に生じた効果を防がない。Task本文や既存repo内容からのprompt injectionについてrrxが主張できるのは、利用できる能力をコマンド実行・ファイル変更（sandbox内、強制は未証明）へ減らすことと事後のHeld化だけで、影響がsandbox内に限られることも内容の安全性も保証しない。成功はImplement完了であり、merge権限（M4）や後続phaseのauthorityを与えない |
| CLI更新とtolerated位置（C1〜C3） | strict位置（送信値、判断値、server request、効果・結果を運ぶitem、終端）は完全一致を要求し、新しいserver request・strict位置の新property・used enumの新値は起動前に拒否する。tolerated位置は有界strict JSONの保持だけで、判断・authority・記録に使わない（例外はterminal原本digestだけで、既存の原本同一性照合に含めるが判断には使わない、§7）。strict到達definitionは共有されていてもstrictに比較し、tolerated anchorから構造をもつstrict到達definitionへの新しい到達は拒否して、黙った再分類を防ぐ。M0のConfigRead consumerは判定pathだけを読み、免責位置の`permissions`等を読まない。許容追加itemは所有turnでHeld。限界: tolerated通知の意味をrrxは知らないため、効果を新しい名前のdefinition経由で報告する通知が加わった場合は事後path検査の対象外になりうる（防止はsandbox強制に依存）。gate許容追加の集合はgateを実行したprogramにだけ結び付け、journalから復元しない |
| spawn経路 | S custody、P helper、Executor transportの新規spawnは`rrx-spawn-hygiene`の6消費関数だけ。`clippy.toml`の直接spawn禁止を維持し、`OwnedProcess::spawn(Command)`へ所有を移す。macOSのpre_exec forkは#98の残余リスクとして記録し、M0で解消しない |

S custodyは自journalのintent／settlementだけ、P SAME actorは既存Admit／CASの範囲でhelperとnongrant closureだけを扱う。S観測のGit／Prepared消費、別actor・CLI peerによるsettlement、revoke済みactorによる新spawn、Probe中のpermission応答は不可。revoke後も既存原本で物理結果を記録することと、新しい実行許可を分ける。ownerの設定選択・一括設計承認、adminのstatus／stopはprobe成功から新authorityを得ない。別Task／Project、同uid別process、unauthにも強制pass・再判定入口を作らない。

source→sinkはVersion stdout→有界観測receipt、wire→validated frame→有限件数／分類、schema files→有限形状比較／digest、program→stat／SHA-256、nonce→メモリ比較だけとする。設定／allocationからargv・環境へ渡す既存境界検査を維持する。helperは利用者の継承権限で動き、exec直前の窓、wrapper内部、escaped descendantsは未証明のままである。

新しい本番権限: 本改訂でrrxが新たに許すのは、承認済みWorktree範囲のsandbox内で、Codex Executorが1 turnの間に操作ごとの承認なしでコマンド実行・ファイル変更を行うことである。MCP・web search・dynamic tool・collabはturn/start前の肯定証拠で無効／不在を確認した場合に限り、その権限を含まない。これはrrxの権限応答ではなくCodexへ渡す実行方針であり、B2（承認済み）による。source→sinkは、config/read・inventory→判定pathだけのメモリ比較（本文非保存）、モデル提案のcommand／path／patch→sandbox内実行とrrxの事後path範囲検査、owned turnのcompleted→成功原本→既存retained Git captureとする。Codexの資格情報・ChatGPT accountはCLIの継承状態のままで、rrxは読取り・保存しない。

### 13. 一括有限承認表

| ID | 全体設計承認に含める決定 | 現状 |
|---|---|---|
| A1 | quota Admit後、準備ごとのClaude canary 1turn。費用・履歴・既存hook・quota消費、Known failureの明示retry帰結を含む。 | 承認済み（2026-10-13、固定HEAD c6413082）。役割入れ替え後はClaude Reviewer Pへ適用し、本番化はM3（B3で再承認）。実装・検証は未完了。 |
| A2 | Probeの専用UUID、固定文面、Executor plan mode、tool／権限応答なし。既定modeや隔離を証明しない。 | 承認済み（2026-10-13、固定HEAD c6413082）。「Executor plan mode」はReviewerの本番argvのplan modeへ読み替え、適用はM3（B3で再承認）。実装・検証は未完了。 |
| A3 | 旧起動 API の機能廃止、不要コード・固有テストの理由付き整理、共有安全性の実管理経路での検証。Reviewer 本番正常系は M3。 | オーナー方向決定済み（2026-10-13）。設計全体承認済み。台帳・実装差分・実装後レビューは未完了。 |
| A4 | S全alias gate、版12 migration、有限deadline／capture、journalの独立upper boundsと上限時BLOCKED、confirmedの限定prune／unknown保持、安全なprivate path、停止wire変更と新旧混在不可。上限の容量を使い切れる保証や旧残骸の自動cleanupは含まない。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。 |
| A5 | 本番strict decoder、finite item／notification／nested variant、分類fieldの型・optional／nullableとwire domain／消費条件の分離、83method所有区分、UUID改変拒否、rate解析不能終端。既知frameの未宣言fieldは有界保持受理し業務非使用、Codex closed object／key-tag／RPC判別keyは§7の例外で拒否。Claude origin公開型と非human消費、実消費6項目／usage・structured_outputは§6の明示契約とし、telemetryからreplay authorityを作らない。未知variantは拒否し、旧黙認から新拒否になるvariant／blockとAC-05正常母集団は§6の一覧で判断する。公開未確認のrrx限定集合と数値refinementのlive適合は未証明。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。Codex部分はC2／C3（§0a）で改訂済み（2026-10-16 オーナー承認）であり、改訂部分は§0aとC2／C3が優先する。 |
| A6 | schemaの未知method／宣言property／discriminatorで起動前拒否。明示Opaque集合とtyped mapだけを例外にし、枝別免責を拡大しない。wire frameの未宣言field保持とは別契約。static catalogは開発時根拠でありruntime仕様生成器ではない。raw hash pinなし。未適合CLI拡張は契約見直しまでBLOCKED。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。C1〜C4（§0a）で改訂済み（2026-10-16 オーナー承認）であり、改訂部分は§0aとC1〜C4が優先する。 |
| B1 | 役割をCodex Executor＋Claude Reviewerへ入れ替える。M0はCodexを最初のExecutorとして提供する。aliasはprovider別に登録し、Claude aliasはReviewer用に受理する。TaskのExecutorがClaude provider aliasへ解決される場合は§8a(0)の主拒否点（Goal計画検証、admissionのDriver composition）で型付き拒否する（helper・Unit・Held・quota Admit 0）。SのClaude argvはReviewer role（plan mode付き）とする。Claude Executorの再提供は別判断とする。 | 承認済み（2026-10-15、固定HEAD 2a9114bc）。実装・検証は未完了。 |
| B2 | Codex Executorの実行方針（§8a）: 承認要求を出さない固定approval方針、Worktree＋Unit temp／output／cacheだけのworkspace-write sandbox、network無効、config上書きなし、応答一致確認。turn/start前のMCP・web search・dynamic tool・collab無効／不在の肯定証拠（config/read・mcpServerStatus/listの2 method追加とCodex Executor B230を含む。得られなければBLOCKED）と許可item集合。rrxの自動approval付与は0。turn/start応答待ちを含む全受信経路のserver request着信・許可集合外／範囲外item・未知variant・不明なturn結果はUnknown→Held。work軸と物理custody軸の分離。承認済みWorktree範囲のsandbox内で、操作ごとの承認なしにコマンド実行・ファイル変更を許す新しい本番権限を含む。 | 承認済み（2026-10-15、固定HEAD 2a9114bc）。実装・検証は未完了。 |
| B2a | 継承設定のままでは§8a(3a)の肯定証拠が得られない場合に、制限方向だけの狭い上書き（`--disable <feature>`、`mcp_servers.<name>.enabled=false`、`web_search="disabled"`、またはThreadStartParams.configでの同等値）を許すか。許す場合も上書き後に同じ証拠検査を通し、approval方針・sandbox・permissionは上書きしない。許さない場合、該当環境のExecutor準備はBLOCKEDのまま（owner-runが成立しない可能性を含む）。 | 許可（2026-10-15、固定HEAD 2a9114bc）。制限方向の上書きだけを実装し、上書き後も同じ証拠検査を通す。方式はapp-serverの起動argv（`--disable`／`-c`）で、継承設定のままで証拠が得られない場合だけ加える。ThreadStartParams.configは使わない（オーナー決定 2026-10-18）。 **U6: 実現はBLOCKED（P契約変更待ち）**。証拠1は対象15 feature全件の明示false・web_search="disabled"・全MCP enabled=falseを要求し、既定設定は拒否見込み（config/readの既定値実体化は未確認）。B2bの文面承認まで現契約の追加argvは0。 |
| B2b | P Handshake末尾のconfig/readで不足クラスだけを決定し、同じP原本をPreparedへ束縛、完全一致command検査と既存registration digestに含め、必要なら最初から制限argv付きの本番transportを1回だけ起動する（§8a(3a) B2b-1〜6）。**本番Codex Executor経路が型付きP決定を受け取る場合だけ要素数上限を18から3＋2×(15＋1＋M)=99へ変更（選択MCP上限M=32）**。他用途への決定渡しと決定なし19要素は拒否する。S／P helper／Codex Reviewer／Claudeは18のまま。各要素128 bytes・選択名102 bytes以下の`[A-Za-z0-9_-]+`・固定3形式の語彙・command64 KiB、B230→246→256を維持。不足MCP33件以上はoverride_limit、不正／相反観測もP拒否。Pではenabled serverの起動通知を相反・選択入力とせず、config/readを含むHandshakeのstdout本文を保持しない。本番は証拠1〜4を全て再取得し、送信直前の全queue検査でturn前相反を拒否、失敗時はExecutorCapabilityUnverified／入力0、retryなし。approval・sandbox・permission上書き、permissions.*／network.enabled送信は0。 | **提案・オーナー承認待ち**（2026-10-18）。Pで決める方向はオーナー決定済みだが、ここに書いたP契約・Codex Executorだけのargv上限変更・上限超過時拒否・有限記録の本文は別承認が必要。承認済みB2／B2aやC1〜C4の追認には数えない。 |
| B3 | Claude P（A1のcanary、A2のProbe条件）をClaude Reviewer準備の検査とし、本番producer・実境界・受入をM3へ移す。ProbeはReviewerの本番argv（plan mode付き）をそのまま使う。M0のClaude coverageはS（Reviewer argv）、strict decoder（M0-U1）、rate selfcheckとする。 | 承認済み（2026-10-15、固定HEAD 2a9114bc）。A1／A2の適用対象・時期の変更。 |
| B4 | Codexモデル実行（Executorのthread/turn）をM0範囲へ入れ、AC-13を「全aliasのS、Codex P、Worktree→Codex Implement成功、SC-N停止」の1runへ改める。Codex Executor群EX-1〜EX-7（EX-3bを含む）を追加し、Claude Probe系controlsはM3へ移す。 | 承認済み（2026-10-15、固定HEAD 2a9114bc）。実装・検証は未完了。 |
| C1 | Codex契約の位置二分類とSchema gateの比較（§7・§8）: strict位置は完全一致（optional／nullableを含むproperty・required・type・enum・枝・制約・defaultの追加／削除／変更で拒否）、tolerated位置は増減を許容、strict到達definitionは共有されていてもstrict比較、tolerated anchorから構造をもつstrict到達definitionへのbasisにない到達で拒否（スカラー葉への到達は可。無変更basisの到達組20組はすべてスカラー葉）、新しいserver request・used enum新値・strict削除は拒否、新ThreadItem type（上限32）と規則内の新通知（上限64）は許容追加（itemは所有turnでHeld）、結果は有限Pass／Refusedと有限reason・位置の記録（生schemaなし）、許容追加はメモリ上だけでdecoderへ渡す。**A6を広げる**（E-01〜E-03）。 | オーナー承認済み（2026-10-16。非執筆側の設計レビューはSolが3巡で承認、固定HEAD 9ce78531）。 |
| C2 | 契約basisを0.162.1のtag export catalog（12 source、303共有definition、`bbdede32…`）へ移し、0.160.1 catalog（`c44baad3…`）は履歴として保持（§12）。84通知、送信8 method／応答8種、closed object 29位置。CodexErrorInfo開放枝の`Unclassified`→failedでUnknown→Held（E-11、**A5を広げる**、結果はHeld）、MessagePhase partial_answerの受理と回答不採用（E-12、**A5を広げる**、回答authorityなし）、thread/settings/updatedの方針照合（E-14、狭める）、Executorのturn/startでのapprovalPolicy・sandboxPolicy明示（B2の範囲内の値確定）。 | オーナー承認済み（2026-10-16。非執筆側の設計レビューはSolが3巡で承認、固定HEAD 9ce78531）。 |
| C3 | 型付き検査の範囲とOpaque／tolerated集合（§7）: 21通知の全payloadと4通知のmethod・所有キーだけを型検査し他59通知は所有キーだけ、ThreadItem内容はagentMessage・commandExecution・fileChangeだけ、Thread coreとTurnError（misalignment除く）、ConfigReadは判定path（web_search宣言＋features／mcp_servers refinement）だけを型検査しConfig開放領域（`permissions`等）を免責表へ追加、M0のconsumerは判定pathだけを読む共有述語で既存`DecisionPolicy`の構築・permissions比較を使わない、terminal原本digest（`terminal_sha256`）はtolerated item内容を含むTurn全体のdigestのまま、既存の原本同一性照合（receipt全体の構造等価・本文の完全一致）には含め、意味上の分類・判断には使わない例外として維持、McpServerStatusListはcoreだけ、免責表を15行から9行へ（7行はtolerated領域へ）、thread/goal/updated refinement廃止。**A6と§7免責表を広げる**（E-05〜E-10、E-13）。 | オーナー承認済み（2026-10-16。非執筆側の設計レビューはSolが3巡で承認、固定HEAD 9ce78531）。 |
| C4 | 契約の更新手順（§8b）: tag exportから記録済み規則（schema位置だけを走査し値位置は原文保持、未知keywordで失敗）で生成、直前basisの再現確認（生成器変更時は、変更後の生成器が値位置controlを全件満たすことも。controlの検出力は固定mutantで別に1回確認し、正しい直前生成器がcontrolを通すことは更新を止めない）、出所記録、旧catalogを書換えず別file追加、型付き契約・テスト更新、非執筆側の設計レビューと通常の実装レビュー。オーナー承認はstrict位置を広げる変更と免責／tolerated集合を広げる変更に限る。A6の「契約見直しまでBLOCKED」を具体化する。 | オーナー承認済み（2026-10-16。非執筆側の設計レビューはSolが3巡で承認、固定HEAD 9ce78531）。 |

C1〜C4は§0aの差分だけを対象とし、A1〜A6・B1〜B4・B2aの他の内容を再判断しない。C1〜C4の承認前は、Codex Schema gate・Codex wire decoder・ConfigRead／McpServerStatusListの実装へ進まない。

R1、M4 runtime merge、public sandboxの作成・使用は決定済み。B1〜B4・B2aの再承認は§0の差分だけを対象とし、A1〜A6の他の内容を再判断しない。本稿（§15を含む）でB1〜B4の再承認と書く箇所はB2aを含む。執筆者は今後の実装者・レビュアーを承認しない。

### 14. pin 設計との差分一覧（orchestrator の転記）

| 項目 | pin 設計 | M0 |
| --- | --- | --- |
| 許可根拠 | 単一の厳密版数 | 実観測。番号は記録のみ |
| 版数検査の消費者 | phase 4、legacy 3 | すべて削除 |
| legacy 経路 | ピンの下で起動 | 旧起動 API を機能廃止として整理し、起動前に CapabilityUnavailable。共有安全性検証は現行の管理経路で維持 |
| 起動時 | 何もしない | 層 S（非モデル、拒否専用） |
| 準備の helper | Version＋Git×13 | Codex（Executor）は＋Schema＋Handshake、Claude（Reviewer）は admission 後に Probe（M3） |
| 登録前の観測 | `--version` | 握手、Codex の schema 構造、Claude の `system/init`／`result`（M3） |
| Codex Executor の approval／sandbox | CLI の継承設定 | 固定値の送信と応答一致、rrx の自動付与 0（§8a） |
| 実行ファイル | canonical パス | stat と sha256 の束縛 |
| 解析 | top `type` のみ有限、未知通知を無視 | 内部 variant、method、payload まで有限 |
| rate decoder | 未知を `None` | 型分離、解析不能は Probe でも本番でも fail closed |
| 宣言 | `cli_version` 必須 | 新 profile、installation で旧宣言を拒否 |
| DB | 版 11 | 版 12（S の journal） |

### 15. この稿の完了境界

A3 の方向はオーナーが旧 API の機能廃止として確定した。`c6413082` の修正差分の独立確認とM0設計全体承認は完了した。2026-10-15 の役割入れ替え改訂（§0、B1〜B4）は、非執筆側の独立確認とオーナー再承認が未完了であり、承認まで役割に依存するM0実装へ進まない。M0-U1は実装・限定検証・修正確認済みで、他の単位は未着手である。具体的な削除・残置・移植台帳、共有安全性ケースの代替検証と実装後レビューは未完了である。M0 で現に提供する振る舞い（Codex Executor の最初の Implement まで）、廃止する振る舞い、M3 で新たに提供する Claude Reviewer（Probe を含む）を区別して確認する。2026-10-16 の Codex 契約改訂（§0a、C1〜C4）は執筆段階（非執筆側レビュー round 1 の指摘を反映済み）であり、非執筆側の独立確認とオーナー承認が未完了である。0.162.1 basis の静的 catalog は verification file（`doc/verification/macos-minimum-m0-catalog-0162.json`、出所と再現確認の結果を含む）として追加済みである。縮約 fixture・Schema gate・Codex wire decoder・ConfigRead／McpServerStatusList の型は未実装、インストール版での live 確認は未実施である。
