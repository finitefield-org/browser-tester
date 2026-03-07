# HTML仕様準拠ロードマップ: 次の着手 TODO

## 現在位置

- `P0: Parsing, Tree Construction, and Serialization` の table fragment / `outerHTML` slice は実装と実行系検証まで完了
- `P1.1: attribute reflection audit 拡張`（enumerated / URL / numeric 追加）は実装と検証まで完了
- `P1.2: reflection coverage hardening`（URL 棚卸し / enumerated invalid matrix / numeric clamp 拡張）は実装と検証まで完了
- `P1.3: reflection parity tightening`（missing-default / fast-path整合 / numeric監査拡張）は実装と検証まで完了
- `P1.4: reflection edge-case tightening`（formAction owner/default・min/maxLength境界・rows/cols上限・enumerated追加）は実装と検証まで完了
- `P1.5: reflection consistency sweep`（URL delimiter正規化・numeric validity相互作用・fast-path拡張）は実装と検証まで完了
- `P1.6: reflection matrix deepening`（URL special/opaque matrix・型別validity再評価・`min/max/step` fast-path）は実装と検証まで完了

## 今回スライスの実施結果（P1.6: reflection matrix deepening）

- [x] URL reflection の特殊ケース matrix を固定した
  - `anchor` の `protocol` / `host` / `hostname` / `port` / `pathname` setter を special URL と opaque URL で比較し、差分を shared test で固定した
  - `username/password` setter を no-host URL（`mailto:` / `data:`）と `file:` で no-op になるように実装し、shared test で回帰化した

- [x] numeric validity の型別再評価を固定した
  - `number` / `range` / `date` / `time` / `datetime-local` で `min/max/step` 変更後の `rangeUnderflow` / `rangeOverflow` / `stepMismatch` 再計算を shared matrix 化した
  - `step base`（`min` 優先・`value` 基準）と丸め境界ケースを shared test に追加した

- [x] parser fast path の候補監査を進めた
  - `DomProp` に `Min` / `Max` / `Step` を追加した
  - parser / resolver / runtime getter / runtime setter を接続し、fast-path と通常 property path の挙動を揃えた
  - member chain + bracket access を含む `min/max/step` parity test を shared に追加した

- [x] shared behavior テストを段階追加した
  - `src/tests/dom_attribute_reflection_shared.rs`
  - `attribute_reflection_html_2_6_1_url_anchor_setter_special_and_opaque_protocol_host_port_pathname_matrix_work`
  - `attribute_reflection_html_2_6_1_url_anchor_username_password_setter_is_noop_for_no_host_and_file_urls`
  - `attribute_reflection_html_2_3_3_numeric_validity_recomputes_after_min_max_step_mutations_across_supported_types`
  - `attribute_reflection_html_2_3_3_numeric_step_base_prefers_min_then_value_attribute_and_rounding_boundary_work`
  - `attribute_reflection_html_2_3_3_parser_fast_path_matches_min_max_step_reflection_with_bracket_and_member_chain_access`

- [x] 検証完了
  - `cargo test --lib dom_attribute_reflection_shared`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib` (`2137 passed, 0 failed`)

- [x] 新規 mock 不要を確認（README 追記なし）

## Traceability

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `8.5`, `13.2.6.4.9` | `src/core_impl/dom/text_html_content.rs`, `src/tests/dom_element_outer_html_property.rs` | table 親配下 `outerHTML` 置換・table context 補正・回帰テストを固定済み | なし | none | `element_outer_html_set_html_8_5_13_2_6_4_9_*`, `element_outer_html_set_html_8_5_13_2_6_4_13_*` | implemented + verified |
| `2.3.1` | shared reflection helper + assignment paths | boolean reflected attribute の presence semantics を shared helper に集約済み | 特になし（維持フェーズ） | none | `attribute_reflection_html_2_3_1_*` | implemented + verified |
| `2.3.2` | shared reflection helper + getter/setter paths | `draggable`/`spellcheck`/`translate` に加え `dir` / `autocapitalize` / `autocomplete` の missing/invalid/case-variant を shared テストで固定済み | form関連 enumerated（`form.autocomplete` など）の owner/default 相互作用監査は継続余地 | none | `attribute_reflection_html_2_3_2_*` | implemented + verified |
| `2.3.3` | shared reflection helper + getter/setter + fast-path paths | `min/max/step` fast-path + `number/range/date/time/datetime-local` の再評価 matrix + `step base` 境界を固定済み | `step='any'` の型別挙動（特に `datetime-local`）と bracket assignment 系の parser/runtime 経路は継続監査余地 | none | `attribute_reflection_html_2_3_3_*` | implemented + verified |
| `2.6.1` | shared reflection helper + URL getter/setter paths | `anchor` special/opaque setter matrix + `username/password` の no-host/file no-op を固定済み | default port 正規化（`80/443`）と scheme 切替時（special↔special/opaque）の保持規則監査は継続余地 | none | `attribute_reflection_html_2_6_1_*` | implemented + verified |

## 次のタスク（P1.7: reflection semantics tightening）

- [ ] URL reflection の正規化ルールを詰める
  - default port（`http:80`, `https:443`）の反映/省略規則を shared test で固定する
  - `protocol` 切替（special↔special, special↔opaque）での authority/path 保持規則を matrix 化する

- [ ] validity の残差分を詰める
  - `datetime-local` を含む型別 `step='any'` 挙動を shared test 化し、必要なら実装修正する
  - `time` の跨日レンジ（`min > max`）と `step` 併用時の境界を追加監査する

- [ ] parser/property path の整合性を詰める
  - static string bracket assignment（`el['prop'] = ...`）の parser 対応範囲を拡張し、dot access と parity を確保する
  - 追加した fast-path key で object path との差分（expando / reflected property）を棚卸しして回帰テスト化する

- [ ] 検証する
  - 追加した targeted tests（URL-normalization / step-any / bracket-assignment parity）
  - `cargo test --lib dom_attribute_reflection_shared`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib`
