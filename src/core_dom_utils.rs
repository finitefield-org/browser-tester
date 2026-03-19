use super::*;
use std::error::Error as StdError;
use std::fmt;

#[path = "core_dom_utils/encoding_utils.rs"]
mod encoding_utils;
#[path = "core_dom_utils/html_css_utils.rs"]
mod html_css_utils;
#[path = "core_dom_utils/input_normalization.rs"]
mod input_normalization;

pub(crate) use encoding_utils::*;
pub(crate) use html_css_utils::*;
pub(crate) use input_normalization::*;

pub(crate) const INTERNAL_RETURN_SLOT: &str = "__bt_internal_return_value__";
pub(crate) const INTERNAL_SYMBOL_KEY_PREFIX: &str = "\u{0}\u{0}bt_symbol_key:";
pub(crate) const INTERNAL_SYMBOL_WRAPPER_KEY: &str = "\u{0}\u{0}bt_symbol_wrapper";
pub(crate) const INTERNAL_STRING_WRAPPER_VALUE_KEY: &str = "\u{0}\u{0}bt_string_wrapper_value";
pub(crate) const INTERNAL_BOOLEAN_WRAPPER_VALUE_KEY: &str = "\u{0}\u{0}bt_boolean_wrapper_value";
pub(crate) const INTERNAL_NUMBER_WRAPPER_VALUE_KEY: &str = "\u{0}\u{0}bt_number_wrapper_value";
pub(crate) const INTERNAL_BIGINT_WRAPPER_VALUE_KEY: &str = "\u{0}\u{0}bt_bigint_wrapper_value";
pub(crate) const INTERNAL_OBJECT_PROTOTYPE_KEY: &str = "\u{0}\u{0}bt_object:prototype";
pub(crate) const INTERNAL_OBJECT_GETTER_KEY_PREFIX: &str = "\u{0}\u{0}bt_object:getter:";
pub(crate) const INTERNAL_OBJECT_SETTER_KEY_PREFIX: &str = "\u{0}\u{0}bt_object:setter:";
pub(crate) const INTERNAL_OBJECT_UNDEFINED_GETTER_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_object:getter_undefined:";
pub(crate) const INTERNAL_OBJECT_UNDEFINED_SETTER_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_object:setter_undefined:";
pub(crate) const INTERNAL_NON_ENUMERABLE_CONSTRUCTOR_KEY: &str =
    "\u{0}\u{0}bt_object:non_enumerable_constructor";
pub(crate) const INTERNAL_NON_ENUMERABLE_PROPERTY_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_object:non_enumerable:";
pub(crate) const INTERNAL_NON_WRITABLE_PROPERTY_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_object:non_writable:";
pub(crate) const INTERNAL_NON_CONFIGURABLE_PROPERTY_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_object:non_configurable:";
pub(crate) const INTERNAL_DELETED_BUILTIN_PROPERTY_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_object:deleted_builtin:";
pub(crate) const INTERNAL_REGEXP_PROTOTYPE_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_regexp:prototype_object";
pub(crate) const INTERNAL_ARRAY_HOLE_KEY_PREFIX: &str = "\u{0}\u{0}bt_array:hole:";
pub(crate) const INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY: &str =
    "\u{0}\u{0}bt_arguments:param_bindings";
pub(crate) const INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY: &str =
    "\u{0}\u{0}bt_class:super_constructor";
pub(crate) const INTERNAL_CLASS_SUPER_PROTOTYPE_KEY: &str = "\u{0}\u{0}bt_class:super_prototype";
pub(crate) const INTERNAL_CONST_BINDINGS_KEY: &str = "\u{0}\u{0}bt_const:bindings";
pub(crate) const INTERNAL_INTL_KEY_PREFIX: &str = "\u{0}\u{0}bt_intl:";
pub(crate) const INTERNAL_INTL_KIND_KEY: &str = "\u{0}\u{0}bt_intl:kind";
pub(crate) const INTERNAL_INTL_LOCALE_KEY: &str = "\u{0}\u{0}bt_intl:locale";
pub(crate) const INTERNAL_INTL_OPTIONS_KEY: &str = "\u{0}\u{0}bt_intl:options";
pub(crate) const INTERNAL_INTL_LOCALE_DATA_KEY: &str = "\u{0}\u{0}bt_intl:localeData";
pub(crate) const INTERNAL_INTL_CASE_FIRST_KEY: &str = "\u{0}\u{0}bt_intl:caseFirst";
pub(crate) const INTERNAL_INTL_SENSITIVITY_KEY: &str = "\u{0}\u{0}bt_intl:sensitivity";
pub(crate) const INTERNAL_INTL_NUMERIC_KEY: &str = "\u{0}\u{0}bt_intl:numeric";
pub(crate) const INTERNAL_INTL_BOUND_COMPARE_KEY: &str = "\u{0}\u{0}bt_intl:boundCompare";
pub(crate) const INTERNAL_INTL_BOUND_FORMAT_KEY: &str = "\u{0}\u{0}bt_intl:boundFormat";
pub(crate) const INTERNAL_INTL_SEGMENTS_KEY: &str = "\u{0}\u{0}bt_intl:segments";
pub(crate) const INTERNAL_INTL_SEGMENT_INDEX_KEY: &str = "\u{0}\u{0}bt_intl:segmentIndex";
pub(crate) const INTERNAL_CALLABLE_KEY_PREFIX: &str = "\u{0}\u{0}bt_callable:";
pub(crate) const INTERNAL_CALLABLE_KIND_KEY: &str = "\u{0}\u{0}bt_callable:kind";
pub(crate) const INTERNAL_BOUND_CALLABLE_TARGET_KEY: &str = "\u{0}\u{0}bt_callable:bound_target";
pub(crate) const INTERNAL_BOUND_CALLABLE_THIS_KEY: &str = "\u{0}\u{0}bt_callable:bound_this";
pub(crate) const INTERNAL_BOUND_CALLABLE_ARGS_KEY: &str = "\u{0}\u{0}bt_callable:bound_args";
pub(crate) const INTERNAL_STATIC_METHOD_NAME_KEY: &str = "\u{0}\u{0}bt_callable:static_method";
pub(crate) const INTERNAL_STATIC_TYPED_ARRAY_KIND_KEY: &str =
    "\u{0}\u{0}bt_callable:static_typed_array_kind";
pub(crate) const INTERNAL_WORKER_KEY_PREFIX: &str = "\u{0}\u{0}bt_worker:";
pub(crate) const INTERNAL_WORKER_OBJECT_KEY: &str = "\u{0}\u{0}bt_worker:object";
pub(crate) const INTERNAL_WORKER_GLOBAL_OBJECT_KEY: &str = "\u{0}\u{0}bt_worker:global";
pub(crate) const INTERNAL_WORKER_TARGET_KEY: &str = "\u{0}\u{0}bt_worker:target";
pub(crate) const INTERNAL_WORKER_TERMINATED_KEY: &str = "\u{0}\u{0}bt_worker:terminated";
pub(crate) const INTERNAL_CANVAS_KEY_PREFIX: &str = "\u{0}\u{0}bt_canvas:";
pub(crate) const INTERNAL_CANVAS_2D_CONTEXT_OBJECT_KEY: &str = "\u{0}\u{0}bt_canvas:2d_context";
pub(crate) const INTERNAL_CANVAS_2D_ALPHA_KEY: &str = "\u{0}\u{0}bt_canvas:2d_alpha";
pub(crate) const INTERNAL_CANVAS_2D_CONTEXT_NODE_EXPANDO_KEY: &str =
    "\u{0}\u{0}bt_canvas:2d_context_value";
pub(crate) const INTERNAL_CANVAS_CONTEXT_MODE_NODE_EXPANDO_KEY: &str =
    "\u{0}\u{0}bt_canvas:context_mode";
pub(crate) const INTERNAL_CANVAS_TRANSFERRED_TO_OFFSCREEN_NODE_EXPANDO_KEY: &str =
    "\u{0}\u{0}bt_canvas:transferred_to_offscreen";
pub(crate) const INTERNAL_CANVAS_2D_LINE_DASH_KEY: &str = "\u{0}\u{0}bt_canvas:2d_line_dash";
pub(crate) const INTERNAL_CANVAS_2D_TRANSFORM_KEY: &str = "\u{0}\u{0}bt_canvas:2d_transform";
pub(crate) const INTERNAL_LOCATION_OBJECT_KEY: &str = "\u{0}\u{0}bt_location";
pub(crate) const INTERNAL_HISTORY_OBJECT_KEY: &str = "\u{0}\u{0}bt_history";
pub(crate) const INTERNAL_NAVIGATION_OBJECT_KEY: &str = "\u{0}\u{0}bt_navigation";
pub(crate) const INTERNAL_WINDOW_OBJECT_KEY: &str = "\u{0}\u{0}bt_window";
pub(crate) const INTERNAL_DOCUMENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_document";
pub(crate) const INTERNAL_POPUP_WINDOW_OBJECT_KEY: &str = "\u{0}\u{0}bt_popup_window";
pub(crate) const INTERNAL_POPUP_DOCUMENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_popup_document";
pub(crate) const INTERNAL_POPUP_DOCUMENT_HTML_KEY: &str = "\u{0}\u{0}bt_popup_document:html";
pub(crate) const INTERNAL_ATTR_OBJECT_KEY: &str = "\u{0}\u{0}bt_attr";
pub(crate) const INTERNAL_NAMED_NODE_MAP_KEY_PREFIX: &str = "\u{0}\u{0}bt_named_node_map:";
pub(crate) const INTERNAL_NAMED_NODE_MAP_OBJECT_KEY: &str = "\u{0}\u{0}bt_named_node_map:object";
pub(crate) const INTERNAL_NAMED_NODE_MAP_OWNER_NODE_KEY: &str =
    "\u{0}\u{0}bt_named_node_map:owner_node";
pub(crate) const INTERNAL_SCOPE_DEPTH_KEY: &str = "\u{0}\u{0}bt_scope_depth";
pub(crate) const INTERNAL_PENDING_SCOPE_START_KEY: &str = "\u{0}\u{0}bt_pending_scope_start";
pub(crate) const INTERNAL_GLOBAL_SYNC_NAMES_KEY: &str = "\u{0}\u{0}bt_global_sync_names";
pub(crate) const INTERNAL_LOCAL_BINDINGS_KEY: &str = "\u{0}\u{0}bt_local_bindings";
pub(crate) const INTERNAL_TOP_LEVEL_LEXICAL_BINDINGS_KEY: &str =
    "\u{0}\u{0}bt_top_level_lexical_bindings";
pub(crate) const INTERNAL_NAVIGATOR_OBJECT_KEY: &str = "\u{0}\u{0}bt_navigator";
pub(crate) const INTERNAL_CLIPBOARD_OBJECT_KEY: &str = "\u{0}\u{0}bt_clipboard";
pub(crate) const INTERNAL_CLIPBOARD_READ_TEXT_DEFAULT_KEY: &str =
    "\u{0}\u{0}bt_clipboard:read_text_default";
pub(crate) const INTERNAL_CLIPBOARD_WRITE_TEXT_DEFAULT_KEY: &str =
    "\u{0}\u{0}bt_clipboard:write_text_default";
pub(crate) const INTERNAL_CLIPBOARD_DATA_OBJECT_KEY: &str = "\u{0}\u{0}bt_clipboard:data";
pub(crate) const INTERNAL_CLIPBOARD_DATA_TEXT_KEY: &str = "\u{0}\u{0}bt_clipboard:data:text";
pub(crate) const INTERNAL_CLIPBOARD_DATA_STORE_KEY: &str = "\u{0}\u{0}bt_clipboard:data:store";
pub(crate) const INTERNAL_CLIPBOARD_DATA_TYPES_KEY: &str = "\u{0}\u{0}bt_clipboard:data:types";
pub(crate) const INTERNAL_DATA_TRANSFER_OBJECT_KEY: &str = "\u{0}\u{0}bt_data_transfer:object";
pub(crate) const INTERNAL_DATA_TRANSFER_EVENT_TYPE_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:event_type";
pub(crate) const INTERNAL_DATA_TRANSFER_FILES_KEY: &str = "\u{0}\u{0}bt_data_transfer:files";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEMS_KEY: &str = "\u{0}\u{0}bt_data_transfer:items";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item:object";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_KIND_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item:kind";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_TYPE_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item:type";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_DATA_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item:data";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_LIST_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item_list:object";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_LIST_OWNER_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item_list:owner";
pub(crate) const INTERNAL_DATA_TRANSFER_ITEM_LIST_EVENT_TYPE_KEY: &str =
    "\u{0}\u{0}bt_data_transfer:item_list:event_type";
pub(crate) const INTERNAL_CLIPBOARD_ITEM_KEY_PREFIX: &str = "\u{0}\u{0}bt_clipboard_item:";
pub(crate) const INTERNAL_CLIPBOARD_ITEM_OBJECT_KEY: &str = "\u{0}\u{0}bt_clipboard_item:object";
pub(crate) const INTERNAL_MOCK_FILE_KEY_PREFIX: &str = "\u{0}\u{0}bt_mock_file:";
pub(crate) const INTERNAL_MOCK_FILE_OBJECT_KEY: &str = "\u{0}\u{0}bt_mock_file:object";
pub(crate) const INTERNAL_MOCK_FILE_BLOB_KEY: &str = "\u{0}\u{0}bt_mock_file:blob";
pub(crate) const INTERNAL_CLASS_LIST_OBJECT_KEY: &str = "\u{0}\u{0}bt_class_list:object";
pub(crate) const INTERNAL_CLASS_LIST_NODE_KEY: &str = "\u{0}\u{0}bt_class_list:node";
pub(crate) const INTERNAL_IMAGE_BITMAP_OBJECT_KEY: &str = "\u{0}\u{0}bt_image_bitmap:object";
pub(crate) const INTERNAL_IMAGE_BITMAP_WIDTH_KEY: &str = "\u{0}\u{0}bt_image_bitmap:width";
pub(crate) const INTERNAL_IMAGE_BITMAP_HEIGHT_KEY: &str = "\u{0}\u{0}bt_image_bitmap:height";
pub(crate) const INTERNAL_TEXT_TRACK_OBJECT_KEY: &str = "\u{0}\u{0}bt_text_track:object";
pub(crate) const INTERNAL_TEXT_TRACK_NODE_KEY: &str = "\u{0}\u{0}bt_text_track:node";
pub(crate) const INTERNAL_TEXT_TRACK_MODE_KEY: &str = "\u{0}\u{0}bt_text_track:mode";
pub(crate) const INTERNAL_DOM_RECT_OBJECT_KEY: &str = "\u{0}\u{0}bt_dom_rect:object";
pub(crate) const INTERNAL_DOM_RECT_LIST_OBJECT_KEY: &str = "\u{0}\u{0}bt_dom_rect_list:object";
pub(crate) const INTERNAL_TIME_RANGES_OBJECT_KEY: &str = "\u{0}\u{0}bt_time_ranges:object";
pub(crate) const INTERNAL_TIME_RANGES_MEDIA_NODE_KEY: &str = "\u{0}\u{0}bt_time_ranges:media_node";
pub(crate) const INTERNAL_TIME_RANGES_KIND_KEY: &str = "\u{0}\u{0}bt_time_ranges:kind";
pub(crate) const INTERNAL_MEDIA_CURRENT_TIME_KEY: &str = "\u{0}\u{0}bt_media:current_time";
pub(crate) const INTERNAL_MEDIA_PAUSED_KEY: &str = "\u{0}\u{0}bt_media:paused";
pub(crate) const INTERNAL_MEDIA_PLAY_CALLABLE_KEY: &str = "\u{0}\u{0}bt_media:play_callable";
pub(crate) const INTERNAL_MEDIA_PAUSE_CALLABLE_KEY: &str = "\u{0}\u{0}bt_media:pause_callable";
pub(crate) const INTERNAL_MEDIA_LOAD_CALLABLE_KEY: &str = "\u{0}\u{0}bt_media:load_callable";
pub(crate) const INTERNAL_MEDIA_CAN_PLAY_TYPE_CALLABLE_KEY: &str =
    "\u{0}\u{0}bt_media:can_play_type_callable";
pub(crate) const INTERNAL_MEDIA_FAST_SEEK_CALLABLE_KEY: &str =
    "\u{0}\u{0}bt_media:fast_seek_callable";
pub(crate) const INTERNAL_MEDIA_VOLUME_KEY: &str = "\u{0}\u{0}bt_media:volume";
pub(crate) const INTERNAL_MEDIA_DURATION_KEY: &str = "\u{0}\u{0}bt_media:duration";
pub(crate) const INTERNAL_MEDIA_PLAYBACK_RATE_KEY: &str = "\u{0}\u{0}bt_media:playback_rate";
pub(crate) const INTERNAL_MEDIA_DEFAULT_PLAYBACK_RATE_KEY: &str =
    "\u{0}\u{0}bt_media:default_playback_rate";
pub(crate) const INTERNAL_DOM_STRING_MAP_KEY_PREFIX: &str = "\u{0}\u{0}bt_dom_string_map:";
pub(crate) const INTERNAL_DOM_STRING_MAP_OBJECT_KEY: &str = "\u{0}\u{0}bt_dom_string_map:object";
pub(crate) const INTERNAL_DOM_STRING_MAP_OWNER_NODE_KEY: &str =
    "\u{0}\u{0}bt_dom_string_map:owner_node";
pub(crate) const INTERNAL_COOKIE_STORE_OBJECT_KEY: &str = "\u{0}\u{0}bt_cookie_store";
pub(crate) const INTERNAL_CACHE_STORAGE_OBJECT_KEY: &str = "\u{0}\u{0}bt_cache_storage";
pub(crate) const INTERNAL_CACHE_OBJECT_KEY: &str = "\u{0}\u{0}bt_cache";
pub(crate) const INTERNAL_CACHE_NAME_KEY: &str = "\u{0}\u{0}bt_cache:name";
pub(crate) const INTERNAL_FETCH_RESPONSE_OBJECT_KEY: &str = "\u{0}\u{0}bt_fetch:response";
pub(crate) const INTERNAL_FETCH_RESPONSE_BODY_KEY: &str = "\u{0}\u{0}bt_fetch:response:body";
pub(crate) const INTERNAL_FETCH_RESPONSE_STATUS_KEY: &str = "\u{0}\u{0}bt_fetch:response:status";
pub(crate) const INTERNAL_FETCH_RESPONSE_STATUS_TEXT_KEY: &str =
    "\u{0}\u{0}bt_fetch:response:status_text";
pub(crate) const INTERNAL_FETCH_RESPONSE_URL_KEY: &str = "\u{0}\u{0}bt_fetch:response:url";
pub(crate) const INTERNAL_FETCH_REQUEST_OBJECT_KEY: &str = "\u{0}\u{0}bt_fetch:request";
pub(crate) const INTERNAL_FETCH_REQUEST_INPUT_KEY: &str = "\u{0}\u{0}bt_fetch:request:input";
pub(crate) const INTERNAL_FETCH_REQUEST_URL_KEY: &str = "\u{0}\u{0}bt_fetch:request:url";
pub(crate) const INTERNAL_FETCH_REQUEST_METHOD_KEY: &str = "\u{0}\u{0}bt_fetch:request:method";
pub(crate) const INTERNAL_HEADERS_OBJECT_KEY: &str = "\u{0}\u{0}bt_fetch:headers";
pub(crate) const INTERNAL_HEADERS_ENTRIES_KEY: &str = "\u{0}\u{0}bt_fetch:headers:entries";
pub(crate) const INTERNAL_DOM_PARSER_OBJECT_KEY: &str = "\u{0}\u{0}bt_dom_parser";
pub(crate) const INTERNAL_XML_SERIALIZER_OBJECT_KEY: &str = "\u{0}\u{0}bt_xml_serializer";
pub(crate) const INTERNAL_PARSED_DOCUMENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_parsed_document";
pub(crate) const INTERNAL_PARSED_DOCUMENT_ROOT_NODE_KEY: &str =
    "\u{0}\u{0}bt_parsed_document:root_node";
pub(crate) const INTERNAL_PARSED_DOCUMENT_CONTENT_TYPE_KEY: &str =
    "\u{0}\u{0}bt_parsed_document:content_type";
pub(crate) const INTERNAL_EVENT_TARGET_OBJECT_KEY: &str = "\u{0}\u{0}bt_event_target:object";
pub(crate) const INTERNAL_MATCH_MEDIA_OBJECT_KEY: &str = "\u{0}\u{0}bt_match_media:object";
pub(crate) const INTERNAL_MATCH_MEDIA_QUERY_KEY: &str = "\u{0}\u{0}bt_match_media:query";
pub(crate) const INTERNAL_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:object";
pub(crate) const INTERNAL_HASH_CHANGE_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:hashchange";
pub(crate) const INTERNAL_ERROR_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:error";
pub(crate) const INTERNAL_BEFORE_UNLOAD_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:beforeunload";
pub(crate) const INTERNAL_KEYBOARD_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:keyboard";
pub(crate) const INTERNAL_WHEEL_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:wheel";
pub(crate) const INTERNAL_NAVIGATE_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:navigate";
pub(crate) const INTERNAL_POINTER_EVENT_OBJECT_KEY: &str = "\u{0}\u{0}bt_event:pointer";
pub(crate) const INTERNAL_EVENT_STOP_PROPAGATION_KEY: &str = "\u{0}\u{0}bt_event:stop_propagation";
pub(crate) const INTERNAL_EVENT_STOP_IMMEDIATE_PROPAGATION_KEY: &str =
    "\u{0}\u{0}bt_event:stop_immediate_propagation";
pub(crate) const INTERNAL_TREE_WALKER_OBJECT_KEY: &str = "\u{0}\u{0}bt_tree_walker";
pub(crate) const INTERNAL_TREE_WALKER_TRAVERSAL_NODES_KEY: &str =
    "\u{0}\u{0}bt_tree_walker:traversal_nodes";
pub(crate) const INTERNAL_TREE_WALKER_INDEX_KEY: &str = "\u{0}\u{0}bt_tree_walker:index";
pub(crate) const INTERNAL_TREE_WALKER_WHAT_TO_SHOW_KEY: &str =
    "\u{0}\u{0}bt_tree_walker:what_to_show";
pub(crate) const INTERNAL_RANGE_OBJECT_KEY: &str = "\u{0}\u{0}bt_range";
pub(crate) const INTERNAL_SELECTION_OBJECT_KEY: &str = "\u{0}\u{0}bt_selection";
pub(crate) const INTERNAL_ANIMATION_OBJECT_KEY: &str = "\u{0}\u{0}bt_animation";
pub(crate) const INTERNAL_RANGE_START_CONTAINER_KEY: &str = "\u{0}\u{0}bt_range:start_container";
pub(crate) const INTERNAL_RANGE_START_OFFSET_KEY: &str = "\u{0}\u{0}bt_range:start_offset";
pub(crate) const INTERNAL_RANGE_END_CONTAINER_KEY: &str = "\u{0}\u{0}bt_range:end_container";
pub(crate) const INTERNAL_RANGE_END_OFFSET_KEY: &str = "\u{0}\u{0}bt_range:end_offset";
pub(crate) const INTERNAL_SELECTION_RANGE_KEY: &str = "\u{0}\u{0}bt_selection:range";
pub(crate) const INTERNAL_READABLE_STREAM_OBJECT_KEY: &str = "\u{0}\u{0}bt_readable_stream";
pub(crate) const INTERNAL_WRITABLE_STREAM_OBJECT_KEY: &str = "\u{0}\u{0}bt_writable_stream";
pub(crate) const INTERNAL_TEXT_ENCODER_OBJECT_KEY: &str = "\u{0}\u{0}bt_text_encoder:object";
pub(crate) const INTERNAL_TEXT_DECODER_OBJECT_KEY: &str = "\u{0}\u{0}bt_text_decoder:object";
pub(crate) const INTERNAL_TEXT_DECODER_ENCODING_KEY: &str = "\u{0}\u{0}bt_text_decoder:encoding";
pub(crate) const INTERNAL_TEXT_DECODER_FATAL_KEY: &str = "\u{0}\u{0}bt_text_decoder:fatal";
pub(crate) const INTERNAL_TEXT_DECODER_IGNORE_BOM_KEY: &str =
    "\u{0}\u{0}bt_text_decoder:ignore_bom";
pub(crate) const INTERNAL_TEXT_ENCODER_STREAM_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_text_encoder_stream:object";
pub(crate) const INTERNAL_TEXT_ENCODER_STREAM_READABLE_KEY: &str =
    "\u{0}\u{0}bt_text_encoder_stream:readable";
pub(crate) const INTERNAL_TEXT_ENCODER_STREAM_WRITABLE_KEY: &str =
    "\u{0}\u{0}bt_text_encoder_stream:writable";
pub(crate) const INTERNAL_TEXT_DECODER_STREAM_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_text_decoder_stream:object";
pub(crate) const INTERNAL_TEXT_DECODER_STREAM_ENCODING_KEY: &str =
    "\u{0}\u{0}bt_text_decoder_stream:encoding";
pub(crate) const INTERNAL_TEXT_DECODER_STREAM_FATAL_KEY: &str =
    "\u{0}\u{0}bt_text_decoder_stream:fatal";
pub(crate) const INTERNAL_TEXT_DECODER_STREAM_IGNORE_BOM_KEY: &str =
    "\u{0}\u{0}bt_text_decoder_stream:ignore_bom";
pub(crate) const INTERNAL_TEXT_DECODER_STREAM_READABLE_KEY: &str =
    "\u{0}\u{0}bt_text_decoder_stream:readable";
pub(crate) const INTERNAL_TEXT_DECODER_STREAM_WRITABLE_KEY: &str =
    "\u{0}\u{0}bt_text_decoder_stream:writable";
pub(crate) const INTERNAL_CSS_STYLE_SHEET_KEY_PREFIX: &str = "\u{0}\u{0}bt_css_style_sheet:";
pub(crate) const INTERNAL_CSS_STYLE_SHEET_OBJECT_KEY: &str = "\u{0}\u{0}bt_css_style_sheet:object";
pub(crate) const INTERNAL_CSS_STYLE_SHEET_OWNER_DOCUMENT_KEY: &str =
    "\u{0}\u{0}bt_css_style_sheet:owner_document";
pub(crate) const INTERNAL_CSS_STYLE_SHEET_RULES_KEY: &str = "\u{0}\u{0}bt_css_style_sheet:rules";
pub(crate) const INTERNAL_ADOPTED_STYLE_SHEETS_ARRAY_KEY: &str =
    "\u{0}\u{0}bt_adopted_style_sheets:array";
pub(crate) const INTERNAL_ADOPTED_STYLE_SHEETS_OWNER_DOCUMENT_KEY: &str =
    "\u{0}\u{0}bt_adopted_style_sheets:owner_document";
pub(crate) const INTERNAL_COMPUTED_STYLE_KEY_PREFIX: &str = "\u{0}\u{0}bt_computed_style:";
pub(crate) const INTERNAL_COMPUTED_STYLE_OBJECT_KEY: &str = "\u{0}\u{0}bt_computed_style:object";
pub(crate) const INTERNAL_COMPUTED_STYLE_TARGET_NODE_KEY: &str =
    "\u{0}\u{0}bt_computed_style:target_node";
pub(crate) const INTERNAL_COMPUTED_STYLE_PSEUDO_KEY: &str = "\u{0}\u{0}bt_computed_style:pseudo";
pub(crate) const INTERNAL_IMPORT_META_OBJECT_KEY: &str = "\u{0}\u{0}bt_import_meta:object";
pub(crate) const INTERNAL_NEW_TARGET_KEY: &str = "\u{0}\u{0}bt_new_target";
pub(crate) const INTERNAL_URL_OBJECT_KEY: &str = "\u{0}\u{0}bt_url:object";
pub(crate) const INTERNAL_URL_OBJECT_ID_KEY: &str = "\u{0}\u{0}bt_url:id";
pub(crate) const INTERNAL_URL_SEARCH_PARAMS_KEY_PREFIX: &str = "\u{0}\u{0}bt_url_search_params:";
pub(crate) const INTERNAL_URL_SEARCH_PARAMS_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_url_search_params:object";
pub(crate) const INTERNAL_URL_SEARCH_PARAMS_ENTRIES_KEY: &str =
    "\u{0}\u{0}bt_url_search_params:entries";
pub(crate) const INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY: &str =
    "\u{0}\u{0}bt_url_search_params:owner_id";
pub(crate) const INTERNAL_STORAGE_KEY_PREFIX: &str = "\u{0}\u{0}bt_storage:";
pub(crate) const INTERNAL_STORAGE_OBJECT_KEY: &str = "\u{0}\u{0}bt_storage:object";
pub(crate) const INTERNAL_STORAGE_ENTRIES_KEY: &str = "\u{0}\u{0}bt_storage:entries";
pub(crate) const INTERNAL_ITERATOR_KEY_PREFIX: &str = "\u{0}\u{0}bt_iterator:";
pub(crate) const INTERNAL_ITERATOR_CONSTRUCTOR_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_iterator:constructor";
pub(crate) const INTERNAL_ITERATOR_OBJECT_KEY: &str = "\u{0}\u{0}bt_iterator:object";
pub(crate) const INTERNAL_ITERATOR_VALUES_KEY: &str = "\u{0}\u{0}bt_iterator:values";
pub(crate) const INTERNAL_ITERATOR_INDEX_KEY: &str = "\u{0}\u{0}bt_iterator:index";
pub(crate) const INTERNAL_ITERATOR_TARGET_KEY: &str = "\u{0}\u{0}bt_iterator:target";
pub(crate) const INTERNAL_ITERATOR_RETURN_VALUE_KEY: &str = "\u{0}\u{0}bt_iterator:return_value";
pub(crate) const INTERNAL_ITERATOR_RETURN_EMITTED_KEY: &str =
    "\u{0}\u{0}bt_iterator:return_emitted";
pub(crate) const INTERNAL_ASYNC_ITERATOR_KEY_PREFIX: &str = "\u{0}\u{0}bt_async_iterator:";
pub(crate) const INTERNAL_ASYNC_ITERATOR_OBJECT_KEY: &str = "\u{0}\u{0}bt_async_iterator:object";
pub(crate) const INTERNAL_ASYNC_ITERATOR_VALUES_KEY: &str = "\u{0}\u{0}bt_async_iterator:values";
pub(crate) const INTERNAL_ASYNC_ITERATOR_INDEX_KEY: &str = "\u{0}\u{0}bt_async_iterator:index";
pub(crate) const INTERNAL_ASYNC_ITERATOR_TARGET_KEY: &str = "\u{0}\u{0}bt_async_iterator:target";
pub(crate) const INTERNAL_ASYNC_GENERATOR_KEY_PREFIX: &str = "\u{0}\u{0}bt_async_generator:";
pub(crate) const INTERNAL_ASYNC_GENERATOR_OBJECT_KEY: &str = "\u{0}\u{0}bt_async_generator:object";
pub(crate) const INTERNAL_ASYNC_GENERATOR_PROTOTYPE_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_async_generator:prototype_object";
pub(crate) const INTERNAL_GENERATOR_KEY_PREFIX: &str = "\u{0}\u{0}bt_generator:";
pub(crate) const INTERNAL_GENERATOR_OBJECT_KEY: &str = "\u{0}\u{0}bt_generator:object";
pub(crate) const INTERNAL_GENERATOR_PROTOTYPE_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_generator:prototype_object";
pub(crate) const INTERNAL_GENERATOR_FUNCTION_KEY_PREFIX: &str = "\u{0}\u{0}bt_generator_function:";
pub(crate) const INTERNAL_GENERATOR_FUNCTION_CONSTRUCTOR_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_generator_function:constructor_object";
pub(crate) const INTERNAL_GENERATOR_FUNCTION_PROTOTYPE_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_generator_function:prototype_object";
pub(crate) const INTERNAL_ASYNC_GENERATOR_FUNCTION_KEY_PREFIX: &str =
    "\u{0}\u{0}bt_async_generator_function:";
pub(crate) const INTERNAL_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_async_generator_function:constructor_object";
pub(crate) const INTERNAL_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OBJECT_KEY: &str =
    "\u{0}\u{0}bt_async_generator_function:prototype_object";
pub(crate) const INTERNAL_GENERATOR_YIELD_LIMIT_REACHED: &str =
    "\u{0}\u{0}bt_generator:yield_limit_reached";
pub(crate) const GENERATOR_MAX_BUFFERED_YIELDS: usize = 2048;
pub(crate) const DEFAULT_COLOR_INPUT_VALUE: &str = "#000000";
pub(crate) const DEFAULT_RANGE_INPUT_MIN: f64 = 0.0;
pub(crate) const DEFAULT_RANGE_INPUT_MAX: f64 = 100.0;
pub(crate) const FILE_INPUT_FAKEPATH_PREFIX: &str = "C:\\fakepath\\";
pub(crate) const DEFAULT_LOCALE: &str = "en-US";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    HtmlParse(String),
    ScriptParse(String),
    ScriptRuntime(String),
    ScriptThrown(ThrownValue),
    SelectorNotFound(String),
    UnsupportedSelector(String),
    TypeMismatch {
        selector: String,
        expected: String,
        actual: String,
    },
    AssertionFailed {
        selector: String,
        expected: String,
        actual: String,
        dom_snippet: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HtmlParse(msg) => write!(f, "html parse error: {msg}"),
            Self::ScriptParse(msg) => write!(f, "script parse error: {msg}"),
            Self::ScriptRuntime(msg) => write!(f, "script runtime error: {msg}"),
            Self::ScriptThrown(value) => {
                write!(f, "script thrown value: {}", value.as_string())
            }
            Self::SelectorNotFound(selector) => write!(f, "selector not found: {selector}"),
            Self::UnsupportedSelector(selector) => write!(f, "unsupported selector: {selector}"),
            Self::TypeMismatch {
                selector,
                expected,
                actual,
            } => write!(
                f,
                "type mismatch for {selector}: expected {expected}, actual {actual}"
            ),
            Self::AssertionFailed {
                selector,
                expected,
                actual,
                dom_snippet,
            } => write!(
                f,
                "assertion failed for {selector}: expected {expected}, actual {actual}, snippet {dom_snippet}"
            ),
        }
    }
}

impl StdError for Error {}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrownValue {
    pub(crate) value: Value,
}

impl ThrownValue {
    pub(crate) fn new(value: Value) -> Self {
        Self { value }
    }

    pub(crate) fn into_value(self) -> Value {
        self.value
    }

    pub(crate) fn as_string(&self) -> String {
        self.value.as_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockFile {
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub last_modified: i64,
    pub webkit_relative_path: String,
    pub bytes: Vec<u8>,
}

impl MockFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: normalize_file_input_name(name),
            size: 0,
            mime_type: String::new(),
            last_modified: 0,
            webkit_relative_path: String::new(),
            bytes: Vec::new(),
        }
    }

    pub fn with_bytes(mut self, bytes: &[u8]) -> Self {
        self.bytes = bytes.to_vec();
        self.size = self.bytes.len() as i64;
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.bytes = text.as_bytes().to_vec();
        self.size = self.bytes.len() as i64;
        if self.mime_type.is_empty() {
            self.mime_type = "text/plain".to_string();
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(pub(crate) usize);

#[derive(Debug, Clone)]
pub(crate) enum NodeType {
    Document,
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) node_type: NodeType,
}

#[derive(Debug, Clone)]
pub(crate) struct Element {
    pub(crate) tag_name: String,
    pub(crate) namespace_uri: Option<String>,
    pub(crate) attrs: HashMap<String, String>,
    pub(crate) value: String,
    pub(crate) default_value: String,
    pub(crate) dirty_value: bool,
    pub(crate) files: Vec<MockFile>,
    pub(crate) checked: bool,
    pub(crate) default_checked: bool,
    pub(crate) checked_dirty: bool,
    pub(crate) indeterminate: bool,
    pub(crate) selected: bool,
    pub(crate) default_selected: bool,
    pub(crate) selected_dirty: bool,
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
    pub(crate) required: bool,
    pub(crate) custom_validity_message: String,
    pub(crate) selection_start: usize,
    pub(crate) selection_end: usize,
    pub(crate) selection_direction: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Dom {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: NodeId,
    pub(crate) id_index: HashMap<String, Vec<NodeId>>,
    pub(crate) active_element: Option<NodeId>,
    pub(crate) active_pseudo_element: Option<NodeId>,
}

pub(crate) fn has_class(element: &Element, class_name: &str) -> bool {
    element
        .attrs
        .get("class")
        .map(|classes| classes.split_whitespace().any(|c| c == class_name))
        .unwrap_or(false)
}

pub(crate) fn is_valid_create_attribute_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

pub(crate) fn is_valid_qualified_attribute_name(name: &str) -> bool {
    let mut segments = name.split(':');
    let first = segments.next().unwrap_or_default();
    let second = segments.next();
    if segments.next().is_some() {
        return false;
    }
    match second {
        None => is_valid_create_attribute_name(first),
        Some(local) => {
            !first.is_empty()
                && !local.is_empty()
                && is_valid_create_attribute_name(first)
                && is_valid_create_attribute_name(local)
        }
    }
}

pub(crate) fn is_color_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("color"))
        .unwrap_or(false)
}

pub(crate) fn is_date_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("date"))
        .unwrap_or(false)
}

pub(crate) fn is_datetime_local_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("datetime-local"))
        .unwrap_or(false)
}

pub(crate) fn is_time_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("time"))
        .unwrap_or(false)
}

pub(crate) fn is_file_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("file"))
        .unwrap_or(false)
}

pub(crate) fn is_number_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("number"))
        .unwrap_or(false)
}

pub(crate) fn is_range_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("range"))
        .unwrap_or(false)
}

pub(crate) fn is_password_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("password"))
        .unwrap_or(false)
}

pub(crate) fn is_image_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| kind.eq_ignore_ascii_case("image"))
        .unwrap_or(false)
}

pub(crate) fn uses_dirty_value_state(element: &Element) -> bool {
    if element.tag_name.eq_ignore_ascii_case("textarea") {
        return true;
    }
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }

    element
        .attrs
        .get("type")
        .map(|kind| {
            matches!(
                kind.to_ascii_lowercase().as_str(),
                "" | "text" | "search" | "tel" | "url" | "email"
            )
        })
        .unwrap_or(true)
}

pub(crate) fn format_float(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value == f64::INFINITY {
        return "Infinity".to_string();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }

    let raw = format!("{value}");
    let Some(exp_idx) = raw.find('e').or_else(|| raw.find('E')) else {
        return raw;
    };
    let mantissa = &raw[..exp_idx];
    let exponent_src = &raw[exp_idx + 1..];
    let exponent = exponent_src.parse::<i32>().unwrap_or(0);
    format!("{mantissa}e{:+}", exponent)
}

pub(crate) fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut it = value.chars();
    let mut out = String::new();
    for _ in 0..max_chars {
        let Some(ch) = it.next() else {
            return out;
        };
        out.push(ch);
    }
    if it.next().is_some() {
        out.push_str("...");
    }
    out
}
