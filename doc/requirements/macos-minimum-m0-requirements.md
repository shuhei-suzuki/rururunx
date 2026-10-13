# macOS 最小一本道 M0: 要件

状態: **DRAFT / A3 は旧 API の機能廃止として整理する方針に確定 / 修正差分の独立確認・オーナー設計承認前**。本番コード・テストは変更していない。

公開sourceのbaseline: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。過去の修正入力: `d452675989df799d29931c7d4f19d73d6b387550`。前回の局所修正はclean固定HEAD `2c9fb68f6f6544ef9c8b3b53c7f936aec2e354e2`から設計草案へ取込済み。今回のR5後補修本文案の入力はclean固定HEAD `0fadf545c45558a05d89377bcb073009875bdb01`であり、設計草案へ取込済み・修正diffの独立確認前である。baselineと各執筆入力HEADを混同しない。

R5結果の事実転記: SolはAPPROVE（確認済required 0）。固定argvでの要求は記録されているがserved model／effortは未報告。OpusはREQUEST CHANGES（確認済required 2: R5-O-01／R5-O-02）、runner ok／model_match=match、報告モデルは`claude-opus-5-5`のみ、effortは未報告。CLI完了と本文の判定は別であり、両レビュー承認済みとはしない。今回の補修はSolが執筆し、指摘側Opusが修正diffだけを独立確認する。無変更範囲のSol R5全文レビューを重複させず、両系の独立レビュー条件は維持する。適用状態はorchestratorが事実転記した。全体オーナー設計承認・実装・台帳・fixture・Linux／macOS／live検証は未完了。

R6局所補修の執筆・取込記録: 専用read-only Sol contextの14差分をorchestratorがsourceと照合して取込済み。要求モデルは`gpt-6.1-sol` / high、served model／effortは未報告。修正diffのOpus独立確認・全体オーナー承認は未完了。[R5結果の公開記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142182746)、[R6開始記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6142322693)。

既存本文の修正執筆履歴: Codex CLI に `gpt-6.1-sol` / high を固定 argv で要求。CLI の実使用モデルと effort は未報告（一致は未確認）。独立レビュアーの context は執筆へ流用していない。これは独立レビュー・実装・実機検証の証拠ではない。

執筆履歴の事実転記: R3後の専用2 contextでは契約補正・欠落節復元をSol、owner rootをOpusが担当し、orchestratorが結合した。Opusはclaude-opus-5-5を報告、effortは未報告。今回のSol R5局所修正で共有metrics／rootとsource照合済みのOpus参考差分をまとめ、42差分を照合して取り込んだ。これは独立レビューやserved model／effortの一致認定ではない。

履歴転記: Sol／Opusのround 3修正差分レビューはともにREQUEST CHANGES。

Sol R4は正式なREQUEST CHANGES。Opus R4はCLI child exit 0でもmodelUsageにOpus＋Haikuがあり、runner exit 9／model_mismatchで必須レビュー未完了。参考本文は承認・票に使わず、orchestratorのsource照合で確認した差分だけを局所補修する。修正後はmodel一致のOpus独立確認と全体オーナー承認が必要である。[R4結果の公開記録](https://github.com/shuhei-suzuki/rururunx/issues/16#issuecomment-6140066238)。

オーナー確定事項（2026-10-13）: 番号 pin 撤廃と capability probe、R1（decoder selfcheck＋実 event strict、未観測は未証明）、M4 の runtime 自動 merge、public [rrx-e2e-sandbox](https://github.com/shuhei-suzuki/rrx-e2e-sandbox) の作成・使用。再承認を求める事項ではない。sandbox は初期 README のみで、fixture・CI・main 保護は未設定。

**macOS最小一本道M0：要件修正全文**

今回のR5後補修本文案の入力HEADは `0fadf545c45558a05d89377bcb073009875bdb01`。前回の取込済み局所修正入力と公開sourceのbaselineは冒頭の別HEADである。本補修は設計草案へ取込済み・修正diffの独立確認前である。状態は **DRAFT／修正差分の独立確認・オーナー設計承認前**。本稿は設計修正の執筆成果であり、レビュー票、承認、実装または検証結果ではない。実装、実CLI資格付け、macOS／Linux検証、CI、owner-run、オーナー設計承認はすべて未完了である。

M0は#5/#6のSTRICT互換変更、M1は#95、M2は#12、M3は#9、M4は#13、M5は#16に対応する。M0の実Mac受入前にM1へ進まない。M4のruntime自動mergeとpublic `shuhei-suzuki/rrx-e2e-sandbox` の作成・使用は決定済みであり、再承認事項にしない。

### 目的と範囲

全CLI番号pinを撤廃し、起動時の能力検査Sと、準備ごとのfreshな検査Pへ置き換える。版数文字列は観測記録であり許可根拠にしない。auth、CAS、owner、marker、#19、process custodyの既存契約を維持する。

Sは設定された全Native aliasを対象に、rrxからモデル入力を送らず、全aliasの合格までTask admissionを停止する。PはSの合格で省略せず、Sのjournalから実行権限を復元しない。

ClaudeはSで`initialize`の相関応答、Pでquota Admit後に`initialize`、`system/init`、stream-jsonの成功`result`を登録前に確認する。`--permission-prompt-tool stdio`を実際のargvに含める。rate eventは起動時decoder selfcheckを必須とし、実eventが届いた場合はstrict解析する。未着は未観測として記録する。

CodexはS/Pで生成schemaの構造、使用methodの宣言、`initialize`、`initialized`、ChatGPT account、environment ready、rate metadataの実応答を検査する。モデルcanaryは追加しない。

Codexモデル実行、2run、#10の権限応答資格付け、#14の自動復旧、追加cleanup工程はM0へ広げない。Task.versionによる新しいauthority束縛、test専用authority constructor、SQL seedによる成功証明、`allow(dead_code)`、skip／ignore／直列化の追加、timeout延長だけの修正は禁止する。

### 受入条件

| ID | 条件 |
|---|---|
| AC-01 | phase／legacyの全番号pinと、そのqualification consumerを撤廃する。任意の合法な版数単一行でも、他の全検査が通ればNativeInputへ到達できる。 |
| AC-02 | 設定された全Native aliasのS合格公開前はTask admission 0。1 aliasでもpending／refused／unknownなら全体を止める。Sが作るTask／Unit／Session／NativeInputは0。 |
| AC-03 | SでClaude initialize、Codex schema・initialize・ChatGPT account・environment ready・rate metadataを確認する。SでClaude system-init／resultやCodexモデル経路を確認済みと記録しない。 |
| AC-04 | PはSから独立したfreshなSAME actor観測。Claude canaryはquota Admit後、Session・NativeInput・transport登録前。Codexは同じ準備actorのSchema／Handshakeを必須とする。 |
| AC-05 | envelope、method、payload、入れ子discriminatorを所有フィルタより前に検査する。未知variant・malformed・UUID内容改変を無視しない。正常ignored／foreign／合法toolのpositiveは設計§6・§7の有限受理構造内を母集団とする。既知frameの有界未宣言field保持、Codex closed object／判別keyの例外、Claude表外variant／blockの意図した拒否を設計§6・§7・A5に対応付け、追加field保持を未知variant受理へ広げない。 |
| AC-06 | installationはdecoder selfcheck成功の証拠を必要とする。実rate eventの解析不能は終端エラー。未着、合法だがwindowなし、usableな観測を別々に記録する。 |
| AC-07 | rrxのモデル入力は既存quota Admit後だけ。canary観測をquota観測としてpool／window／backoffへ反映しない。既存lease・probe ownership・期限・CAS bookkeepingは維持する。 |
| AC-08 | **登録前拒否**ではSession、native invocation、native input/setup/permission、transport登録、IPC Grant、および当該helper以外の効果追加が0。S／installation拒否ではUnitも0。Pでは既存Preparing Unitとhelper行の存在を許容し、terminal dispositionを検査する。 |
| AC-09 | Native programのidentityを各Native helper／transport spawn直前に検査する。CLIをexecしないGit helperはそのCLI hash再計算の対象外とし、既存Git sealを維持する。 |
| AC-10 | intent先行、SAME actor／custody、epoch、CAS、Confirmed／Unknown settlementを維持する。probe合格はpermission・ownership・quota・Session authorityを発行しない。 |
| AC-11 | 旧起動 API は機能廃止として起動前に拒否する（オーナー方向決定 2026-10-13）。対象 test の移植／残置／機能廃止に伴う削除を ID・件数・理由・旧 consumer・現在の代替 coverage で記録する。共有する wire/quota/auth/permission/custody の安全性ケースは、現在提供する本番の管理経路で検証する。削除や拒否への置換だけで共有ケースを検証済みとしない。旧 Reviewer 等の実行機能を M0 が提供したとは扱わず、Reviewer 本番正常系は M3 の受入条件とする。 |
| AC-12 | 旧profile／`cli_version`ありの宣言をinstallationで型付き拒否し、新規Unit・Heldは0。既存Frameの旧宣言拒否は別契約であり、Preparing UnitがHeldになる場合を含む。 |
| AC-13 | owner-run実Mac、1 Project／1 Active Task／QUICK、Claude Executor＋Codex Reviewer設定の**1run**で、全aliasのS、Claude P、Worktree→Implement成功、SC-N停止を確認する。CLI番号は観測値として出力する。 |
| AC-14 | 失敗runは不合格として状態を保持し、同じ状態ディレクトリの再実行を拒否する。Known不合格とUnknownを混同せず、#14の復旧は実装しない。 |
| AC-15 | M0差分でskip／ignore／直列化を追加せず、禁止されたauthority代替や死んだproducerを導入しない。廃止台帳で残置／移植する既存synthetic childは、IDと実wrapperの対応を保存し、既存`#[ignore]`を必要な実childへ到達するwrapper入口としてpositive検証する。旧機能固有の理由付き削除を全旧positive維持と数えず、#19の既存機構・検査と共有安全性のcoverageを廃止で緩めない。 |
| AC-16 | 実producerを通る境界positive／+1 negative、因果controls／mutants、独立実装・安全性レビュー、ubuntu／macosの必須CIを通す。 |

本番Sessionでのstrict拒否はAC-08の累積0を要求しない。既存状態をbaselineとして、拒否後の新しい入力・権限応答・Grant・quota誤更新がないこと、および既存terminal契約に従うdispositionを検査する。

### 承認・資格付けの順序

修正差分の独立確認とオーナー設計承認をM0コード着手前に完了する。実装後にowner-run実CLI資格付けを行い、M0実Mac合格後に次段階へ進む。設計承認前のlive採取を新しい必須条件にはしない。

Sの保証は「rrxがモデル入力を送らないこと」である。CLIや既存hookによる内部backend呼出しの不存在を観測済みとは保証しない。設定・認証・hookをスキップしてSを成立させない。

### BLOCKED条件と未証明事項

以下では合格を出さず設計へ戻る。

- Claude canaryを含む一括設計承認が得られない。
- Sの実argv／空cwd／停止方式が成立しない、またはrrxがモデル入力を送らなければ成立しない。
- 実frame／schemaが有限契約の表外、または構造不適合。
- 現在提供する管理経路の共有安全性ケースの検出範囲を維持できない。旧機能の廃止そのものと未検証の安全性を混同しない。
- 必須検証・レビュー・両OSの結果が欠ける。

未証明事項はCodexモデル経路・review実行・承認・error分類・sandbox実効、Claudeの権限往復・既定permission mode・未着runのrate event、escaped descendants、wrapper内部identityである。静的schema適合やsynthetic positiveでlive成功を証明したとは扱わない。
