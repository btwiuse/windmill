use uuid::Uuid;
use windmill_common::{agent_workers::QueueInitJob, utils::HTTP_CLIENT, BASE_URL};

pub async fn queue_init_job(worker_name: &str, content: &str) -> anyhow::Result<Uuid> {
    let client = HTTP_CLIENT.clone();
    let url = format!("{}/agent_workers/queue_init_job", BASE_URL.read().await);
    let response = client
        .post(url)
        .json(&QueueInitJob { worker_name: worker_name.to_string(), content: content.to_string() })
        .send()
        .await?;
    let status = response.status();
    if status.is_success() {
        Ok(Uuid::parse_str(&response.text().await?)?)
    } else {
        Err(anyhow::anyhow!("Failed to create initial job"))
    }
}
