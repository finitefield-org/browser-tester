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
- `P1.25: computed-call parser and constructor surface alias residual sweep`（dynamic computed call receiver 維持・constructor static identity・worker constructor alias surface）は実装と検証まで完了
- `P1.26: constructor function-surface identity and worker breadth sweep`（constructor `call/apply/bind/toString/name/length` parity・grouped/new callee 境界・worker/global/window core constructor exposure）は実装と検証まで完了
- `P1.27: constructor raw-static/prototype breadth and bound-new residual sweep`（`RegExp` / `Promise` / `ArrayBuffer` / `Blob` raw static/property path・stable prototype cache・bound constructor `instanceof` residual）は実装と検証まで完了
- `P1.28: builtin prototype-chain and bound callable surface sweep` is implemented and verified
- `P1.29: function/object prototype-chain and callable metadata residual sweep` is implemented and verified
- `P1.30: global Function exposure and generator-family constructor surface sweep` is implemented and verified

## 今回スライスの実施結果（P1.27: constructor raw-static/prototype breadth and bound-new residual sweep）

- [x] constructor raw static/property path を広げた
  - `RegExp` を core constructor binding に追加し、main env / `window` / worker global から `globalThis['RegExp']` として読めるようにした
  - `RegExp.escape`、`Promise.resolve/reject/all/allSettled/any/race/try/withResolvers`、`ArrayBuffer.isView` は static callable cache 経路に寄せ、dot/bracket/alias access の identity を揃えた

- [x] prototype raw getter と receiver builtin dispatch を補強した
  - `Blob` / `ArrayBuffer` / `Promise` / `RegExp` は stable `prototype` cache を持つようにし、`constructor.prototype.method.call(...)` と repeated `prototype` access の identity を固定した
  - instance raw getter は `Blob.text/arrayBuffer/bytes/stream/slice`、`ArrayBuffer.byteLength/maxByteLength/resizable/detached/slice/resize/transfer/transferToFixedLength`、`Promise.then/catch/finally`、`RegExp.exec/test/toString` を generic property path から返せるようにした

- [x] callable/new residual を詰めた
  - `RegExp` constructor value は alias 経由でも callable / constructable にし、`RegExpCtor('a', 'g')` と `new RegExpCtor(...)` の両方を generic dispatch で通すようにした
  - `instanceof` は bound function を target constructor へ unwrap するようにして、`new Foo.bind(... )()` の `instanceof Bound` を JS と同じ結果にした

- [x] 回帰テストを広げた
  - `src/tests/collections_url_typed_arrays.rs`
  - `constructor_raw_static_and_prototype_property_paths_work`
  - `src/tests/language_core_expressions.rs`
  - `bound_constructor_new_target_and_instanceof_work`
  - `src/tests/issue_121_127_finitefield_site_regressions.rs`
  - `regex_match_before_async_functions_does_not_break_following_await_flow` の期待値を promise raw getter 追加後の挙動へ更新した
  - 既存の `regexp_constructor_properties_and_escape_work`、`constructor_function_surface_and_global_bindings_work`、worker constructor surface 回帰も含めて breadth を維持した

- [x] 検証完了
  - `cargo test --lib constructor_raw_static_and_prototype_property_paths_work`
  - `cargo test --lib bound_constructor_new_target_and_instanceof_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2213 passed, 0 failed`)

- [x] 新規 mock 不要を確認（README 追記なし）

## Traceability

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `8.5`, `13.2.6.4.9` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_outer_html_property.rs` | table 親配下 `outerHTML` 置換・table context 補正・回帰テストを固定済み | なし | none | `element_outer_html_set_html_8_5_13_2_6_4_9_*`, `element_outer_html_set_html_8_5_13_2_6_4_13_*` | implemented + verified |
| `2.3.1` | shared reflection helper + assignment paths | boolean reflected attribute の presence semantics を shared helper に集約済み | 特になし（維持フェーズ） | none | `attribute_reflection_html_2_3_1_*` | implemented + verified |
| `2.3.2` | shared reflection helper + getter/setter paths | `draggable`/`spellcheck`/`translate` に加え `dir` / `autocapitalize` / `autocomplete` の missing/invalid/case-variant を shared テストで固定済み | form関連 enumerated（`form.autocomplete` など）の owner/default 相互作用監査は継続余地 | none | `attribute_reflection_html_2_3_2_*` | implemented + verified |
| `2.3.3` | shared reflection helper + getter/setter + fast-path paths | `datetime-local` に加え `time` の fractional-second precision / millisecond step / wrapped range まで回帰化済み | `time` の token edge cases（過剰精度・境界トークン）と `step='any'` の追加監査は継続余地 | none | `attribute_reflection_html_2_3_3_*`, `html_input_datetime_local_*`, `html_input_time_*` | implemented + verified |
| `2.6.1` | shared reflection helper + URL getter/setter paths | default port 正規化 + special/file/opaque protocol switch + file/generic invalid authority reject + invalid absolute anchor subproperty semantics + protocol-relative base/fetch/navigation parity + invalid hyperlink activation no-op + special-host empty-host/backslash/hostless canonicalization + area/link null-URL getter parity + credential/delimiter encoding matrix + authority raw `%` reject / host percent-triplet decode / bare `%` preservation + malformed-percent searchParams decode + true IDNA/punycode host canonicalization + invalid-label reject/no-op + trailing-dot/full-stop variant parity + `file:` mixed IDNA host/location/history/document URL parity + URL/URLSearchParams/FormData/Map/Set/Storage object-path extra-arg ignore/evaluation + extracted/prototype `.call()` parity + raw URL/location getter parity + primitive/collection inherited receiver parity + array/string/typed array/NodeList raw getter・collection `Symbol.iterator` property path・boxed primitive `constructor.prototype` parity + `Number`/`BigInt`/`Symbol` global constructor exposure + stable `String` / `Symbol` / typed array `prototype` identity + static bracket/property path parity + dynamic computed call receiver preservation + grouped/new optional-chain callee boundary + constructor `call/apply/bind/toString/name/length/prototype.constructor` parity + window/global/worker core constructor exposure (`Blob` / `URL` / `URLSearchParams` / `ArrayBuffer` / `Promise` / `Map` / `WeakMap` / `Set` / `WeakSet` / `RegExp`) + `Blob` / `ArrayBuffer` / `Promise` / `RegExp` raw static/property-path breadth + bound constructor `instanceof` unwrap まで固定済み | variant-backed builtin の internal prototype / `instanceof` / `Object.getPrototypeOf` parity と bound function `name`/`length`/`prototype` surface は継続余地 | none | `attribute_reflection_html_2_6_1_*`, `url_*matrix_work`, `location_*no_op*_work`, `*_special_host_*`, `*_null_url_*`, `*_credentials_*`, `*_authority_and_percent_*`, `*_malformed_query_and_host_code_point_*`, `fetch_*canonical_mock_key*`, `fetch_*residuals*`, `form_data_*extra_args*`, `collection_member_chain_and_extra_arg_parity_work`, `collection_extracted_method_call_and_prototype_parity_work`, `storage_extracted_method_call_parity_work`, `form_data_extracted_method_call_parity_work`, `raw_url_location_getter_and_collection_bracket_parity_work`, `primitive_raw_getter_and_incompatible_receiver_work`, `array_typed_array_and_collection_iterator_property_paths_work`, `string_nodelist_and_boxed_prototype_property_paths_work`, `form_data_symbol_iterator_property_path_work`, `constructor_identity_and_string_raw_getter_breadth_work`, `typed_array_raw_getter_breadth_and_constructor_prototype_work`, `stable_constructor_prototype_identity_and_symbol_bracket_access_work`, `constructor_static_bracket_and_property_path_work`, `computed_calls_preserve_receiver_across_dynamic_keys_and_super`, `constructor_static_identity_and_new_callee_alias_paths_work`, `constructor_function_surface_and_global_bindings_work`, `new_operator_supports_grouped_computed_and_optional_chain_callee`, `constructor_raw_static_and_prototype_property_paths_work`, `bound_constructor_new_target_and_instanceof_work`, `worker_global_exposes_constructor_aliases_and_static_method_identity`, `worker_global_exposes_constructor_surface_breadth` | implemented + verified |

## Completed Task (P1.28: builtin prototype-chain and bound callable surface sweep)

- [x] Align variant-backed builtin prototype chains and `instanceof`
  - `Blob` / `ArrayBuffer` / `Promise` / `RegExp` / `Map` / `WeakMap` / `Set` / `WeakSet` / `URLSearchParams` / `URL` / typed array instances now resolve stable cached constructor `prototype` objects through `Object.getPrototypeOf`, inherited `constructor`, and `instanceof`
  - concrete typed array prototypes now chain through the abstract `TypedArray` prototype, and object-backed `URL` / `URLSearchParams` instances use the same prototype lookup path as variant-backed builtins

- [x] Align bound callable surface
  - `Function.prototype.bind` callables now expose `name` / `length` consistently, keep `prototype` as `undefined`, and avoid inheriting builtin static methods by accident
  - main realm and worker constructor aliases now share the same bound surface and `instanceof` behavior

- [x] Verification completed
  - `cargo test --lib builtin_instanceof_and_object_get_prototype_of_parity_work`
  - `cargo test --lib bound_callable_name_length_and_static_surface_work`
  - `cargo test --lib worker_bound_builtin_constructor_surface_and_instanceof_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2216 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.29: function/object prototype-chain and callable metadata residual sweep)

- [x] Finish function/object prototype-chain parity
  - plain objects now fall back to the shared `Object.prototype`, with inherited `constructor` and `instanceof Object` behavior matching callable and non-callable objects instead of returning placeholder prototype objects
  - callable objects, variant-backed constructors, bound functions, and ordinary functions now share a cached hidden `Function.prototype`, and worker constructor bindings reuse the same `Object` constructor identity as the main realm

- [x] Deepen callable metadata coverage
  - ordinary function declarations, class constructors, and `new Function(...)` results now expose stable `.name` / `.length` metadata, while extracted builtin constructors inherit `constructor` from the shared function prototype chain
  - callable constructor/property lookup now falls through to the function prototype chain instead of bypassing generic lookup when own `.length` / `.name` / `constructor` handling misses

- [x] Verification completed
  - `cargo test --lib function_and_object_prototype_chain_and_constructor_metadata_work`
  - `cargo test --lib function_constructor_name_and_callable_prototype_chain_work`
  - `cargo test --lib worker_function_object_prototype_chain_and_metadata_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2219 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.30: global Function exposure and generator-family constructor surface sweep)

- [x] Expose stable function-family constructors across realms
  - `Function`, `GeneratorFunction`, and `AsyncGeneratorFunction` are now surfaced directly on the main realm, `window`, and worker globals through shared constructor bindings instead of hidden one-off objects
  - constructor identity, `.prototype`, `Object.getPrototypeOf`, and callable metadata are aligned across the main realm and worker bootstrap paths

- [x] Deepen ordinary/generator-family function surface parity
  - ordinary functions now repair their public `prototype` object links so `prototype.constructor`, `Object.getPrototypeOf(prototype)`, and named function-expression aliases stay consistent after extraction and rebinding
  - generator-family constructor outputs now expose stable constructor/prototype chains, `"anonymous"` naming for constructor-built functions, and non-enumerable `constructor` behavior through the shared enumerable-key filters

- [x] Verification completed
  - `cargo test --lib global_function_constructor_and_ordinary_function_prototype_links_work`
  - `cargo test --lib generator_function_helpers`
  - `cargo test --lib async_generator_function_helpers`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo test --lib for_in_loop_includes_inherited_properties_and_skips_shadowed_keys`
  - `cargo fmt`
  - `cargo test --lib` (`2223 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.31: native callable source-text and function-prototype descriptor sweep)

- [x] Align native callable source-text breadth
  - shared callable source-text generation now covers ordinary functions, bound callables, builtin constructors, and function-family constructors so `.toString()`, `Function.prototype.toString.call(...)`, `String(...)`, and alias/bracket access paths return stable native text
  - variant-backed constructors and object-backed callables now use the same source-text path across the main realm and worker globals, closing parity gaps for `Function`, `GeneratorFunction`, `AsyncGeneratorFunction`, `Map`, `URL`, `URLSearchParams`, `ArrayBuffer`, `Promise`, `RegExp`, and `Blob`

- [x] Deepen function/generator prototype descriptor parity
  - non-enumerable property tracking now supports generic property keys instead of only `constructor`, and the shared constructor/prototype builders mark exposed surface properties as hidden where required
  - `Function.prototype`, ordinary function prototype objects, generator-family constructor/prototype objects, and iterator-adjacent generator prototypes now stay aligned for `Object.keys`, spread, `for...in`, and `JSON.stringify`

- [x] Verification completed
  - `cargo test --lib native_function_source_text_and_prototype_enumerability_work`
  - `cargo test --lib native_variant_backed_constructor_source_text_is_stable_across_paths_work`
  - `cargo test --lib generator_function_helpers`
  - `cargo test --lib async_generator_function_helpers`
  - `cargo test --lib worker_global_function_family_constructors_are_exposed_and_callable_work`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2227 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.32: callable string-coercion breadth and descriptor residual sweep)

- [ ] Expand callable string-coercion parity outside direct constructor paths
  - audit remaining generic stringification/coercion sites so callable values reached through concatenation, template interpolation, and indirect coercion use the same native/user-defined source-text rules
  - verify object-backed callables and bound wrappers stay aligned when coerced through shared utility paths instead of specialized constructor logic

- [ ] Close remaining descriptor gaps on constructor and prototype surfaces
  - sweep object-backed constructors and prototype objects that still rely on ad hoc enumerable-property behavior, especially around static methods, `.prototype`, and inherited `constructor` exposure
  - verify descriptor visibility stays stable for `Object.keys`, spread, `JSON.stringify`, and `for...in` after alias access and prototype repair

- [ ] Verify
  - targeted tests for indirect callable string coercion and remaining constructor/prototype descriptor visibility
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib generator_function_helpers`
  - `cargo test --lib async_generator_function_helpers`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo test --lib`
