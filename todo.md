# HTML仕様準拠ロードマップ: 次の着手 TODO

## 現在位置

- `P0: Parsing, Tree Construction, and Serialization` の最初の slice として、table context における `innerHTML` / `insertAdjacentHTML` の direct `tr` 正規化は実装済み
- `cargo test --lib` は green
- 次は同じ P0 の残件として、`setHTMLUnsafe(table)` と `tbody` / `tr` / `td` 文脈の fragment parsing を固定する

## 今回のゴール

- `8.5.2 Unsafe HTML parsing methods` と `13.2.6.4.13` から `13.2.6.4.15` までを次の監査対象として固定する
- `setHTMLUnsafe` と table row/cell context の挙動を、spec section 付きの failing test で先に固める
- table 関連の fragment parsing を API ごとの個別対応ではなく、共有アルゴリズムとして寄せる

## TODO

- [ ] `html-standard.txt` から次の節を再確認し、期待挙動をテストメモに落とす
  - `8.5.2 Unsafe HTML parsing methods`
  - `13.2.6.4.13 The "in table body" insertion mode`
  - `13.2.6.4.14 The "in row" insertion mode`
  - `13.2.6.4.15 The "in cell" insertion mode`
  - 必要なら `13.2.6.4.9 The "in table" insertion mode` を再参照して、既存 table 正規化との一貫性を確認する

- [ ] 次の targeted test 追加先を固定する
  - `src/tests/dom_element_set_html_unsafe_method.rs`
  - `src/tests/dom_tbody_element.rs`
  - `src/tests/dom_tr_element.rs`
  - `src/tests/dom_td_element.rs`
  - 必要なら `src/tests/dom_table_element.rs` と `src/tests/dom_element_outer_html_property.rs`

- [ ] 先に再現するギャップを 2 系統に分ける
  - 系統 A: `table.setHTMLUnsafe("<tr>...")` が table-specific fragment parsing を通るか
  - 系統 B: `tbody` / `tr` / `td` 文脈で row/cell-start tag を含む HTML 文字列が正しい階層になるか

- [ ] failing test の候補を 4 件以内に絞る
  - `setHTMLUnsafe(table)` で direct `tr` を残さないケース
  - `tbody.innerHTML` が table body insertion mode を通るケース
  - `tr.innerHTML` が row insertion mode を通るケース
  - `td.innerHTML` が cell insertion mode を通るケース

- [ ] 追加するテストに spec section を必ず残す
  - test 名かコメントで `html_8_5_2_*`, `html_13_2_6_4_13_*`, `html_13_2_6_4_14_*`, `html_13_2_6_4_15_*` を使う

- [ ] 実装方針を先に決める
  - `setHTMLUnsafe` だけの個別パッチにしない
  - `innerHTML` / `outerHTML` / `insertAdjacentHTML` / `setHTMLUnsafe` が同じ fragment parsing か同じ post-parse fixup を通る構造を優先する
  - row/cell context を post-parse 正規化では吸収できない場合は、`src/core_impl/parser` 側の fragment context を先に直す

- [ ] 新規 mock の要否を判定する
  - この slice では原則として不要
  - もし test-only mock を増やすなら、同時に `README.md` の `Test Mocks` 節へ利用方法を追記する

- [ ] 実装後の検証コマンドを固定する
  - `cargo test --lib dom_element_set_html_unsafe_method`
  - `cargo test --lib dom_tbody_element`
  - `cargo test --lib dom_tr_element`
  - `cargo test --lib dom_td_element`
  - 必要なら `cargo test --lib dom_table_element`
  - 最後に `cargo test --lib`
  - parser を触った場合は property/fuzz も確認する

## Traceability

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `8.5`, `13.2.6.4.9` | `src/core_impl/dom/text_html_content.rs` | `dom_table_element` に table context の HTML string insertion 回帰テストを追加済み | `table.innerHTML` / `table.insertAdjacentHTML()` の direct `tr` 正規化 | none | `table_inner_html_html_8_5_13_2_6_4_9_*`, `table_insert_adjacent_html_html_13_2_6_4_9_*` | implemented |
| `8.5.2` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_set_html_unsafe_method.rs` | `setHTMLUnsafe` の sanitizer / declarative shadow root テストはある | `setHTMLUnsafe(table)` が table-specific fragment parsing を通るか未固定 | none | `set_html_unsafe_table_html_8_5_2_*` | active |
| `8.5`, `13.2.6.4.13`-`13.2.6.4.15` | `src/core_impl/parser`, `src/core_impl/dom/text_html_content.rs` | table 系要素の静的構造テストはある | `tbody` / `tr` / `td` 文脈の fragment parsing が未固定 | none | `tbody_inner_html_html_13_2_6_4_13_*`, `tr_inner_html_html_13_2_6_4_14_*`, `td_inner_html_html_13_2_6_4_15_*` | active |
| `8.5`, `8.5.2` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_outer_html_property.rs` | detached node / document child 制約の既存テストはある | table 親配下の `outerHTML` 置換後に direct `tr` が残らないことは未固定 | none | table 配下の `outerHTML` 置換テスト | next |
| `2.3`, `2.6.1` | shared attribute reflection helpers | 個別要素テストは多い | boolean / enumerated / numeric / URL reflection の shared audit は未着手 | none | shared microsyntax/reflection test set | queued |

## 次の着手順

1. `8.5.2` と `13.2.6.4.13` から `13.2.6.4.15` の期待挙動を読む
2. `setHTMLUnsafe(table)` と `tbody` / `tr` / `td` の現在挙動を最小ケースで再現する
3. spec section 付きの failing test を先に追加する
4. fragment parsing の共有経路を決めて最小実装を入れる
5. targeted test と `cargo test --lib` を回す

## 完了条件

- `setHTMLUnsafe(table)` の期待挙動が spec section 付きテストで固定されている
- `tbody` / `tr` / `td` 文脈の fragment parsing が少なくとも 1 ケースずつテスト化されている
- API ごとの差分ではなく、共有アルゴリズムとして実装箇所が定まっている
- `cargo test --lib` が通る

この TODO が終わった時点で、次の作業は「table 親配下の `outerHTML` 置換と、その後の P1 attribute reflection audit」に進む。
