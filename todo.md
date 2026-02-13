# browser-tester 未実装タスク一覧

この一覧は実装上の未完了項目およびテストカバレッジ不足を示します。

## 優先度: S1

- [ ] イベント参照の `bubbles` / `cancelable` を実装する。`EventExprProp` と `EventState` に追加し、`event.bubbles` / `event.cancelable` の式評価と既定動作判定をテストする。
- [ ] `dispatchEvent` 由来のイベントの `isTrusted` を未信頼状態として扱う。現在は `EventState::new` が常に `is_trusted = true` を返すため、`dispatchEvent` 経路での `event.isTrusted` 挙動を実装し、既存の `EventMethod` と組み合わせて検証する。

## 優先度: S2

- [ ] `HTMLFormElement` の `submit` / `reset` など、代表的フォーム関連DOMメソッドを DOM method ステートメントで追加し、`addEventListener`/`dispatchEvent` 連携を含むテストを追加する。

## 優先度: S3

- [ ] `decode_html_character_references` の既知名前付き参照カバレッジを拡張する（不足しがちな `pound`, `times`, `divide`, `laquo`, `raquo`, `frac*` 系など）。既知/未知参照の境界ケースが落ちないよう、追加ケースの回帰テストを追加する。
