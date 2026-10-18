# Issue #106 B1: base branch for Plan B

| 項目 | 結果 |
|---|---|
| 判定 | Plan B は origin/main を基準にする |
| 基準 | origin/main `79e4d273` |
| CI | `check (macos-latest)` success、`check (ubuntu-latest)` success（GitHub の check-runs で確認、2026-10-18） |
| 必要な部品 | すべて main にある（設計書 §0 の表）。M0 の branch にしかない部品で必須のものはない |
| M0 の branch | `feature/issue-5-m0` は U6a で一時停止し、そのまま残す。U6b まで失敗する 62 件のテストは Plan B の branch に入らない |
| schema | main は v11。Plan B は v12 を足す（設計書 §2）。M0 を再開するときは M0 側の版を振り直す |
