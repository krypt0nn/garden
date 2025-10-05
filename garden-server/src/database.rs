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

use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use spin::{Mutex, MutexGuard};
use time::UtcDateTime;

use libflowerpot::crypto::hash::Hash;
use libflowerpot::crypto::sign::VerifyingKey;
use libflowerpot::block::BlockContent;
use libflowerpot::storage::Storage;

use garden_protocol::{Events, EventsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    pub name: String,
    pub timestamp: UtcDateTime,
    pub author: VerifyingKey
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub content: String,
    pub timestamp: UtcDateTime,
    pub author: VerifyingKey,
    pub reactions: Box<[Reaction]>,
    pub comments: Box<[Hash]>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub content: String,
    pub tags: Box<[String]>,
    pub timestamp: UtcDateTime,
    pub author: VerifyingKey,
    pub reactions: Box<[Reaction]>,
    pub comments: Box<[Hash]>
}

fn query_reactions(
    lock: &MutexGuard<'_, Connection>,
    address: &Hash
) -> anyhow::Result<Option<Box<[Reaction]>>> {
    let mut query = lock.prepare_cached("
        SELECT
            name,
            timestamp,
            author
        FROM v1_reactions
        WHERE ref = ?1
    ")?;

    let result = query.query_map([address.as_bytes()], |row| {
        Ok((
            row.get::<_, String>("name")?,
            row.get::<_, i64>("timestamp")?,
            row.get::<_, [u8; VerifyingKey::SIZE]>("author")?
        ))
    });

    let result = match result {
        Ok(result) => result,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(err) => anyhow::bail!(err)
    };

    let mut reactions = Vec::new();

    for reaction in result {
        let (name, timestamp, author) = reaction?;

        reactions.push(Reaction {
            name,
            timestamp: UtcDateTime::from_unix_timestamp(timestamp)?,
            author: VerifyingKey::from_bytes(&author)
                .ok_or_else(|| anyhow::anyhow!("invalid verifying key format"))?
        });
    }

    Ok(Some(reactions.into_boxed_slice()))
}

fn query_comments_list(
    lock: &MutexGuard<'_, Connection>,
    address: &Hash
) -> anyhow::Result<Option<Box<[Hash]>>> {
    let mut query = lock.prepare_cached("
        SELECT transaction FROM v1_comments WHERE ref = ?1
    ")?;

    let result = query.query_map([address.as_bytes()], |row| {
        row.get::<_, [u8; Hash::SIZE]>("transaction")
    });

    let result = match result {
        Ok(result) => result,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(err) => anyhow::bail!(err)
    };

    let mut comments = Vec::new();

    for comment in result {
        comments.push(Hash::from(comment?));
    }

    Ok(Some(comments.into_boxed_slice()))
}

fn query_post(
    lock: &MutexGuard<'_, Connection>,
    address: &Hash
) -> anyhow::Result<Option<Post>> {
    let mut query = lock.prepare_cached("
        SELECT
            content,
            timestamp,
            author
        FROM v1_posts
        WHERE transaction = ?1
    ")?;

    let result = query.query_row([address.as_bytes()], |row| {
        Ok((
            row.get::<_, String>("content")?,
            row.get::<_, i64>("timestamp")?,
            row.get::<_, [u8; VerifyingKey::SIZE]>("author")?
        ))
    });

    let (content, timestamp, author) = match result {
        Ok(result) => result,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(err) => anyhow::bail!(err)
    };

    let mut query = lock.prepare_cached("
        SELECT tag FROM v1_post_tags WHERE post = ?1
    ")?;

    let mut tags = Vec::new();

    let result = query.query_map([address.as_bytes()], |row| {
        row.get::<_, String>("tag")
    })?;

    for tag in result {
        tags.push(tag?);
    }

    Ok(Some(Post {
        content,
        tags: tags.into_boxed_slice(),
        timestamp: UtcDateTime::from_unix_timestamp(timestamp)?,
        author: VerifyingKey::from_bytes(&author)
            .ok_or_else(|| anyhow::anyhow!("invalid verifying key format"))?,
        reactions: query_reactions(lock, address)?.unwrap_or_default(),
        comments: query_comments_list(lock, address)?.unwrap_or_default()
    }))
}

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

#[derive(Debug, Clone)]
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
            CREATE TABLE IF NOT EXISTS v1_handled_blocks (
                hash BLOB NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS v1_posts (
                transaction BLOB    NOT NULL UNIQUE,
                content     TEXT    NOT NULL,
                timestamp   INTEGER NOT NULL,
                author      BLOB    NOT NULL,

                PRIMARY KEY (transaction)
            );

            CREATE TABLE IF NOT EXISTS v1_post_tags (
                post BLOB NOT NULL,
                tag  TEXT NOT NULL,

                UNIQUE (post, tag)
            );

            CREATE TABLE IF NOT EXISTS v1_comments (
                ref         BLOB    NOT NULL,
                transaction BLOB    NOT NULL UNIQUE,
                content     TEXT    NOT NULL,
                timestamp   INTEGER NOT NULL,
                author      BLOB    NOT NULL,

                PRIMARY KEY (transaction)
            );

            CREATE TABLE IF NOT EXISTS v1_reactions (
                ref         BLOB    NOT NULL,
                transaction BLOB    NOT NULL UNIQUE,
                name        TEXT    NOT NULL,
                timestamp   INTEGER NOT NULL,
                author      BLOB    NOT NULL,

                PRIMARY KEY (transaction)
            );
        "#)?;

        Ok(Self {
            storage,
            index: Arc::new(Mutex::new(index))
        })
    }

    /// Check if blockchain block is handled in the index.
    pub fn is_handled(
        &self,
        block: &Hash
    ) -> rusqlite::Result<bool> {
        let result = self.index.lock()
            .prepare_cached("SELECT 1 FROM v1_handled_blocks WHERE hash = ?1")?
            .query_one([block.as_ref().as_bytes()], |_| Ok(true));

        match result {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(err) => Err(err)
        }
    }

    /// Sync index state with the blockchain storage.
    pub fn sync(&self) -> Result<(), DatabaseError<S>> {
        for block_hash in self.storage.history() {
            let block_hash = block_hash.map_err(DatabaseError::Storage)?;

            if self.is_handled(&block_hash)? {
                continue;
            }

            let block = self.storage.read_block(&block_hash)
                .map_err(DatabaseError::Storage)?;

            let Some(block) = block else {
                continue;
            };

            let mut lock = self.index.lock();

            let commit = lock.transaction()?;

            commit.prepare_cached("INSERT INTO v1_handled_blocks (hash) VALUES (?1)")?
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
                        Events::Post(post) => {
                            let mut query = commit.prepare_cached("
                                INSERT INTO v1_posts (
                                    transaction,
                                    content,
                                    timestamp,
                                    author
                                ) VALUES (?1, ?2, ?3, ?4)
                            ")?;

                            query.execute((
                                transaction_hash.as_bytes(),
                                post.content().as_bytes(),
                                block_timestamp,
                                transaction_author.to_bytes()
                            ))?;

                            for tag in post.tags() {
                                let mut query = commit.prepare_cached("
                                    INSERT INTO v1_post_tags (
                                        post,
                                        tag
                                    ) VALUES (?1, ?2)
                                ")?;

                                query.execute((
                                    transaction_hash.as_bytes(),
                                    tag.as_bytes()
                                ))?;
                            }
                        }

                        Events::Comment(comment) => {
                            let mut query = commit.prepare_cached("
                                INSERT INTO v1_comments (
                                    ref,
                                    transaction,
                                    content,
                                    timestamp,
                                    author
                                ) VALUES (?1, ?2, ?3, ?4, ?5)
                            ")?;

                            query.execute((
                                comment.ref_address().as_bytes(),
                                transaction_hash.as_bytes(),
                                comment.content().as_bytes(),
                                block_timestamp,
                                transaction_author.to_bytes()
                            ))?;
                        }

                        Events::Reaction(reaction) => {
                            let mut query = commit.prepare_cached("
                                INSERT INTO v1_reactions (
                                    ref,
                                    transaction,
                                    name,
                                    timestamp,
                                    author
                                ) VALUES (?1, ?2, ?3, ?4, ?5)
                            ")?;

                            query.execute((
                                reaction.ref_address().as_bytes(),
                                transaction_hash.as_bytes(),
                                reaction.reaction().to_name(),
                                block_timestamp,
                                transaction_author.to_bytes()
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

    // TODO: better error handling

    /// Try to query list of reactions for a post or comment with provided
    /// flowerpot blockchain transaction hash.
    ///
    /// Return `Ok(None)` if there's no such transaction.
    pub fn query_reactions(
        &self,
        address: &Hash
    ) -> anyhow::Result<Option<Box<[Reaction]>>> {
        query_reactions(&self.index.lock(), address)
    }

    /// Try to query list of flowerpot transactions' hashes which are comments
    /// for the provided post/comment transaction hash.
    ///
    /// Return `Ok(None)` if there's no such transaction.
    pub fn query_comments_list(
        &self,
        address: &Hash
    ) -> anyhow::Result<Option<Box<[Hash]>>> {
        query_comments_list(&self.index.lock(), address)
    }

    /// Try to query post with provided flowerpot blockchain transaction hash.
    ///
    /// Return `Ok(None)` if there's no such transaction.
    pub fn query_post(&self, address: &Hash) -> anyhow::Result<Option<Post>> {
        query_post(&self.index.lock(), address)
    }

    /// Get iterator of all the indexed posts.
    pub fn posts(&self) -> PostsIter {
        PostsIter {
            index: self.index.clone(),
            curr_id: i64::MAX
        }
    }
}

// TODO: search filters

/// Iterator over the posts stored in the blockchain index. The posts are
/// returned in descending chronology order, so new posts are returned first.
pub struct PostsIter {
    index: Arc<Mutex<Connection>>,
    curr_id: i64
}

impl Iterator for PostsIter {
    type Item = anyhow::Result<Post>;

    fn next(&mut self) -> Option<Self::Item> {
        let lock = self.index.lock();

        let mut query = lock.prepare_cached("
            SELECT
                rowid,
                transaction
            FROM v1_posts
            WHERE rowid < ?1
            ORDER BY rowid DESC
        ").ok()?;

        let (
            rowid,
            transaction
        ) = query.query_row([self.curr_id], |row| {
            Ok((
                row.get::<_, i64>("rowid")?,
                row.get::<_, [u8; Hash::SIZE]>("transaction")?
            ))
        }).ok()?;

        self.curr_id = rowid;

        match query_post(&lock, &Hash::from(transaction)) {
            Ok(Some(post)) => Some(Ok(post)),
            Ok(None) => None,
            Err(err) => Some(Err(err))
        }
    }
}
