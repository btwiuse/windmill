use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct QueueInitJob {
    pub content: String,
}
