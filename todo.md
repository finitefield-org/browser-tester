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

## 今回スライスの実施結果（P1.18: file-host/arity parity residual sweep）

- [x] `file:` host と IDNA の混在 residual を固定した
  - Unicode / percent-decoded / trailing-dot host を伴う `file:` URL は constructor / `host` / `hostname` setter / `location` / `history.replaceState` / `document.URL` で同じ canonical host を返すように揃えた
  - invalid punycode token と bidi joiner を含む `file:` host setter は no-op、`localhost` は empty host へ正規化される挙動を URL object と location で揃えた

- [x] method-arity / extra-arg parity の残差を固定した
  - direct-variable `URL.toString(extra)` と `URLSearchParams.toString(extra)` は URL/URLSearchParams receiver を優先して直列化し、数値 `toString(radix)` 側への誤 dispatch を吸収した
  - `URLSearchParams.entries` / `keys` / `values` / `toString`、FormData `entries` / `keys` / `values` / `get` / `getAll` / `has` は extra args を無視しつつ side effect を評価するように揃えた
  - inline `new FormData(...).get*()` / `has()` は parse fast-path で弾かず generic member-call へ落とし、direct variable path と同じ arity semantics で実行されるようにした

- [x] 回帰テストを広げた
  - `src/tests/collections_url_typed_arrays.rs`
  - `url_file_idna_host_and_method_extra_args_work`
  - `src/tests/dom_navigation_dialog.rs`
  - `location_and_history_file_idna_host_residuals_work`
  - `src/tests/window_forms_trace.rs`
  - `form_data_inline_constructor_*_ignore_extra_args_work`
  - `form_data_{entries,keys,values}_ignore_extra_arguments_and_evaluate_side_effects_work`
  - 既存の `form_data_overlap_dispatch_ignores_extra_args_work` と合わせて direct variable / inline constructor / iterator path を固定した

- [x] 検証完了
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib window_forms_trace`
  - `cargo fmt`
  - `cargo test --lib` (`2190 passed, 0 failed`)

- [x] 新規 mock 不要を確認（README 追記なし）

## Traceability

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `8.5`, `13.2.6.4.9` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_outer_html_property.rs` | table 親配下 `outerHTML` 置換・table context 補正・回帰テストを固定済み | なし | none | `element_outer_html_set_html_8_5_13_2_6_4_9_*`, `element_outer_html_set_html_8_5_13_2_6_4_13_*` | implemented + verified |
| `2.3.1` | shared reflection helper + assignment paths | boolean reflected attribute の presence semantics を shared helper に集約済み | 特になし（維持フェーズ） | none | `attribute_reflection_html_2_3_1_*` | implemented + verified |
| `2.3.2` | shared reflection helper + getter/setter paths | `draggable`/`spellcheck`/`translate` に加え `dir` / `autocapitalize` / `autocomplete` の missing/invalid/case-variant を shared テストで固定済み | form関連 enumerated（`form.autocomplete` など）の owner/default 相互作用監査は継続余地 | none | `attribute_reflection_html_2_3_2_*` | implemented + verified |
| `2.3.3` | shared reflection helper + getter/setter + fast-path paths | `datetime-local` に加え `time` の fractional-second precision / millisecond step / wrapped range まで回帰化済み | `time` の token edge cases（過剰精度・境界トークン）と `step='any'` の追加監査は継続余地 | none | `attribute_reflection_html_2_3_3_*`, `html_input_datetime_local_*`, `html_input_time_*` | implemented + verified |
| `2.6.1` | shared reflection helper + URL getter/setter paths | default port 正規化 + special/file/opaque protocol switch + file/generic invalid authority reject + invalid absolute anchor subproperty semantics + protocol-relative base/fetch/navigation parity + invalid hyperlink activation no-op + special-host empty-host/backslash/hostless canonicalization + area/link null-URL getter parity + credential/delimiter encoding matrix + authority raw `%` reject / host percent-triplet decode / bare `%` preservation + malformed-percent searchParams decode + true IDNA/punycode host canonicalization + invalid-label reject/no-op + trailing-dot/full-stop variant parity + `file:` mixed IDNA host/location/history/document URL parity + URL/URLSearchParams/FormData extra-arg ignore/evaluation まで固定済み | generic member-call / object-path receiver に残る collection/WebIDL arity parity、non-URL receiver `toString` / `valueOf` 残差の追加監査は継続余地 | none | `attribute_reflection_html_2_6_1_*`, `url_*matrix_work`, `location_*no_op*_work`, `*_special_host_*`, `*_null_url_*`, `*_credentials_*`, `*_authority_and_percent_*`, `*_malformed_query_and_host_code_point_*`, `fetch_*canonical_mock_key*`, `fetch_*residuals*`, `form_data_*extra_args*` | implemented + verified |

## 次のタスク（P1.19: generic member-call collection/WebIDL parity sweep）

- [ ] generic member-call に残る collection/WebIDL arity residual を詰める
  - `Map` / `Set` / `WeakMap` / `WeakSet` / `Storage` / `URLSearchParams` の object-path / member-chain `entries` / `keys` / `values` / `clear` / `toString` が direct-variable path と同じ extra-arg ignore+evaluation になるか棚卸しする
  - zero-arg WebIDL method と `toString` / `valueOf` 系で parser specialization に依存している path を洗い、runtime dispatch で揃えられる箇所を広げる

- [ ] 回帰テストを追加する
  - `collections_url_typed_arrays` / `webapi_data_builtins` へ collection iterator / serializer の object-path parity を追加する
  - `window_forms_trace` へ FormData / collection overlap の member-chain path を追加する

- [ ] 検証する
  - 追加した targeted tests（collection/WebIDL arity parity）
  - `cargo test --lib collections_url_typed_arrays`
  - `cargo test --lib webapi_data_builtins`
  - `cargo test --lib window_forms_trace`
  - `cargo test --lib`
