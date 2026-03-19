use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTimer {
    pub id: i64,
    pub due_at: i64,
    pub order: i64,
    pub interval_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationNavigationKind {
    Assign,
    Replace,
    HrefSet,
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationNavigation {
    pub kind: LocationNavigationKind,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadArtifact {
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardPayloadArtifact {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardWriteArtifact {
    pub payloads: Vec<ClipboardPayloadArtifact>,
}

#[path = "runtime_state/browser_platform_state.rs"]
mod browser_platform_state;
#[path = "runtime_state/callback_handler_state.rs"]
mod callback_handler_state;
#[path = "runtime_state/event_script_runtime_state.rs"]
mod event_script_runtime_state;
#[path = "runtime_state/location_url_state.rs"]
mod location_url_state;

pub(crate) use browser_platform_state::*;
pub(crate) use callback_handler_state::*;
pub(crate) use event_script_runtime_state::*;
pub(crate) use location_url_state::*;
