# macOS 最小一本道 M0: capability probe 設計案

状態: **DRAFT / A3 は旧 API の機能廃止として整理する方針に確定 / 両系の独立確認・オーナー設計承認済み、実装待ち**。本番コード・テストは変更していない。

オーナー全体設計承認（2026-10-13）: 固定HEAD `c64130828453cb38b742c9b31d08d9e488e85ee1` の3文書を承認し、必須検証を維持してM0実装へ進むとの明示回答を取得した。A1／A2／A4／A5／A6を含む設計承認であり、実装・実CLI適合・controls／mutants・両OS／実Mac受入・実装レビュー・マージ条件の免除ではない。

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

**provider別S手順**

Sはdaemon継承環境と専用の空cwdを使い、既存の設定・認証・hookを変更・省略しない。設定上のmodel／effortを記録するが、projectによる上書き、Unit環境overlay、実role別argvのP観測をSで済ませたことにしない。各stepは専用custody、intent先行、program identity再確認、§10の期限・捕捉・停止契約を使う。

| provider | stepとargv／送信 | 合否・停止点 |
|---|---|---|
| Codex | Versionは`--version`。Schemaは`app-server generate-json-schema --experimental --out <new-dir>`。Handshakeは`command_argv(codex, Executor, model, effort, None)`の`app-server --listen stdio://`。 | Schema gate後、initialize→paramsなしinitialized→account/read（refreshToken=false）→environment/status（local ID）→account/rateLimits/read。chatgpt／ready／strict-valid rateを確認し、thread/start／turn/startを送らず停止。 |
| Claude | Versionは`--version`。Handshake stepにinitialize観測を記録。`command_argv(claude, Executor, 設定model, 設定effort, fresh UUID)`の`-p --input-format stream-json --output-format stream-json --verbose --session-id <UUID> --permission-prompt-tool stdio --settings {"forceLoginMethod":"claudeai"}`に、存在するmodel／effortを既存の有界argv検査で付加する。 | 専用request IDのinitialize（既存hooks=null）だけ送信し、相関したsuccess control_responseを検査して停止。user／canary／権限応答は0。Sでsystem-init／resultを必須にも観測済みにもしない。 |

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
| Codex Executor／Reviewer | 232 | 233 | 235（Schema＋Handshake後） | 248 | 248 |
| Claude Executor／Reviewer | 237 | 238 | 238 | 251 | 252 |

Codexは`232+Version1+Schema1+Handshake1+Git13=248`。Claudeは`237+Version1+Git13+Probe1=252`。Claude Probe末尾はPrepared上限に含め、prefixだけを`check_prepared`へ渡さない。

最初の入力までの後続reserveはCodexがtransport1＋setup6＋input1、Claudeがtransport1＋setup1＋input1。したがってCodexは256、Claudeは255。terminalは既存input effectのsettlement／result publicationであり、新しいmanaged effect行を追加する計算にはしない。terminal consumerが追加行を必要とする変更はこの契約を再設計する。

同じbudget定数をVersion intent、metadata helper、Git reserve、Probe intent、Prepared、registration、setup/input consumerへ渡す。byte reserveは追加helperの最大imageも含む。

境界positiveとB+1 negativeは実installed producerが生成する履歴で検査する。`vec![row; N]`やSQL seedだけでは予算契約の正例にならない。M0 は現に提供する Executor の実 producer で境界を検証する。Reviewer の同一予算政策は純粋計算として照合するが、実 Reviewer helper 鎖の境界合格は M3 で検証する。純粋計算を実 Reviewer 起動の成功証明としない。

### 6. Claude Probeとstrict wire

Probeは本番`command_argv`を使い、専用session UUID、同じmodel／effort／overlay／worktreeを用いる。Executorにはplan modeを追加する。既存設定・認証・hookを変更・スキップしない。

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

全rowのtypeは表のtop type、system rowのsubtypeは表の名前を要求する。共通optional uuidはUUID、session_idはpresentなら有界string（init／assistant／result／rateでは必須）。optional originはnullまたはobjectで、非nullはkindを要求する。assistant.errorの12値は`authentication_failed/oauth_org_not_allowed/account_on_hold/billing_error/rate_limit/overloaded/invalid_request/model_not_found/server_error/max_output_tokens/cloud_credential_error/unknown`。現consumerがhint分類に使う4値とこのwire集合を混同しない。api_error_statusの公開numberに対するuint16 refinement、usageの非負性、配列上限はrrx契約でありlive適合は未証明。欠落とnullはdecoder上で別状態を保持する。

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

optional fieldもpresentなら型検査する。特にthreadName、projectId、turnId、error、reason、details、targetItemId、environmentIdなどのnullableとoptionalを混同しない。以下の所有列を上のpayload表へmethodで自然結合し、全83methodがちょうど1区分に属することを検査する。列挙中のmethodはそれぞれ独立methodであり、prefixによる未知methodの受理ではない。

| method集合（件数） | 所有区分／キーと型 | S/P握手中 |
|---|---|---|
| error、thread/tokenUsage/updated、turn/diff/updated、turn/plan/updated、item/started、item/autoApprovalReview/started、item/autoApprovalReview/completed、autoApprovalReview/strictReviewRequired、item/completed、item/agentMessage/delta、item/plan/delta、item/commandExecution/outputDelta、item/commandExecution/terminalInteraction、item/fileChange/outputDelta、item/fileChange/patchUpdated、item/mcpToolCall/progress、item/reasoning/summaryTextDelta、item/reasoning/summaryPartAdded、item/reasoning/textDelta、thread/compacted、model/rerouted、model/verification、modelProvider/authRecoveryStarted、modelProvider/authRecoveryCompleted、turn/moderationMetadata、model/safetyBuffering/updated（26） | Thread＋Turn。params.threadId:string、params.turnId:stringとも必須。 | strict検査後に拒否 |
| thread/status/changed、thread/archived、thread/deleted、thread/unarchived、thread/closed、thread/reverted、thread/name/updated、thread/attachment/updated、thread/goal/cleared、thread/queue/changed、thread/project/updated、thread/environment/connected、thread/environment/disconnected、thread/settings/updated、serverRequest/resolved、guardianWarning、thread/realtime/started、thread/realtime/itemAdded、thread/realtime/item/started、thread/realtime/item/transcript/delta、thread/realtime/item/completed、thread/realtime/transcript/delta、thread/realtime/transcript/done、thread/realtime/outputAudio/delta、thread/realtime/sdp、thread/realtime/error、thread/realtime/closed（27） | Thread。params.threadId:string必須。 | strict検査後に拒否 |
| thread/goal/updated、hook/started、hook/completed（3） | Thread＋任意Turn。params.threadId:string必須、params.turnIdはoptional string/null。非nullだけ追加の所有比較。 | turnId欠落／nullでも拒否 |
| thread/started（1） | Nested Thread。params.thread.id:string必須。Thread全体も検査。top-level threadId不在を全体通知としない。 | 拒否 |
| turn/started、turn/completed（2） | Thread＋Nested Turn。params.threadId:string、params.turn.id:string必須。Turn全体も検査。 | 拒否 |
| mcpServer/oauthLogin/completed、mcpServer/startupStatus/updated、warning（3） | Optional Thread。params.threadIdはoptional string/null。欠落／nullなら全体、非nullならThread。 | 欠落／nullなら正常全体通知として続行、非nullなら拒否 |
| skills/changed、project/changed、command/exec/outputDelta、process/outputDelta、process/exited、mcpServer/event/stream/notification、account/updated、account/gatewayOAuth/changed、account/rateLimits/updated、app/list/updated、remoteControl/status/changed、externalAgentConfig/import/progress、externalAgentConfig/import/completed、fs/changed、deprecationNotice、configWarning、fuzzyFileSearch/sessionUpdated、fuzzyFileSearch/sessionCompleted、windows/worldWritableWarning、windowsSandbox/setupCompleted、account/login/completed（21） | Global。Thread／Turn所有キーなし。processHandle、processId、watchId、subscriptionId、sessionId等をNative Thread／Turn authorityへ変換しない。 | strict-validなら件数に算入して続行。rateは§9も適用 |

本番では存在する所有キーをすべて比較し、foreignはpayload全体の検査後にだけ除外する。任意Turnが欠落／nullでもThread比較は省略しない。Global通知はwire合法性だけで入力・回答・権限authorityを発行しない。 `thread/goal/updated`はparams.threadIdと必須goal.threadIdの一致を所有filter前に検査し、両方を同じThread所有へ束縛する。この等値制約はsource catalogの制約ではなくrrx refinementであり、全SDK対応の追加scopeにはしない。不一致はforeignとして捨てずProtocolError。S/P握手ではaccount/updated.authModeがpresentでchatgpt以外（null含む）、またはaccount/login/completed.success=falseなら相反観測として拒否し、後の正常account/readで打ち消して資格付けない。authMode欠落は肯定証拠にせず、成功通知自体もchatgpt資格付けの代替にしない。

補助型は、固定HEADの公開static catalogでsource schemaごとの`local_definition_digests`から`shared_definitions`へ解決したdefinitionを開発時根拠にする。rrx予定型名とsource definition名は同名とは限らない。

| source definition | rrx予定型 |
|---|---|
| GuardianApprovalReviewAction | GuardianAction |
| GuardianApprovalReview | GuardianReview |
| HookRunSummary | HookRun |
| ThreadRealtimeItem | RealtimeItem |
| ExternalAgentConfigImportTypeResult | ImportResult |
| FuzzyFileSearchResult | SearchResult |
| TurnPlanStep | PlanStep |
| FunctionCallOutputBody | FunctionOutput |
| RateLimitSnapshot | RateSnapshot |
| FileSystemSpecialPath | SpecialPath |

その他の同名型もsource schema名とdefinition名を明記する。転記範囲は上表、11 server request、送信6 method、6 responseから到達する**宣言済みproperty／required／enum／型／数値域の全閉包**であり、optional／nullable、配列要素、map値も含めてrequiredだけの投影に縮めない。出所definition→rrx型→decoder→consumer対応表を実装成果物に含める。static catalogは開発時の観測資料であり、runtime validatorの仕様生成器ではない。全JSON dump、runtime生成schemaからの型生成、schemaをvalidatorとして実行する方式でこの固定rrx契約を代替しない。

内部discriminatorは少なくとも次を有限化する。

- ThreadStatus：notLoaded／idle／systemError／active。activeはactiveFlags配列を要求。
- GuardianActionは`GuardianApprovalReviewAction`の7枝。全枝でtypeを必須とし、枝別構造は次表に固定する。server request methodの`applyPatchApproval`を内部action tagとして受理しない。

| type | 追加必須field | optional field |
|---|---|---|
| command | command:string、cwd:LegacyAppPathString、source=`shell/unifiedExec` | なし |
| execve | argv:string[]、cwd:AbsolutePathBuf、program:string、source=`shell/unifiedExec` | なし |
| writeStdin | approvalId:string、cwd:LegacyAppPathString、processId:string、stdin:string | なし |
| applyPatch | cwd:LegacyAppPathString、files:LegacyAppPathString[] | なし |
| networkAccess | host:string、port:uint16、protocol=`http/https/socks5Tcp/socks5Udp`、target:string | なし |
| mcpToolCall | server:string、toolName:string | connectorId／connectorName／toolTitle:string/null |
| requestPermissions | permissions:RequestPermissionProfile | reason:string/null |

LegacyAppPathStringとAbsolutePathBufはsourceのstring形を保ち、後者には既存absolute-path業務条件を適用する。RequestPermissionProfileはtyped閉包を検査し、任意JSONへ落とさない。
- GuardianReview status：inProgress／approved／denied／timedOut／aborted。
- HookRun：eventName、executionMode、handlerType、scope、source、status、entries.kindの各有限enum。
- FileSystemPath／SpecialPath、AskForApproval、SandboxPolicy、SessionSource／SubAgentSource：各oneOfのtagと枝別必須構造。
- RealtimeItem：共通id／realtimeSessionIdに加え、realtimeSessionStarted／transcriptSegment／bemItemPromoted／realtimeSessionClosedの枝別構造。presentationのwholeItem／inlineMarkdown／inlineVisualizationも検査。
- ImportResult、AppInfo、SearchResult、ThreadGoal、ThreadSettings、token usage、permission profile：配列要素・map値・nullable参照を再帰検査し、objectだけでは通さない。

`OpaqueJson`の免責集合は次表で閉じる。位置はsource definitionのproperty pathであり、枝をまたいで拡大しない。任意JSON自身にはnullを含められるが、外側のtyped容器の型・required／optional／nullableは維持する。

| source位置 | 外側の契約 |
|---|---|
| McpServerEventNotification.params | 必須任意JSON |
| McpToolCallResult._meta、structuredContent | 各optional任意JSON |
| McpToolCallResult.content[] | contentは必須array、各要素だけ任意JSON |
| ThreadItem[type=mcpToolCall].arguments、ThreadItem[type=dynamicToolCall].arguments | 各枝で必須任意JSON |
| ThreadItem[type=webSearch].results[] | resultsはoptional array/null、非null arrayの各要素だけ任意JSON |
| ThreadRealtimeItemAddedNotification.item | 必須任意JSON。ThreadRealtimeItemのtyped枝とは別位置 |
| TurnModerationMetadataNotification.metadata | 必須任意JSON |
| DynamicToolCallParams.arguments | 必須任意JSON |
| McpServerElicitationRequestParams[mode=openai/userVerification・form・openai/form・openaiForm・url]._meta | 5枝それぞれoptional任意JSON |
| McpServerElicitationRequestParams[mode=openai/form・openaiForm].requestedSchema | この2枝だけ必須任意JSON |
| DynamicToolNamespaceTool[type=function].inputSchema、DynamicToolSpec[type=function].inputSchema | それぞれoneOf/0の必須任意JSON |
| InitializeCapabilities.extensionsのmap値 | extensionsはoptional object/null、値だけ任意JSON |
| ThreadStartParams.configのmap値 | configはoptional object/null、値だけ任意JSON |
| TurnStartParams.outputSchema | optional任意JSON（sourceは型なし`{}`） |
| GetAccountRateLimitsResponse.rateLimitUpsell | optional任意JSON（sourceは型なし`{}`）。quota／可用性判断に使用しない |

McpServerElicitationRequestParamsのmode=formではrequestedSchemaは必須`McpElicitationSchema`であり、型付きproperties／required／primitive schema閉包を検査する。mode=openai/userVerification／urlへrequestedSchemaの免責を追加しない。共通必須serverName／threadIdとoptional nullable turnIdも検査する。

選択した10source schemaと到達definition閉包に明示された`true`／型なしschema位置について、この表との双方向集合一致を検査する。typed枝を免責へ広げず、欠落免責を暗黙追加しない。未知method・item.type・status・discriminatorをopaqueへ落とさない。opaqueもstrict JSONと既存byte／depth／nodes／文字列／容器上限の対象であり、構造外のauthority・業務成功証拠にしない。 Thread.extraのThreadExtraはsourceの`type:object`を保つ有界非使用mapであり、任意JSON全体のOpaqueJsonではない。外側Thread.extraのrequired／optional／nullableはcatalogに従い、非null値はobject、map値だけ有界strict JSONとして保持し業務非使用とする。この位置を明示typed mapとして照合し、unknown-schema exemptionを別位置へ拡大しない。

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

本番処理順はstrict envelope→method→payload全構造→所有フィルタ→consumer。正常toolは構造検査後に回答から除外できる。malformed foreign frameはProtocolError。S/P握手では全server requestをparams検査後に拒否し、通知は所有列の握手扱いを使う。正常なGlobal通知と、Optional ThreadでthreadId欠落／nullの通知だけを数えて続行する。

既知Codex wire frame内の未宣言fieldは、Claudeと同じく有界strict JSONとして保持受理し、業務・authorityには使わない。既知fieldの不正型、未知method／tag／enum、branch不適合は拒否する。このwire追加field規則と、Schema gateで未知の宣言property／method／discriminatorを拒否する規則は別である。schemaのadditionalProperties未指定を、未知methodやtyped field不正を受理する根拠にしない。明示typed mapとOpaque位置以外に、新しい宣言propertyを自動追加しない。 例外としてsourceでadditionalProperties=falseのclosed objectは宣言key以外を拒否する。固定catalogのshared definitions全体では27位置であり、数から契約を生成せず、対象source rootのref解決後閉包にある各位置のproperties／required／additionalPropertiesを保持する。externally-tagged／key-tag unionはtag keyの存在もdiscriminatorであり、既知tag keyはちょうど1つ、0または複数は拒否する。CodexErrorInfoの5 object枝（4 HTTP＋activeTurnNotSteerable）にも適用する。

JSON-RPC envelopeの判別keyはid／method／params／result／errorであり未宣言追加field扱いしない。受信94 methods（server request 11／通知83）は、requestならid＋method＋params、notificationならmethod＋paramsかつidなしを要求する。success responseはid＋resultかつmethod／params／errorなし、error responseはid＋errorかつmethod／params／resultなしを要求する。送信requestのparams presenceは各source branchのrequiredに従い、`account/rateLimits/read`はid／method required・params optional nullableで、現rrxのnative.rs:2158は`{}`を送る。送信ClientNotificationの`initialized`はparamsなしであり、optional nullableなrequest paramsとは別契約である。送信使用6 methodsの既存wire検査とCX-3／CX-4を緩めない。idは有界stringまたはsource整数域でnull不可、methodは有限method。result＋error、method＋result/error、id:null、initialized＋paramsを拒否し、result:null等のpayload nullabilityは選択済response型で別検査する。判別key以外の追加fieldはclosed objectでない既知位置だけ既存のbounded保持規則を適用する。

server requestは既知11種すべてのparamsを先に検査する。本番で対応する承認2種だけ既存ledgerへ渡し、残り9種は既存の非対応拒否を維持する。M0で権限応答能力を増やさない。

### 8. Schema gateと実握手

Schema helperは`app-server generate-json-schema --experimental --out <new-dir>`。生hash一致を要求しない。

4 envelopeに加え、Initialize／Account／Rate／Environment／ThreadStart／TurnStart responseの生成ファイルを読み、§7のfinite contractと比較する。file名はfixture manifestで固定し、任意探索で同名definitionを拾わない。

生成schema filesのsinkは、自uidの新規0700 directory、symlink禁止、1024項目、総16 MiB、directory深さ4、各読取file1 MiBまで。JSONは深さ64、262144 nodes、文字列64 KiB、object／array4096。16 MiBは生成file総量の上限であり、process stdout＋stderrの64 KiB捕捉や準備aggregate8 MiBとは別sinkである。既存上限を増やさず、生成fileをprocess捕捉枠に混算しない。

local refは同一fileの`#/definitions/`のみ。512は各source fileで解決・memo化する異なるdefinition target数の上限であり、再参照回数や非メモ化展開node数ではない。keyは検査済みfile identity＋definition名、memoは最大512 entry、現在の解決stack／ref深さは16までとする。stack内target再訪を循環として拒否し、completed memoへの再参照は512を再消費しない。比較側も既処理のschema-node／rrx-type対をmemo化し、全再参照・分岐の処理を含むwork-node数を各file262144以下に制限する。raw JSON node数とwork-node数を別々に検査し、memoによる無制限な反復処理を許さない。循環・未解決・外部refを拒否する。

PのSchema出力先はSAME Unitの既存tmp直下にrrxが新規作成する専用childとする。既存pathを再利用せず、custodyは作成したdirectory identityとUnit／actorの対応を保持する。S/Pともcleanupは当該runで作成した同一directoryだけに限定する。Unknownや強制死では旧epoch／Unitの生成fileが残りうる。後続runで再利用・自動復旧・旧残骸の自動削除をせず、残存を未清掃として記録する。M0に追加cleanup工程を要求しない。

**適合意味**

- method枝は単一enumの一意枝。入れ子oneOfもtagの単一enumまたは相互排他的構造で一意に選択する。
- `$ref`は解決してから比較する。allOfは全制約の積、anyOfはunion、oneOfはちょうど1枝。type配列はunion、nullableは明示null枝だけ。
- boolean schema／型なしschemaを任意型扱いするのは、rrx契約が明示OpaqueJsonとする位置だけ。ThreadExtraの明示object mapはobject容器を維持し、任意型免責へ落とさない。closed object／key-tagの制約もref閉包ごとに比較する。
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
| Claude canary | user送信から120秒 | Probe枠に含める |
| Claude Probe全体 | intent／custody開始から180秒、initialize応答は送信から30秒 | 4 MiB |
| Git helper／batch | 現行各応答30秒／batch180秒 | 現行各action上限を維持 |
| S全体 | custody登録から10分 | 全alias合計8 MiB。各helper上限も独立に適用 |

入れ子deadlineは常に最も早い期限を使う。準備PではVersion／Schema／Handshake／Git／Probeのstdout＋stderrの全受信bytesを、既存`AGGREGATE_BYTES=8 MiB`へ一度だけ累積算入する。stderrを保存しなくても算入する。UUIDなし、同一UUID再送、foreign、ignored、合法tailもdedup／所有filter前に算入する。各helperは自身の上限とaggregate残量の小さい方で制限し、capture stateと固定size read bufferも有界にする。Sは全aliasで同じ8 MiB counterを共有し、予約・加算を競合時にも超過させない。PのHELPER_LIMIT=32、Sのstep上限96、既存line／JSON上限を維持する。生成schema filesは別sinkであり、この8 MiB捕捉counterへ混算しない。

期限／byte超過では新しい送信・spawn・モデル入力を止め、合格を出さず既存停止手順へ進む。必須観測後もstdin close、5秒の自発終了猶予、必要ならgroup signal、2秒drain、10秒reapを適用する。親deadlineが先ならそこで切り、猶予でS／Probe上限を延長しない。両pipe EOFとleader reapが既知ならKnown不合格としてsettleし、どちらか不明ならUnknown。Sはadmission 0、Pは既存Known closureまたはUnknown Heldへ接続する。停止要求／shutdownの既存deadlineを延長しない。

Confirmedは生成結果既知＋両pipe EOF＋leader reap既知。qualifiedはConfirmed＋protocol pass＋合法tail＋自発exit 0または猶予後signal。未知tail／partial JSON／自発nonzeroはKnown不合格。pipe／reap不明はUnknown。

Claude成功後の新規tail許容集合は、strict-validなrate event、auth_status、stream ping／message_stop、system state=idleのみとする。完全同一UUIDの再送はdedup可能。新しいassistant／user／result／task／tool／control要求はtail不合格。late rate rejected等はcanary成功を覆し、Known不合格へ分類する。Codex tailは握手で許す正常全体／rate通知だけである。

Native programはcanonical path、dev／ino／size／mtime／ctime、SHA-256をVersion intent前に記録し、Native helper／transport前にhash、spawn直前にstatを再検査する。Git helperはCLI identityの再hash対象外。execとの非原子的な窓、wrapper内部、escaped descendantsは未証明として残す。

監査は有限語彙、method件数、digest、nullable観測状態だけ。account本文、codexHome、nonce、canary本文、raw frame、stderrを保存しない。receiptは16項目以下、値128 bytes以下、既存private imageの8192 bytes上限を維持する。S bodyは4096 bytes以下。版数はexit 0、完全捕捉、256 bytes以下、単一行、制御文字なし。記録は96 bytes以下かつ`[A-Za-z0-9 ._()+/:-]`なら原文、それ以外はSHA-256とnonrecordableを記録する。記録可否を版数一致による合否へ変換しない。

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

CI schema positiveは今回提供された公開生成structureから、使用6 request、initialized、全83通知、全11 server request、6種responseとその到達definition閉包を保った有限縮約fixtureとする。合成の最小schemaだけで代替しない。

fixture sourceは固定HEAD `b22f22a521424eefc3e61fb1a109bfc85965e5ef`のstatic資料`/schema_contract_catalog`と、そのsource_schemasの対象root／local definition閉包に固定する。catalog subtreeの非循環digestは、JSON読取り後にPython `json.dumps(catalog, sort_keys=True, separators=(",", ":"), ensure_ascii=False)`のUTF-8 bytesをSHA-256にする規則で`c44baad39a81231b2a1bd567b21ede7e7d155b08bf705df1ff8b087728944c7e`。これは取得済みcatalogの出所照合であり、未作成fixtureのdigestでもruntime hash pinでもない。

catalogの10source／268共有definition、ClientNotification、6response、notification branch構造は既に補完済みであり、未作成とは扱わない。ClientNotificationはoneOfの1 object枝であるためschema_inspectionのshape文字列だけを訂正し、catalog本体は再生成・書換えしない。static.baselineは公開source baselineを指し、fixtureのauthor入力HEADとは別である。旧static全体digestを今回fixtureの出所として流用しない。

fixture実体・縮約manifest・decoder対応表・適合結果は未作成／未検証。実体digestは作成後に計算・保存し、今は値や結果を捏造しない。live各項目はnot_runを保持する。

| control群 | positive／negativeと必要mutant |
|---|---|
| Startup | 全alias合格／1 alias拒否、gate除去、journalからpass復元、P省略。 |
| Strict | 全83通知・19itemの正常形、各required欠落・wrongtype・nested tag改変、自／foreign双方。正常ignored／tool維持。所有filter先行mutant。 |
| Schema | 実fileの枝複製・削除、required追加、型・nullable・enum・数値域・ref先改変、無関係同名property、未解決／循環ref。 |
| R1 | 合法empty／usable window／wrongtypeを区別。foreign rate先drop、unknownをAvailable／not_observed化、selfcheck呼出し除去mutant。vector mutationは試験用authority seamを作らずsource差分として実施。 |
| Quota | Wait、普通Admit、recovery Admitそれぞれのモデル入力数、lease／probe ownership／期限／CAS、観測非更新。 |
| Budget | M0 が提供する Executor の実 producer の境界／+1、Git13→9仮定mutant、Probe末尾省略mutant。Reviewer は純粋政策の照合と実起動証明を区別し、本番 helper 鎖は M3 で検証。 |
| IPC | Claude ProbeとCodex Schema／Handshakeそれぞれ、shim呼出し、Grant0、helper以外効果追加0。 |
| Custody | stop／Drop／panic／forced deathの別期待、stdin close、tail打切り、reap不明をConfirmed化するmutant。 |
| Migration/path | new、v1〜9、v10、v11、current、同時initializer、guard欠落・旧reference新表混入、alias traversal／symlink／ownership／境界cleanup。 |
| Consumer | RuntimeMetadata／ShutdownSite wire、型付きinstallation refusal、collector usability、synthetic child実呼出し。 |
| Legacy | 機能廃止・残置・移植の完全台帳、旧入口と他 consumer の起動／入力0、共有する安全性ケースの実管理経路での positive。拒否だけを共有ケースの検証としない。 |

上の11群を次の個別control／対応mutantで具体化する。各negativeは正常な対を同じproducerで通し、指定欠陥を1つだけ変える。mutantで別の拒否が残っても、対象検査へ実到達してその検査の欠落を検出することを要求する。

| ID | positive／negativeと期待 | 落ちるべきmutant |
|---|---|---|
| S-1 | 全alias fresh合格／完了前Task投入・1alias拒否。未公開時admission0、status pending／refused | S gate除去 |
| S-2 | Codexの相関した全応答、chatgpt、ready、合法empty rate／各応答欠落、ID違い、error、-32601、非chatgpt、非ready。欠陥alias拒否 | RPCをschemaだけで代替、相関比較除去、account／ready比較除去 |
| S-3 | Claude相関initialize／未着、ID違い、error。user入力0、system-init／resultを観測済みにしない | initialize相関除去、未着を合格化 |
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
| CX-1 | 固定source縮約／枝欠落・複製、method.enum複数、無関係な同名property | branch検査を文字列存在検査へ変更 |
| CX-2 | local ref閉包／未解決・外部・循環、深さ・memo・work-node上限+1 | ref解決省略、memo／stack／work counter除去 |
| CX-3 | 実送信6method＋initialized／送信キー未宣言、型不適合、required追加、read-only欠落 | schema省略、required／型比較除去 |
| CX-4 | 全受信閉包／nested required・nullable・enum・数値域・ref先改変、未知method／宣言property、Opaque枝拡大。closed object各位置の正常対／追加key、key-tag 1個／0個・複数（httpConnectionFailed＋activeTurnNotSteerable等）、RPC正常4形／result＋error・method＋result/error・id:null。ThreadExtra object／scalar・過大map、ordinaryUsageAllowedの各状態／wrongtype | 消費requiredだけの投影、typed枝をopaque化、additionalProperties=false無視、複数tagを最初のkeyで選択、RPC判別keyを追加field化、ThreadExtraを任意JSON化 |
| CX-5 | 全所有区分の正常形／thread.startedのnested ID、turn.id、nullable turnId、任意threadId各状態、malformed foreign、thread/goal/updatedのparams.threadId＝goal.threadId正例／不一致 | top-levelキー有無だけで分類、所有filter先行、goal.threadId照合除去 |
| CX-6 | 握手Global通知／Thread通知・server request。両者ともpayload検査後の固有扱い。chatgptに相反しないaccount通知／authMode非chatgpt・null、login success=falseはS/P資格付け拒否 | 握手所有区分除去、server request params検査省略、相反account観測を後続正常応答で相殺 |
| Q-1 | Wait／Admitの実producer。Waitはモデル入力0、Parked | Probeをquota admission前へ移動 |
| Q-2 | 普通Admit／recovery AdmitのKnown canary失敗。§9の全before／after、pool自probe解放、期限、readiness CASと観測非反映 | closureからProbe末尾を除外、自probe／期限更新省略、before画像比較を1列除去 |
| X-1 | 同一Native program／S合格後、各Native helper前、transport前にpath・stat・hash差替え。拒否後spawn0 | 束縛比較を各1点除去 |
| X-2 | 現Git sealの正例と差替え負例。GitはCLI hash再検査の対象外 | CLI identityとGit sealを混同、Git seal比較除去 |
| I-S | SのSchema／Handshakeからshim要求を実serverへ到達させる。SにはUnitなしなので有効managed contextなしの拒否、Grant0、helper外効果0。marker拒否とは記録しない | 到達確認／S Unit0検査除去 |
| I-P | genuine SAME actor、現Unit／epoch／正しいcookieでshim要求をserverへ到達させ、cookie検査後にmarker-bound Driver拒否へ達した有限証跡を確認。Grant0、helper外効果0 | state/runtime/driver.rsのmarker-bound拒否除去。元の拒否境界消失を検出し、別の拒否や接続失敗でpassにしない |
| M-1 | TOML新profile・cli_version不在／旧profile、旧キーpresence（表現可能な各値）。canonical前とinstallationで型付き拒否、helper／Unit0。TOML null負例は作らない | legacy宣言検査をhelper後へ移動、canonical前／installationのpresence検査を各々除去 |
| M-2 | 新Frameの旧key不在serialize→deserialize roundtripとdigest一致／旧保持Frame、新profile＋cli_version旧keyあり（JSON null含む）。型付き拒否、Preparing Unit Held、Session0。新旧profile混在不可 | Frame presence検査除去、旧keyを省略してcanonical化、Frame digest比較除去、新Frameへ旧key再出力 |
| M-3 | 任意の合法な単一行版数で全検査通過→ExecutorのNativeInput。番号外だけで拒否しない | 旧版数一致・番号付きqualified_profile復活 |
| MP-1 | new、v1〜9、v10、v11、current、同時initializer。旧版別layout、guard、旧binary open拒否 | 旧referenceへstartup混入、新表guard省略、v11をlegacy一般枝へ落とす |
| MP-2 | §2の既存正負例を同じproducerで束ねる。0755／0700 root・検査済みstartup共有親・umask000 private child0700はpositive。root g/o書込み、別inode lock、他uid、startup symlink／file／0755、既存epoch／alias、spawn前置換、run外cleanupはnegative。追加対はuid=euid／不一致（非特権CIの不一致は§2の単一述語確認として別計数し、実producer／権限証明に数えない）、special bits付きでも0777 maskが0700のprivate child、空Version／Handshake cwd／残存物、Schema child identity一致／置換。実instance＋epoch成分の新規作成と同名拒否を確認 | root書込み／uid検査除去、lock inode束縛除去、startupの0700検査除去、旧epoch／alias採用、通常create_dir_allへ戻す、raw alias join、空cwd／Schema child再確認除去、identity確認省略、symlink追跡、旧残骸まで削除 |
| B-1 | Executor実producerのB境界／B+1と末尾。Reviewerは純粋政策照合だけ | Git13を9として計算、Probe末尾省略 |
| R-1 | selfcheck全vector／source差分で1vector改変。installation拒否 | 成功token不要化、selfcheck呼出し除去 |
| LG-1 | 本番の正常ignored／foreign／tool／追加field／task_type任意string／patch.status欠落／pending／running／各malformed対。公開origin正常非human／不正、§6の旧黙認variant／blockの新拒否、合法text tool_result／image等表外拒否を分離し、型名だけのwire対応をpositiveにしない | ignored検査省略、consumer条件をwire enumへ戻す、未知fieldの改変をdigestから除外、表外黙認復活 |
| C-1 | Metadata／ShutdownSite／型付きrefusalのwireとcollector usability、台帳で維持するsynthetic child実wrapper | consumer配線欠落、正常child起動省略 |
| L-1 | 全旧入口のhelper／Session／新入力0、#19残置検査、共有ケースの現管理producer positive、完全台帳 | 拒否をhelper後へ移動、resume／checkpoint抜け、共有保持領域削除 |
| T-1 | 各deadlineの無応答、UUIDなし通知洪水、同UUID再送、stderrのみ、各byte境界／+1、残aggregate不足。有限停止とKnown／Unknownを別検査 | 通知でdeadline更新、dedup後だけ加算、stderr除外、捕捉枠をfile枠と混同 |

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
4. 新profile、実設定のClaude Executor＋Codex Reviewerでdaemonを起動する。
5. selfcheckと全aliasのS合格を10分上限内で確認する。Sでモデル実行を確認済みにしない。
6. 1 Project／1 Active Task／QUICKを投入する。
7. Claude P receipt、Worktree gate、Implement work=success、SC-N Waiting、sessions=1、Unknown effects=0を確認する。Codex ReviewerはSのみで、review成功とは記録しない。
8. daemonを停止し、PASS時だけ当該scriptが作成したdirectoryを既存規則で削除し、残存なしを確認する。

出力は`M0 selfcheck=pass`、provider別startup passと観測版数／exe digest／実際に確認したstep、Claude preparation probe/init/result pass、rate=`not_observed/empty_valid/usable`、`M0 implement=success stop=SC-N sessions=1 unknown_effects=0`、`M0 RESULT=PASS`を持つ。未証明一覧にはCodex model methods／review／approval／error分類／sandbox実効、Claude permission roundtrip／既定mode、escaped descendants、wrapper内部identityを残し、rate未着ならclaude_rate_limit_eventも加える。static／synthetic合格をこれらのlive証明へ昇格しない。

失敗時は有限stage／reason／unit=held|retired|noneと保持した状態・一時directoryを表示し、`M0 RESULT=FAIL`として保存する。失敗runは削除・再利用せず、原因確認後の再挑戦は新run IDで行う。保持directoryの削除はowner判断に残す。追加cleanup工程や2runをM0受入条件へ増やさない。

### 12a. 影響範囲と競合順序

実装着手前に固定sourceからconsumer inventoryを再照合し、並行担当の進捗を確認する。現在の担当完了を推測しない。producerのないmoduleを先に完成扱いにせず、承認後に実producerからconsumerまで接続する。

| 変更／手段 | 影響consumerと対応 |
|---|---|
| helper変種・capture budget | `execution/native/readonly.rs`、`version.rs`、`preparation.rs:778`、`state/execution/native_phase/version.rs`。32本／8 MiB／Git13／各action上限、期限、settlement、capture境界fixtureを同時更新。既存上限は増やさない |
| prefix／Probe末尾と行予算 | `native/budget.rs`、`prepared.rs`、`native/preparation.rs`、`state/execution/native_phase/version.rs:281-319,486-501,621-641,687-698,804-866`、registration／setup／input。Codex B232、Claude B237へ統一し、Reviewer9本の旧閾値・純粋testsも変更 |
| profile／cli_version／qualified_profile廃止 | 公開source `config.rs`のNativeCompatConfig／canonical／tests、`adapter.rs`、`native/compat.rs:79-100,138-144,253`、`native/version.rs:94,138,150,156,202`、`state/execution/native_phase/version.rs:625`、installation宣言fixture。設定・Frame digest消費者も照合 |
| Claude strict／rate／分類 | `execution/claude_wire.rs:176-429,467以降tests`、`execution/quota.rs:175-274`、`execution/native.rs:2320-2392`、collector、permission consumer、native_fixture.py。現as_str／as_u64／配列抽出前にvalidated型を要求 |
| Codex finite契約・所有区分 | `execution/native.rs:939-952,2135,2204,2209-2264`、`codex/protocol.rs:105-165,1085-1093`、approval／collector、Schema／Handshake fixture。shared protocol16のcoverage保持 |
| quota Known closure | `state/execution/native_phase/quota.rs`、preparation／readiness／no-dispatchの末尾consumer、`state/execution/quotas.rs`、`native_phase/live_quota.rs`。ordinary／recoveryの全画像CASと観測非反映を別assertionにする |
| S gate・停止wire | `runtime/service.rs`、`runtime/admission.rs`、`runtime/installation.rs`、`runtime/stop.rs`、§4のRuntimeMetadata／ShutdownSite全consumer。start lock外実行、admission前提、停止／表示／deserializeを更新 |
| migration v12 | `state/mod.rs`、`state/execution.rs`、`state/managed_binding/schema.rs`、§3のMUTABLE_TABLES全consumer。historical集合とcurrent集合を分離し、旧binary混在不可を確認 |
| 旧入口廃止 | §11の全入口、standalone／Native／adapter／workflow tests、examples、既存protected拒否。entries／starts／preparationsと現status／cancelは保持 |
| 共通fixture | `execution/native/native_fixture.py`、installation tests／pin／success／activation/composition、`crates/rrx/tests/daemon_native.rs`。S・P helper・本番Sessionを区別し、BOOTSTRAP_CASEの失敗がSession前へ移る期待を更新。旧最小payloadやUUIDを新strict正例の証拠として流用しない |

| 競合所有集合 | 編集順序 |
|---|---|
| #93: `execution/native/tests.rs`、`runtime/installation/tests/fm_d2.rs` | 移植／旧宣言変更が必要。担当進捗と保存済み変更を確認し、#93の取り込み後に編集 |
| #93: `execution/results/tests/retained_tests.rs`、`workflow/committed_source_tests.rs` | この修正では編集予定なし。参照／coverage影響が確認されたら担当と調整 |
| #94: `runtime/phase_jobs.rs`、`runtime/installation/tests/epoch_fence.rs` | phase_jobs自体の契約変更は予定しない。S gateでepoch_fence fixture前提が変わるため担当進捗を確認し、必要差分は取り込み順を調整 |
| F3: `adapter/inspection.rs`、`inspection/diagnostics.rs` | 編集予定なし。共有refusal型の参照は確認 |
| P4: `crates/rrx/tests/project_api.rs`、production側未確定集合 | 編集予定なし。production集合が確定するまで無影響を断言しない |
| 上記外のfixture／installation／daemon tests | sourceと消費者を再照合して編集。並行worktreeの未保存変更を上書きしない |

修正手段自体の影響も検査する。登録前CLI本起動は継承設定・hook・MCP・履歴・IPC環境を消費し、Claude canaryは実quotaと費用を消費する。ProbeをAdmit後に置くためpre-Session lease／same-Unit gateの保持時間が延びるが、§10の有限上限内とする。strict化は本番Sessionにも作用し、未適合eventを終端させる。schema免責集合、行・byte閾値、argv本数、UUID／path形はそれを数えるfixtureも更新する。marker authority、既存transport登録手順、Core／terminalの契約を緩める変更は予定しないが、共有consumerの回帰検証を省略しない。

### 12b. セキュリティ自己点検

| 面 | 維持する対策・検証と限界 |
|---|---|
| CLI偽装／入力 | strict JSON、有限method／variant、分類前のfield型検査、相関ID、nonce完全一致、UUID内容digest、成功後tail、deadline／累積bytes。追加fieldからauthorityを作らない |
| canary副作用 | Admit後だけ送信、Executor plan mode、tool／権限要求で拒否、応答0。既存hookや自動許可toolの開始を隔離したとは保証しない |
| helper IPC | SはUnitなし、Pはgenuine contextからmarker-bound拒否へ実到達。cookie失敗／接続失敗をmarker検査の証明にしない |
| S流用／権限 | freshメモリ公開だけ、journalからpass復元しない。Sは拒否専用でP／Prepared／permission／Session authorityを発行しない |
| filesystem／process | raw aliasを使わず、private child identityと限定cleanup、intent先行custody、stdin保持、EOF／leader reap／joinの実観測。root境界は§2とMP-2の同じ正負例・mutantで独立確認する。canonical pathだけ、Drop、forced deathから全子孫死亡を推定しない |
| quota／CAS | SAME actor原本、epoch、revoke、no_registration、末尾inventory、全before／after画像、既存freshness。canary観測非反映と機械的closure更新を分離 |
| 秘密のsink | frame／account／codexHome／nonce／stderrを永続化せず、有限receipt・件数・digestのみ。file／capture／JSONの既存上限維持 |
| legacy／#19 | 起動・新入力前拒否、全consumer台帳、共有3保持領域と停止保持、#19既存部品・検査保持。削除や拒否だけで共有安全性のpositiveを代替しない |

S custodyは自journalのintent／settlementだけ、P SAME actorは既存Admit／CASの範囲でhelperとnongrant closureだけを扱う。S観測のGit／Prepared消費、別actor・CLI peerによるsettlement、revoke済みactorによる新spawn、Probe中のpermission応答は不可。revoke後も既存原本で物理結果を記録することと、新しい実行許可を分ける。ownerの設定選択・一括設計承認、adminのstatus／stopはprobe成功から新authorityを得ない。別Task／Project、同uid別process、unauthにも強制pass・再判定入口を作らない。

source→sinkはVersion stdout→有界観測receipt、wire→validated frame→有限件数／分類、schema files→有限形状比較／digest、program→stat／SHA-256、nonce→メモリ比較だけとする。設定／allocationからargv・環境へ渡す既存境界検査を維持する。helperは利用者の継承権限で動き、exec直前の窓、wrapper内部、escaped descendantsは未証明のままである。

### 13. 一括有限承認表

| ID | 全体設計承認に含める決定 | 現状 |
|---|---|---|
| A1 | quota Admit後、準備ごとのClaude canary 1turn。費用・履歴・既存hook・quota消費、Known failureの明示retry帰結を含む。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。 |
| A2 | Probeの専用UUID、固定文面、Executor plan mode、tool／権限応答なし。既定modeや隔離を証明しない。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。 |
| A3 | 旧起動 API の機能廃止、不要コード・固有テストの理由付き整理、共有安全性の実管理経路での検証。Reviewer 本番正常系は M3。 | オーナー方向決定済み（2026-10-13）。設計全体承認済み。台帳・実装差分・実装後レビューは未完了。 |
| A4 | S全alias gate、版12 migration、有限deadline／capture、journalの独立upper boundsと上限時BLOCKED、confirmedの限定prune／unknown保持、安全なprivate path、停止wire変更と新旧混在不可。上限の容量を使い切れる保証や旧残骸の自動cleanupは含まない。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。 |
| A5 | 本番strict decoder、finite item／notification／nested variant、分類fieldの型・optional／nullableとwire domain／消費条件の分離、83method所有区分、UUID改変拒否、rate解析不能終端。既知frameの未宣言fieldは有界保持受理し業務非使用、Codex closed object／key-tag／RPC判別keyは§7の例外で拒否。Claude origin公開型と非human消費、実消費6項目／usage・structured_outputは§6の明示契約とし、telemetryからreplay authorityを作らない。未知variantは拒否し、旧黙認から新拒否になるvariant／blockとAC-05正常母集団は§6の一覧で判断する。公開未確認のrrx限定集合と数値refinementのlive適合は未証明。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。 |
| A6 | schemaの未知method／宣言property／discriminatorで起動前拒否。明示Opaque集合とtyped mapだけを例外にし、枝別免責を拡大しない。wire frameの未宣言field保持とは別契約。static catalogは開発時根拠でありruntime仕様生成器ではない。raw hash pinなし。未適合CLI拡張は契約見直しまでBLOCKED。 | 承認済み（2026-10-13、固定HEAD c6413082）。実装・検証は未完了。 |

R1、M4 runtime merge、public sandboxの作成・使用は決定済み。執筆者は今後の実装者・レビュアーを承認しない。

### 14. pin 設計との差分一覧（orchestrator の転記）

| 項目 | pin 設計 | M0 |
| --- | --- | --- |
| 許可根拠 | 単一の厳密版数 | 実観測。番号は記録のみ |
| 版数検査の消費者 | phase 4、legacy 3 | すべて削除 |
| legacy 経路 | ピンの下で起動 | 旧起動 API を機能廃止として整理し、起動前に CapabilityUnavailable。共有安全性検証は現行の管理経路で維持 |
| 起動時 | 何もしない | 層 S（非モデル、拒否専用） |
| 準備の helper | Version＋Git×13 | Codex は＋Schema＋Handshake、Claude は admission 後に Probe |
| 登録前の観測 | `--version` | 握手、Claude の `system/init`／`result`、Codex の schema 構造 |
| 実行ファイル | canonical パス | stat と sha256 の束縛 |
| 解析 | top `type` のみ有限、未知通知を無視 | 内部 variant、method、payload まで有限 |
| rate decoder | 未知を `None` | 型分離、解析不能は Probe でも本番でも fail closed |
| 宣言 | `cli_version` 必須 | 新 profile、installation で旧宣言を拒否 |
| DB | 版 11 | 版 12（S の journal） |

### 15. この稿の完了境界

A3 の方向はオーナーが旧 API の機能廃止として確定した。修正差分の独立確認とM0設計全体承認は完了した。具体的な削除・残置・移植台帳、共有安全性ケースの代替検証と実装後レビューは未完了である。M0 で現に提供する振る舞い、廃止する振る舞い、M3 で新たに提供する Reviewer を区別して確認する。
