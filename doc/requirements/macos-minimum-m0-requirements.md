# macOS 最小一本道 M0: 要件

基準: `cb9c17e023601b3dbe721495bbdf50c9f65d92bb`。状態: **DRAFT / 修正差分の独立レビュー・オーナー設計承認前**。本番コードは変更していない。

M0 は #5/#6 の互換性変更（STRICT）、M1 は #95、M2 は #12、M3 は #9、M4 は #13、M5 は #16 に対応する。M0 の実 Mac 受入まで M1 の実装へ進まない。

本文執筆: Claude Opus 5.5 (`claude-opus-5-5`)、要求 effort high。CLI は同じ model を報告、effort の実測値は未報告。改訂4の本文を転記した。オーナー決定・静的事実の記録は orchestrator が実施した。いずれも独立レビューの代替ではない。

オーナー確定事項（2026-10-13）:

- 版数の厳密固定を capability probe に置換する。auth/CAS/owner/marker/#19/process custody は維持する。
- rate_limit_event: 起動時 decoder selfcheck、実イベントが出た場合は厳密解析。未観測は記録し、実イベント確認済みとは扱わない。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/5#issuecomment-6136393618)
- M4 を rrx 自身の自動マージまで拡張する。Draft PR → 必須 CI/review/safety/repository gate → ready → PR merge → 結果照合。[回答記録](https://github.com/shuhei-suzuki/rururunx/issues/13#issuecomment-6136256684)
- 試験用リポジトリは public の [shuhei-suzuki/rrx-e2e-sandbox](https://github.com/shuhei-suzuki/rrx-e2e-sandbox)。作成・使用承認済み。初期 README のみで、fixture・CI・main 保護は未設定。rururunx 本体で試運転しない。

初回の Sol・Opus 独立設計レビューは両方 REQUEST CHANGES。修正版の独立レビューは未完了。本文の未確認事項を承認済み・実証済みと扱わない。実装には設計承認、両 OS の検証、causal controls/mutants、独立実装・安全性レビューが必要。

### 目的

rrx の全起動経路から CLI の厳密バージョン一致を撤廃し、実観測で置換する。Claude 2.1.296 と codex-cli 0.160.1 は観測値として記録するだけで、番号を許可根拠にしない。使用する通信契約の有限で厳格な解析を維持する。

### スコープ

| ID | 内容 |
| --- | --- |
| S-1 | 版数ピンの全消費者の撤廃（phase 4 箇所、legacy 3 箇所） |
| S-2 | 層 S: daemon 起動時の非モデル probe（両 CLI）。専用 custody と journal。拒否専用 |
| S-3 | 層 P: 準備ごとの fresh な SAME actor 観測。Claude は quota Admit 後に nonce canary、Codex は schema と握手 |
| S-4 | Claude／Codex の共通 strict 解析、rate decoder の型分離（本番にも適用）、起動時 selfcheck |
| S-5 | 実行ファイル同一性の束縛 |
| S-6 | legacy 未合成経路の起動前 CapabilityUnavailable と、正例の phase レーンへの移植 |
| S-7 | 互換宣言の移行、監査保存、Mac スクリプト |

### 非スコープ

- M1 以降（#95 起票済み。M0 実機完了まで実装しない）。M4 は決定済み事項として記すだけ（rrx が Draft PR → 必須条件 → ready → merge → 照合、main 直 push 禁止、試験 repo は public `shuhei-suzuki/rrx-e2e-sandbox` を作成済み）。
- Codex のモデル canary、Codex executor の run、Codex review の実行（M3）。
- 自動復旧（#14）、`#10` を代替する権限付与、process custody の強化、#19 の変更。
- quota の外でのモデル呼出し、canary 結果の quota pool への書き戻し。
- テスト専用の権限 constructor、SQL seed による成功例、Task.version への束縛。

### 受入条件

| ID | 受入条件 |
| --- | --- |
| AC-01 | 本番コードに、版数リテラルとの比較による許可・拒否が残らない。印字可能な単一行である任意の版数文字列でも、全検査が通れば NativeInput に到達する |
| AC-02 | 層 S が、設定された全 Native agent について合格を公開するまで、Task admission は 0。S は Task／Unit／Session／NativeInput を作らず、モデルを呼ばない |
| AC-03 | 層 S の Codex: version、schema 構造述語、`initialize`、`initialized` 送信、`account.type=chatgpt`、`environment/status=ready`、`account/rateLimits/read` を実応答で観測。Claude: version と `initialize` の相関成功。S は `system/init`／`result` を観測したと記録しない |
| AC-04 | 層 P は S の結果で省略されない。Codex は SAME actor で S と同じ非モデル検査を行う。Claude は quota Admit の後・transport 登録の前に、`initialize`、`system/init`、nonce 完全一致の `result` を観測する |
| AC-05 | 未知または読めないフレーム（Claude の内部 variant、Codex の表外 method・不正 payload・不正な他 thread 通知）、同一 UUID の内容改変、Probe 中の tool 使用、成功後の未知フレーム・途中で切れた JSON・reap 不明は不合格。本番 consumer も未知・不正・改変を fail closed にする |
| AC-06 | rate limit: 起動時 selfcheck が通る。実 event は Probe でも本番でも strict decode し、解析不能は fail closed。未着だけを `not_observed` とし、観測済みや互換の証明として扱わない |
| AC-07 | モデル入力は既存 quota admission が Admit を返した後にだけ送る |
| AC-08 | 不合格時は Session 0、`native_invocations` 0、`native_input`／`native_setup`／`native_permission` 効果 0、transport 登録 0、helper 自身以外の `managed_effects` 行 0、IPC Grant 0 |
| AC-09 | 実行ファイルの同一性が、各層の全 helper と transport spawn の直前で一致する |
| AC-10 | helper は intent 先行、custody 保持、Confirmed／Unknown の settlement、SAME actor（層 S は SAME custody）の検査に従う。probe 成功は permission／ownership／quota／Session を代替しない |
| AC-11 | legacy 経路は helper も Session も作る前に CapabilityUnavailable を返す。失われる正例は phase producer を使うテストへ移植し、検出範囲を保つ |
| AC-12 | 旧 profile と `cli_version` ありの宣言は installation で型付きに拒否され、Unit 0・Held 0 |
| AC-13 | 実 Mac、1 Project／1 Active Task／QUICK、executor=Claude 2.1.296、reviewer=Codex 0.160.1 を設定した 1 run で、S が両 CLI 合格、P（Claude）合格、Worktree → Implement 成功、`… typed Driver continuation unavailable (SC-N)`（`workflow/driven_settled.rs:44-49`）で停止。Codex は S の実 metadata 結果で示す |
| AC-14 | 失敗した run は状態を保持して不合格とし、同じ状態ディレクトリの再実行をスクリプトが拒否する |
| AC-15 | skip／ignore、直列化、timeout 延長だけの修正、`allow(dead_code)`、producer の無い module の完成扱いが無い |
| AC-16 | CI（`.github/workflows/ci.yml:12` の ubuntu／macos）が緑 |

### 証明しないこと（完了報告に必ず載せる。条件緩和への同意ではない）

- Codex のモデル経路: `thread/start`、`turn/start`、通知、承認、error 分類、sandbox の実効。schema 構造と本番 strict parser で守るが、実成功は未観測。
- Codex review の実行。
- Claude の `can_use_tool` 往復、既定 permission モード、`rate_limit_event`（未着の run）。
- group を離れた子孫の不在、ラッパー型 CLI の内側の同一性。

### BLOCKED 条件

- §20 の A1 不承認: Claude の `system/init`／`result` を登録前に観測できない。
- 実フレームまたは schema 宣言に有限表外の variant／method がある: 表の見直しまで不合格。
- §21 の参照先定義が rrx の送信値と合わない: schema gate が通らない。

---
