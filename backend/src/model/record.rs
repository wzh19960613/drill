use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    pub id: u64,
    #[serde(rename = "questionId")]
    pub question_id: String,
    /// Id of the source (question bank) the question belongs to.
    pub source: String,
    pub correct: bool,
    pub at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ms: Option<u64>,
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
