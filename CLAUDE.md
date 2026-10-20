# rururunx の作業規則

## CI の方針

PC（ローカル）でできる確認は PC で行い、GitHub でしか行えないものだけを GitHub Actions で行う。

### PC で行う確認（push の前に必ず行う）

- `scripts/local-ci.sh` を、確認したいコミットで、clean な worktree から実行する。中身は次のとおり。
  - macOS の実機での `cargo fmt --all -- --check`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace`、`cargo build --locked --release --workspace`
  - Linux（Docker の `rust:1.91.1` の使い捨てコンテナ）での fmt、clippy、test
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
