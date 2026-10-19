# Issue #106 B5: Executor の段階、記録した rrx の commit だけの採用、PipelineGit

| 項目 | 結果 |
|---|---|
| 範囲 | Executor の段階（起動前の確認、`codex exec` の起動と custody の下の終了）、§6.2 の検査（scope・保護するパス・通常ファイル・大きさ・秘密情報の scan）、rrx が組み立てる commit（一時の index・`--no-filters`・`commit-tree`）と、SHA・tree の記録 → ref の CAS → 採用の順、committed からの再開、prepare（Goal の作成の journal・Task の worktree の登録と検査）、`PipelineGit`（global・system の設定と attributes を読まない Git、上限と期限、プロセスの後始末）、diff の根拠の原子的な公開、last-message の fd による封印 |
| 実装のコミット | 2989113a から 0e959561 まで |
| 設計の追記 | D18（gitlink・submodule・global の attributes と excludes・lazy fetch・`binary`・`ident`・大文字小文字・`--no-checkout` の後の検査）と D19（ignored の保護するパスの拒否、APFS と同じ Unicode の名前の畳み込み、Executor は scope にかかわらず `.gitignore` を変えられない、index の `assume-unchanged`・`skip-worktree` の拒否、残余）。どちらも orchestrator が書き、実装のレビューで確かめた。ignored の保護するパスのうち、Goal の base の ignore 規則が畳むディレクトリ（`node_modules/` など）の中だけは残余として受け入れた |
| レビュー | 初回は Sol（B5-SOL-01〜05）と Opus（F1〜F15）の 2 系。2 巡目は Opus が修正の全体（Sol が自分で直した SOL の項目を含む）、Sol が自分の指摘を確かめた（新しい指摘 N1〜N3、B5-SOL-R2-01・02）。3 巡目で Opus は承認し、Sol は B5-SOL-R3-01・02 を出した。4 巡目は指摘した側が修正の差分だけを見て、両系とも承認した。修正は指摘していない側が書いた（Opus は read-only で差分を出し、orchestrator がそのまま当てた） |
| macOS のホスト | cd6bd32d で fmt・clippy（`-D warnings`）・build と、`execution::pipeline::adoption::` 52、`::git::` 19、`::custody::` 14、`state::execution::pipeline::` 191、`runtime::pipeline::` 1、`execution::process::` 14 が合格。各巡で足したテストは 1 件ずつ実行して合格 |
| Linux | rust:1.91.1 のコンテナを非 root（uid 501）で実行（cd6bd32d）。fmt・clippy（`-D warnings`）・`cargo test --workspace` がすべて合格（失敗 0） |

## ホストで見つけて直した欠陥

| 欠陥 | 影響 | 修正 |
|---|---|---|
| Git の終了の後、zombie だけのプロセスグループへの `killpg` が macOS で EPERM を返した | すべての Git の呼び出しが後始末の確認で失敗した | 起動時の確認と同じ方法で、zombie だけのグループを解決する |
| `extensions.worktreeConfig` が無いと `git config --worktree` が拒否され、あっても `config.worktree` が無いと読めなかった | 設定の検査が常に失敗した | 拡張を先に読み、ファイルが無ければ worktree の設定は無いものとする |
| Agent の index にだけある gitlink を、tree の検査が見なかった | status が `--ignore-submodules=all` でそのディレクトリを一覧から隠した | status の前に index の gitlink を拒否する（D18） |
| テストの正の対照で、長さの違う書き換えでは status が clean filter を動かさなかった | gitlink のテストが前提の段階で失敗した | 同じ長さの書き換えにした |

## 変異検証

最後に 1 回、レビュー用の detached worktree（cd6bd32d）で実行した。各変異は、元の文がちょうど 1 回現れることを確かめてから当て、対象のテストを 1 件だけ実行し、hash と clean を確かめて元に戻した。元のコードでは、対象のテスト 88 種類がすべて合格した。

| 対象 | 件数 | 結果 |
|---|---|---|
| B5 の全巡の変異（cd6bd32d） | 142 | 138 件を検出 |
| 生き残りのうちテストを足したもの | 2 | `b5-f1-gitlink-refusal`（tree の gitlink の拒否。後ろの index の検査も拒否していたが、prepare では tree の検査だけが使われる）と `b5-recorded-identity-parent`（finish の記録した commit の親の照合。既存のテストは違う tree しか記録していなかった）。0e959561 で直接のテストを足し、どちらの変異も検出した |
| 同等の変異 | 2 | `b5-f1-adoption-status-submodules`・`b5-f1-clean-status-submodules`（status の `--ignore-submodules=all` を `none` に戻す）。gitlink は tree でも index でも status より前に拒否されるので、status まで届かない。この印は後ろに控える多層防御として残す |

巡ごとの変異の定義のうち、後の修正で元の文が変わった 8 件は、今のコードに合わせて書き直した（1 件は 3 巡目の同じ検査の変異と重なるので外した）。

## 既知の限界と残件

| ID | 内容 | 扱い |
|---|---|---|
| B5-F11 | Git の期限と出力の上限で、大きなリポジトリが held になりうる | D18 に記録した既知の限界。Issue #108 |
| B5-SOL-R2-02 | diff の根拠の書き込みの途中の状態を、テストが観測していない | Issue #108 |
| B5-N3 | prepare が worktree の登録と checkout の間で止まると、自動では再開しない（held） | Issue #108 |
| B5-R3-4 | 名前の畳み込みは Unicode の full case folding と同じではない（今の保護する名前では悪用できない） | Issue #108 |
| B5-R4-1 | retry の初期化は index のフラグを消さないので、フラグが残った run は held のままになる | 運用者の復旧手順を D19 に書いた。Issue #108 |

## テストの harness の制限

- 本物のプロセスと Git を使うテストは macOS のホストで実行した。Sol の sandbox ではプロセスの後始末が確認できず実行できない。
- load average が 25〜40 のとき、起動時の確認（Q）を使う Store のテストの fixture が期限切れで準備を拒否された。負荷が下がった後の実行と 1 件ずつの実行では合格した。
