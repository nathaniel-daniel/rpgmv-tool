use std::collections::HashMap;

/// Note: This struct is incomplete
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SystemAdvanced {
    #[serde(rename = "mainFontFilename")]
    pub main_font_filename: String,

    #[serde(rename = "fontSize")]
    pub font_size: u16,

    #[serde(rename = "screenWidth")]
    pub screen_width: u16,

    /// Extra k/v entries
    pub extra: HashMap<String, serde_json::Value>,
}

/// Note: This struct is incomplete
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct System {
    /// This field is MZ only.
    pub advanced: Option<SystemAdvanced>,

    /// Extra k/v entries
    pub extra: HashMap<String, serde_json::Value>,
}
