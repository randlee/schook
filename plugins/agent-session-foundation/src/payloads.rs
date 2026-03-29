use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SessionStartPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[allow(dead_code)]
    pub transcript_path: Option<String>,
    pub source: String,
    #[allow(dead_code)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionEndPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[allow(dead_code)]
    pub transcript_path: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreCompactPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[allow(dead_code)]
    pub transcript_path: Option<String>,
    #[allow(dead_code)]
    pub trigger: Option<String>,
    #[allow(dead_code)]
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StopPayload {
    pub session_id: String,
    #[serde(rename = "cwd")]
    pub _cwd: String,
    #[allow(dead_code)]
    pub transcript_path: Option<String>,
    #[allow(dead_code)]
    pub stop_hook_active: bool,
    #[allow(dead_code)]
    pub permission_mode: Option<String>,
    #[allow(dead_code)]
    pub last_assistant_message: Option<String>,
}
