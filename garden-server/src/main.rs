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

use std::path::PathBuf;
use std::net::{SocketAddr, Ipv6Addr, TcpListener, TcpStream};
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use axum::Router;
use axum::routing::{get, post};

use libflowerpot::crypto::key_exchange::SecretKey;
use libflowerpot::storage::Storage;
use libflowerpot::storage::sqlite_storage::SqliteStorage;
use libflowerpot::protocol::network::{PacketStream, PacketStreamOptions};
use libflowerpot::node::{Node, NodeOptions};

pub mod database;
pub mod handlers;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional path to a file where to write debug information.
    #[arg(long, alias = "debug")]
    log: Option<PathBuf>,

    /// Path to the flowerpot blockchain storage.
    #[arg(short = 's', long)]
    storage: PathBuf,

    /// Path to the garden-server index database.
    #[arg(short = 'i', long)]
    index: PathBuf,

    /// Connect to another flowerpot node.
    #[arg(short = 'n', long = "node", alias = "connect")]
    nodes: Vec<String>,

    /// Listen address for incoming flowerpot node connections.
    #[arg(
        long,
        short = 'f',
        default_value_t = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 13874)
    )]
    flowerpot_addr: SocketAddr,

    /// Listen address for garden protocol HTTP API server.
    #[arg(
        long,
        short = 'a',
        default_value_t = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 8080)
    )]
    api_addr: SocketAddr
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(log) = cli.log {
        if let Some(parent) = log.parent() {
            std::fs::create_dir_all(parent)
                .context("failed to create log file's parent folder")?;
        }

        let file = std::fs::File::create(log)
            .context("failed to create log file")?;

        tracing_subscriber::fmt()
            .with_writer(file)
            .with_ansi(false)
            .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
            .init();
    }

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

    println!("syncing garden-server index...");

    let database = database::Database::new(storage.clone(), cli.index)
        .context("failed to open flowerpot storage index")?;

    database.sync().context("failed to sync flowerpot storage index")?;

    println!("open flowerpot node listener...");

    let flowerpot_listener = TcpListener::bind(cli.flowerpot_addr)
        .context("failed to start flowerpot tcp listener")?;

    let api_listener = tokio::net::TcpListener::bind(cli.api_addr).await
        .context("failed to start http api tcp listener")?;

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

    println!("start garden protocol server on http://{}", cli.api_addr);

    let handler = node.start(NodeOptions::default())
        .context("failed to start flowerpot blockchain node")?;

    {
        let handler = Arc::new(handler.clone());

        tokio::spawn(async move {
            let app = Router::new()
                .route("/", get("hi"))
                .route("/api/v1/post", post(handlers::api_send_post))
                .route("/api/v1/post/{address}", get(handlers::api_get_post));

            let serve = axum::serve(
                api_listener,
                app.with_state(handlers::App {
                    database,
                    handler
                })
            );

            if let Err(err) = serve.await {
                panic!("{err}");
            }
        });
    }

    println!("start flowerpot listener on {}", cli.flowerpot_addr);

    loop {
        let (stream, addr) = flowerpot_listener.accept()
            .context("failed to accept incoming tcp connection")?;

        println!("tcp: accept connection from {addr}");

        let stream = PacketStream::init(&secret_key, &options, stream)
            .context("failed to initiate packet stream")?;

        handler.add_stream(stream);
    }
}
