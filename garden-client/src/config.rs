// SPDX-License-Identifier: GPL-3.0-or-later
//
// garden-client
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

use std::net::{SocketAddr, Ipv6Addr};

use anyhow::Context;
use serde_json::{json, Value as Json};

use flowerpot::crypto::hash::Hash;
use flowerpot::crypto::sign::VerifyingKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Address of the local flowerpot node.
    pub node_address: SocketAddr,

    /// List of bootstrap flowerpot nodes addresses.
    pub node_bootstrap: Vec<String>,

    /// Root block hash of a blockchain where the garden protocol is stored.
    pub blockchain_root_block: Hash,

    /// Verifying key of a blockchain where the garden protocol is stored.
    pub blockchain_verifying_key: VerifyingKey
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_address: SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 13400),
            node_bootstrap: Vec::new(),
            blockchain_root_block: Hash::from_base64("ErKCHaWAqeQDcty2qEo07tr8xQJrYGJ9LpRXAIDV41U=").unwrap(),
            blockchain_verifying_key: VerifyingKey::from_base64("AiFiwoYx3vzTu-VhU7AdrTIfKgF3hC8pd6cBjC0AfFd2").unwrap()
        }
    }
}

impl Config {
    pub fn to_json(&self) -> Json {
        json!({
            "node": {
                "address": self.node_address.to_string(),
                "bootstrap": self.node_bootstrap
            },
            "blockchain": {
                "root_block": self.blockchain_root_block.to_base64(),
                "verifying_key": self.blockchain_verifying_key.to_base64()
            }
        })
    }

    pub fn from_json(value: &Json) -> Self {
        let default = Self::default();

        Self {
            node_address: value.get("node")
                .map(|node| {
                    node.get("address")
                        .and_then(Json::as_str)
                        .and_then(|address| address.parse().ok())
                        .unwrap_or(default.node_address)
                })
                .unwrap_or(default.node_address),

            node_bootstrap: value.get("node")
                .map(|node| {
                node.get("bootstrap")
                    .and_then(Json::as_array)
                    .map(|bootstrap| {
                        bootstrap.iter()
                            .flat_map(Json::as_str)
                            .map(String::from)
                            .collect()
                    })
                    .unwrap_or(default.node_bootstrap.clone())
            })
            .unwrap_or(default.node_bootstrap),

            blockchain_root_block: value.get("blockchain")
                .map(|blockchain| {
                    blockchain.get("root_block")
                        .and_then(Json::as_str)
                        .and_then(Hash::from_base64)
                        .unwrap_or(default.blockchain_root_block)
                })
                .unwrap_or(default.blockchain_root_block),

            blockchain_verifying_key: value.get("blockchain")
                .map(|blockchain| {
                    blockchain.get("verifying_key")
                        .and_then(Json::as_str)
                        .and_then(VerifyingKey::from_base64)
                        .unwrap_or(default.blockchain_verifying_key.clone())
                })
                .unwrap_or(default.blockchain_verifying_key)
        }
    }
}

/// Try to read config file.
pub fn read() -> anyhow::Result<Config> {
    if !crate::CONFIG_FILE_PATH.is_file() {
        return Ok(Config::default());
    }

    let config = std::fs::read(crate::CONFIG_FILE_PATH.as_path())
        .context("failed to read config file")?;

    let config = serde_json::from_slice::<Json>(&config)
        .context("failed to deserialize config file")?;

    Ok(Config::from_json(&config))
}

/// Try to write config file.
pub fn write(config: &Config) -> anyhow::Result<()> {
    let config = serde_json::to_vec_pretty(&config.to_json())
        .context("failed to serialize config")?;

    std::fs::write(crate::CONFIG_FILE_PATH.as_path(), config)
        .context("failed to write config file")?;

    Ok(())
}
