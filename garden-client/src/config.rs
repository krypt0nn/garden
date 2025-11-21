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

use flowerpot::address::Address;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Address of the local flowerpot node.
    pub node_address: SocketAddr,

    /// List of bootstrap flowerpot nodes addresses.
    pub node_bootstrap: Vec<String>,

    /// Garden protocol blockchain address.
    pub blockchain_address: Address
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_address: SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 13400),
            node_bootstrap: Vec::new(),
            blockchain_address: Address::from_base64("AwVwKRoob1NIyRhn5vXtTD6H3yxpDO5Y7JRMruE8g25U5nbZGQ==").unwrap()
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
                "address": self.blockchain_address.to_base64()
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

            blockchain_address: value.get("blockchain")
                .map(|blockchain| {
                    blockchain.get("address")
                        .and_then(Json::as_str)
                        .and_then(Address::from_base64)
                        .unwrap_or(default.blockchain_address.clone())
                })
                .unwrap_or(default.blockchain_address)
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
