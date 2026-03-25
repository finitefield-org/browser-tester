# HTML Spec Conformance TODO

## Status

- `P0` through `P4` are complete.
- Rolling maintenance work is complete for the current pass.
- Latest full verification on the old root workspace: `cargo test --lib` with `2561 passed, 0 failed`.
- No new test-only mock is currently required.

## Current Posture

- The backlog is dormant/on-demand.
- Reopen work only if one of these happens:
  - a new public API family is exposed
  - a browser-comparison-backed regression cluster appears
  - a harness/modeling change broadens a stabilized contract

## Next Task

- [ ] `Maintenance: Trigger-driven selective intake reopening`
  - reopen the smallest justified selective intake slice only when a concrete trigger appears
  - otherwise keep the roadmap dormant

## Test Migration Details

At the time of migration, the remaining `tests/integration_cases/*.rs` files not listed below were a 1:1 filename/function match between `tests/` and `crates/browser-tester/tests/`.

### `contract_harness_core.rs`
Root test functions: `stable_core_actions_and_assertions_work_together`, `stable_core_assert_exists_reports_presence_and_missing_selectors`, `stable_core_constructors_and_time_controls_work`, `stable_test_mock_fetch_contract_is_direct`, `stable_test_mock_clipboard_contract_is_direct`, `stable_test_mock_location_contract_is_direct`, `stable_test_mock_file_input_contract_is_direct`, `stable_core_scheduler_controls_are_direct`, `stable_core_trace_and_determinism_controls_are_direct`, `stable_core_limit_validation_errors_are_direct`, `stable_test_mock_dialog_and_match_media_controls_are_direct`, `stable_test_mock_clipboard_error_controls_are_direct`
Next test functions: `stable_core_actions_and_assertions_work_together`, `stable_core_assert_exists_reports_presence_and_missing_selectors`, `stable_core_constructors_and_time_controls_work`, `stable_test_mock_fetch_contract_is_direct`, `stable_test_mock_clipboard_contract_is_direct`, `stable_test_mock_location_contract_is_direct`, `stable_test_mock_file_input_contract_is_direct`, `stable_core_scheduler_controls_are_direct`, `stable_test_mock_dialog_and_match_media_controls_are_direct`, `stable_core_debug_view_reports_metadata`, `stable_core_selector_dump_dom_returns_matching_node_markup`, `stable_core_keyboard_dispatch_reaches_bubbling_listeners`, `stable_core_negative_time_rejected`
Status: `partial`
Missing in next: `stable_core_trace_and_determinism_controls_are_direct`, `stable_core_limit_validation_errors_are_direct`, `stable_test_mock_clipboard_error_controls_are_direct`

### `regression_real_world_html.rs`
Root test functions: `ignores_json_ld_script_blocks_and_runs_executable_script`, `json_ld_with_escaped_quotes_does_not_break_script_end_detection`, `script_end_extractor_handles_regex_literals_with_quotes`, `unicode_literal_in_regex_match_works_in_input_handler`, `fragment_input_still_exposes_document_body`, `array_from_supports_nodelist_and_map_callback`, `trailing_commas_in_literals_are_supported_without_allowing_sparse_entries`, `nested_object_path_access_on_runtime_objects_is_supported`, `csv_deduplicator_inline_script_does_not_fail_with_unclosed_block`, `malformed_escaped_empty_string_literals_are_normalized_before_parse`, `cron_descriptor_inline_script_does_not_fail_with_unsupported_expression`, `function_declaration_can_be_called_before_its_definition`, `create_lot_row_seed_name_property_uses_object_semantics`, `function_reassignment_of_global_is_visible_across_functions`, `function_can_call_global_function_declared_later`, `a_then_b_reads_updated_global_binding`, `function_a_update_is_visible_to_function_b_in_same_event`, `bind_function_listener_closure_can_call_local_close`, `foreach_map_reduce_sort_callbacks_reflect_outer_updates`, `run_calculation_pipeline_keeps_candidates_for_render_outputs`, `window_url_static_properties_are_object_like_and_assignable`, `local_storage_basic_methods_are_available`, `window_local_storage_is_assignable_for_stub_usage`, `from_html_with_local_storage_seeds_values_before_script_execution`, `document_member_calls_with_dynamic_arguments_are_supported`, `get_attribute_returns_null_for_missing_attribute_in_delegated_click_handler`, `async_click_handler_observes_updated_let_capture_for_clipboard`, `dom_expando_properties_round_trip_on_nodes`, `regex_lookahead_in_replace_parses_and_runs`, `utf8_script_assigned_text_is_preserved`, `non_executable_script_types_are_inert_and_text_is_preserved`, `template_content_clone_node_true_is_supported`
Status: `partial`
Note: the current implementation keeps the same scenario under `click_handler_observes_updated_let_capture_for_clipboard`; the async clipboard version remains only in the archived root snapshot.

### `parser_property_fuzz_test.rs`
Root test functions: `env_proptest_cases`, `parser_proptest_cases`, `identifier_strategy`, `literal_strategy`, `regex_literal_strategy`, `binary_operator_strategy`, `expression_strategy`, `simple_statement_strategy`, `statement_strategy`, `callback_body_strategy`, `escaped_script_end_tag_strategy`, `script_boundary_fragment_strategy`, `html_with_callback_body`, `assert_parser_path_never_panics`, `assert_script_boundary_path_never_reports_unclosed_script`, `parser_generated_statement_blocks_do_not_panic`, `parser_generated_expression_combinations_do_not_panic`, `parser_script_boundary_combinations_do_not_report_unclosed_script`
Next test functions: `#[path]` wrapper to `tests/parser_property_fuzz_test.rs`
Status: `wrapper`

### `runtime_property_fuzz_test.rs`
Root test functions: `env_proptest_cases`, `runtime_proptest_cases`, `text_input_strategy`, `ui_action_strategy`, `ui_action_sequence_strategy`, `run_action`, `assert_runtime_sequence_is_stable`, `runtime_rerendering_form_actions_do_not_panic`
Next test functions: `#[path]` wrapper to `tests/runtime_property_fuzz_test.rs`
Status: `wrapper`

### Former next-only additions
These contract test files were previously only in `next/` and are now part of the root workspace: `contract_dom_phase1.rs`, `contract_harness_builder.rs`, `contract_phase3.rs`, `contract_phase4.rs`, `contract_phase6.rs`, `contract_phase7.rs`, `contract_phase8.rs`, and `contract_script_phase2.rs`.
