# Issue #106: 本物の CLI での確認（real-CLI pass）と D24〜D27

## 結果

オーナーが承認した real-CLI pass の 5 回目（`20261030T213809Z`、HEAD `84545391`）で、Executor（`codex-cli 0.164.0`）→ テスト → Reviewer（`2.1.314 (Claude Code)`）が通り、run は `adopted` になった。D26 §5.6.3 の結果 (a)（両方の profile を確認）に当たる。

| 段階 | 結果 |
|---|---|
| prepare | 採用 |
| implement（codex） | exit 0、profile `codex-jsonl-v1`、`provider_outcome=success`、last-output `passed`、採用 |
| test | `pass` |
| review（claude） | exit 0、profile `claude-result-v1`、`provider_outcome=success`、verdict `approve` |

## real-CLI pass の経過

| 実行 | HEAD | 結果 | 原因と対応 |
|---|---|---|---|
| `20261029T033829Z` | 52f5fc00 | Q で失敗、Agent は未起動 | `claude --help` が 22 KB で probe の上限 4 KiB を超えた。help を求める probe だけ 64 KiB にした（D24、432eea22・90591fca・8b03ee65） |
| `20261029T042850Z` | 8b03ee65 | implement が `launch_preflight_failed`、Agent は未起動 | 起動直前の再 hash（2 つの実行ファイル、合わせて 5 秒）が debug ビルドで間に合わなかった。sha2 を dev profile で `opt-level = 3` にした（D25、12a06faf・2105beff） |
| `20261029T045606Z` | 2105beff | implement が `agent_failed` | 分類器が合成の pin しか受け付けず、本物の版は常に `other` だった。pin → wire profile の閉じた表と成功判定の強化（D26、d33e4b9a・75b847d4・a06c5edd） |
| `20261030T160517Z` | a06c5edd | 結果 (c)：codex は採用・テスト合格、claude は `unmapped` → `other` → `review_agent_failed` | Claude Code が自動更新で 2.1.312 から 2.1.314 になっていた。観測した封筒の shape をもとに、完全一致の pin 2.1.314 を足した（D27、c096acfb・d66de773・b9658cff・84545391） |
| `20261030T213809Z` | 84545391 | 結果 (a)：`adopted` | — |

## 第 2 窓（`20261030T213809Z`）の版の照合

| CLI | 手動の開始前の確認 | script の記録値（Q が認定した値） | run 後の値 |
|---|---|---|---|
| codex | 取得していない | `codex-cli 0.164.0` | `codex-cli 0.164.0` |
| claude | 取得していない | `2.1.314 (Claude Code)` | `2.1.314 (Claude Code)` |

- 手動の開始前の確認は行われなかった。script の記録値は、その直後に Q が完全一致で認定した値で、2 つの pin と一致する。
- run 後の値は、script の終了の直後ではなく、結果の報告の後に同じ端末で取得した。版は上がる一方で、遅れて取得した値も pin と一致したので、run の途中に版は変わっていない。
- 最後の起動（review）の実行ファイルは、起動直前の再 hash（`before_launch`）で Q の値と照合されている。

## 手順からの逸脱

- 5 回すべてで、ほかの Claude / Codex のセッション（第 2 窓では約 40 の process）が動いたまま、オーナーが `yes` と答えて実行した。D22 の開始条件（資格情報を持つ同じ利用者の process と同時に動かさない）を満たしていない。実行中、Agent の CLI は同じ利用者のほかの process の引数と環境変数を読める状態だった。送った入力は合成のサンプルだけで、送信の記録は `send-record.json` のとおり（prompt 2 つと sample の tree 2 つ）。
- D27 の第 2 窓の手順のうち、手動の開始前の確認と `DISABLE_AUTOUPDATER=1` の設定は、行われたことを確かめられていない。run 後の版の取得は遅れた（上の表）。

## 観測した事実

- codex 0.164.0 の event の順序（両窓で同じ）：`thread.started` → `turn.started` → `item.completed` → `item.started` / `item.completed` の組 → `item.completed` → `turn.completed`。
- claude 2.1.314 の封筒：`type=result`、`subtype=success`、`is_error=false`、`api_error_status` は null（キーあり）、`api_error` と `errors` は無い。top-level のキーは 27 個（`crates/rrx/tests/fixtures/pipeline/claude-2.1.314-success.json` と同じ構造）。第 2 窓で製品の分類器が `success` と判定したので、`structured_output` が object であることも確かめられた。
- B7 の確認事項：`modelUsage` のキーはモデル名（`claude-opus-5-5`）。Claude CLI は子の git に `GIT_CONFIG_NOSYSTEM=1` を渡し、`GIT_CONFIG_COUNT` などは渡さない。Reviewer の PATH の git は `/usr/bin/git`。
- 上限（limit）の分岐は、どちらの CLI でも実観測していない。2.1.312 は文書由来の pin のまま、実観測していない。

## レビューと検証

| 範囲 | レビュー | 検証 |
|---|---|---|
| D24 | Sol 承認、Opus 2 巡で承認 | Q と probe の試験 45 件、変異 4 件をすべて検出 |
| D25 | Sol・Opus とも 1 巡で承認、Opus の Low 2 件を直した | 実測（未最適化 13.2 秒・10.6 秒 → 0.62 秒・0.56 秒） |
| D26 設計 | Opus 3 巡で承認（Sol が執筆） | — |
| D26 実装 | 初回は 2 系（Sol は Low 2 件、Opus は承認と Low 5 件）、Opus の再レビューで承認 | 変異 23 件をすべて検出（host でしか確かめられないものは、狙った assert で落ちたことを確かめた） |
| D27 | 初回は 2 系（Sol は指摘なしで承認、Opus は承認と Medium 2 件・Low 1 件）、文書の修正を Opus が 3 巡確かめて承認 | classify の試験 11 件が合格、変異 2 件を検出 |

## 残件

| ID | 内容 | 扱い |
|---|---|---|
| HARNESS-R1 | 試験の helper が、動いている pipeline のサービスと `control_admission` を取り合い、「state changed」で落ちることがある（変更前の c84ef638 でも起きる） | #108 |
| W2-L1・W2-L2 | D26 の試験が、将来の変更で別の理由で通りうる 2 か所 | #108 |
| HB-R2-L2・SHA2-S1・SHA2-S4 | D24・D25 の Low | #108 |
| D27-N6a・N6b | 設計書の言葉づかい（3 つの値に「一方」）と、版の記録の末尾の改行 | 任意。対応しない |
| PIN-AUTOUPDATE | Claude Code は自動で版が上がるので、完全一致の pin は版ごとに実観測と追加が要る | B11 の受入と運用で扱う |
| LIMIT-UNOBSERVED | 上限の分岐は合成の fixture だけが根拠 | 実際の上限の形が違えば `other` に倒れ、待ちは延びる（安全側） |
