# HTML仕様準拠ロードマップ: 次の着手 TODO

## 現在位置

- `P0: Parsing, Tree Construction, and Serialization` の table fragment / `outerHTML` slice は実装と実行系検証まで完了
- `P1.1: attribute reflection audit 拡張`（enumerated / URL / numeric 追加）は実装と検証まで完了
- `P1.2: reflection coverage hardening`（URL 棚卸し / enumerated invalid matrix / numeric clamp 拡張）は実装と検証まで完了
- `P1.3: reflection parity tightening`（missing-default / fast-path整合 / numeric監査拡張）は実装と検証まで完了
- `P1.4: reflection edge-case tightening`（formAction owner/default・min/maxLength境界・rows/cols上限・enumerated追加）は実装と検証まで完了
- `P1.5: reflection consistency sweep`（URL delimiter正規化・numeric validity相互作用・fast-path拡張）は実装と検証まで完了
- `P1.6: reflection matrix deepening`（URL special/opaque matrix・型別validity再評価・`min/max/step` fast-path）は実装と検証まで完了
- `P1.7: reflection semantics tightening`（default port / protocol switch / step-any / static bracket assignment）は実装と検証まで完了
- `P1.8: datetime/file-url precision sweep`（`datetime-local` 秒精度・`file:` protocol 切替・opaque/no-host setter）は実装と検証まで完了
- `P1.9: time/file-url setter parity finish`（`time` 小数秒・`file:` host setter・location no-op navigation）は実装と検証まで完了
- `P1.10: file-url parse/origin hardening`（invalid authority reject・file origin/document URL 正規化・mixed-case/location alias parity）は実装と検証まで完了
- `P1.11: URL invalid-input parity deepening`（generic invalid authority / port token・invalid anchor subproperty・protocol-relative base/fetch parity）は実装と検証まで完了
- `P1.12: hyperlink activation and special-host edge sweep`（invalid hyperlink activation no-op・special-host empty-host/backslash/hostless canonicalization・area/link null-URL parity）は実装と検証まで完了
- `P1.13: URL credential and delimiter encoding sweep`（credential delimiter encoding・special/non-special/file/opaque delimiter serialization・fetch/history/navigation canonical key parity）は実装と検証まで完了
- `P1.14: URL parser authority and opaque-path residual sweep`（raw `%` host reject・host percent-triplet decode・userinfo/path/query/hash bare `%` preservation・fetch credential reject parity）は実装と検証まで完了
- `P1.15: URLSearchParams malformed-percent and host-code-point sweep`（forgiving query decode・searchParams live sync `%zz` round-trip・fullwidth ASCII host fold・unsupported unicode host reject parity）は実装と検証まで完了
- `P1.16: IDNA host parity and searchParams live-mutation sweep`（Unicode host punycode parity・dot variant / combining mark canonicalization・duplicate malformed-percent live mutation・member dispatch overlap hardening）は実装と検証まで完了
- `P1.17: IDNA invalid-label and overlapping-dispatch residual sweep`（invalid punycode / joiner / trailing-dot parity・extra-arg evaluation・DOM/FormData/Map overlap dispatch）は実装と検証まで完了
- `P1.18: file-host/arity parity residual sweep`（`file:` + IDNA mixed host parity・URL/URLSearchParams/FormData extra-arg ignore/evaluation・location/history/document URL sync）は実装と検証まで完了
- `P1.19: generic member-call collection/WebIDL parity sweep`（Map/Set/WeakMap/WeakSet/Storage object-path parity・URLSearchParams member-chain iterator parity・FormData chain extra-arg parity）は実装と検証まで完了
- `P1.20: prototype/property-access parity residual sweep`（extracted/prototype method `.call()` parity・receiver builtin dispatch・URL/URLSearchParams property path補強）は実装と検証まで完了
- `P1.21: raw string-wrapper getter and inherited receiver residual sweep`（raw bracket getter parity・primitive/location receiver builtin・incompatible receiver 回帰）は実装と検証まで完了
- `P1.22: array/string iterator and boxed-prototype property parity sweep`（array/string/typed array/NodeList raw getter・collection `Symbol.iterator` property path・boxed primitive `constructor.prototype` 露出）は実装と検証まで完了
- `P1.23: constructor identity and raw-getter breadth sweep`（`Number`/`BigInt`/`Symbol` global constructor exposure・primitive constructor identity・string/typed array raw getter breadth 拡張）は実装と検証まで完了
- `P1.24: stable constructor prototype identity and static bracket-access sweep`（stable `prototype` identity・typed array constructor first-class exposure・static bracket/property path parity）は実装と検証まで完了

## 今回スライスの実施結果（P1.24: stable constructor prototype identity and static bracket-access sweep）

- [x] stable constructor `prototype` identity を固定した
  - `String` / `Symbol` / typed array constructor の `prototype` は runtime cache を通すようにし、`Constructor.prototype === Constructor.prototype` と `value.constructor.prototype === Constructor.prototype` を stable にした
  - concrete typed array constructor は first-class value として global/window に露出し、instance / property path / object literal 経由でも同じ constructor/prototype identity を返すようにした

- [x] static bracket/property path parity を詰めた
  - `Number['parseInt']` / `BigInt['asIntN']` / `String['fromCodePoint']` / `Symbol['for']` / typed array static members を constructor object 側へ exposed し、special AST と同じ callable / constant surface に揃えた
  - computed bracket-call 用 parser fallback を receiver-aware に寄せつつ `obj['m'](...)` だけを拾うように締め、通常の dot call / ASI hazard / invalid optional chaining を壊さないようにした

- [x] DOM text の visible rendering も揃えた
  - JS 内部の surrogate marker 表現は維持したまま、harness の `assert_text` / `dump_dom` / snippet 出力では display 用に externalize し、emoji を含む static bracket 回帰がそのまま読める形にした

- [x] 回帰テストを広げた
  - `src/tests/collections_url_typed_arrays.rs`
  - `constructor_static_bracket_and_property_path_work`
  - `src/tests/webapi_data_builtins.rs`
  - `stable_constructor_prototype_identity_and_symbol_bracket_access_work`
  - 既存の `language_core_expressions` の ASI / optional chaining invalid-syntax 回帰も含めて computed-call parser 境界を維持した

- [x] 検証完了
  - `cargo test --lib stable_constructor_prototype_identity_and_symbol_bracket_access_work`
  - `cargo test --lib constructor_static_bracket_and_property_path_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2205 passed, 0 failed`)

- [x] 新規 mock 不要を確認（README 追記なし）

## Traceability

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `8.5`, `13.2.6.4.9` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_outer_html_property.rs` | table 親配下 `outerHTML` 置換・table context 補正・回帰テストを固定済み | なし | none | `element_outer_html_set_html_8_5_13_2_6_4_9_*`, `element_outer_html_set_html_8_5_13_2_6_4_13_*` | implemented + verified |
| `2.3.1` | shared reflection helper + assignment paths | boolean reflected attribute の presence semantics を shared helper に集約済み | 特になし（維持フェーズ） | none | `attribute_reflection_html_2_3_1_*` | implemented + verified |
| `2.3.2` | shared reflection helper + getter/setter paths | `draggable`/`spellcheck`/`translate` に加え `dir` / `autocapitalize` / `autocomplete` の missing/invalid/case-variant を shared テストで固定済み | form関連 enumerated（`form.autocomplete` など）の owner/default 相互作用監査は継続余地 | none | `attribute_reflection_html_2_3_2_*` | implemented + verified |
| `2.3.3` | shared reflection helper + getter/setter + fast-path paths | `datetime-local` に加え `time` の fractional-second precision / millisecond step / wrapped range まで回帰化済み | `time` の token edge cases（過剰精度・境界トークン）と `step='any'` の追加監査は継続余地 | none | `attribute_reflection_html_2_3_3_*`, `html_input_datetime_local_*`, `html_input_time_*` | implemented + verified |
| `2.6.1` | shared reflection helper + URL getter/setter paths | default port 正規化 + special/file/opaque protocol switch + file/generic invalid authority reject + invalid absolute anchor subproperty semantics + protocol-relative base/fetch/navigation parity + invalid hyperlink activation no-op + special-host empty-host/backslash/hostless canonicalization + area/link null-URL getter parity + credential/delimiter encoding matrix + authority raw `%` reject / host percent-triplet decode / bare `%` preservation + malformed-percent searchParams decode + true IDNA/punycode host canonicalization + invalid-label reject/no-op + trailing-dot/full-stop variant parity + `file:` mixed IDNA host/location/history/document URL parity + URL/URLSearchParams/FormData/Map/Set/Storage object-path extra-arg ignore/evaluation + extracted/prototype `.call()` parity + raw URL/location getter parity + primitive/collection inherited receiver parity + array/string/typed array/NodeList raw getter・collection `Symbol.iterator` property path・boxed primitive `constructor.prototype` parity + `Number`/`BigInt`/`Symbol` global constructor exposure + stable `String` / `Symbol` / typed array `prototype` identity + static bracket/property path parity まで固定済み | generic computed-call syntax（dynamic key / chained call）と constructor surface alias/identity edge の追加監査は継続余地 | none | `attribute_reflection_html_2_6_1_*`, `url_*matrix_work`, `location_*no_op*_work`, `*_special_host_*`, `*_null_url_*`, `*_credentials_*`, `*_authority_and_percent_*`, `*_malformed_query_and_host_code_point_*`, `fetch_*canonical_mock_key*`, `fetch_*residuals*`, `form_data_*extra_args*`, `collection_member_chain_and_extra_arg_parity_work`, `collection_extracted_method_call_and_prototype_parity_work`, `storage_extracted_method_call_parity_work`, `form_data_extracted_method_call_parity_work`, `raw_url_location_getter_and_collection_bracket_parity_work`, `primitive_raw_getter_and_incompatible_receiver_work`, `array_typed_array_and_collection_iterator_property_paths_work`, `string_nodelist_and_boxed_prototype_property_paths_work`, `form_data_symbol_iterator_property_path_work`, `constructor_identity_and_string_raw_getter_breadth_work`, `typed_array_raw_getter_breadth_and_constructor_prototype_work`, `stable_constructor_prototype_identity_and_symbol_bracket_access_work`, `constructor_static_bracket_and_property_path_work` | implemented + verified |

## 次のタスク（P1.25: computed-call parser and constructor surface alias residual sweep）

- [ ] generic computed-call parser の残差を詰める
  - `obj[key](...)` の dynamic key path、call-result chain、`new` callee 周りの bracket/property path が ASI / optional chaining invalid-syntax を壊さず通るか監査する
  - 既存の dot member call fast-path と computed fallback の責務を整理し、receiver 維持と syntax reject の境界を回帰で固定する

- [ ] constructor surface alias/identity の残差を詰める
  - constructor static member object identity、well-known symbol alias、global/window/worker 露出差分、typed array constructor alias path の残りを監査する
  - callable constructor object と variant-backed constructor の二重表現でまだ special-case が必要な箇所を洗い、property path をさらに共通化する

- [ ] 検証する
  - 追加した targeted tests（computed-call parser / constructor alias residual）
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib`
