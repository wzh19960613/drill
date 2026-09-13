use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookItemDef {
    pub id: String,

    pub source: String,
    #[serde(rename = "optionOrder", default)]
    pub option_order: Option<Vec<usize>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub seed: String,
    pub date: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub items: Vec<BookItemDef>,
}
