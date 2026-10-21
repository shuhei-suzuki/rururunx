# rururunx の作業規則

## CI の方針

PC（ローカル）でできる確認は PC で行い、GitHub でしか行えないものだけを GitHub Actions で行う。

### PC で行う確認（push の前に必ず行う）

- `scripts/local-ci.sh` を、確認したいコミットで、clean な worktree から実行する。中身は次のとおり。
  - macOS の実機での `cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace`、`cargo build --locked --release --workspace`
  - Linux（Docker の `rust:1.91.1` の使い捨てコンテナ）での fmt、clippy、test。HEAD の追跡内容だけを `git archive` で保存し、毎回新しい `/src` に新しい mtime で展開する。worktree はマウントせず、無視された `worktree/`・`.rrx/`・`target/` はコンテナに持ち込まない。
  - ローカルで把握している `origin/main` が HEAD の祖先であることを `main-contained` で確認する。欠落は NOT RUN、非包含は FAIL となり、どちらも実行全体を失敗にする。
- マージ直前の最終ローカル実行では、先に fetch し、現在の `origin/main` を含む HEAD を使う。必要なら main を作業ブランチへ merge または rebase し、clean な最終 HEAD でスクリプト全体を再実行する。Actions では main push と PR の merge 結果のテストを行わないため、この手順で統合後のテストを補う。`main-contained` 自体は fetch しない。
- `--host-only`（`--allow-partial` 併用を含む）は診断用で、push 前の必須確認を満たさない。`doc/` だけを変える PR はコード変更がなく、CI の文書専用経路も成功を報告するため、ローカルスクリプトは不要。
- ホストの build script・proc macro・テストは、ユーザーの権限・認証情報・ネットワークで実行される。除去する環境変数は `GITHUB_TOKEN`・`GH_TOKEN`・`GH_ENTERPRISE_TOKEN`・`GITHUB_ENTERPRISE_TOKEN` の4つだけで、ホストを隔離するものではない。この作業フローで作成したコミット、または差分をレビュー済みのコミットだけを実行する。外部 PR のコードは、レビューするか隔離環境を用意するまでローカル実行しない。
- Linux の HOME・TMPDIR・CARGO_HOME は実行ごとの tmpfs とし、`config.toml` を永続化しない。`RUSTUP_HOME` は上書きせず、イメージ内の root 所有の rustfmt/clippy 付きツールチェーンを使う。image digest（無ければ image ID）をメタデータと要約に記録する。
- `rururunx-local-ci-registry`・`rururunx-local-ci-git`・`rururunx-local-ci-target` は、Cargo registry・Git データベース・Linux target 用の常設プロジェクトキャッシュで、一般の作業後クリーンアップによる削除対象ではない。キャッシュのルート権限だけを準備するソース未マウントの補助コンテナも `--rm` で終了する。キャッシュは信頼済みコミットにだけ使い、信頼できないコードを実行した場合は、利用中でないことを確認して `docker volume rm rururunx-local-ci-registry rururunx-local-ci-git rururunx-local-ci-target` で3つとも削除する。
- Linux 結果マーカーは厳密な1行の `PASS <数字>` または `FAIL <数字>` と末尾の改行だけを受理し、読み取り失敗・欠落・不正形式は NOT RUN とする。コードが悪意を持つ場合はどのマーカーも偽造できるため、上記のコミット信頼条件が必要。リポジトリパスにカンマ・二重引用符・改行を含む場合は Docker mount の CSV 解釈を避けるため実行を拒否する。
- 結果（コマンド、対象の SHA、件数、成否）を PR の Test plan に書く。失敗したままの push はしない。
- 個別の試験を実機で 1 件ずつ実行する必要がある変更は、その手順と結果も PR に書く。

### GitHub Actions に残すもの

- `.github/workflows/ci.yml` の Ubuntu の 1 ジョブだけ。中身は fmt と clippy で、自動マージの判定が求める「独立した成功のチェック」を作るためのもの。
- テスト、macOS、release ビルドは Actions では行わない（PC で行う）。
- `doc/` だけを変える PR では、ジョブは起動するが重い手順を飛ばして成功で終わる。

### してはいけないこと

- CI を試行錯誤やテストの実行の代わりに使う。確かめるためだけの push、変更も新しい根拠もない再実行をしない。
- オーナーの承認なしに、ワークフローを足したり、Actions で行う範囲を広げたりする（計測用のワークフローを含む）。承認を得て一時的に足したものは、使い終わったらブランチごと片付ける。
- 手動実行・定期実行などの任意のワークフローを起動する。
