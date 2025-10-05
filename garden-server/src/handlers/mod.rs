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

use serde_json::{json, Value as Json};

use axum::extract::{Path, State};
use axum::response::{Json as JsonResponse};

use libflowerpot::crypto::hash::Hash;
use libflowerpot::block::BlockContent;
use libflowerpot::storage::Storage;

use garden_protocol::Events;

#[derive(Debug, Clone)]
pub struct App<S: Storage> {
    storage: S
}

// TODO: in flowerpot implement direct transactions querying

pub async fn api_get_post<S: Storage>(
    State(state): State<App<S>>,
    Path(address): Path<String>
) -> anyhow::Result<JsonResponse<Json>> {
    let Some(hash) = Hash::from_base64(address) else {
        anyhow::bail!("invalid address hash format");
    };

    let mut block_hash = state.storage.root_block()
        .map_err(|err| {
            anyhow::anyhow!(err.to_string())
                .context("failed to read root block")
        })?;

    while let Some(curr_block_hash) = block_hash {
        let block = state.storage.read_block(&curr_block_hash)
            .map_err(|err| {
                anyhow::anyhow!(err.to_string())
                    .context("failed to read block")
            })?;

        if let Some(block) = block {
            if let BlockContent::Transactions(transactions) = block.content() {
                for transaction in transactions {
                    let transaction_hash = transaction.hash();

                    if transaction_hash != hash {
                        continue;
                    }

                    let Events::Post(post) = Events::from_bytes(transaction.data())? else {
                        return Ok(JsonResponse(Json::Null));
                    };
                }
            }
        }

        block_hash = state.storage.next_block(&curr_block_hash)
            .map_err(|err| {
                anyhow::anyhow!(err.to_string())
                    .context("failed to read next block")
            })?;
    }

    Ok(JsonResponse(Json::Null))
}
