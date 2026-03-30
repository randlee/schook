use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStartSource {
    Startup,
    Resume,
    Compact,
    Clear,
}

impl SessionStartSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Resume => "resume",
            Self::Compact => "compact",
            Self::Clear => "clear",
        }
    }

    pub fn establishes_root(self) -> bool {
        matches!(self, Self::Startup | Self::Resume | Self::Clear)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionStartPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub transcript_path: Option<String>,
    pub source: SessionStartSource,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionEndPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub transcript_path: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreCompactPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub transcript_path: Option<String>,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub trigger: Option<String>,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StopPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub transcript_path: Option<String>,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub stop_hook_active: bool,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub permission_mode: Option<String>,
    #[expect(
        dead_code,
        reason = "fixture-backed optional field is captured but not consumed in HP3"
    )]
    pub last_assistant_message: Option<String>,
}
