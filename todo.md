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

## Completed Task (P1.32: callable string-coercion breadth and descriptor residual sweep)

- [x] Expand callable string-coercion parity outside direct constructor paths
  - shared callable-aware string coercion now covers array joins, string concatenation, `String.raw`, and string replacement callback results so variant-backed constructors, object-backed host callables, and bound functions use the same source-text path in indirect string contexts
  - indirect coercion for callable values no longer falls back to raw `Value::as_string()` output such as `[object Object]` or constructor short names when the source-text path should be used instead

- [x] Close remaining descriptor gaps on constructor and prototype surfaces
  - object-backed constructor builders now hide own `.prototype` and prototype-side `constructor` consistently, and constructor surfaces with static methods or constants use the shared non-enumerable marker path instead of ad hoc public entries
  - `Event`, `KeyboardEvent`, `WheelEvent`, `Document`, `TextEncoder`, `File`, and related object-backed constructors/prototypes now stay aligned for `Object.keys`, spread, `JSON.stringify`, and `for...in`

- [x] Verification completed
  - `cargo test --lib callable_string_coercion_uses_source_text_across_indirect_string_contexts_work`
  - `cargo test --lib object_backed_constructor_descriptor_visibility_stays_hidden_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib generator_function_helpers`
  - `cargo test --lib async_generator_function_helpers`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2229 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.33: object-backed callable naming and residual string-coercion sweep)

- [x] Align object-backed callable naming and metadata surfaces
  - object-backed host constructors and host callables now use shared callable naming/arity metadata so `.name`, `.length`, and native source text line up with variant-backed constructors and builtin callable surfaces
  - host constructor aliases and static host callables stay stable across direct access, `window` aliases, and worker-exposed bindings instead of falling back to generic object formatting

- [x] Extend callable-aware coercion across remaining string-search and argument paths
  - callable-aware string coercion now covers remaining search/separator overlap paths, including array/typed-array parser fallbacks that land on string `includes`, `indexOf`, and `lastIndexOf`
  - indirect string contexts no longer regress object-backed callables to raw `Value::as_string()` output when overlap dispatch or generic helpers route through string operations

- [x] Verification completed
  - `cargo test --lib callable_search_separator_and_padding_args_use_source_text_work`
  - `cargo test --lib object_backed_host_callable_name_length_and_source_text_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2231 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.34: string prototype raw-getter breadth and overlap parser cleanup)

- [x] Expand remaining string prototype raw getters through the callable surface cache
  - added cached/raw getter coverage for `String.prototype.indexOf`, `lastIndexOf`, `padStart`, `padEnd`, and `repeat` so extracted/prototype `.call(...)`, `constructor.prototype`, and alias/property-path access share the same callable metadata and receiver behavior as direct calls
  - extended generic string member-call execution so these helpers keep callable-aware coercion instead of falling back to plain stringification when host callables flow through search, padding, or repeat operations

- [x] Reduce parser overlap reliance on runtime string fallbacks
  - moved ambiguous bare-identifier `includes`, `indexOf`, and `lastIndexOf` parsing onto the shared `MemberCall` path so string/array/typed-array dispatch is resolved by the common member-call runtime instead of overlap-specific AST nodes
  - removed the dead `ArrayIncludes` AST/runtime path after the parser cleanup, leaving one dispatch route for the remaining overlap cases

- [x] Verification completed
  - `cargo test --lib ambiguous_search_methods_parse_to_shared_member_calls_work`
  - `cargo test --lib shared_member_call_search_dispatch_keeps_string_array_and_typed_array_behavior_work`
  - `cargo test --lib callable_search_separator_and_padding_args_use_source_text_work`
  - `cargo test --lib string_search_and_padding_raw_getter_metadata_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2234 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.35: string generic receiver coercion and remaining prototype surface sweep)

- [x] Harden generic receiver coercion across the remaining string prototype helpers
  - string receiver builtin calls now distinguish strict `toString` / `valueOf` receiver validation from generic string methods, so `search`, `match`, `matchAll`, `replace`, `replaceAll`, `localeCompare`, and the remaining string helpers accept non-nullish receivers through shared ToString-style coercion while still rejecting Symbol receivers
  - the string-specialized AST paths now use the same receiver coercion helper, so callable receivers pick up native source text consistently instead of falling back to raw internal placeholders on direct string-method execution

- [x] Finish the remaining string prototype property-surface breadth gaps
  - cached string prototype/raw getter coverage now includes the remaining callable surface breadth such as `charAt`, `at`, `search`, `match`, `matchAll`, `replace`, `replaceAll`, `localeCompare`, `trim`, `toUpperCase`, `isWellFormed`, and `toWellFormed`
  - shared string member-call execution now covers those methods for extracted/prototype/bracket paths, including regexp-sensitive behavior and callback-based replacement flows

- [x] Verification completed
  - `cargo test --lib string_generic_receiver_coercion_and_remaining_prototype_paths_work`
  - `cargo test --lib string_search_and_padding_raw_getter_metadata_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2235 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.36: string argument ToString and regexp delegation parity sweep)

- [x] Tighten string and regexp argument ToString semantics where JS requires errors
  - routed string helper arguments that require ToString through the strict coercion helper so Symbol inputs now throw instead of silently stringifying across direct AST, shared member-call, raw getter, and extracted/prototype call paths
  - aligned `RegExp` construction, `RegExp.escape`, `RegExp.prototype.exec/test`, string replacement callback results, and `String.raw` with the same Symbol-sensitive coercion behavior

- [x] Deepen `Symbol.match*` delegation parity for string helpers
  - added shared `Symbol.match`, `Symbol.matchAll`, `Symbol.replace`, and `Symbol.search` method lookup/call helpers so direct string AST paths and shared member-call execution both delegate through the pattern object when present
  - preserved `String.prototype.matchAll` cloned-regexp `lastIndex` semantics by copying the original global regexp state onto the clone instead of resetting it to zero

- [x] Verification completed
  - `cargo test --lib string_argument_tostring_and_regexp_delegation_work`
  - `cargo test --lib string_generic_receiver_coercion_and_remaining_prototype_paths_work`
  - `cargo test --lib string_search_and_padding_raw_getter_metadata_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2236 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.37: string split delegation and ordinary-object ToString residual sweep)

- [x] Finish `String.prototype.split` delegation and RegExp-like boundary parity
  - added `Symbol.split` delegation across direct string AST execution and shared member-call execution, including correct forwarding of the `limit` argument to custom splitters
  - aligned `includes`, `startsWith`, and `endsWith` with `@@match`-based RegExp-like detection so custom objects with truthy `Symbol.match` are rejected while real regexes with `Symbol.match = false` fall back to normal string coercion

- [x] Deepen ordinary-object ToString parity in string-facing helpers
  - upgraded strict string coercion to use `toString` then `valueOf` for ordinary objects, including proper `Cannot convert object to primitive value` failure when both callable paths stay non-primitive
  - routed generic string receivers through the same strict coercion path so plain-object receivers and arguments now behave consistently across search, separator, split, and replacement flows

- [x] Verification completed
  - `cargo test --lib string_split_delegation_and_object_tostring_residuals_work`
  - `cargo test --lib string_generic_receiver_coercion_and_remaining_prototype_paths_work`
  - `cargo test --lib string_argument_tostring_and_regexp_delegation_work`
  - `cargo test --lib shared_member_call_search_dispatch_keeps_string_array_and_typed_array_behavior_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2237 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.38: RegExp symbol-method surface and string coercion edge-case sweep)

- [x] Expose and align RegExp symbol-method callable surface
  - exposed `RegExp.prototype[Symbol.match]`, `Symbol.matchAll`, `Symbol.replace`, `Symbol.search`, and `Symbol.split` through shared receiver-aware builtins so direct symbol access, extracted calls, and `Function.prototype.call` routes now share receiver validation, lastIndex handling, and result shapes
  - preserved instance override precedence for symbol-keyed regexp methods so custom `regex[Symbol.replace] = fn` continues to work for both direct invocation and string delegation

- [x] Deepen remaining string coercion edge cases
  - verified regexp symbol methods now use the shared ToString path for omitted inputs and host-object inputs such as `URL`, instead of falling back to ad hoc placeholder behavior
  - added raw-getter metadata coverage for RegExp symbol methods so `.name`, `.length`, and native source text stay aligned with the rest of the callable surface

- [x] Verification completed
  - `cargo test --lib regexp_symbol_method_property_paths_and_coercion_edge_cases_work`
  - `cargo test --lib regexp_symbol_method_raw_getter_metadata_work`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2239 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.39: Date string-hint ToPrimitive and RegExp inheritance/identity residual sweep)

- [x] Close remaining string-hint coercion gaps for object-like values
  - unified `Date` and object-like string coercion across `String(...)`, `new String(...)`, fast-path `Expr::StringConstruct`, and direct `toString()` evaluation so string-hint ToPrimitive uses shared primitive-conversion logic instead of placeholder text
  - preserved `String(Symbol(...))` descriptive-string behavior while still keeping Symbol rejection for string-method receiver coercion, and aligned canvas 2D context stringification with the same native object `toString` path

- [x] Tighten RegExp instance/prototype parity
  - routed direct regexp fast paths through instance/prototype property lookup so `exec`, `test`, and `toString` now respect own overrides, inherited `RegExp.prototype` fallback, and extracted-call parity
  - aligned omitted-versus-`undefined` behavior for `RegExp.prototype.test()` and removed the now-unused direct regex resolver after the fast paths moved onto the shared callable/property route

- [x] Verification completed
  - `cargo test --lib date_string_hint_and_raw_getter_coercion_work`
  - `cargo test --lib regexp_instance_lookup_respects_prototype_fallback_and_own_overrides_work`
  - `cargo test --lib string_constructor_and_static_methods_work`
  - `cargo test --lib canvas_rendering_context_2d_exposes_core_defaults`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib dom_canvas_rendering_context_2d`
  - `cargo fmt`
  - `cargo test --lib` (`2241 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.40: host-object toString/valueOf bracket-call and raw-getter parity sweep)

- [x] Close remaining host-object `toString` / `valueOf` call-surface gaps
  - replaced the remaining object-backed host `toString` placeholders on `Selection` and `CanvasRenderingContext2D` with receiver-aware native callables so dot-call, bracket-call, extracted-method, and `String(obj)` paths converge on the same host implementation
  - routed receiver-builtin dispatch through the existing `Selection` and canvas member-call evaluators so incompatible-receiver errors, native callable metadata, and host-specific stringification stay aligned

- [x] Expand host-object callable parity coverage
  - added targeted tests for `Selection` and `CanvasRenderingContext2D` covering direct calls, bracket calls, extracted `Function.prototype.call`, string coercion, and raw-getter metadata
  - updated the canvas 2D default surface expectation to the canonical host stringification text now returned by the shared callable path

- [x] Verify
  - `cargo test --lib selection_tostring_bracket_call_and_raw_getter_parity_work`
  - `cargo test --lib canvas_rendering_context_2d_tostring_bracket_call_and_raw_getter_parity_work`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_canvas_rendering_context_2d`
  - `cargo test --lib window_get_selection`
  - `cargo fmt`
  - `cargo test --lib` (`2243 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.41: Object.prototype default toString/valueOf receiver parity sweep)

- [x] Close remaining default object-stringification gaps
  - added receiver-aware `Object.prototype.toString` / `valueOf` builtins so ordinary objects, primitives, and object-backed host values now share the same direct-call, bracket-call, extracted-call, and string-coercion behavior
  - restored `Intl.Locale` generic method/property access by attaching `Intl.Locale.prototype` to instances and exposing receiver-aware locale methods on the prototype, which brings ambiguous `toString()` calls back onto the locale-specific path instead of falling through to `[object Object]`

- [x] Expand parity coverage
  - added targeted tests for `Object.prototype.toString` / `valueOf` metadata, incompatible receivers, collection/host-object inheritance, and `Intl.Locale` raw-getter plus extracted-call paths
  - kept `Symbol.toStringTag`-based object tagging aligned with the new default-object receiver surface

- [x] Verify
  - `cargo test --lib object_prototype_to_string_and_value_of_receiver_paths_work`
  - `cargo test --lib object_prototype_raw_getter_metadata_and_incompatible_receiver_work`
  - `cargo test --lib object_prototype_to_string_inherits_across_collections_and_host_tags_work`
  - `cargo test --lib intl_locale_properties_and_methods_work`
  - `cargo test --lib intl_locale_raw_getter_and_call_paths_work`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo fmt`
  - `cargo test --lib` (`2247 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.42: Intl prototype generic getter and extracted-call parity sweep)

- [x] Close remaining Intl prototype surface gaps
  - attached shared `Intl.*.prototype` links to formatter instances and exposed receiver-aware prototype methods for the remaining `Intl.Collator`, `Intl.DateTimeFormat`, `Intl.DisplayNames`, `Intl.DurationFormat`, `Intl.ListFormat`, `Intl.NumberFormat`, `Intl.PluralRules`, `Intl.RelativeTimeFormat`, and `Intl.Segmenter` method surface
  - aligned raw getters, bracket calls, extracted `Function.prototype.call`, inherited prototype dispatch, and incompatible-receiver failures so the generic property path no longer falls back to unrelated shared callable or `Object.prototype` behavior

- [x] Expand Intl receiver-parity coverage
  - added focused main-realm regressions for prototype-property identity, extracted-call parity, native metadata, and incompatible receivers across formatter and segmenter families
  - added worker coverage for `Intl` namespace identity plus `DisplayNames` / `RelativeTimeFormat` / `Collator` raw getter execution, and fixed worker `self.Intl` to share the same namespace object as bare `Intl`

- [x] Verify
  - `cargo test --lib intl_formatter_prototype_methods_and_bound_format_metadata_work`
  - `cargo test --lib intl_generic_prototype_getters_and_incompatible_receivers_work`
  - `cargo test --lib worker_intl_prototype_raw_getters_and_receiver_parity_work`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib numeric_intl_dom_mutations`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2250 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.43: Intl bound getter accessor and callable identity sweep)

- [x] Tighten remaining bound getter accessor parity
  - moved `Intl.Collator.prototype.compare`, `Intl.DateTimeFormat.prototype.format`, and `Intl.NumberFormat.prototype.format` onto prototype getter accessors backed by cached internal bound callables instead of enumerable instance data properties
  - aligned direct dot access, bracket access, parser-special getter paths, repeated access identity, and incompatible-receiver failures so accessor-backed callables now share the same stable callable object and metadata as their direct-call paths

- [x] Expand coverage for Intl callable identity and worker breadth
  - added focused main-realm regressions for `compare` / `format` identity stability, lack of own data properties, incompatible receivers, and native `name` / `length` / `toString()` parity
  - added worker coverage for bound getter identity and hidden-surface parity, keeping `self.Intl` / bare `Intl` access aligned with the same accessor-backed callable behavior

- [x] Verify
  - `cargo test --lib intl_bound_format_getter_accessor_identity_and_receiver_parity_work`
  - `cargo test --lib intl_collator_compare_getter_accessor_identity_and_receiver_parity_work`
  - `cargo test --lib worker_intl_bound_getter_accessor_identity_work`
  - `cargo test --lib issue_105_intl_number_format_format_method`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib numeric_intl_dom_mutations`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2253 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.44: Intl accessor assignment no-op and enumerable surface parity sweep)

- [x] Tighten assignment semantics for Intl accessor-backed callables
  - aligned `Intl.Collator.prototype.compare`, `Intl.DateTimeFormat.prototype.format`, and `Intl.NumberFormat.prototype.format` with getter-only browser behavior so instance assignment is a no-op, deletion leaves the prototype accessor path intact, and repeated access keeps returning the same cached bound callable
  - verified direct dot access, bracket access, extracted-call paths, and worker/global entry points continue to use the native accessor-backed callable instead of leaking an assigned RHS through parser fast paths

- [x] Expand enumerable-surface coverage for Intl accessors
  - marked Intl instance/prototype `constructor` links as non-enumerable so `Object.keys`, spread, `for...in`, and `JSON.stringify` stay clean while accessor-backed `format` / `compare` remain inherited-only
  - added parser and runtime regressions for `hasOwnProperty`, bracket access, `self.Intl` parity, and shared `ObjectGet` / `MemberCall` parsing around Intl accessor properties

- [x] Verify
  - `cargo test --lib intl_date_time_format_accessor_assignment_noop_and_enumeration_work`
  - `cargo test --lib intl_number_format_accessor_assignment_noop_and_enumeration_work`
  - `cargo test --lib intl_collator_compare_accessor_assignment_noop_and_enumeration_work`
  - `cargo test --lib worker_intl_accessor_assignment_noop_and_enumeration_work`
  - `cargo test --lib intl_accessor_member_get_and_call_parse_to_shared_paths_work`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib numeric_intl_dom_mutations`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo fmt`
  - `cargo test --lib` (`2258 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.45: Intl accessor descriptor introspection and property-descriptor parity sweep)

- [x] Audit Intl accessor descriptor introspection
  - added dedicated parsing and evaluation for `Object.getOwnPropertyDescriptor(...)` so Intl prototype accessors expose stable accessor descriptors with `get` / `set` / `enumerable` / `configurable` shape and no hidden cache-slot leakage
  - covered main realm and worker prototype-walk descriptor checks for `Intl.NumberFormat.prototype.format` and `Intl.Collator.prototype.compare`, including alias parity through `window.Object` and `self.Intl`

- [x] Tighten property-definition and reflective-set parity
  - added dedicated parsing and evaluation for `Object.defineProperty(...)` and `Reflect.set(...)`, including getter/setter-aware descriptor application, own-property shadow descriptors, inherited getter-only no-op behavior, and delete-based fallback to the prototype accessor path
  - verified worker call paths honor explicitly defined own overrides for `format` / `compare`, while main-realm regression coverage now pins descriptor/overwrite/delete surfaces and inherited fallback behavior

- [x] Verify
  - `cargo test --lib intl_descriptor_and_reflect_static_calls_parse_work`
  - `cargo test --lib descriptor_define_property_and_reflect_set_work`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib numeric_intl_dom_mutations`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo fmt`
  - `cargo test --lib` (`2262 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.46: Intl main-realm own-override lookup and Reflect/Object surface breadth sweep)

- [x] Tighten main-realm own-override lookup parity
  - added direct main-realm regression coverage for `Intl.NumberFormat.prototype.format` and `Intl.Collator.prototype.compare` own overrides after `Object.defineProperty(...)`, `Reflect.set(...)`, bracket access, extracted-call reads, and delete-based fallback
  - confirmed main-realm `format` / `compare` now stay aligned with worker behavior across bare variables and `window.*` object paths

- [x] Broaden Reflect/Object descriptor API surface
  - exposed `Reflect` as a real shared global object on main realm, `window`, and worker, and surfaced actual callable entries for supported `Object.*` static methods instead of relying only on dedicated parser fast paths
  - fixed parser fallback so `Object.defineProperty`, `Object.entries`, `Reflect.set`, and similar property gets can be extracted or aliased without being misparsed as mandatory static calls

- [x] Verify
  - `cargo test --lib object_and_reflect_alias_surface_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2267 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.47: Object/Reflect object-like target breadth and function-array descriptor parity sweep)

- [x] Extend Object/Reflect target coverage beyond plain objects
  - broadened `Object.getOwnPropertyDescriptor(...)`, `Object.defineProperty(...)`, `Object.keys(...)`, `Object.values(...)`, `Object.entries(...)`, `Object.hasOwn(...)`, `Object.getOwnPropertySymbols(...)`, and `Reflect.set(...)` so arrays, functions, maps, sets, weak collections, and regex-backed values now follow the same object-like rules instead of failing as non-objects
  - aligned array index / `length`, function `name` / `length`, callable-object surfaces, collection `size`, and regex builtins with descriptor, enumerability, reflective set, and deletion behavior expected by the generic `Object` / `Reflect` helpers

- [x] Tighten function/object-like own-property parity
  - made function public-name storage non-enumerable so `Object.keys(fn)` no longer leaks `name`, while `Object.hasOwn(...)` and `Object.getOwnPropertyDescriptor(...)` still report the builtin own property correctly
  - added delete support for function public properties so custom data properties added through `Object.defineProperty(...)` or `Reflect.set(...)` behave like ordinary own properties on callable targets

- [x] Verify
  - `cargo test --lib object_and_reflect_support_array_function_and_collection_targets_work -- --nocapture`
  - `cargo test --lib object_descriptor_and_reflect_work_on_callable_object_surfaces -- --nocapture`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2269 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.48: Object/Reflect own-key ordering and descriptor-attribute residual sweep)

- [x] Deepen own-key and descriptor coverage
  - surfaced `Object.getOwnPropertyNames(...)` and `Reflect.ownKeys(...)` through the parser, callable runtime, and generic property-get paths so extracted calls and alias access now stay on the same `Object` / `Reflect` execution surface
  - aligned own-key ordering for arrays, functions, callable objects, collections, and symbol-backed values so integer-like string keys, builtin non-enumerable keys, custom string keys, and symbol keys serialize in the expected reflective order
  - tightened builtin descriptor attributes for array `length`, function/callable `name` / `length` / `prototype`, collection `size`, and regex-backed own properties so `Object.getOwnPropertyDescriptor(...)`, `Object.hasOwn(...)`, and related helpers report stable browser-like flags

- [x] Verify
  - `cargo test --lib object_and_reflect_own_keys_and_descriptor_attributes_work -- --nocapture`
  - `cargo test --lib intl_descriptor_and_reflect_static_calls_parse_work -- --nocapture`
  - `cargo test --lib object_and_reflect_property_get_parse_falls_back_to_generic_paths_work -- --nocapture`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo fmt`
  - `cargo test --lib` (`2270 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.49: Object/Reflect descriptor mutation and symbol-key residual sweep)

- [x] Deepen descriptor-mutation parity
  - persisted `writable` / `configurable` / `enumerable` metadata across `Object.defineProperty(...)` writes on plain objects, arrays, callable targets, collections, and regex-backed property bags so follow-up `Object.getOwnPropertyDescriptor(...)` reads reflect the mutated flags instead of builtin defaults
  - aligned `Reflect.set(...)`, direct assignment, and `delete` with the stored descriptor metadata for array indices and `length`, function `name` / `length`, collection `size`, regex `lastIndex`, and entry-backed own properties so non-writable and non-configurable cases now stay browser-like after reflective mutation
  - fixed property fast paths to honor overridden own descriptors before builtin virtual slots, including function `length`, collection `size`, and regex-backed properties

- [x] Extend symbol-key and own-key residual coverage
  - verified mixed string/symbol reflective ordering and descriptor persistence after mutation-heavy flows on arrays, callables, and collection-backed targets, including cases where builtin non-enumerable keys and custom symbol keys coexist

- [x] Verify
  - `cargo test --lib object_and_reflect_descriptor_mutation_and_symbol_key_residuals_work -- --nocapture`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib numeric_intl_dom_mutations`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2271 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.50: callable own-property delete semantics and property-fast-path residual sweep)

- [x] Close callable own-property delete gaps
  - added deleted-builtin markers so removing overridden callable `name` / `length`, collection `size`, and regexp builtin surfaces clears descriptor and own-key exposure while direct property reads fall back to the builtin value
  - aligned `Object.getOwnPropertyDescriptor(...)`, `Object.hasOwn(...)`, own-key enumeration, `Reflect.set(...)`, direct assignment, and `delete` across callable objects, user functions, maps, sets, and regexp-backed values after explicit own overrides are removed

- [x] Sweep property fast-path and parser residuals
  - fixed property fast paths so function/callable `name` / `length`, collection `size`, and regexp builtin keys consult explicit own overrides first and deleted-builtin fallbacks second
  - stopped DOM access parsing from consuming known non-DOM globals like `Object` and `Reflect`, so `delete Object.keys.name` now follows the generic object-path pipeline instead of the DOM fast path

- [x] Verify
  - `cargo test --lib object_and_reflect_property_get_parse_falls_back_to_generic_paths_work -- --nocapture`
  - `cargo test --lib callable_delete_and_builtin_surface_residuals_work -- --nocapture`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo fmt`
  - `cargo test --lib` (`2272 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.51: function prototype writable/configurable and regex own-surface parity sweep)

- [x] Tighten function prototype descriptor parity
  - aligned ordinary-function `prototype` writes so direct assignment and `Reflect.set(...)` preserve the builtin descriptor surface instead of creating enumerable/configurable ad hoc own properties, while `delete` now correctly stays `false`
  - added dedicated `Object.defineProperty(...)` handling for ordinary-function `prototype` so omitted fields preserve the current value/flags, non-configurable invariants stay in place, and `new` keeps using the overridden prototype object for instance linkage

- [x] Revisit regexp own-surface modeling
  - reduced regexp instance builtin own keys to `lastIndex`, moving `source`, `flags`, and the boolean flag accessors onto `RegExp.prototype` via accessor descriptors so `Object.getOwnPropertyNames(...)`, `Reflect.ownKeys(...)`, `Object.hasOwn(...)`, and direct property reads now match browser structure
  - fixed regexp delete/set parity so deleting inherited accessor properties is a no-op success, explicit own overrides can still be defined and deleted, and `RegExp.prototype` getters plus `toString()` work on the prototype object itself

- [x] Verify
  - `cargo test --lib function_prototype_descriptor_and_write_parity_work -- --nocapture`
  - `cargo test --lib regexp_prototype_accessor_and_own_surface_parity_work -- --nocapture`
  - `cargo test --lib callable_delete_and_builtin_surface_residuals_work -- --nocapture`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2274 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.52: generic accessor own-key synthesis and non-configurable redefine invariant sweep)

- [x] Synthesize accessor-backed own keys more consistently
  - synthesized public string and symbol own keys from getter/setter-backed storage entries so `Object.getOwnPropertyNames(...)`, `Object.keys(...)`, `Reflect.ownKeys(...)`, and descriptor introspection stay aligned even when there is no shadow data slot
  - filtered internal builtin storage markers back out of reflective own-key enumeration so host internals do not leak through the new accessor-key synthesis

- [x] Tighten non-configurable redefine invariants across builtin surfaces
  - normalized `Object.defineProperty(...)` handling for generic objects, arrays, functions, regexp instances, and collection-like builtin surfaces so mixed accessor/data descriptors, forbidden flag flips, and non-writable value rewrites fail consistently
  - preserved omitted descriptor fields on redefinition while keeping spec-default `false` flags for newly defined properties, which also fixes follow-up `Reflect.set(...)`, assignment, delete, and descriptor reads

- [x] Verify
  - `cargo test --lib accessor_only_own_key_synthesis_work -- --nocapture`
  - `cargo test --lib non_configurable_redefine_invariant_sweep_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib timers_numbers_intl_basics`
  - `cargo test --lib numeric_intl_dom_mutations`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo test --lib` (`2276 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.53: defineProperty descriptor-object coercion and accessor-undefined parity sweep)

- [x] Tighten descriptor-object coercion for reflective property mutation
  - broadened `Object.defineProperty(...)` descriptor intake from plain objects to object-like descriptor values while preserving browser-like property access order across inherited descriptor fields
  - normalized mixed accessor/data descriptor validation so getter/setter/value/writable conflicts fail consistently after the new coercion path

- [x] Normalize explicit `undefined` accessor semantics
  - distinguished omitted `get`/`set` from explicit `get: undefined` / `set: undefined` with dedicated internal markers so descriptor reads, own-key exposure, `Reflect.set(...)`, assignment, and delete stay aligned across generic objects and builtin-backed surfaces
  - fixed object-literal overwrite and accessor-pair merge behavior so spread/data redefinitions clear stale accessor metadata without breaking duplicate-key insertion order or getter/setter pairing

- [x] Verify
  - `cargo test --lib define_property_descriptor_object_coercion_parity_work -- --nocapture`
  - `cargo test --lib define_property_accessor_undefined_parity_work -- --nocapture`
  - `cargo test --lib spread_syntax_in_object_literals_supports_merge_override_and_primitive_sources -- --nocapture`
  - `cargo test --lib object_literal_property_access_and_methods_work -- --nocapture`
  - `cargo test --lib operators_advanced_selectors`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2278 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.54: descriptor boxing and object-literal accessor overwrite residual sweep)

- [x] Tighten boxed descriptor coercion paths
  - taught `Object.defineProperty(...)` descriptor reads to work through real boxed `Boolean` / `Number` wrapper objects instead of ad hoc stringified placeholder objects, preserving browser-like inherited field lookup order for descriptor flags and accessors
  - aligned `Object(...)`, `new Boolean(...)`, and `new Number(...)` boxing paths with runtime wrapper objects so prototype lookup, `valueOf()`, `toString()`, and constructor-backed descriptor use now share the same surface

- [x] Sweep remaining object-literal accessor overwrite residuals
  - centralized object-literal data/getter/setter writes so duplicate getter/setter/data/spread combinations preserve insertion order while clearing stale accessor metadata and keeping getter/setter pairing intact
  - added dedicated overwrite coverage to ensure direct reads and reflective surfaces stay aligned after accessor-to-data and data-to-accessor transitions

- [x] Verify
  - `cargo test --lib define_property_boxed_descriptor_wrappers_work -- --nocapture`
  - `cargo test --lib object_constructor_boxes_numbers_with_number_wrapper_surface_work -- --nocapture`
  - `cargo test --lib object_literal_accessor_overwrite_matrix_keeps_browser_semantics -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib` (`2281 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.55: wrapper-object reflective surface and primitive-boxing residual sweep)

- [x] Expand boxed primitive reflective parity
  - synthesized string-wrapper exotic own keys and descriptors across `Object.keys(...)`, `Object.getOwnPropertyNames(...)`, `Reflect.ownKeys(...)`, `Object.assign(...)`, and object spread so wrapper index properties and `length` show up in browser-like order while still merging custom own properties
  - aligned boxed `Boolean`, `Number`, `BigInt`, and `Symbol` instances for `Object.getPrototypeOf(...)`, `Object.prototype.toString.call(...)`, constructor identity, and `valueOf()` so runtime wrapper objects expose the expected reflective surface

- [x] Sweep primitive-boxing residual call and coercion paths
  - taught fast-path `valueOf()` and receiver-builtin evaluation to unwrap non-string primitive wrappers instead of only string wrappers, fixing boxed symbol and numeric wrapper behavior through generic member access and extracted call paths
  - routed reflective lookup and copy helpers through shared wrapper-aware key synthesis so exotic wrapper own properties are visible without bypassing explicit own overrides

- [x] Verify
  - `cargo test --lib string_wrapper_reflective_surface_and_copy_paths_work -- --nocapture`
  - `cargo test --lib boxed_primitive_wrapper_tags_and_prototype_introspection_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib` (`2283 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.56: wrapper descriptor invariants and string-exotic override residual sweep)

- [x] Tighten wrapper descriptor invariants
  - treated string-wrapper index keys and `length` as real non-writable / non-configurable exotic own properties through direct assignment, `delete`, `Reflect.set(...)`, and `Object.defineProperty(...)`, including foreign-receiver `Reflect.set(...)` cases
  - kept compatible descriptor redefinitions as no-ops while rejecting incompatible value and attribute changes with stable redefine errors instead of leaking synthetic overrides into wrapper storage

- [x] Sweep string-exotic override and reflective residuals
  - fixed string-wrapper numeric property reads so out-of-range indices fall through to explicit own properties and prototype lookup instead of returning hardcoded `undefined`, restoring browser-like access for custom numeric keys such as `"2"`
  - aligned own-property descriptor reads with wrapper exotic semantics by preferring virtual string-wrapper descriptors over stale explicit entries, preventing reflective mismatches after failed override attempts

- [x] Verify
  - `cargo test --lib string_wrapper_descriptor_invariants_and_override_attempts_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib` (`2284 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.57: wrapper prototype mutation and string-exotic introspection residual sweep)

- [x] Tighten wrapper prototype mutation parity
  - exposed `Object.setPrototypeOf(...)` on the real `Object` callable surface and routed it through shared prototype mutation logic with cycle checks so wrapper objects and ordinary objects follow the same mutation rules
  - taught wrapper-aware property lookup and the `in` operator to honor default wrapper prototype chains plus explicit prototype reassignment, restoring inherited method and numeric-key lookup without breaking string-exotic own properties

- [x] Sweep string-exotic introspection residuals
  - exposed receiver-aware `Object.prototype.hasOwnProperty`, `isPrototypeOf`, and `propertyIsEnumerable`, and routed the special `hasOwnProperty(...)` fast path through shared own-property logic so wrapper virtual keys such as `"0"` and `"length"` report browser-like results
  - kept string-exotic own keys stable across custom and `null` prototype transitions so direct calls, extracted calls, reflective checks, and prototype-backed lookups stay aligned after mutation

- [x] Verify
  - `cargo test --lib wrapper_prototype_mutation_and_string_exotic_introspection_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib` (`2285 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.58: wrapper primitive-prototype method surface and Object static prototype-mutation breadth sweep)

- [x] Expand wrapper primitive-prototype method parity
  - linked boxed `Boolean`, `Number`, and `BigInt` prototype objects to the shared `Object.prototype` surface so inherited `Object.prototype` helpers remain available through custom prototype chains and extracted calls
  - exposed `Number.prototype.toExponential`, `toFixed`, and `toPrecision` through raw getter, bracket, and extracted-call paths, keeping callable metadata and wrapper receiver validation aligned with parser-special number method execution

- [x] Broaden `Object` static prototype-mutation coverage
  - extended `Object.setPrototypeOf(...)` to ordinary functions, arrays, maps, sets, weak collections, regexps, and boxed wrapper objects while returning primitive targets unchanged for browser-like non-object behavior
  - taught `Object.getPrototypeOf(...)` / prototype-backed lookup to honor explicit function prototype overrides, preserving inherited reads and `isPrototypeOf(...)` checks after callable prototype mutation and `null` transitions

- [x] Verify
  - `cargo test --lib wrapper_primitive_prototype_methods_survive_custom_prototype_chains_work -- --nocapture`
  - `cargo test --lib object_static_prototype_mutation_covers_functions_primitives_and_regexp_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib` (`2287 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Completed Task (P1.59: variant-backed callable prototype mutation and array prototype-chain parity sweep)

- [x] Extend variant-backed callable prototype mutation coverage
  - added explicit hidden prototype storage for variant-backed constructor values so `Object.getPrototypeOf(...)` / `Object.setPrototypeOf(...)` now preserve custom prototype overrides for `String`, `Symbol`, `Map`, `Set`, `Promise`, `URL`, `RegExp`, `ArrayBuffer`, `Blob`, `URLSearchParams`, and typed-array constructors
  - kept constructor own surface access stable while letting inherited reads follow the overridden prototype chain, and allowed object-like prototypes such as the abstract typed-array constructor to round-trip through `Object.setPrototypeOf(...)`

- [x] Align array prototype-chain lookup after explicit mutation
  - taught array property reads to fall through to explicit custom prototypes for holes, out-of-range numeric keys, inherited methods, and `in` checks instead of always stopping at built-in fast paths
  - suppressed synthesized array builtin methods once an explicit prototype override is present so custom or `null` prototype chains control inherited lookup the same way they do for ordinary objects

- [x] Verify
  - `cargo test --lib variant_backed_callable_prototype_mutation_keeps_inherited_reads_work -- --nocapture`
  - `cargo test --lib array_prototype_chain_reads_and_in_semantics_follow_explicit_mutation_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib` (`2289 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.60: object-like prototype target breadth and array-like inherited lookup residual sweep
  - broadened `Object.setPrototypeOf(...)` / `Object.getPrototypeOf(...)` across arrays, functions, constructors, typed arrays, and `NodeList` by storing explicit prototype overrides on the relevant runtime values instead of only ordinary objects
  - split prototype traversal into owner-aware and receiver-aware paths so inherited lookup through explicit prototype mutation keeps the original receiver while starting from the current object-like owner
  - taught typed arrays and `NodeList` to honor explicit prototype reassignment for property reads and `in` without bypassing custom prototype chains, while keeping their built-in own index/length behavior intact
  - fixed `NodeList` live-list borrow scope so default DOM mutation and selection flows still pass after prototype-aware lookup was added

- [x] Verify
  - targeted prototype-mutation and inherited-lookup regression tests
  - `cargo fmt`
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib` (`2296 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.61: receiver-aware prototype traversal for remaining object-like built-ins
  - extended owner/receiver-aware lookup across remaining non-ordinary object-like values so collections, promises, regexps, blobs, array buffers, and related object-like built-ins no longer leak instance-only fast-path state through explicit prototype mutation
  - taught parser-specialized `Map` / `Set` / `WeakMap` / `WeakSet` / promise / array-buffer member dispatch to fall back to generic receiver-aware lookup when an explicit prototype override or inherited callable must win, while still preserving legacy specialized behavior for placeholder-backed host surfaces such as `DataTransferItemList`
  - suppressed placeholder-callable execution in the collection parser fast paths so reflected host-method surfaces continue to route into their specialized runtime implementations instead of being consumed by generic placeholder functions

- [x] Verify
  - `cargo test --lib collection_and_regexp_explicit_prototype_override_disables_builtin_fast_paths_work -- --nocapture`
  - `cargo test --lib non_ordinary_prototype_traversal_preserves_receiver_and_hides_instance_state_work -- --nocapture`
  - `cargo test --lib data_transfer_item_list_methods_are_noop_outside_dragstart -- --nocapture`
  - `cargo test --lib data_transfer_item_list_add_rejects_non_file_single_argument -- --nocapture`
  - `cargo test --lib data_transfer_item_list_add_file_appends_to_files_and_items -- --nocapture`
  - `cargo test --lib data_transfer_item_list_add_replaces_string_item_without_reordering -- --nocapture`
  - `cargo test --lib data_transfer_item_list_remove_can_remove_string_and_file_items -- --nocapture`
  - `cargo test --lib dom_data_transfer_item_list`
  - `cargo test --lib window_forms_trace`
  - `cargo fmt`
  - `cargo test --lib` (`2298 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.62: placeholder-backed host method dispatch and inherited callable residual sweep
  - routed placeholder-backed `Document`, parsed-document, `DOMParser`, `TreeWalker`, `Range`, and `Selection` method surfaces through receiver-aware builtins so extracted calls, `.call(...)`, and inherited lookups hit the specialized host runtime instead of placeholder functions
  - exposed `Object.create(...)` on the actual `Object` static callable surface so inherited host-method regressions can construct prototype-linked receivers through generic property access instead of relying on parser-only static-call support
  - taught the DOM property parser to fall back from `document.*` placeholder-backed method getters to generic member access so raw getter/property-path use sites no longer fail before runtime receiver validation

- [x] Verify
  - `cargo test --lib dom_parser_tree_walker_and_parsed_document_placeholder_methods_support_extracted_and_inherited_calls_work -- --nocapture`
  - `cargo test --lib document_range_and_selection_placeholder_methods_support_extracted_and_inherited_calls_work -- --nocapture`
  - `cargo test --lib dom_selection_interface`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2300 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.63: remaining host raw-getter/property-path breadth sweep
  - exposed receiver-aware raw getter callables for `cookieStore`, `CacheStorage`, and `Cache` so placeholder-backed method properties reached through dot access, bracket access, extracted calls, and inherited property reads now route into their specialized host implementations instead of leaking placeholder functions
  - aligned incompatible-receiver validation and callable metadata for the residual secure-context host surfaces that previously only worked through direct member invocation or alias variables
  - added raw getter regressions covering `cookieStore` and `caches` / `Cache`, including bracket property access, extracted `.call(...)`, and inherited property reads created through `Object.create(...)`

- [x] Verify
  - `cargo test --lib cookie_store_raw_getter_and_inherited_receiver_parity_work -- --nocapture`
  - `cargo test --lib cache_storage_and_cache_raw_getter_and_inherited_receiver_parity_work -- --nocapture`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_selection_interface`
  - `cargo test --lib window_forms_trace`
  - `cargo fmt`
  - `cargo test --lib` (`2302 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.64: residual EventTarget-like and host-specialized raw-getter sweep
  - exposed receiver-aware raw getter callables for `matchMedia`, `DataTransfer`, `clipboardData`, `DataTransferItem`, and `DataTransferItemList` so bracket access, inherited property reads, and extracted method metadata no longer fall back to placeholder functions
  - aligned incompatible-receiver validation and callable name/length metadata for the remaining EventTarget-like and drag-and-clipboard host surfaces that previously diverged from generic property access
  - added regressions covering `matchMedia` raw getter/property paths, constructor-backed `DataTransfer` method extraction, item/item-list inherited property reads, and paste `clipboardData` raw getter calls

- [x] Verify
  - `cargo test --lib match_media_raw_getter_and_inherited_property_paths_work -- --nocapture`
  - `cargo test --lib data_transfer_raw_getters_and_inherited_property_paths_work -- --nocapture`
  - `cargo test --lib dispatch_paste_clipboard_data_raw_getter_paths_work -- --nocapture`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_data_transfer_item_list`
  - `cargo test --lib dom_dispatch_paste_clipboard_data`
  - `cargo fmt`
  - `cargo test --lib` (`2305 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.65: event-object placeholder method raw-getter and receiver sweep
  - exposed receiver-aware raw getter callables for `Event`, `KeyboardEvent`, `PointerEvent`, and `NavigateEvent` placeholder-backed methods so bare property reads, bracket access, extracted calls, and inherited property paths all route through the same callable surface
  - aligned direct `event.preventDefault()` / `stopPropagation()` / `stopImmediatePropagation()` with extracted-call state updates by reflecting cancellation and stop flags onto the event object immediately, which keeps same-callback property reads in sync with listener dispatch state
  - added regressions for base `Event` raw getter paths plus `KeyboardEvent.getModifierState()`, `PointerEvent.getCoalescedEvents()` / `getPredictedEvents()`, and `NavigateEvent.intercept()` / `scroll()` callable metadata and incompatible-receiver handling

- [x] Verify
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_keyboard_event`
  - `cargo test --lib dom_pointer_event`
  - `cargo test --lib dom_navigate_event`
  - `cargo test --lib dom_event_target_dispatch_event_method`
  - `cargo fmt`
  - `cargo test --lib` (`2310 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.66: event-object property fast-path override and host-event accessor residual sweep
  - routed specialized `EventExprProp` reads through the live listener event object first, so direct reads like `event.type`, `event.target`, `event.currentTarget`, `event.defaultPrevented`, `event.isTrusted`, `event.bubbles`, `event.cancelable`, `event.eventPhase`, `event.timeStamp`, `event.state`, `event.oldState`, and `event.newState` now stay aligned with generic object-property semantics after same-callback overrides
  - removed string-only fallback behavior from the `target.name` / `target.id` and `currentTarget.name` / `currentTarget.id` fast paths, preserving raw overridden property values instead of coercing everything through DOM attribute defaults
  - added regression coverage for live event-object overrides so direct event-property fast paths, nested target/currentTarget reads, and same-callback `preventDefault()` observation all remain consistent with the generic object path

- [x] Verify
  - `cargo test --lib event_fast_paths_respect_live_event_object_overrides_work -- --nocapture`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2311 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.67: event-object delete/descriptor parity and remaining host-event property-path sweep
  - routed `delete event.*` and nested deletes like `delete event.target.id` / `delete event.currentTarget.name` through the live listener event object, so parser-specialized event-property fast paths now honor generic own-property deletion semantics instead of falling through to a noop `true`
  - taught non-node DOM-read delete paths to reuse the generic fallback property chain, which fixed event-adjacent host properties such as `BeforeUnloadEvent.returnValue` when they are shadowed with `Object.defineProperty(...)` and then deleted
  - added regression coverage for live event-object `defineProperty(...)` overrides, nested target/currentTarget shadowing, and `beforeunload` `returnValue` descriptor/delete behavior so specialized event access stays aligned with the generic object-property path

- [x] Verify
  - `cargo test --lib event_fast_path_delete_and_define_property_parity_work -- --nocapture`
  - `cargo test --lib before_unload_return_value_define_property_and_delete_work -- --nocapture`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_before_unload_event`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2313 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.68: host-event descriptor breadth and event-adjacent shadowing residual sweep
  - synthesized receiver-aware descriptor values for placeholder-backed host-event methods so `Object.getOwnPropertyDescriptor(...)` on `Event`, `KeyboardEvent`, `PointerEvent`, `NavigateEvent`, `clipboardData`, `DataTransfer`, `DataTransferItem`, and `DataTransferItemList` returns the same callable surface exposed by raw property access
  - taught `delete`, direct property reads, and parser-specialized direct member calls to honor deleted markers and explicit own-property overrides for placeholder-backed host-event methods, preventing fallback into generic array/object builtins after shadowing or deletion
  - added descriptor/delete/shadowing regressions for event methods, paste `clipboardData`, and drag-and-drop item-list methods so specialized host paths stay aligned with the generic object-property path

- [x] Verify
  - `cargo test --lib host_event_method_descriptors_and_delete_shadowing_work -- --nocapture`
  - `cargo test --lib dispatch_paste_clipboard_data_descriptor_and_delete_shadowing_work -- --nocapture`
  - `cargo test --lib data_transfer_placeholder_method_descriptor_and_delete_shadowing_work -- --nocapture`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_before_unload_event`
  - `cargo test --lib dom_dispatch_paste_clipboard_data`
  - `cargo test --lib dom_data_transfer_item_list`
  - `cargo test --lib dom_keyboard_event`
  - `cargo test --lib dom_pointer_event`
  - `cargo test --lib dom_navigate_event`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2316 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.69: placeholder-backed host method own-key/enumerability and specialized-call shadowing residual sweep
  - made placeholder-backed host methods non-enumerable own properties across document/window, cookie-store/cache surfaces, range/selection, paste/drag-and-drop objects, and event-like objects so raw getters, descriptors, and own-key APIs stay aligned after shadowing and deletion
  - taught specialized member-call paths for document, DOMParser, parsed documents, TreeWalker, Range, Selection, `matchMedia`, `cookieStore`, `CacheStorage`, and `Cache` to honor explicit own-property overrides and deleted markers before falling back to specialized runtime dispatch
  - restored browser-like object-literal accessor overwrite semantics so getter/setter redefinitions drop stale data slots, preserve accessor pairing correctly, and keep compound/logical assignment property references on the single-read accessor path

- [x] Verify
  - `cargo test --lib placeholder_backed_host_methods_are_non_enumerable_and_shadowable_work -- --nocapture`
  - `cargo test --lib dom_parser_parsed_document_and_tree_walker_methods_respect_shadowing_work -- --nocapture`
  - `cargo test --lib document_range_and_selection_methods_are_non_enumerable_and_shadowable_work -- --nocapture`
  - `cargo test --lib host_event_method_descriptors_and_delete_shadowing_work -- --nocapture`
  - `cargo test --lib dispatch_paste_clipboard_data_descriptor_and_delete_shadowing_work -- --nocapture`
  - `cargo test --lib data_transfer_placeholder_method_descriptor_and_delete_shadowing_work -- --nocapture`
  - `cargo test --lib object_literal_`
  - `cargo test --lib property_reference_once`
  - `cargo test --lib getter_once_for_property_reference`
  - `cargo test --lib reference_target`
  - `cargo test --lib skips_setter_when_left_`
  - `cargo fmt`
  - `cargo test --lib` (`2319 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

- [x] Completed P1.70: placeholder-backed host sync/rebuild persistence and synthesized-surface parity sweep
  - preserved placeholder-backed document method overrides, descriptor flags, and deleted markers across history-driven document resync so `pushState(...)` / `replaceState(...)` no longer resurrect or drop shadowed document methods
  - reused lazily-created clipboard/data-transfer wrapper objects across repeated event reconstruction so multi-listener dispatches keep stable host wrapper identity and expando/method overrides
  - treated `matchMedia` live `matches` / `media` as synthesized properties that still honor explicit own overrides, accessor redefinitions, deletes, and inherited reads without desynchronizing raw getter behavior

- [x] Verify
  - `cargo test --lib document_placeholder_method_shadowing_survives_history_resync_work -- --nocapture`
  - `cargo test --lib match_media_synthesized_properties_respect_override_delete_and_inherited_reads_work -- --nocapture`
  - `cargo test --lib drag_event_reuses_data_transfer_wrapper_across_listeners_work -- --nocapture`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_dispatch_drag_event_data_transfer`
  - `cargo fmt`
  - `cargo test --lib` (`2322 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.71: remaining synthesized host-property descriptor/receiver parity sweep)

- [x] Completed P1.71: remaining synthesized host-property descriptor/receiver parity sweep
  - preserved hidden canonical clipboard/data-transfer wrapper objects so live `types`, `items`, and `files` wrappers continue to update after `setData(...)`, `clearData(...)`, `add(...)`, `remove(...)`, and `clear()` without reviving deleted or shadowed public properties
  - aligned inherited reads for event-adjacent synthesized wrapper properties so `Object.create(...)` and saved wrapper references observe the same shadowing/delete state as direct reads while runtime mutations still reach the canonical backing objects
  - kept paste/drag event wrapper refresh paths receiver-stable by reusing canonical arrays and item-list objects instead of replacing the public surface during host-side synchronization

- [x] Verify
  - `cargo test --lib dispatch_paste_clipboard_types_wrapper_survives_shadow_delete_and_mutation_work -- --nocapture`
  - `cargo test --lib data_transfer_live_wrappers_survive_shadow_delete_and_item_list_mutations_work -- --nocapture`
  - `cargo test --lib dom_dispatch_paste_clipboard_data`
  - `cargo test --lib dom_data_transfer_item_list`
  - `cargo test --lib dom_dispatch_drag_event_data_transfer`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo fmt`
  - `cargo test --lib` (`2324 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.72: residual synthesized host-property own-key/descriptor breadth sweep)

- [x] Completed P1.72: residual synthesized host-property own-key/descriptor breadth sweep
  - moved `DOMStringMap` own-key synthesis onto live `data-*` attributes so `Object.keys(...)`, `Object.getOwnPropertyNames(...)`, `Reflect.ownKeys(...)`, and `Object.hasOwn(...)` reflect current dataset state instead of stale cached entries
  - stopped caching dataset attribute values inside wrapper object storage and made direct reads honor explicit own property/accessor overrides before falling back to synthesized `data-*` lookups
  - aligned `Object.getOwnPropertyDescriptor(...)` and object spread for live dataset keys so descriptor metadata and enumerable copy behavior follow the same synthesized surface as direct property reads

- [x] Verify
  - `cargo test --lib dataset_dom_string_map_own_keys_and_descriptors_track_live_attributes_work -- --nocapture`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib operators_advanced_selectors`
  - `cargo fmt`
  - `cargo test --lib` (`2325 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.73: synthesized array-like host own-key/descriptor parity sweep)

- [x] Completed P1.73: synthesized array-like host own-key/descriptor parity sweep
  - synthesized live own-key and descriptor surfaces for `DOMTokenList` and `NamedNodeMap` so indexed, named, and length/value properties now line up across `Object.keys(...)`, `Object.getOwnPropertyNames(...)`, `Reflect.ownKeys(...)`, `Object.getOwnPropertyDescriptor(...)`, and `Object.hasOwn(...)`
  - made `classList` method entries non-enumerable and taught direct reads to honor explicit own data/accessor overrides on `DOMTokenList`, `NamedNodeMap`, and `DOMStringMap` before falling back to computed live collection values
  - added regressions for `classList` and `attributes` shadow/delete/recovery flows so object spread, descriptors, and live collection refreshes stay aligned after `Object.defineProperty(...)`, `delete`, and host-side mutations

- [x] Verify
  - `cargo test --lib element_attributes_own_keys_and_descriptors_track_live_entries_work -- --nocapture`
  - `cargo test --lib class_list_own_keys_and_descriptors_track_live_tokens_work -- --nocapture`
  - `cargo test --lib dom_element_attributes_property`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo fmt`
  - `cargo test --lib` (`2327 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.74: residual DOM collection iterator/prototype and object-copy parity sweep)

- [x] Completed P1.74: residual DOM collection iterator/prototype and object-copy parity sweep
  - moved `DOMTokenList` and `NamedNodeMap` iterator/method raw getters onto receiver-aware callables so extracted property reads, bracket access, `Symbol.iterator`, and inherited reads use the same live collection semantics instead of owner-captured placeholders
  - filled the remaining `NodeList` reflective surface so `Object.keys(...)`, `Object.getOwnPropertyNames(...)`, `Reflect.ownKeys(...)`, `Object.getOwnPropertyDescriptor(...)`, `Object.hasOwn(...)`, `Object.assign(...)`, and object spread all synthesize live indexed/length properties from the current list snapshot
  - aligned collection copy paths by teaching object-literal spread and `Object.assign(...)` to enumerate synthesized DOM collection keys, which keeps copied results in sync with live state while still respecting explicit own overrides and accessors

- [x] Verify
  - `cargo test --lib named_node_map_raw_getter_methods_and_iterators_are_receiver_aware_work -- --nocapture`
  - `cargo test --lib class_list_iterator_property_paths_and_object_copy_work -- --nocapture`
  - `cargo test --lib node_list_reflective_surface_and_object_copy_work -- --nocapture`
  - `cargo test --lib dom_named_node_map`
  - `cargo test --lib dom_element_attributes_property`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo fmt`
  - `cargo test --lib` (`2330 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.75: DOM collection prototype identity and named-property collision residual sweep)

- [x] Completed P1.75: DOM collection prototype identity and named-property collision residual sweep
  - stabilized `NodeList` default prototype lookup so `Object.getPrototypeOf(...)` on static lists no longer manufactures fresh empty objects and now reuses the shared object prototype when no explicit override is present
  - cached live `classList` wrappers by owner node so repeated `element.classList` reads reuse the same host collection object instead of recreating wrapper state on every access
  - filtered `NamedNodeMap` synthesized named properties through built-in/prototype visibility checks so collisions with `item`, `getNamedItem`, `values`, `toString`, `constructor`, and similar surface members no longer leak into own keys, descriptors, or object-copy paths

- [x] Verify
  - `cargo test --lib static_node_list_default_prototype_is_stable_work -- --nocapture`
  - `cargo test --lib named_node_map_named_property_collisions_follow_builtin_visibility_work -- --nocapture`
  - `cargo test --lib dom_named_node_map`
  - `cargo test --lib dom_element_attributes_property`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib webapi_data_builtins`
  - `cargo fmt`
  - `cargo test --lib` (`2332 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.76: DOM collection expando-assignment and explicit prototype-mutation residual sweep)

- [x] Completed P1.76: DOM collection expando-assignment and explicit prototype-mutation residual sweep
  - cached live `DOMStringMap` wrappers by owner node so repeated `element.dataset` reads reuse the same object and preserve expando state across explicit prototype mutation just like `classList` and `attributes`
  - taught direct and inherited property lookup for `DOMStringMap`, `DOMTokenList`, and `NamedNodeMap` to fall through to explicit prototype objects for missing synthesized keys while still keeping live indexed/named/value surfaces authoritative when present
  - allowed `NodeList`/`HTMLCollection`-like live lists to accept non-index expando assignment and explicit prototype overrides without turning indexed items or `length` into writable data properties
  - added regressions for `dataset`, `classList`, `attributes`, and live `children` collections so expando reads, inherited custom methods, `Object.create(...)`, and `in` semantics stay aligned after `Object.setPrototypeOf(...)`

- [x] Verify
  - `cargo test --lib class_list_expando_assignment_and_explicit_prototype_mutation_work -- --nocapture`
  - `cargo test --lib dataset_expando_assignment_and_explicit_prototype_mutation_work -- --nocapture`
  - `cargo test --lib element_attributes_expando_assignment_and_explicit_prototype_mutation_work -- --nocapture`
  - `cargo test --lib live_children_node_list_expando_assignment_and_explicit_prototype_mutation_work -- --nocapture`
  - `cargo test --lib dom_element_attributes_property`
  - `cargo test --lib dom_named_node_map`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2336 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.77: DOM collection defineProperty/delete and live-wrapper descriptor residual sweep)

- [x] Completed P1.77: DOM collection defineProperty/delete and live-wrapper descriptor residual sweep
  - added `Object.defineProperty(...)` support for live `NodeList` wrappers so explicit own data/accessor properties can shadow synthesized indexed items and `length` without breaking live collection identity
  - taught `delete` and `in` on `DOMStringMap` and `NodeList` to respect explicit own shadow properties first, revealing live synthesized keys again after shadow deletion instead of deleting underlying `data-*` state or losing indexed parity
  - updated the `NodeList` `length` fast path to honor explicit own getter/value overrides before falling back to synthesized live length, keeping bare-identifier reads aligned with generic object-property lookup
  - added regressions for dataset shadow/delete flows and live `children` descriptor shadowing so repeated wrapper reads, `Object.hasOwn(...)`, `Object.getOwnPropertyDescriptor(...)`, `delete`, and `in` stay aligned across cached live wrappers

- [x] Verify
  - `cargo test --lib dataset_define_property_delete_and_live_wrapper_identity_work -- --nocapture`
  - `cargo test --lib live_children_node_list_define_property_delete_and_in_parity_work -- --nocapture`
  - `cargo test --lib dom_element_attributes_property`
  - `cargo test --lib dom_named_node_map`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2338 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.78: DOM collection fast-path breadth and remaining named/indexed shadow parity sweep)

- [x] Completed P1.78: DOM collection fast-path breadth and remaining named/indexed shadow parity sweep
  - routed direct `element.dataset.foo` reads through the cached live `DOMStringMap` wrapper so named fast paths now honor explicit own overrides, deleted markers, prototype fallback, and live `data-*` updates instead of bypassing wrapper state
  - routed direct `element.classList.length` and `element.children.length` reads through the cached live wrapper objects so `length` fast paths respect explicit own getter/value shadowing before falling back to synthesized live lengths
  - added regressions for direct dataset dot/bracket access, classList indexed/length shadowing, and live `children` indexed/length shadowing so repeated direct reads stay aligned with wrapper-based property access after define/delete flows and live DOM mutations

- [x] Verify
  - `cargo test --lib dataset_direct_fast_path_respects_live_wrapper_shadow_and_proto_work -- --nocapture`
  - `cargo test --lib direct_dom_collection_fast_paths_respect_live_wrapper_shadowing_work -- --nocapture`
  - `cargo test --lib dom_element_attributes_property`
  - `cargo test --lib dom_named_node_map`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2340 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.79: document-backed collection fast-path and wrapper-identity residual sweep)

- [x] Completed P1.79: document-backed collection fast-path and wrapper-identity residual sweep
  - routed `document.forms`, `images`, `links`, and `scripts` through cached live wrappers in both parser-specialized fast paths and generic document property lookup so direct `length`/indexed/property reads preserve wrapper identity, explicit own overrides, and deleted markers
  - added dedicated live query-selector-backed node-list refresh support so document-backed collections stay live while repeated `querySelectorAll(...)` reads still return fresh static wrappers that do not leak prior expando shadow state across rereads or DOM mutation
  - added regressions covering document-backed collection identity/shadowing/live update behavior plus static-vs-live query wrapper parity for object-copy, own-key, and descriptor surfaces

- [x] Verify
  - `cargo test --lib document_backed_collections_are_live_cached_and_shadow_aware_work -- --nocapture`
  - `cargo test --lib query_selector_all_reread_returns_fresh_static_wrappers_after_shadowing_work -- --nocapture`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_element_query_selector_all_method`
  - `cargo test --lib dom_link_element`
  - `cargo test --lib dom_script_element`
  - `cargo test --lib dom_area_element`
  - `cargo test --lib window_forms_trace`
  - `cargo fmt`
  - `cargo test --lib` (`2342 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.80: document collection named-property and live HTMLCollection residual sweep)

- [x] Completed P1.80: document collection named-property and live HTMLCollection residual sweep
  - introduced an explicit HTMLCollection-like collection kind for live `children`, `getElementsByClassName`, `getElementsByTagName`, `getElementsByTagNameNS`, and document-backed `forms`/`images`/`links`/`scripts` wrappers so named properties and `namedItem(...)` are only exposed on live HTMLCollection surfaces, not on static `querySelectorAll(...)` node lists
  - aligned named-property visibility with builtin collisions, prototype fallback, expando shadowing, `delete`, `Object.hasOwn(...)`, `in`, descriptor introspection, `Reflect.ownKeys(...)`, and object-copy/spread surfaces so id/name-backed properties stay live while explicit own overrides still win
  - updated existing wrapper-parity regressions and added dedicated coverage for document-backed named properties plus live descendant HTMLCollection named access after mutation

- [x] Verify
  - `cargo test --lib document_backed_collections_expose_named_properties_and_hide_builtin_collisions_work -- --nocapture`
  - `cargo test --lib live_html_collection_named_properties_and_builtin_collisions_work -- --nocapture`
  - `cargo test --lib document_backed_collections_are_live_cached_and_shadow_aware_work -- --nocapture`
  - `cargo test --lib live_children_node_list_define_property_delete_and_in_parity_work -- --nocapture`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_element_get_elements_by_tag_name_method`
  - `cargo test --lib dom_element_get_elements_by_class_name_method`
  - `cargo test --lib dom_element_children_property`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2344 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.81: HTMLCollection prototype/tag surface and static NodeList contrast sweep)

- [x] Completed P1.81: HTMLCollection prototype/tag surface and static NodeList contrast sweep
  - split cached default constructor/prototype/tag surfaces for static `NodeList` wrappers versus live HTMLCollection-like collections so `Object.prototype.toString`, `constructor`, `constructor.prototype`, and inherited callable identity no longer collapse both families into the same runtime shape
  - routed `item(...)`, `namedItem(...)`, iterator access, and prototype-path method reads through stable receiver-aware prototype objects so raw getter identity, inherited lookup, and incompatible receiver behavior match the intended static-vs-live distinction
  - tightened `in`/prototype traversal for collection wrappers so HTMLCollection named properties and static NodeList inherited methods are observed through the correct default prototype chain without reintroducing direct fast-path masquerading

- [x] Verify
  - `cargo test --lib static_node_list_default_prototype_is_stable_work -- --nocapture`
  - `cargo test --lib node_list_explicit_prototype_override_controls_inherited_lookup_work -- --nocapture`
  - `cargo test --lib live_children_node_list_expando_assignment_and_explicit_prototype_mutation_work -- --nocapture`
  - `cargo test --lib node_list_reflective_surface_and_object_copy_work -- --nocapture`
  - `cargo test --lib language_core_expressions`
  - `cargo test --lib dom_element_query_selector_all_method`
  - `cargo fmt`
  - `cargo test --lib` (`2344 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.82: HTMLCollection constructor exposure breadth and specialized collection residual sweep)

- [x] Completed P1.82: HTMLCollection constructor exposure breadth and specialized collection residual sweep
  - exposed stable `NodeList`, `HTMLCollection`, `HTMLFormControlsCollection`, and `HTMLOptionsCollection` constructors on env/window paths and split their default prototype chains so static lists and specialized live collections report the correct constructor, tag, and inherited HTMLCollection surface
  - upgraded `form.elements`, `select.options`, `select.selectedOptions`, and `datalist.options` to cached live wrappers with specialized collection kinds, stable identity, and browser-like stringification while preserving live refresh through generic property lookup
  - aligned DOM property parsing with the specialized wrapper path so collection properties fall back to the generic object/property machinery instead of failing parser-side when accessed through direct DOM member expressions

- [x] Verify
  - `cargo test --lib dom_collection_constructors_are_exposed_and_specialized_prototypes_chain_work -- --nocapture`
  - `cargo test --lib form_elements_is_live_cached_and_specialized_collection_surface_work -- --nocapture`
  - `cargo test --lib select_options_and_selected_options_are_live_cached_specialized_collections_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2347 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.83: specialized collection readonly/index shadowing and constructor call-surface residual sweep)

- [x] Completed P1.83: specialized collection readonly/index shadowing and constructor call-surface residual sweep
  - fixed synthesized `NodeList` / HTMLCollection-like property descriptors so live collection `length`, indexed entries, and named properties expose readonly-but-configurable flags that match the assignment fast-path and survive `Object.defineProperty(...)` shadow/delete round-trips
  - verified `HTMLFormControlsCollection` and `HTMLOptionsCollection` inherit the shared HTMLCollection callable surface through raw getter, bracket access, extracted call, and incompatible-receiver paths while keeping constructor illegal-call behavior stable
  - added focused regressions for `form.elements` and `select.options` shadowing so explicit own overrides on `0`, `length`, and named properties delete cleanly and reveal the live synthesized collection surface again

- [x] Verify
  - `cargo test --lib form_elements_define_property_delete_and_shadow_parity_work -- --nocapture`
  - `cargo test --lib select_options_define_property_delete_and_shadow_parity_work -- --nocapture`
  - `cargo test --lib specialized_collection_constructors_share_html_collection_callable_surface_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2350 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.84: specialized collection named-collision breadth and selectedOptions/html-collection contrast sweep)

- [x] Completed P1.84: specialized collection named-collision breadth and selectedOptions/html-collection contrast sweep
  - confirmed `HTMLFormControlsCollection` and `HTMLOptionsCollection` already preserve builtin callable visibility when control names collide with `item`, `namedItem`, `length`, `constructor`, iterator helpers, and other builtin collection keys, then locked that behavior with focused regression coverage
  - verified `selectedOptions` and `datalist.options` continue to use shared `HTMLCollection` constructor/tag semantics, stable wrapper identity, and live `namedItem(...)` updates without leaking colliding names into reflective own-key surfaces
  - added dedicated collision regressions for `form.elements`, `select.selectedOptions`, and `datalist.options` so specialized-versus-shared collection behavior stays explicit even when names overlap builtin members

- [x] Verify
  - `cargo test --lib form_elements_named_property_collisions_keep_builtin_surface_visible_work -- --nocapture`
  - `cargo test --lib selected_options_and_datalist_options_keep_html_collection_collision_rules_work -- --nocapture`
  - `cargo test --lib dom_form_element`
  - `cargo test --lib dom_select_element`
  - `cargo test --lib dom_datalist_element`
  - `cargo fmt`
  - `cargo test --lib` (`2352 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.85: specialized collection multi-match namedItem and RadioNodeList residual sweep)

- [x] Completed P1.85: specialized collection multi-match namedItem and RadioNodeList residual sweep
  - upgraded `HTMLFormControlsCollection` multi-match named lookups so `form.elements[name]` and `namedItem(...)` return stable live `RadioNodeList` wrappers when multiple controls share the same `name`/`id`, while preserving wrapper identity, ordering, liveness, illegal-constructor behavior, and grouped `value` semantics for radio groups
  - exposed `RadioNodeList` on env/window paths with the correct constructor/prototype chain and wired grouped form-control wrappers through the shared live collection cache so specialized `form.elements[...]` parsing falls back to generic property access for named/dynamic lookups instead of collapsing back to single-element fast paths
  - confirmed `selectedOptions`, `options`, and other specialized/shared live collections keep their existing single-element or `HTMLCollection` behavior and do not accidentally adopt grouped-return `RadioNodeList` semantics

- [x] Verify
  - `cargo test --lib form_elements_multi_match_named_lookup_returns_live_radio_node_lists_work -- --nocapture`
  - `cargo test --lib selected_options_and_options_duplicate_names_do_not_switch_to_radio_node_lists_work -- --nocapture`
  - `cargo test --lib form_elements_index_supports_expression -- --nocapture`
  - `cargo test --lib dom_form_element`
  - `cargo test --lib dom_select_element`
  - `cargo test --lib dom_datalist_element`
  - `cargo test --lib language_core_expressions`
  - `cargo fmt`
  - `cargo test --lib` (`2354 passed, 0 failed`)

- [x] Confirmed no new mock was required (no README update)

## Next Task (P1.86: RadioNodeList reflective surface and grouped form-control descriptor parity sweep)

- [ ] Audit `RadioNodeList` reflective/object-copy behavior
  - verify `Object.keys(...)`, `Object.getOwnPropertyNames(...)`, `Reflect.ownKeys(...)`, `Object.getOwnPropertyDescriptor(...)`, `Object.assign(...)`, object spread, and prototype/introspection paths match browser behavior for grouped form-control results

- [ ] Tighten grouped form-control shadowing and indexed-property parity
  - confirm grouped `form.elements[name]` wrappers handle expando assignment, `Object.defineProperty(...)`, `delete`, indexed access, `item(...)`, and named collisions without regressing live updates or shared collection fast paths

- [ ] Verify
  - targeted regressions for `RadioNodeList` reflective surface and grouped shadow/delete parity
  - related form/select collection suites
  - `cargo test --lib`
