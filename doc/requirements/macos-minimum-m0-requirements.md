# macOS 最小一本道 M0: 要件

状態: **役割入れ替え改訂（2026-10-15）はオーナー承認済み（固定HEAD `2a9114bc`、B1〜B4・B2a許可）** / `c6413082`の全体設計承認とA1〜A6は維持する。M0-U1（Claude strict wire decoder／metrics、O03、I01〜I04、共有queue予算補正）は実装・限定検証・独立修正確認済み（`doc/verification/macos-minimum-m0-u1.md`）で、役割入れ替え後はReviewer側のdecoderとして使う。それ以外のM0本番実装・検証は未着手。

オーナー全体設計承認（2026-10-13）: 固定HEAD `c64130828453cb38b742c9b31d08d9e488e85ee1` の3文書を承認し、必須検証を維持してM0実装へ進むとの明示回答を取得した。A1／A2／A4／A5／A6を含む設計承認であり、実装・実CLI適合・controls／mutants・両OS／実Mac受入・実装レビュー・マージ条件の免除ではない。

役割入れ替え改訂（オーナー決定 2026-10-15）: rururunxの役割をオーナーの日常開発に合わせ、**CodexをExecutor、ClaudeをReviewer**とする。`c6413082`の承認はClaude Executor＋Codex Reviewerを前提としていた。A1〜A6の他の決定、R1、M4のruntime自動merge、public `rrx-e2e-sandbox`、安全規則、M0→M5対応、M0実Mac受入前にM1へ進まない順序は変えない。再承認の対象は設計§0の差分一覧と§13のB1〜B4・B2aだけである。本稿は執筆成果であり、レビュー票・承認・実装・検証結果ではない。

Codex契約改訂（2026-10-16、**提案中・オーナー承認待ち**）: インストール済みCodex CLIが0.162.1へ上がり、版数は今後も上がる。Codex有限契約を位置のstrict／tolerated二分類へ改め、契約basisを0.162.1へ移し、strict位置に触れる更新の手順を定める（設計§0a、§13のC1〜C4）。本要件の該当箇所（目的と範囲のCodex段落、AC-05、AC-18、BLOCKED条件、未証明事項）はこの改訂案を反映した。承認前はこの改訂に依存するU4実装へ進まない。

公開sourceのbaseline: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。過去の修正入力: `d452675989df799d29931c7d4f19d73d6b387550`。前回の局所修正はclean固定HEAD `2c9fb68f6f6544ef9c8b3b53c7f936aec2e354e2`から設計草案へ取込済み。今回のR5後補修本文案の入力はclean固定HEAD `0fadf545c45558a05d89377bcb073009875bdb01`であり、設計草案へ取込済み・固定HEAD b746dfd4の補修diffはOpus承認済みである。baselineと各執筆入力HEADを混同しない。

R5結果の事実転記: SolはAPPROVE（確認済required 0）。固定argvでの要求は記録されているがserved model／effortは未報告。OpusはREQUEST CHANGES（確認済required 2: R5-O-01／R5-O-02）、runner ok／model_match=match、報告モデルは`claude-opus-5-5`のみ、effortは未報告。CLI完了と本文の判定は別であり、両レビュー承認済みとはしない。今回の補修はSolが執筆し、指摘側Opusが修正diffだけを独立確認する。無変更範囲のSol R5全文レビューを重複させず、両系の独立レビュー条件は維持する。適用状態はorchestratorが事実転記した。全体オーナー設計承認・実装・台帳・fixture・Linux／macOS／live検証は未完了。

R6局所補修の執筆・取込記録: 専用read-only Sol contextの14差分をorchestratorがsourceと照合して取込済み。要求モデルは`gpt-6.1-sol` / high、served model／effortは未報告。修正diffのOpus独立確認と全体オーナー承認は完了。[R5結果の公開記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142182746)、[R6開始記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142322693)。

R6独立確認結果の事実転記: 固定HEAD `b746dfd4c1c0de49fe07f173f40b2c236d2c97dd` の補修diffにOpusがAPPROVE（確認済み必須欠落0）。runner ok、model matchで`claude-opus-5-5`のみ報告、effort未報告。R5-O-01／R5-O-02は解消し、Sol R5の既承認範囲と今回の補修diff確認を合わせて扱えるとの判定。任意の検証補足は実装台帳で扱い、追加の必須設計条件やレビュー反復にはしない。レビュー後、#94のmain `b8344ab951fa24f545174917eee7be067284c625`へrebaseし、取込後HEAD `deb9909c661e98c12a7e7a8207c9eb6a80bad0d0`でも3文書のbyte不変を確認した。#94はマージ済み・同SHAの両OS CI成功であり、固定依頼文のOPEN記述は過去の状態。M0本番実装・M0検証は未完了。全体オーナー設計承認は下記の固定HEADで完了。[R6結果記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142873096)。

既存本文の修正執筆履歴: Codex CLI に `gpt-6.1-sol` / high を固定 argv で要求。CLI の実使用モデルと effort は未報告（一致は未確認）。独立レビュアーの context は執筆へ流用していない。これは独立レビュー・実装・実機検証の証拠ではない。

執筆履歴の事実転記: R3後の専用2 contextでは契約補正・欠落節復元をSol、owner rootをOpusが担当し、orchestratorが結合した。Opusはclaude-opus-5-5を報告、effortは未報告。今回のSol R5局所修正で共有metrics／rootとsource照合済みのOpus参考差分をまとめ、42差分を照合して取り込んだ。これは独立レビューやserved model／effortの一致認定ではない。

履歴転記: Sol／Opusのround 3修正差分レビューはともにREQUEST CHANGES。

Sol R4は正式なREQUEST CHANGES。Opus R4はCLI child exit 0でもmodelUsageにOpus＋Haikuがあり、runner exit 9／model_mismatchで必須レビュー未完了。参考本文は承認・票に使わず、orchestratorのsource照合で確認した差分だけを局所補修する。修正後はmodel一致のOpus独立確認と全体オーナー承認が必要である。[R4結果の公開記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6140066238)。

オーナー確定事項（2026-10-13）: 番号 pin 撤廃と capability probe、R1（decoder selfcheck＋実 event strict、未観測は未証明）、M4 の runtime 自動 merge、public [rrx-e2e-sandbox](https://github.com/shuhei-suzuki/rrx-e2e-sandbox) の作成・使用。再承認を求める事項ではない。sandbox は初期 README のみで、fixture・CI・main 保護は未設定。

**macOS最小一本道M0：要件修正全文**

今回のR5後補修本文案の入力HEADは `0fadf545c45558a05d89377bcb073009875bdb01`。前回の取込済み局所修正入力と公開sourceのbaselineは冒頭の別HEADである。本補修は設計草案へ取込済み・固定HEAD b746dfd4の補修diffはOpus承認済みである。状態は冒頭のとおりで、役割入れ替え改訂はオーナー承認済みである。本稿は設計修正の執筆成果であり、レビュー票、承認、実装または検証結果ではない。M0-U1以外の実装、実CLI資格付け、macOS／Linux検証、CI、owner-runはすべて未完了である。`c6413082`のオーナー設計承認は冒頭の固定HEADで完了した。

M0は#5/#6のSTRICT互換変更、M1は#95、M2は#12、M3は#9、M4は#13、M5は#16に対応する。M0の実Mac受入前にM1へ進まない。M4のruntime自動mergeとpublic `shuhei-suzuki/rrx-e2e-sandbox` の作成・使用は決定済みであり、再承認事項にしない。

### 目的と範囲

全CLI番号pinを撤廃し、起動時の能力検査Sと、準備ごとのfreshな検査Pへ置き換える。版数文字列は観測記録であり許可根拠にしない。auth、CAS、owner、marker、#19、process custodyの既存契約を維持する。

Sは設定された全Native aliasを対象に、rrxからモデル入力を送らず、全aliasの合格までTask admissionを停止する。PはSの合格で省略せず、Sのjournalから実行権限を復元しない。

ClaudeはReviewerとして扱う。Sで`initialize`の相関応答を、Reviewerの実argv（`--permission-prompt-tool stdio`と`--permission-mode plan`を含む）で確認する。Claude P（quota Admit後に`initialize`、`system/init`、stream-jsonの成功`result`を登録前に確認するcanary）はReviewer準備の検査として定義を維持し、本番化と受入はReviewer helper鎖を提供するM3で行う。rate eventは起動時decoder selfcheckを必須とし、実eventが届いた場合はstrict解析する。未着は未観測として記録する。

CodexはExecutorとして扱う。S/Pで生成schemaの構造、使用methodの宣言、`initialize`、`initialized`、ChatGPT account、environment ready、rate metadataの実応答を検査する。生成schemaは、rrxが送る値・判断に使う値・server request・効果や結果を運ぶitem・終端を表すstrict位置では固定契約との完全一致を要求し、rrxが送らず判断にも使わないtolerated位置では増減を許容する（設計§7・§8）。CLIの通常の更新がtolerated位置だけに触れる場合は契約を作り直さずに動き続け、strict位置に触れる場合は設計§8bの手順で契約を更新するまで起動前に拒否する。版数文字列はこの判定にも使わない。モデルcanaryは追加しない。PはExecutor準備の必須検査であり、M0で本番化する。Codex Executorの実行（app-serverのthread/turn、approval方針、sandbox、Worktree束縛、結果のcapture、Unknown／Held）はM0の範囲とし、設計§8aの固定契約に従う。

2run、#10の権限応答資格付け、#14の自動復旧、追加cleanup工程、Claude Executor設定の提供、Reviewer実行はM0へ広げない。Codexモデル実行はExecutorの最初のImplementまでをM0へ入れ、rrxが自動付与するCodex approvalは0件とする。Task.versionによる新しいauthority束縛、test専用authority constructor、SQL seedによる成功証明、`allow(dead_code)`、skip／ignore／直列化の追加、timeout延長だけの修正は禁止する。

### 受入条件

| ID | 条件 |
|---|---|
| AC-01 | phase／legacyの全番号pinと、そのqualification consumerを撤廃する。任意の合法な版数単一行でも、他の全検査が通ればNativeInputへ到達できる。 |
| AC-02 | 設定された全Native aliasのS合格公開前はTask admission 0。1 aliasでもpending／refused／unknownなら全体を止める。Sが作るTask／Unit／Session／NativeInputは0。 |
| AC-03 | SでClaude initialize、Codex schema・initialize・ChatGPT account・environment ready・rate metadataを確認する。SでClaude system-init／resultやCodexモデル経路を確認済みと記録しない。 |
| AC-04 | PはSから独立したfreshなSAME actor観測。Codex Executor Pは同じ準備actorのSchema／Handshakeを必須とし、M0の本番Executor準備で実施する。Claude Reviewer Pは、quota Admit後、Session・NativeInput・transport登録前のcanaryとして定義を維持する。その本番producer・実境界・受入はM3へ移し、M0ではClaude P合格を記録せず、producerも追加しない。aliasはproviderごとに登録し、Claude aliasはReviewer用に受理する。TaskのExecutorがClaude provider aliasに解決される場合は、設計§8a(0)の拒否点（Goal計画検証、admissionのDriver composition）で型付き拒否する（helper・Unit・Held・quota Admit 0）。 |
| AC-05 | envelope、method、payload、入れ子discriminatorを所有フィルタより前に検査する。未知variant・malformed・UUID内容改変を無視しない。正常ignored／foreign／合法toolのpositiveは設計§6・§7の有限受理構造内を母集団とする。既知frameの有界未宣言field保持、Codex closed object／判別keyの例外、Claude表外variant／blockの意図した拒否を設計§6・§7・A5に対応付け、追加field保持を未知variant受理へ広げない。Codexで全構造を型検査するのは設計§7のstrict位置であり、tolerated位置は有界strict JSONとして保持し判断に使わない。 |
| AC-06 | installationはdecoder selfcheck成功の証拠を必要とする。実rate eventの解析不能は終端エラー。未着、合法だがwindowなし、usableな観測を別々に記録する。 |
| AC-07 | rrxのモデル入力は既存quota Admit後だけ。canary観測をquota観測としてpool／window／backoffへ反映しない。既存lease・probe ownership・期限・CAS bookkeepingは維持する。 |
| AC-08 | **登録前拒否**ではSession、native invocation、native input/setup/permission、transport登録、IPC Grant、および当該helper以外の効果追加が0。S／installation拒否ではUnitも0。Pでは既存Preparing Unitとhelper行の存在を許容し、terminal dispositionを検査する。 |
| AC-09 | Native programのidentityを各Native helper／transport spawn直前に検査する。CLIをexecしないGit helperはそのCLI hash再計算の対象外とし、既存Git sealを維持する。 |
| AC-10 | intent先行、SAME actor／custody、epoch、CAS、Confirmed／Unknown settlementを維持する。probe合格はpermission・ownership・quota・Session authorityを発行しない。 |
| AC-11 | 旧起動 API は機能廃止として起動前に拒否する（オーナー方向決定 2026-10-13）。対象 test の移植／残置／機能廃止に伴う削除を ID・件数・理由・旧 consumer・現在の代替 coverage で記録する。共有する wire/quota/auth/permission/custody の安全性ケースは、現在提供する本番の管理経路で検証する。削除や拒否への置換だけで共有ケースを検証済みとしない。旧 Reviewer 等の実行機能を M0 が提供したとは扱わず、Reviewer 本番正常系は M3 の受入条件とする。 |
| AC-12 | 旧profile／`cli_version`ありの宣言をinstallationで型付き拒否し、新規Unit・Heldは0。既存Frameの旧宣言拒否は別契約であり、Preparing UnitがHeldになる場合を含む。 |
| AC-13 | owner-run実Mac、1 Project／1 Active Task／QUICK、Codex Executor＋Claude Reviewer設定の**1run**で、全aliasのS、Codex P、Worktree→**Codex Implement成功**（結果を既存retained Git経路でcapture）、SC-N停止を確認する。ClaudeはSと、Reviewer argvの配線検査（`--permission-mode plan`を含むこと）で扱い、Claude P・review実行は確認済みと記録しない。CLI番号は観測値として出力する。 |
| AC-14 | 失敗runは不合格として状態を保持し、同じ状態ディレクトリの再実行を拒否する。Known不合格とUnknownを混同せず、#14の復旧は実装しない。 |
| AC-15 | M0差分でskip／ignore／直列化を追加せず、禁止されたauthority代替や死んだproducerを導入しない。廃止台帳で残置／移植する既存synthetic childは、IDと実wrapperの対応を保存し、既存`#[ignore]`を必要な実childへ到達するwrapper入口としてpositive検証する。旧機能固有の理由付き削除を全旧positive維持と数えず、#19の既存機構・検査と共有安全性のcoverageを廃止で緩めない。 |
| AC-16 | 実producerを通る境界positive／+1 negative、因果controls／mutants、独立実装・安全性レビュー、ubuntu／macosの必須CIを通す。 |
| AC-17 | Codex Executorは設計§8aの固定approval方針・sandbox・Worktree束縛で起動し、ThreadStart応答がその値と一致しなければturn/startを送らない。turn/start前に、同じ本番Sessionの実効設定とinventoryでMCP・web search・dynamic tool・collabが無効または不在である肯定証拠（設計§8a(3a)）を得る。得られなければ型付き拒否でturn/start・NativeInput 0とし、継承設定の上書きで補わない（上書きの可否は設計§13のB2a）。rrxはapprovalを自動付与しない。turn/start送信から所有turnの終了まで、turn/start応答待ちを含む全受信経路でのserver request、所有turnの許可item集合（設計§8a(3a)）外のitem、書込み可能root外のpath、所有turnから別thread／turn／agentへ作業を広げるitem、未知variant、中断・不明なturn結果は成功にせずUnknown→Heldとし、自動再送しない。構造検査を通った正常なforeign通知・itemは設計§7の所有フィルタで除外して続行し、malformed foreignはProtocolErrorとする。成功はowned turnの`completed`で成功原本を捕捉した場合だけで、既存retained Git経路へ接続する。成功原本の捕捉後に生じたpipe EOF／leader reapの不明は物理custody軸に記録し、work軸を反転しない。spawnは`rrx-spawn-hygiene`の消費関数だけを使う。 |
| AC-18 | Codex Schema gateは設計§7・§8の二分類で判定する（提案中、C1〜C4）。strict位置（送信8 method＋initializedのparams全宣言、握手・ConfigRead判定path・inventory判定値・ThreadStart／TurnStart応答・owned thread／turnの同一性と状態・終端、21通知の全payload、全server request、効果や結果を運ぶitem）では、property（optional／nullableを含む）・required・type・enum・枝・制約・defaultの追加／削除／変更をすべて起動前に拒否する。新しいserver request method、used enumの新値、strict位置の削除も拒否する。tolerated位置の増減、規則内の新しい通知method、新しいThreadItem typeは通過させ、新しいThreadItem typeは所有turnで成功にせずUnknown→Heldとする。tolerated位置の値は分類・回答・quota・authority・記録に使わない。gate結果は有限のPass／Refusedと有限reason・件数・記録可能な位置だけをjournalへ残し、生schemaを保存しない。basis catalogは0.162.1のtag exportから記録済み規則で作り、直前basisの再現確認と出所記録を持ち、旧catalogを書換えない。CLI更新がtolerated位置だけに触れる場合に契約を作り直さず通ること、strict位置に触れる場合にkind別reasonで拒否されることをcontrol（設計CX-7／CX-8）で示す。 |

本番Sessionでのstrict拒否はAC-08の累積0を要求しない。既存状態をbaselineとして、拒否後の新しい入力・権限応答・Grant・quota誤更新がないこと、および既存terminal契約に従うdispositionを検査する。

### 承認・資格付けの順序

修正差分の独立確認とオーナー設計承認をM0コード着手前に完了する（`c6413082`で完了）。役割入れ替え改訂も、非執筆側の独立確認とオーナー再承認（設計§13のB1〜B4・B2a）を、役割に依存する残りのM0コード着手前に完了する。Codex契約改訂（設計§13のC1〜C4）も、非執筆側の独立確認とオーナー承認を、Codex Schema gate・Codex wire decoder・ConfigRead／McpServerStatusListの実装着手前に完了する。以後のCLI更新で契約を更新する場合は設計§8bに従い、オーナー承認はstrict位置または免責／tolerated集合を広げる更新に限る。M0-U1の実装済み差分はReviewer側のdecoderとして保持する。実装後にowner-run実CLI資格付けを行い、M0実Mac合格後に次段階へ進む。設計承認前のlive採取を新しい必須条件にはしない。

Sの保証は「rrxがモデル入力を送らないこと」である。CLIや既存hookによる内部backend呼出しの不存在を観測済みとは保証しない。設定・認証・hookをスキップしてSを成立させない。

### BLOCKED条件と未証明事項

以下では合格を出さず設計へ戻る。

- 役割入れ替え差分（設計§13のB1〜B4・B2a）のオーナー再承認が得られない。
- Codex Executorの固定approval方針・sandbox値が固定catalogに宣言されていない、またはThreadStart応答で一致を確認できない。
- Codex Executorの本番Sessionで、継承設定のままMCP・web search・dynamic tool・collabの無効／不在の肯定証拠をturn/start前に得られない（制限方向の上書きを許すかは設計§13のB2a。判断までその環境のExecutor準備をBLOCKEDとし、上書きで補わない）。
- Sの実argv／空cwd／停止方式が成立しない、またはrrxがモデル入力を送らなければ成立しない。
- Codex契約改訂（設計§13のC1〜C4）のオーナー承認が得られない（承認済みA6の規則のままでは、0.162.1のConfigRead／McpServerStatusListと通常のCLI更新を扱えない）。
- 実frameが有限契約の表外、または構造不適合。Codex生成schemaがstrict位置で不一致、または許容追加の規則外（設計§8bの更新手順を通すまで）。
- 現在提供する管理経路の共有安全性ケースの検出範囲を維持できない。旧機能の廃止そのものと未検証の安全性を混同しない。
- 必須検証・レビュー・両OSの結果が欠ける。

未証明事項は、Codexのapproval往復・error分類・sandbox実効（OSでの強制）・network遮断の実効、インストール版0.162.1の`generate-json-schema --experimental`出力が0.162.1契約を通ること（tag exportとの一致はowner-runのgate結果で確認する）、ThreadStart応答のsandbox（writableRoots）が送信方針と照合できる形で返ること、tolerated通知の実際の意味、owner-runの1run以外でのCodex Executorの挙動、ClaudeのReviewer P（canary／system-init／result）・review実行・権限往復・既定permission mode・rate event、escaped descendants、wrapper内部identityである。owner-runの1run成功をCodex Executor経路全般の資格付けへ広げない。静的schema適合やsynthetic positiveでlive成功を証明したとは扱わない。
