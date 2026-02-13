# browser-tester 未実装タスク一覧

この一覧は実装上の未完了項目およびテストカバレッジ不足を示します。

## 優先度: S1

- [x] `for ... in` / `for ... of` ループに対応し、`parse_for_stmt` の `for (;;)` のみ扱いを拡張する。`forEach` を使わないループ用途の `for` 文系動作テストも追加する。
- [x] `break` / `continue` 文の構文と実行フローを追加し、`for`/`while`/`forEach`/`while` の制御移譲テストを追加する。
- [x] `do ... while` を実装し、`do ... while` 固有の構文分解と実行テストを追加する。
- [x] アロー関数の簡易本体（`=> expr`）を受け付けるようにし、`parse_callback` / `parse_for_each_callback` / タイマーハンドラ経路に式ボディのテストを追加する。

## 優先度: S2

- [x] `setTimeout`/`setInterval` の callback 引数形式を拡張し、関数リファレンスや追加引数形の許容を実装する。現状で未対応の指定形に対するパース/実行テストを追加する。
- [x] `setTimeout` / `setInterval` のコールバック引数上限や引数解決ロジックを一般化し、引数不足・追加引数・可変引数ケースを網羅する。
- [x] `parse_element_target` のインデックス参照を数値リテラル固定から拡張し、変数や式ベースのインデックス（`list[i]`）を追加する。
- [x] `parse_document_element_call` の対応メソッドを拡張し、`document.getElementsBy...` 系などの基本DOM取得 APIの追加実装とテストを追加する。

## 優先度: S3

- [x] `parse_dom_method_call_stmt` のメソッド網羅を拡大し、`click` 以外の主要DOM API（`scrollIntoView` 等）を追加。対応したDOM操作テストを追加する。
- [x] イベントオブジェクト参照を `type/target/currentTarget` 以外へ拡張し（例: `defaultPrevented`, `isTrusted`, `target` 系サブプロパティ）、`EventMethod` と組み合わせたイベント関連テストを追加する。
- [x] HTML エンティティデコード(`decode_html_character_references`)の名前付き参照を既定参照リストへ拡張し、既知/未知参照の挙動テストを追加する。
