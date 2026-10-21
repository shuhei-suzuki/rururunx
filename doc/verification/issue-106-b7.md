# Issue #106 B7: Reviewer の段階、VerdictDecoder、読み取り専用のスナップショット、秘密情報の走査

| 項目 | 結果 |
|---|---|
| 範囲 | Reviewer の段階（対象のコミットから rrx が作る読み取り専用のスナップショット `work_root/ro/<run>-<seq>-<attempt>`、§5.3 の固定の argv、B4 の launcher と custody の下での起動）、VerdictDecoder（result.json の上限 1 MiB、外側の envelope、strict_json、`deny_unknown_fields`、`rrx.review.v1` の schema と §8.3 の制約）、prompt・生の結果・stderr・デコードした文字列・保存する本文の秘密情報の走査、nonce による Task と diff の囲い込み、共通の launch の境界での必須のレビューの証拠、Goal が Running でないときに書き込まない規則（rule G） |
| 実装のコミット | 0abfec88 から c477238c まで |
| 設計の追記 | D21（Reviewer の argv と Q に `--no-session-persistence` を足す）。§15 の B10 の行に、一時停止と再開のレビューの受け入れ試験と、再開で `PipelineProfile.policy_sha256` を比べることを記録 |
| レビュー | 初回は Sol（S1〜S3）と Opus（O1〜O9）の 2 系。修正は e368f412（Sol）と 347cd91c（S2・S3、Opus）。2 巡目は Sol が S2・S3 の修正を確かめて承認し、Opus が O2-1〜O2-8 を出した（b0b890f2 で Sol が修正）。3 巡目の Opus は、6 件を修正済み、O2-4 を多層防御として受け入れ、O2-2 を本物の CLI での確認待ちとし、新しい Low 3 件を出した（3cb0288e で Sol が修正）。その差分の確認で Opus が Low 1 件（B7-O4-1）を出し、eaa26ecc で Sol が直し、Opus が指摘なしで承認した |
| オーケストレーターの変更 | 8c04ff70（一時停止の試験を B7 が持つ範囲に絞り、再開は B10 に回す）、c477238c（変異の検証で見つけた試験の穴を埋める 1 ケース） |
| macOS のホスト | 0abfec88 で 382 件を 1 件ずつ実行し 379 件が合格。3 件（`b5_f1_binary_tree`・`he08_platform_escape_held`・`b7_snapshot_writable_file`）は高い負荷のときに落ち、単独の再実行で合格。347cd91c で 89 件のうち 87 件が合格（`b7_line_order` は 347cd91c で消した試験、`b7_paused_boundaries_resume` は 8c04ff70 で絞った）。8c04ff70 で 34 件、3cb0288e で 8 件（`b7_runtime_review_diff` は負荷のときに落ち、単独で合格）、eaa26ecc で 5 件がすべて合格。c477238c で `b7_decoder_secrets` が合格。`safe.bareRepository=explicit` の下で git 2.54.0 が bare の形のリポジトリを拒否することを canary で確かめた |
| Linux | rust:1.91.1 のコンテナを非 root（uid 501）で実行（eaa26ecc）。fmt と clippy（`-D warnings`）は合格。`cargo test --workspace` は 1 件を除いて合格（lib は 948 件合格、20 件 ignored）。落ちた `b7_reviewer_git_environment` は、確認用のコンテナに `USER` と `LOGNAME` が無いことが原因で、同じコンテナで 2 つを設定すると合格し、外すと `missing USER` で落ちることを確かめた。macOS の実機と GitHub の Ubuntu のランナーには、どちらもある |

## 変異検証

最後に 1 回、レビュー用の detached worktree（eaa26ecc）で実行した。各変異は、元の文がちょうど 1 回現れることを確かめてから当て、対象のテストを 1 件だけ実行し、hash と clean を確かめて元に戻した。元のコードでの確認は、試験ごとに 1 回行った。

| 対象 | 件数 | 結果 |
|---|---|---|
| B7 の全巡の変異の定義 | 236 | 重複 83 件を除き、今のコードに当たる 140 件と、古くなった 13 件に分けた |
| 古くなった定義を、今のコードに合わせて Sol（読み取り専用）が作り直したもの | 12 | 当てた件数をオーケストレーターが確かめた。残る 1 件 `b7-line-order` は、行の順序の検査が B8 の filtering に移り、`b7_inverted_range_kept` が逆順を保存することを求めるので外した |
| 実行した変異 | 152 | 144 件を検出 |
| 元のコードでの確認が落ちたもの | 5 | `b7_runtime_envelope` など Runtime の試験 4 本が、load average が 50 前後のときに `review_adoption_state`（採用の時点で段階がまだ exited でない）で落ちた。load average が 20 前後のときにやり直すと、元のコードでは合格し、5 件とも検出した |
| 同等の変異 | 2 | `b7-strict-frame`（`review.rs:110` が strict_json の frame より先に同じ 1 MiB の上限で拒否する）、`b7-snapshot-tree-equality`（`runtime/pipeline/review.rs:189-192` が、準備のときに記録した `snapshot_tree` ともう一度照合する） |
| 試験の穴を直した後に検出 | 1 | `b7-decoded-envelope-scan`。生のバイト列の走査は `\uXXXX` しか戻さないので、`\t` を含む `"a=b\tEXAMPLE_SECRET=fixture"` はデコードした文字列の走査でしか見つからない。既存の入力はどれも生の走査で見つかっていた。c477238c でこのケースを足し、デコードした文字列の走査を外すと試験が落ちることを確かめた |

## 既知の限界と残件

| ID | 内容 | 扱い |
|---|---|---|
| B7-O2-2 | 固定した Claude CLI が、子の git に `GIT_CONFIG_*` を渡すか。Reviewer の PATH で `path_prefix` に git があるとき、どの git が動くか | 下の real-CLI harness で記録する道具を追加。owner の実行結果は未取得 |
| B7-L-model | `modelUsage` のキーの形を、本物の claude の envelope で確かめていない | 下の real-CLI harness でキーだけ記録する道具を追加。owner の実行結果は未取得 |
| B7-L-budget | スナップショットの 100,000 項目の上限での所要時間 | `e0c1307a` の host 計測では準備が 360 ms で出力上限により失敗し、独立した合成 tree の死亡後検証は 111 秒で合格（load average 約 47）。下記のとおり、owner run で低負荷時の上限での検証時間を再計測する |
| B7-L-resume | 一時停止の後の再開のレビューの受け入れ試験、再開で accepted policy を比べること | B10 の受け入れの条件 |
| B7-L-order | finding の行の順序（`line` と `end_line`）の扱い | B8 の filtering |
| B7-O2-4 | 入口の検査と handler の guard の重なり | 多層防御として受け入れ |

### owner による real-CLI pass と上限の計測

これは検証用の道具で、製品の起動条件や Q を変更しない。通常の host test の `state::execution::pipeline::tests::real_cli::fake_cli_pass` は、同じ Runtime フローを既存の native fake CLI だけで実行する。実 CLI は `#[ignore]` と `RRX_REAL_CLI_PASS=1` の両方が必要である。この実装作業では実 CLI を起動していない。

1. owner は新しい端末を開き、他の Claude/Codex セッションと、資格情報を持つ同一利用者のプロセスを終了する。Agent は同一利用者の他プロセスの引数・環境を読めるので、この条件を満たさない状態では実行しない。
2. 検証済みの commit を持つ clean なリポジトリの toplevel で、`command -v codex` / `command -v claude` が実行したい native binary を指すことを確認する（npm の script 入口は Q が拒否する）。token-like 変数（名前に TOKEN・SECRET・PASSWORD・API_KEY、大文字小文字を問わない。空の値も含む）を環境に残さない。通常の owner の shell が設定する `GITHUB_TOKEN` と `GH_TOKEN` は、事前に unset するか、下の `env -u` で取り除く。ほかの該当変数も事前に unset する。
3. 専用の絶対パスのディレクトリを `mkdir -m 700 /absolute/private/rrx-codex-home` で作り、`CODEX_HOME=/absolute/private/rrx-codex-home codex login` で認証する。既存のものを使う場合も、所有者が自分で mode が 0700、symlink 解決後も `$HOME/.codex` と異なり、Projects と rrx の state ディレクトリに重ならないことを確かめる。`config.toml` に許されるのは文字列の `model` と `model_reasoning_effort` だけ。スクリプトはこのディレクトリを作成・変更しない。`claude` もログイン済みであることを確認する。
4. 同じ clean なリポジトリの toplevel から次を 1 回実行する。`RRX_REAL_CODEX_HOME` は必須である。モデルの任意指定は `RRX_REAL_CODEX_MODEL`（既定 `gpt-6.1-sol`）、`RRX_REAL_CODEX_EFFORT`（`medium`）、`RRX_REAL_CLAUDE_MODEL`（`claude-opus-5-5`）。

   ```sh
   env -u GITHUB_TOKEN -u GH_TOKEN RRX_REAL_CODEX_HOME=/absolute/private/rrx-codex-home bash scripts/real-cli-pass.sh
   ```

   スクリプトは関連プロセスの PID と名前だけを表示し、上記の条件を満たすことへの明示的な `yes` を要求する。version を pin として取得し、offline・locked build 後に実テスト 1 本だけを起動する。追加 pass や自動再試行はしない。
5. `.rrx/real-cli/<UTC timestamp>/summary.json` と `send-record.json` を確認し、オーケストレーターへはこの 2 ファイルだけを送る。`run.log` は絶対に送らない。summary は各 step の kind・state・reason・時間、CLI pin/model、送信した prompt と初期/採用後の sample HEAD の `git archive` の hash・bytes、probe 結果を含む。生の Agent 出力・本文・環境全体は転記しない。git の probe が呼ばれなければ `called=false` として残り、転送を確認済みとはしない。途中で採用されなければ後続は `not_run`、summary を書いた後に失敗とする。Q の拒否も記録し、安全条件を緩めて進めない。通常の Runtime が保持する生の出力は一時 fixture 内だけで、終了時に片付ける。スクリプトの build/run/version のログは private な出力ディレクトリに置き、端末へ Agent 出力を流さない。

snapshot の時間計測は CLI・ネットワークを使わず、別に実行する。出力の親ディレクトリを作り、未使用の絶対ファイルパスを指定する。

```sh
RRX_SNAPSHOT_BUDGET_TIMING=1 RRX_SNAPSHOT_BUDGET_OUT=/absolute/path/in/worktree/.rrx/snapshot-budget.json cargo test --offline -p rrx --lib state::execution::pipeline::tests::snapshot_budget::snapshot_budget_timing -- --ignored --exact --test-threads=1
```

production の `SNAPSHOT_ENTRY_BUDGET` を使い、root + 2 個の中間ディレクトリ + 残りの小さい file で正確に上限の commit を作る。fixture の Git オブジェクト作成と件数の確認には、固定した Git 環境の通常の Git プロセスを使い、production の deadline を掛けない。setup の失敗は `fixture_build_failed` として JSON に残す。production `rebuild` と採用時と同じ `verify` を別々に計測し、失敗も時間・理由とともに JSON に残す。既存の Git plumbing の通常出力上限 4 MiB は、100,000 項目の `ls-tree` の出力より小さいため、この fixture は準備で拒否される。その場合も verifier 自体を上限で測るため、同じ合成 tree の読み取り専用 fixture を別途作り、`verification_fixture` で区別して production `verify` を計測する。準備の失敗は保持し、JSON を書いた後にテストを失敗とする。この道具では製品の上限を広げず、準備に成功した時間を測れたとは扱わない。

**B7-L-budget の計測結果**: オーケストレーターの `e0c1307a` の host 実行では、100,000 項目の準備（`review_snapshot::rebuild`）が 360 ms で `command output exceeded its bound` により失敗した。準備失敗後に別途作った同じ合成 tree の死亡後検証（production `verify`）は 111 秒で合格し、load average は約 47 だった。これは準備済み snapshot の測定成功ではない。実効上限は 100,000 項目または 4 MiB の `ls-tree` 一覧出力のうち先に達する方とし、どちらの超過も `review_snapshot_limit` で review を保留する。B7-L-budget の残件として、owner run で低負荷時の 100,000 項目での検証時間を再計測する。

## テストの harness の制限

- 本物のプロセス、sandbox-exec、daemon を使うテストは macOS のホストで 1 件ずつ実行した。Sol の sandbox では実行できない。
- load average が 40〜90 のとき、Runtime の fixture が採用の時点で段階の終了を確認できずに落ちることがある。単独の実行や負荷の低いときには合格した。
- 観測として、`fake_cli_pass` は load average が約 20 のときに 4/4 回合格した。約 50〜60 では前段が `death_unconfirmed` で hold し、1 回は review の段階が汎用の `review_validation_failed` で hold した。review の準備または検証内の Git deadline は、`Git timeout` が review の名前付き理由にないため、この汎用理由に対応する。製品コードは変更していない。
- Linux のコンテナで試験するときは `USER` と `LOGNAME` を設定する（Reviewer の環境の組み立てが、無いときに起動を拒むため）。
