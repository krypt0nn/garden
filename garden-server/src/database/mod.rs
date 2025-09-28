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

use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use spin::Mutex;

use libflowerpot::crypto::hash::Hash;
use libflowerpot::block::BlockContent;
use libflowerpot::storage::Storage;

use garden_protocol::events::{Events, EventsError};

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError<S: Storage> {
    #[error("storage error: {0}")]
    Storage(#[source] S::Error),

    #[error("index error: {0}")]
    Index(#[from] rusqlite::Error),

    #[error("failed to decode event: {0}")]
    Events(#[from] EventsError),

    #[error("failed to verify transaction signature: {0}")]
    VerifySignature(String)
}

/// Garden protocol database.
///
/// The database consist of two key components:
/// 1. flowerpot blockchain storage, and
/// 2. sqlite-powered index.
///
/// Since garden works locally we don't need incredibly fast processing speeds,
/// although we kinda do have them anyway. Thus, to reduce space overhead, we
/// do not store any data in the sqlite database. Instead, we use sqlite
/// database as index of the blockchain data: we store each event as some
/// metadata values like timestamp and author's verifying key, and a link to
/// the flowerpot blockchain transaction where this event is stored. Then we
/// can use flowerpot storage to request this transaction and decode it in
/// runtime.
///
/// This architecture allows us to use abstract blockchain storage and have
/// minimal disk space overhead of just some metadata fields. The runtime
/// overhead is also minimal and absolutely acceptable for local, one-user
/// solution.
///
/// This architecture also natively supports soft history modifications
/// handling. Since we reference blockchain storage and don't store any data -
/// if blockchain changes at any point we won't have desync issues.
pub struct Database<S: Storage> {
    storage: S,
    index: Arc<Mutex<Connection>>
}

impl<S: Storage> Database<S> {
    pub fn new(
        storage: S,
        index_path: impl AsRef<Path>
    ) -> rusqlite::Result<Self> {
        let index = Connection::open(index_path)?;

        index.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS handled_blocks (
                hash BLOB NOT NULL UNIQUE,

                PRIMARY KEY (hash)
            );

            CREATE TABLE IF NOT EXISTS create_community (
                block       BLOB NOT NULL,
                transaction BLOB NOT NULL,
                author      BLOB NOT NULL,
                timestamp   INTEGER NOT NULL,

                UNIQUE (block, transaction),

                FOREIGN KEY (block) REFERENCES handled_blocks (hash)
                ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS create_community_post (
                block       BLOB NOT NULL,
                transaction BLOB NOT NULL,
                author      BLOB NOT NULL,
                timestamp   INTEGER NOT NULL,

                UNIQUE (block, transaction),

                FOREIGN KEY (block) REFERENCES handled_blocks (hash)
                ON DELETE CASCADE
            );
        "#)?;

        Ok(Self {
            storage,
            index: Arc::new(Mutex::new(index))
        })
    }

    /// Check if blockchain block is handled.
    pub fn is_handled(
        &self,
        block: impl AsRef<Hash>
    ) -> rusqlite::Result<bool> {
        let result = self.index.lock()
            .prepare_cached("SELECT 1 FROM handled_blocks WHERE hash = ?1")?
            .query_one([block.as_ref().as_bytes()], |_| Ok(true));

        match result {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(err) => Err(err)
        }
    }

    /// Sync index state with the blockchain storage.
    pub fn sync(&mut self) -> Result<(), DatabaseError<S>> {
        for block_hash in self.storage.history() {
            let block_hash = block_hash.map_err(DatabaseError::Storage)?;

            if self.is_handled(block_hash)? {
                continue;
            }

            let block = self.storage.read_block(&block_hash)
                .map_err(DatabaseError::Storage)?;

            let Some(block) = block else {
                continue;
            };

            let mut lock = self.index.lock();

            let commit = lock.transaction()?;

            commit.prepare_cached("INSERT INTO handled_blocks (hash) VALUES (?1)")?
                .execute([block_hash.as_bytes()])?;

            if let BlockContent::Transactions(transactions) = block.content() {
                let block_timestamp = block.timestamp().unix_timestamp();

                for transaction in transactions {
                    let transaction_hash = transaction.hash();

                    let (_, transaction_author) = transaction.sign()
                        .verify(transaction_hash)
                        .map_err(|err| {
                            DatabaseError::VerifySignature(err.to_string())
                        })?;

                    match Events::from_bytes(transaction.data())? {
                        Events::CreateCommunity(_) => {
                            let mut query = commit.prepare_cached("
                                INSERT INTO create_community (
                                    block,
                                    transaction,
                                    author,
                                    timestamp
                                ) VALUES (?1, ?2, ?3, ?4)
                            ")?;

                            query.execute((
                                block_hash.as_bytes(),
                                transaction_hash.as_bytes(),
                                transaction_author.to_bytes(),
                                block_timestamp
                            ))?;
                        }

                        Events::CreateCommunityPost(_) => {
                            let mut query = commit.prepare_cached("
                                INSERT INTO create_community_post (
                                    block,
                                    transaction,
                                    author,
                                    timestamp
                                ) VALUES (?1, ?2, ?3, ?4)
                            ")?;

                            query.execute((
                                block_hash.as_bytes(),
                                transaction_hash.as_bytes(),
                                transaction_author.to_bytes(),
                                block_timestamp
                            ))?;
                        }
                    }
                }
            }

            commit.commit()?;

            drop(lock);
        }

        Ok(())
    }
}
