use serde::Deserialize;

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
    pub source: String,
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
