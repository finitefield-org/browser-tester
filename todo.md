# browser-tester 未実装タスク一覧

この一覧は現状の実装（`cargo test` 全件通過時点）から、次の実装対象を抽出したものです。

## 推奨実装順（先着順）

- [x] [S1] 最優先: `querySelectorAll` の実体化・再利用（NodeList化、変数格納、forEach 以外でも利用）
- [x] [S1] 最優先: 要素イベントとクラス操作系の実体拡張（`click`, `classList` 強化）
- [x] [S2] タイマーとスクリプト実行基盤の補強（タイマー引数/コメント/script抽出）
- [x] [S2] セレクタ機能の実用強化（追加の疑似クラス）
- [x] [S3] DOM/HTML 表現の仕様準拠補強（id 重複、イベント情報、style、エンティティ）
- [x] [S4] JS 実行モデルの高機能化（for/while 等、microtask）
- [x] [S5] 高度機能（`insertAdjacentHTML`）

- [x] [P0] `querySelectorAll` の結果を NodeList として汎用的に扱えるようにする（現在は `document.querySelectorAll(...).forEach(...)` 形式を想定した固定パスのみ）。`src/lib.rs:5570` (`Stmt::ForEach` / `parse_query_selector_all_foreach_stmt`) では直接呼び出し経路のみ対応。
- [x] [P0] 要素メソッド `element.click()` を JS から実行可能にする（`focus`/`blur` のみ実装）。`src/lib.rs:2796`, `src/lib.rs:5483`.
- [x] [P1] `DomQuery` の表現を拡張して、`querySelectorAll` の戻りを変数へ退避し再利用できるようにする（例: 代入・長さ取得・インデックス取得）。`src/lib.rs:2477`, `src/lib.rs:5570`, `src/lib.rs:5606`.
- [x] [P1] Script パーサに行コメント / ブロックコメント対応を統合する（`Cursor::skip_ws_and_comments` が未使用）。`src/lib.rs:8487`.
- [x] [P1] 属性セレクタを拡張する（`=`, `^=`, `$=`, `*=`, `~=`, `|=`）。`src/lib.rs:2349`（`parse_selector_attr_condition`）。
- [x] [P1] HTML パーサの `<script>` 抽出を厳密化する（`script` 本文中の見かけ上の `</script>` を誤認識しない、コメント・CDATA などへ拡張）。`src/lib.rs:8052`.
- [x] [P1] 重複する `id` 属性の取り扱いを仕様どおりに扱う（現在は上書き）。`src/lib.rs:145-150`.
- [x] [P1] `parse_set_timeout` / `setInterval` の引数仕様を拡張し、JS 側で一般的なコール形式に近づける（現在は callback + optional delay の固定）。`src/lib.rs:5956`.
- [x] [P1] イベントオブジェクト情報を拡張する（`timeStamp`, `eventPhase` 等の基本フィールド）。`src/lib.rs:7175`.
- [x] [P1] `classList` の `forEach` 系や `add/remove` の可変長引数・複数引数対応など、標準 API との差分を縮小する。`src/lib.rs:5754`.
- [x] [P1] `style` 値パースを改善する（現在は `;` と `:` の単純分割）。`src/lib.rs:1574`.
- [x] [P2] CSS セレクタへ `:where`, `:is`, `:has` を追加する。`src/lib.rs:1971`.
- [x] [P2] selector 属性値のエッジケース（空文字、エスケープ、ハイフン含みキー、ケース・ホワイトスペース扱い）を拡張する。`src/lib.rs:2349`.
- [x] [P2] `HTML エンティティ` と一部の文字参照（`&nbsp;` 等）を簡易デコードする。`src/lib.rs:8010`.
- [x] [P2] `for`/`while` 等の一般ループと `return`/`function` など、DOMイベント用途で増える可能性のある JS 構文を段階追加する。`src/lib.rs:6310` 付近の式/文パーサ。
- [x] [P2] microtask キュー/`Promise` 系の基本を導入し、`setTimeout` 系以外の非同期フローを模倣する。`src/lib.rs:4284` を含む実行エンジン全体。
- [x] [P3] `insertAdjacentHTML` を追加し、`innerHTML` の文字列差し替えだけでなく増分挿入をサポートする。`src/lib.rs:5850`（既存 `insertAdjacent*` と `set_inner_html`）。
