use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QOption {
    pub id: u8,
    pub text: String,
}
