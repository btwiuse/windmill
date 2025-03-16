/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2042
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use crate::{
    db::{ApiAuthed, DB},
    users::{maybe_refresh_folders, require_owner_of_path},
    utils::require_super_admin,
};

use axum::{
    async_trait,
    extract::{Extension, FromRequestParts, Path},
    routing::{delete, post},
    Json, Router,
};
use http::request::Parts;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use windmill_common::{
    agent_workers::QueueInitJob,
    db::UserDB,
    error::Result,
    jwt::{decode_with_internal_secret, encode_with_internal_secret, JWT_SECRET},
    utils::StripPath,
};
use windmill_queue::push_init_job;

pub fn global_service() -> Router {
    Router::new()
        .route("/queue_init_job", post(queue_init_job))
        .route("/create_agent_token", post(create_agent_token))
}

async fn queue_init_job(
    AgentClaims { worker_name_prefix, .. }: AgentClaims,
    Extension(db): Extension<DB>,
    Json(QueueInitJob { content, worker_name }): Json<QueueInitJob>,
) -> Result<StatusCode> {
    if !worker_name.starts_with(&worker_name_prefix) {
        return Err(anyhow::anyhow!("Worker name must start with {}", worker_name_prefix).into());
    }
    push_init_job(&db, content, &worker_name).await?;
    Ok(StatusCode::OK)
}

#[derive(Serialize, Deserialize)]
struct AgentClaims {
    worker_name_prefix: String,
    tags: Vec<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AgentClaims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        if parts.method == http::Method::OPTIONS {
            return Ok(AgentClaims { worker_name_prefix: "".to_string(), tags: Vec::new() });
        };

        let auth_header = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        if let Some(token) = auth_header {
            let claims = decode_with_internal_secret::<AgentClaims>(&token).await;
            match claims {
                Ok(claims) => Ok(claims),
                Err(_) => Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_owned())),
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_owned()))
        }
    }
}

async fn create_agent_token(
    authed: ApiAuthed,
    Extension(db): Extension<DB>,
    Json(claims): Json<AgentClaims>,
) -> Result<String> {
    require_super_admin(&db, &authed.email).await?;
    let token = encode_with_internal_secret(claims).await?;
    Ok(token)
}
