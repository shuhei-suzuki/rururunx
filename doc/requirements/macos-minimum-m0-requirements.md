# macOS 最小一本道 M0: 要件

基準: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。状態: **DRAFT / 独立レビュー・オーナー設計承認前**。本番コードは変更していない。

M0 は #5/#6 の互換性変更（STRICT）、M1 は #95、M2 は #12、M3 は #9、M4 は #13、M5 は #16 に対応する。M0 の実 Mac 受入まで M1 の実装へ進まない。

本文執筆: Claude Opus 5.5 (`claude-opus-5-5`)、要求 effort high。CLI 出力の modelUsage は同じ model を報告。effort の実測値は未報告。オーナー回答の転記と静的観測の追記は orchestrator が実施。これらは独立レビューの代替ではない。

オーナー確定事項（2026-10-13）:

- 版数の厳密固定を capability probe に置換する。auth/CAS/owner/marker/#19/process custody は維持する。
- rate_limit_event: 起動時 decoder selfcheck、実イベントが出た場合は厳密解析。未観測は記録し、実イベント確認済みとは扱わない。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/5#issuecomment-6136393618)
- M4 を rrx 自身の自動マージまで拡張する。Draft PR → 必須 CI/review/safety/repository gate → ready → PR merge → 結果照合。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/13#issuecomment-6136256684)
- 試験用リポジトリ候補は新規 private の `shuhei-suzuki/rrx-e2e-sandbox`。作成・使用は未確定。rururunx 本体で試運転しない。

実装時には両 OS の検証と causal controls/mutants、独立実装・安全性レビューが必要。設計本文に残る未確認事項は完了扱いしない。


### 目的

managed phase 経路の CLI 厳密バージョン一致を撤廃し、実際の CLI との対話で得た capability 観測で置換する。Claude 2.1.296 と codex-cli 0.160.1 は観測値として記録するだけで、番号を許可根拠にしない。使用する通信契約の有限で厳格な解析は維持し、機能確認は省略も緩和もしない。

### スコープ

| ID | 内容 |
| --- | --- |
| S-1 | phase 経路（`start_phase` `execution/native.rs:248`）の版数ピン撤廃 |
| S-2 | 層 P: 準備 actor ごとの確認用 helper 鎖（Codex は schema 生成、両者とも握手と canary ターン） |
| S-3 | 層 S: daemon 起動時に、設定された全 Native agent へ独立 custody の probe を行う。拒否専用で、権限の代替にしない |
| S-4 | Codex 通知の有限分類を probe と本番 consumer で共有する |
| S-5 | 互換宣言の明示的移行、監査保存、Mac スクリプトと期待出力 |
| S-6 | legacy 経路の扱いをオーナー判断として提示する（A3） |

### 非スコープ

- M1 以降の実装。SC-N Issue は #95 として起票済みで、M0 での起票は不要。進捗は #16、scope 回答は #13 に記録済み。
- M4 の内容は決定済み事項として記すだけ: Draft 作成 → 必須 CI/review/safety/repo gate → ready → PR merge → 結果照合、main/master への直接 push は禁止。試験 repo 候補 `shuhei-suzuki/rrx-e2e-sandbox` の作成と使用は未承認で、M0 では触れない。runtime cleanup 等の別 scope は追加しない。
- `#10` を代替する権限付与。canary は権限応答を一切行わない。
- auth/CAS/owner/marker/#19/process custody の意味変更、Task.version への束縛。
- Held になった準備の回復経路の新設（現行契約のまま）。

### 受入条件

| ID | 受入条件 |
| --- | --- |
| AC-01 | phase 経路に版数リテラルとの比較による許可・拒否が無い。任意の版数文字列を返す模擬 CLI でも全検査が通れば NativeInput に到達し、従来ピンの番号でも検査が落ちれば拒否される |
| AC-02 | Claude（登録前 helper）: 相関する `initialize` 成功応答、helper 専用 UUID と cwd が一致する `system/init`、nonce と完全一致する `result`（`subtype=success`、`is_error=false`）を観測する。いずれかが欠けたら不合格 |
| AC-03 | Codex（登録前 helper 鎖）: 同一 canonical program の schema 生成で必須 method と必須 params が宣言され、かつ実応答で `initialize`、`account.type=chatgpt`、`environment/status=ready`、`account/rateLimits/read`、`thread/start`、`turn/start`、nonce 一致の `turn/completed` を観測する。宣言 schema は実応答の代替にしない |
| AC-04 | 必須応答の欠落、未知または読めないフレーム（Codex の未知 notification method を含む）、tool 要求、期限超過、対話完了前の終了、reap 不明は不合格。再試行も成功への丸めも無い |
| AC-05 | 不合格時は rrx Session 記録 0、`native_invocations` 0、`native_input`/`native_setup`/`native_permission` 効果 0、transport 登録 0。許容: 準備 readiness commit、`native_phase_*` helper の intent と settlement、確認用 helper プロセスの起動、helper 専用のモデル入力（A1 承認時のみ） |
| AC-06 | `rate_limit_event` の未観測を互換性の証明として記録しない。receipt は「codec selfcheck」と「実 event 観測」を別フィールドで持つ |
| AC-07 | 観測版と検査別結果が秘密なしで残る。アカウント識別子、token、プロバイダ本文、canary 応答本文、フレーム原文、stderr は残らない |
| AC-08 | helper は既存の権限鎖（admission 内 intent → 1 回 spawn → custody 保持 → Confirmed/Unknown settlement → closure CAS → SAME actor の ptr-eq）に入る。別 actor の観測も層 S の結果も `PreparedNativePhase` の conjunct にならない |
| AC-09 | probe 成功は permission/ownership/quota/Session を代替しない。権限応答規則（`execution/native.rs:2270, 2397`）、Git 対応検査、quota admission は不変 |
| AC-10 | テスト専用の権限 constructor、SQL seed、`allow(dead_code)`、skip/ignore、直列化、timeout 延長だけの修正を追加しない。producer の無い module を完成扱いしない |
| AC-11 | 旧宣言（旧 profile、`cli_version` あり）は型付きで拒否され、黙って受理・書換えされない |
| AC-12 | 実 Mac、1 Project／1 Active Task、QUICK、executor=Claude 2.1.296、reviewer=Codex 0.160.1 を設定した 1 レーンで、層 S が両 CLI とも合格、層 P（Claude executor）合格、Worktree → Implement 成功、`… typed Driver continuation unavailable (SC-N)`（`workflow/driven_settled.rs:44-49`）で停止する。Codex の確認は層 S の観測で示す |
| AC-13 | CI（`.github/workflows/ci.yml:12` の ubuntu/macos）が緑 |

### BLOCKED 条件

範囲を緩めて合格と呼ばないため、次を明記します。

- **A1 が未承認**: `system/init` と `result` は登録前に観測できない。M0 は BLOCKED。
- **A2 はオーナーが R1 を選択済み（2026-10-13）**: decoder の起動時 selfcheck と、実イベントが出た場合の厳密解析を必須にする。未観測は未観測として記録し、実イベント確認済みとは扱わない。
- **`can_use_tool` の往復**: canary は tool 要求を即不合格にするため観測しない。登録前に言えるのは「`--permission-prompt-tool stdio` を付けた起動で握手と canary が成立した」まで。

---

