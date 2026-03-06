# HTML仕様準拠ロードマップ: 最初の一歩 TODO

最初の着手対象は `P0: Parsing, Tree Construction, and Serialization` のうち、既存 API と既存テストをそのまま活用しやすい `fragment parsing + serialization` に絞る。

## 今回のゴール

- `html-standard.txt` の章番号を根拠に、最初の仕様差分を 1 つ以上特定する
- その差分を再現する failing test を追加できる状態にする
- 実装対象を `innerHTML` / `outerHTML` / `insertAdjacentHTML` / `setHTMLUnsafe` 周辺の fragment parsing に固定する

## TODO

- [x] `html-standard.txt` から次の節を確認し、最初の監査対象としてメモする
  - `8.5 DOM parsing and serialization APIs`
  - `8.5.2 Unsafe HTML parsing methods`
  - `13.2.4.1 The insertion mode`
  - `13.2.6.4.7 The "in body" insertion mode`
  - `13.2.6.4.9 The "in table" insertion mode`
  - `13.2.6.4.13` から `13.2.6.4.15` までの table body / row / cell insertion modes

- [x] 既存の関連実装とテストを読み、最初の変更対象ファイルを固定する
  - 実装:
    - `src/core_impl/dom/text_html_content.rs`
    - `src/core_impl/parser`
  - 既存テスト:
    - `src/tests/dom_element_outer_html_property.rs`
    - `src/tests/dom_element_insert_adjacent_html_method.rs`
    - `src/tests/dom_element_set_html_unsafe_method.rs`
    - `src/tests/dom_table_element.rs`
    - `src/tests/dom_tbody_element.rs`
    - `src/tests/dom_tr_element.rs`
    - `src/tests/dom_td_element.rs`

- [x] `todo.md` か issue メモに、最初の traceability 行を 3 件以上作る
  - 列は `Spec section / Repo surface / Current coverage / Missing behavior / Required mock / Acceptance test`
  - 最低 1 件は table context の fragment parsing を入れる
  - 最低 1 件は serialization or mutation restriction を入れる

- [x] 最初の実装スライスを次のどちらかに固定する
  - 第一候補: table context での `innerHTML` / `insertAdjacentHTML` の fragment parsing
  - 第二候補: `outerHTML` / `setHTMLUnsafe` の document child restriction と serialization edge case
  - 原則として、より failing test を作りやすい方を採用する

- [x] 最初の failing test を追加する前提で、検証シナリオを 2 から 4 件まで具体化する
  - 例:
    - `table.innerHTML` や `tbody.innerHTML` が table-specific insertion mode を通るか
    - `tr` / `td` 文脈で期待通りのノード階層が作られるか
    - `insertAdjacentHTML` が table 関連要素の前後で誤った sibling を作らないか
    - `outerHTML` が document child や detached node に対して既存仕様どおりに失敗または no-op になるか

- [x] 追加するテストには、対象の HTML 章番号を test 名かコメントに必ず残すルールを入れる
  - 例: `html_8_5_2_*`, `html_13_2_6_4_9_*`

- [x] deterministic mock が必要か先に判定する
  - この最初のスライスでは、原則として新規 mock を増やさない
  - もし新規 mock が必要と判明したら、同時に `README.md` の `Test Mocks` 追記も TODO に追加する

- [x] 実装前の実行コマンドを固定する
  - 対象テストのみ実行
  - `cargo test`
  - 必要なら `cargo test --test parser_property_fuzz_test --test runtime_property_fuzz_test`

## Traceability

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `8.5`, `13.2.6.4.9` | `src/core_impl/dom/text_html_content.rs` | `dom_table_element` に table 構造テストはあるが、HTML string insertion の table 文脈は未固定 | `table.innerHTML` / `table.insertAdjacentHTML()` が direct `tr` を `tbody` に正規化しない | none | `table_inner_html_html_8_5_13_2_6_4_9_*`, `table_insert_adjacent_html_html_13_2_6_4_9_*` | implemented |
| `8.5.2` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_set_html_unsafe_method.rs` | `setHTMLUnsafe` の sanitizer と declarative shadow root はある | table-specific fragment parsing が `setHTMLUnsafe()` でも正しく使われるか未固定 | none | table 文脈の `setHTMLUnsafe()` テスト追加 | next |
| `8.5`, `13.2.6.4.13`-`13.2.6.4.15` | `src/core_impl/parser`, `src/core_impl/dom/text_html_content.rs` | table / tbody / tr / td の静的テストはある | `tbody.innerHTML = "<td>..."` など row/cell context の fragment parsing が未固定 | none | `tbody` / `tr` / `td` 文脈の insertion mode テスト追加 | next |
| `8.5`, `8.5.2` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_outer_html_property.rs` | `outerHTML` の detached / document-child 制約は既存テストあり | table 親要素配下で `outerHTML` 置換後に direct `tr` が残らないことは未固定 | none | table 配下の `outerHTML` 置換テスト追加 | next |

## 最初の着手順

1. `8.5` / `8.5.2` / `13.2.6.4.x` の対象節を読む
2. 既存テストと実装を対応付ける
3. traceability 行を作る
4. 最初の 1 ギャップを選ぶ
5. spec section 付きの failing test を先に書く

## 完了条件

- 最初の対象ギャップが 1 つに決まっている
- 対応する spec section が章番号付きで記録されている
- 変更対象の実装ファイルとテストファイルが決まっている
- failing test のシナリオが具体化されている
- mock 追加の要否が判断されている

この TODO が終わった時点で、次の作業は「最初の failing test を追加して、最小実装に進む」になる。
