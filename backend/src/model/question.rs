use serde::{Deserialize, Serialize};

use super::QOption;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Question {
    pub id: String,
    pub source: String,

    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub locate: String,
    pub chapter: String,
    pub qtype: String,
    pub stem: Vec<String>,
    pub options: Vec<QOption>,
    pub correct_id: Option<u8>,
    pub correct_ids: Vec<u8>,

    pub answer: Vec<String>,
    pub solution: Vec<String>,
    pub notes: Vec<String>,
}
