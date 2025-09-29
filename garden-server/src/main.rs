// SPDX-License-Identifier: GPL-3.0-or-later
//
// garden-protocol
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

use std::path::PathBuf;
use std::net::{SocketAddr, Ipv6Addr, TcpListener, TcpStream};

use anyhow::Context;
use clap::Parser;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use libflowerpot::crypto::key_exchange::SecretKey;
use libflowerpot::storage::Storage;
use libflowerpot::storage::sqlite_storage::SqliteStorage;
use libflowerpot::protocol::network::{PacketStream, PacketStreamOptions};
use libflowerpot::node::{Node, NodeOptions};

pub mod database;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the flowerpot blockchain storage.
    #[arg(short = 's', long)]
    storage: PathBuf,

    #[arg(short = 'n', long = "node", alias = "connect")]
    nodes: Vec<String>,

    #[arg(
        short = 'l',
        long,
        alias = "local",
        alias = "bind",
        alias = "listen",
        default_value_t = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 13874)
    )]
    local_addr: SocketAddr
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(parent) = cli.storage.parent() {
        std::fs::create_dir_all(parent)
            .context("failed to create blockchain storage's parent folder")?;
    }

    let storage = SqliteStorage::open(cli.storage)
        .context("failed to open blockchain storage")?;

    let root_block = storage.root_block()
        .context("failed to read root block of the blockchain")?;

    let Some(root_block) = root_block else {
        anyhow::bail!("no root block found");
    };

    let listener = TcpListener::bind(cli.local_addr)
        .context("failed to start local tcp listener")?;

    let mut node = Node::new(root_block);

    node.attach_storage(storage);

    let mut rng = ChaCha20Rng::from_entropy();

    let secret_key = SecretKey::random(&mut rng);
    let options = PacketStreamOptions::default();

    for address in cli.nodes {
        let stream = TcpStream::connect(address)
            .context("failed to connect to the node")?;

        let stream = PacketStream::init(&secret_key, &options, stream)
            .context("failed to initiate packet stream")?;

        node.add_stream(stream);
    }

    let handler = node.start(NodeOptions::default())
        .context("failed to start flowerpot blockchain node")?;

    println!("start listener on {}", cli.local_addr);

    loop {
        let (stream, addr) = listener.accept()
            .context("failed to accept incoming tcp connection")?;

        println!("tcp: accept connection from {addr}");

        let stream = PacketStream::init(&secret_key, &options, stream)
            .context("failed to initiate packet stream")?;

        handler.add_stream(stream);
    }
}
