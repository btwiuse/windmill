use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct QueueInitJob {
    pub content: String,
    pub worker_name: String,
}

use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

lazy_static! {
    pub static ref AGENT_HTTP_CLIENT: Client = ClientBuilder::new()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");
}
