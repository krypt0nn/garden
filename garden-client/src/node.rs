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

use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{SeedableRng, RngCore};

use anyhow::Context;

use flowerpot::crypto::key_exchange::SecretKey;
use flowerpot::storage::sqlite_storage::SqliteStorage;
use flowerpot::protocol::network::{
    PacketStream, PacketStreamOptions, PacketStreamEncryption
};
use flowerpot::node::{Node, NodeOptions, NodeHandler};
use flowerpot::node::tracker::Tracker;

use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Progress {
    /// Open sqlite blockchain storage and create flowerpot blockchain tracker.
    CreateTracker(PathBuf),

    /// Establish connection with another flowepot node.
    EstablishConnection(SocketAddr),

    /// Synchronize flowerpot blockchain.
    SynchronizeBlockchain,

    /// Start flowerpot node.
    StartNode,

    /// Start background connections listener.
    StartListener(SocketAddr)
}

/// Try to start flowerpot node.
///
/// This method will establish packet streams with bootstrap nodes listed in the
/// config file, synchronize blockchain, start background thread to listen to
/// incoming stream connections and return started node handler.
///
/// It's recommended to use this function in a separate thread.
pub fn start(
    config: &Config,
    mut progress: impl FnMut(Progress)
) -> anyhow::Result<NodeHandler> {
    // Create the node.
    let mut node = Node::default();

    // Attach blockchain storage.
    let storage_path = crate::STORAGES_FOLDER_PATH
        .join(format!("{}.db", config.blockchain_root_block.to_base64()));

    progress(Progress::CreateTracker(storage_path.clone()));

    let storage = SqliteStorage::open(storage_path)
        .context("failed to open flowerpot blockchain storage")?;

    let tracker = Tracker::from_storage(storage);

    node.add_tracker(
        tracker,
        Some(config.blockchain_root_block),
        Some(config.blockchain_verifying_key.clone())
    );

    // Generate ECDH secret key.
    let mut rng = ChaCha20Rng::from_entropy();

    let mut rng = ChaCha20Rng::seed_from_u64(
        rng.next_u64() ^
        time::UtcDateTime::now().unix_timestamp() as u64
    );

    let secret_key = SecretKey::random(&mut rng);

    // Prepare packet stream options.
    let options = PacketStreamOptions {
        encryption_algorithms: vec![
            PacketStreamEncryption::ChaCha20,
            PacketStreamEncryption::ChaCha12,
            PacketStreamEncryption::ChaCha8
        ],

        force_encryption: true
    };

    // Establish connections.
    for address in &config.node_bootstrap {
        let addresses = address.to_socket_addrs()
            .context("failed to lookup bootstrap node addresses")?;

        for address in addresses {
            // TODO: errors logging.

            progress(Progress::EstablishConnection(address));

            let Ok(stream) = TcpStream::connect(address) else {
                continue;
            };

            let Ok(stream) = PacketStream::init(&secret_key, &options, stream) else {
                continue;
            };

            node.add_stream(stream);
        }
    }

    // Sync the node.
    progress(Progress::SynchronizeBlockchain);

    node.sync().map_err(|err| {
        anyhow::anyhow!(err.to_string())
            .context("failed to synchronize flowerpot blockchain")
    })?;

    // Start the node.
    progress(Progress::StartNode);

    let handler = node.start(NodeOptions {
        messages_filter: Some(garden_protocol::messages_filter),

        ..NodeOptions::default()
    });

    let handler = handler.map_err(|err| {
        anyhow::anyhow!(err.to_string())
            .context("failed to start flowerpot node")
    })?;

    // Start background thread to listen to incoming connections.
    progress(Progress::StartListener(config.node_address));

    if let Ok(listener) = TcpListener::bind(config.node_address) {
        let handler = handler.clone();

        std::thread::spawn(move || {
            loop {
                let Ok((stream, _)) = listener.accept() else {
                    continue;
                };

                let Ok(stream) = PacketStream::init(
                    &secret_key,
                    &options,
                    stream
                ) else {
                    continue;
                };

                handler.add_stream(stream);
            }
        });
    }

    Ok(handler)
}
