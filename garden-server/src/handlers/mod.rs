// SPDX-License-Identifier: GPL-3.0-or-later
//
// garden-server
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

use serde_json::{json, Value as Json};

use axum::http::Request;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{Json as JsonResponse};

use libflowerpot::crypto::hash::Hash;
use libflowerpot::crypto::sign::SigningKey;
use libflowerpot::transaction::Transaction;
use libflowerpot::storage::Storage;
use libflowerpot::node::NodeHandler;

use garden_protocol::{Events, PostEvent, Content, Tag};

use crate::database::Database;

#[derive(Clone)]
pub struct App<S: Storage> {
    pub database: Database<S>,
    pub handler: Arc<NodeHandler<S>>
}

pub async fn api_send_post<S: Storage>(
    State(state): State<App<S>>,
    request: Request<Body>
) -> JsonResponse<Json> {
    let result = axum::body::to_bytes(request.into_body(), 65535).await;

    let body = match result {
        Ok(body) => body,

        Err(err) => return JsonResponse(json!({
            "error": {
                "code": "body_read_error",
                "message": err.to_string()
            }
        }))
    };

    let post = match serde_json::from_slice::<Json>(&body) {
        Ok(post) => post,

        Err(err) => return JsonResponse(json!({
            "error": {
                "code": "invalid_json_format",
                "message": err.to_string()
            }
        }))
    };

    let Some(signing_key) = post.get("signing_key").and_then(Json::as_str) else {
        return JsonResponse(json!({
            "error": {
                "code": "missing_field",
                "field": "signing_key",
                "message": "missing signing key"
            }
        }));
    };

    let Some(content) = post.get("content").and_then(Json::as_str) else {
        return JsonResponse(json!({
            "error": {
                "code": "missing_field",
                "field": "content",
                "message": "missing post content"
            }
        }));
    };

    let Some(tags) = post.get("tags").and_then(Json::as_array) else {
        return JsonResponse(json!({
            "error": {
                "code": "missing_field",
                "field": "tags",
                "message": "missing post tags"
            }
        }));
    };

    let Some(signing_key) = SigningKey::from_base64(signing_key) else {
        return JsonResponse(json!({
            "error": {
                "code": "invalid_signing_key",
                "message": "invalid signing key"
            }
        }));
    };

    let Some(content) = Content::new(content) else {
        return JsonResponse(json!({
            "error": {
                "code": "invalid_content",
                "message": "invalid post content"
            }
        }));
    };

    let tags = tags.iter()
        .map(|tag| {
            tag.as_str()
                .and_then(Tag::new)
        })
        .collect::<Option<Vec<Tag>>>();

    let Some(tags) = tags else {
        return JsonResponse(json!({
            "error": {
                "code": "invalid_tags",
                "message": "invalid post tags"
            }
        }));
    };

    let Some(event) = PostEvent::new(content, tags) else {
        return JsonResponse(json!({
            "error": {
                "code": "invalid_post",
                "message": "invalid post"
            }
        }));
    };

    let event = Events::from(event);

    let transaction = match Transaction::create(signing_key, event.to_bytes()) {
        Ok(transaction) => transaction,
        Err(err) => {
            return JsonResponse(json!({
                "error": {
                    "code": "create_transaction_error",
                    "message": format!("failed to create transaction: {err}")
                }
            }));
        }
    };

    state.handler.send_transaction(transaction);

    JsonResponse(Json::Null)
}

pub async fn api_get_post<S: Storage>(
    State(state): State<App<S>>,
    Path(address): Path<String>
) -> JsonResponse<Json> {
    let Some(hash) = Hash::from_base64(address) else {
        return JsonResponse(json!({
            "error": {
                "code": "invalid_hash_format",
                "message": "Invalid flowerpot transaction hash"
            }
        }));
    };

    let post = match state.database.query_post(&hash) {
        Ok(Some(post)) => post,

        Ok(None) => return JsonResponse(json!({
            "error": {
                "code": "transaction_not_found",
                "message": "There's no flowerpot transaction with such hash"
            }
        })),

        Err(err) => return JsonResponse(json!({
            "error": {
                "code": "internal_error",
                "message": err.to_string()
            }
        }))
    };

    JsonResponse(json!({
        "status": "staged",
        "content": post.content,
        "tags": post.tags,
        "comments": post.comments.into_iter()
            .map(|comment| comment.to_base64())
            .collect::<Vec<_>>(),
        "reactions": post.reactions.into_iter()
            .map(|reaction| {
                json!({
                    "name": reaction.name,
                    "author": reaction.author.to_base64()
                })
            })
            .collect::<Vec<_>>()
    }))
}
