# Rust と Zig の実装リフレクション

対象にしたのは `crates/browser-tester/src/lib.rs`, `crates/bt-runtime/src/lib.rs`, `doc/subsystem-map.md`, `doc/capability-matrix.md`, `doc/mock-guide.md`, `zig/src/root.zig`, `zig/src/harness.zig`, `zig/src/session.zig`, `zig/src/mocks.zig`。

## 良かった点

- `Harness` を薄い facade に保ち、状態を `Session` / `DomStore` / `ScriptRuntime` / `MockRegistry` に分けたのは良かった。Rust は subsystem map で置き場を先に決めるので、公開 API を増やす前の判断基準がぶれにくい。
- モックを family 単位で切り、seed / capture / failure / reset を揃えたのは良かった。Rust の `MockRegistryView` と `zig/` の `MockRegistry` は、テストの可読性を保ちながらブラウザ依存を閉じ込めている。
- `zig/` の `Session` はコピー済み状態を arena で一元管理していて、ライフタイムと再現性が分かりやすい。Rust でも同じ考え方が貫かれており、実装意図が揃っている。
- phase 単位の contract / regression テストが厚く、実装した機能の境界がそのままテストになっている。移植と回帰確認がしやすい。
- Rust の capability matrix と mock guide が、何を stable にするか、何を test-only に留めるかを明文化している。

## 改善点

- Rust の `bt-runtime/src/lib.rs` と `zig/` の `session.zig` は責務が広い。navigation/history、collections、form state、serialization、scheduler のような塊でさらに切ると、変更影響を追いやすい。
- Rust の builder が一部の mock 設定を隠しキー付きの `local_storage` に詰めているのは、動くが少し分かりにくい。`zig/` の `SessionConfig` のように専用フィールドで持つ方が、後から見たときに意味が明確になる。
- `zig/` は `root.zig` から低レベル型を多く再exportしているので、stable API と test-only / debug API の境界が曖昧になりやすい。Rust の `DebugView` / `MockRegistryView` のような薄い facade を増やした方がよい。
- テストは多いが、`zig/` は単一ファイルに寄りやすい。Rust のように phase / concern ごとにファイルを分けると、追跡と差分確認が楽になる。
- mock family ごとの命名、reset、失敗メッセージにはまだ少しばらつきがある。`respond` / `fail` / `calls` / `reset` の形を揃えるほど、API の予測可能性が上がる。
- README / capability matrix / mock guide は充実しているが、説明の重複が多い。将来は単一の機能一覧から生成する形に寄せると、更新漏れを減らせる。

## 総評

- 方針は妥当で、特に「公開面を薄く保つ」「状態を内部に閉じ込める」「モックを型付きの family に分ける」の 3 点は今後も維持したい。
- 次の改善は、巨大化しつつある runtime / session の分割と、公開 API と test-only API の境界のさらなる明確化。
