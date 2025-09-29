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

use std::borrow::Cow;

use time::UtcDateTime;

use libflowerpot::crypto::hash::Hash;
use libflowerpot::crypto::sign::VerifyingKey;
use libflowerpot::storage::Storage;

use garden_protocol::types::PrintableText;
use garden_protocol::events::{Events, CreateCommunityPostEvent};

use super::{Database, QueryDatabaseError};

#[derive(Debug, Clone)]
pub struct CommunityPost<S: Storage> {
    pub(super) database: Database<S>,
    pub(super) community_block: Hash,
    pub(super) community_transaction: Hash,
    pub(super) post_block: Hash,
    pub(super) post_transaction: Hash,
    pub(super) post_author: VerifyingKey,
    pub(super) post_timestamp: UtcDateTime,
    pub(super) event: Option<CreateCommunityPostEvent>
}

impl<S: Storage> CommunityPost<S> {
    /// Hash of block where this community was created.
    #[inline(always)]
    pub const fn community_block(&self) -> &Hash {
        &self.community_block
    }

    /// Hash of transaction where this community was created.
    #[inline(always)]
    pub const fn community_transaction(&self) -> &Hash {
        &self.community_transaction
    }

    /// Hash of block where the community post was created.
    #[inline(always)]
    pub const fn post_block(&self) -> &Hash {
        &self.post_block
    }

    /// Hash of transaction where the community post was created.
    #[inline(always)]
    pub const fn post_transaction(&self) -> &Hash {
        &self.post_transaction
    }

    /// Verifying key of the user who created community post.
    #[inline(always)]
    pub const fn post_author(&self) -> &VerifyingKey {
        &self.post_author
    }

    /// Timestamp of the block in which this community post was created.
    #[inline(always)]
    pub const fn post_timestamp(&self) -> &UtcDateTime {
        &self.post_timestamp
    }

    /// Prefetch event from the blockchain storage.
    pub fn prefetch_event(&mut self) -> Result<&mut Self, QueryDatabaseError<S>> {
        if self.event.is_some() {
            return Ok(self);
        }

        let transaction = self.database.query_transaction(
            self.post_block,
            self.post_transaction
        )?;

        let Ok(Events::CreateCommunityPost(event)) = Events::from_bytes(transaction.data()) else {
            return Err(QueryDatabaseError::InvalidEvent {
                block: self.post_block,
                transaction: self.post_transaction
            });
        };

        self.event = Some(event);

        Ok(self)
    }

    fn query_event(&self) -> Result<Cow<'_, CreateCommunityPostEvent>, QueryDatabaseError<S>> {
        match &self.event {
            Some(event) => Ok(Cow::Borrowed(event)),
            None => {
                let transaction = self.database.query_transaction(
                    self.post_block,
                    self.post_transaction
                )?;

                let Ok(Events::CreateCommunityPost(event)) = Events::from_bytes(transaction.data()) else {
                    return Err(QueryDatabaseError::InvalidEvent {
                        block: self.post_block,
                        transaction: self.post_transaction
                    });
                };

                Ok(Cow::Owned(event))
            }
        }
    }

    /// Try to query title of the community post from the blockchain storage.
    pub fn query_title(&self) -> Result<PrintableText, QueryDatabaseError<S>> {
        Ok(self.query_event()?.title().clone())
    }

    /// Try to query body of the community post from the blockchain storage.
    pub fn query_body(&self) -> Result<PrintableText, QueryDatabaseError<S>> {
        Ok(self.query_event()?.body().clone())
    }
}

#[derive(Debug, Clone)]
pub struct CommunityPostsIter<S: Storage> {
    pub(super) database: Database<S>,
    pub(super) community_block: Hash,
    pub(super) community_transaction: Hash,
    pub(super) last_rowid: u64
}

impl<S: Storage> Iterator for CommunityPostsIter<S> {
    type Item = CommunityPost<S>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: respect deleted community posts once they will be supported.

        loop {
            let (
                rowid,
                post_block,
                post_transaction,
                post_author,
                post_timestamp
            ) = self.database.index.lock()
                .prepare_cached("
                    SELECT
                        rowid,
                        post_block,
                        post_transaction,
                        post_author,
                        post_timestamp
                    FROM create_community_post
                    WHERE
                        community_block = ?1 AND
                        community_transaction = ?1 AND
                        rowid > ?1
                    ORDER BY rowid ASC
                    LIMIT 1
                ").ok()?
                .query_row([self.last_rowid], |row| {
                    Ok((
                        row.get::<_, u64>("rowid")?,
                        row.get::<_, [u8; Hash::SIZE]>("post_block")?,
                        row.get::<_, [u8; Hash::SIZE]>("post_transaction")?,
                        row.get::<_, [u8; VerifyingKey::SIZE]>("post_author")?,
                        row.get::<_, i64>("post_timestamp")?
                    ))
                }).ok()?;

            self.last_rowid = rowid;

            let post_block = Hash::from(post_block);

            if self.database.storage.has_block(&post_block).ok()? {
                return Some(CommunityPost {
                    database: self.database.clone(),
                    community_block: self.community_block,
                    community_transaction: self.community_transaction,
                    post_block,
                    post_transaction: Hash::from(post_transaction),
                    post_author: VerifyingKey::from_bytes(&post_author)?,
                    post_timestamp: UtcDateTime::from_unix_timestamp(post_timestamp).ok()?,
                    event: None
                })
            }
        }
    }
}
