use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SessionStartPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub transcript_path: Option<String>,
    pub source: String,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
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
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
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
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub transcript_path: Option<String>,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub trigger: Option<String>,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
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
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub transcript_path: Option<String>,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub stop_hook_active: bool,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub permission_mode: Option<String>,
    #[expect(
        dead_code,
        reason = "captured provider payload keeps this field even when the scaffold does not read it yet"
    )]
    pub last_assistant_message: Option<String>,
}
