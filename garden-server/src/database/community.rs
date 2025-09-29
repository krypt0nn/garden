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

use garden_protocol::types::Name;
use garden_protocol::events::{Events, CreateCommunityEvent};

use super::{Database, QueryDatabaseError};
use super::community_post::CommunityPostsIter;

#[derive(Debug, Clone)]
pub struct Community<S: Storage> {
    pub(super) database: Database<S>,
    pub(super) block: Hash,
    pub(super) transaction: Hash,
    pub(super) author: VerifyingKey,
    pub(super) timestamp: UtcDateTime,
    pub(super) event: Option<CreateCommunityEvent>
}

impl<S: Storage> Community<S> {
    /// Hash of block where this community was created.
    #[inline(always)]
    pub const fn block(&self) -> &Hash {
        &self.block
    }

    /// Hash of transaction where this community was created.
    #[inline(always)]
    pub const fn transaction(&self) -> &Hash {
        &self.transaction
    }

    /// Verifying key of the user who created community.
    #[inline(always)]
    pub const fn author(&self) -> &VerifyingKey {
        &self.author
    }

    /// Timestamp of the block in which this community was created.
    #[inline(always)]
    pub const fn timestamp(&self) -> &UtcDateTime {
        &self.timestamp
    }

    /// Prefetch event from the blockchain storage.
    pub fn prefetch_event(&mut self) -> Result<&mut Self, QueryDatabaseError<S>> {
        if self.event.is_some() {
            return Ok(self);
        }

        let transaction = self.database.query_transaction(
            self.block,
            self.transaction
        )?;

        let Ok(Events::CreateCommunity(event)) = Events::from_bytes(transaction.data()) else {
            return Err(QueryDatabaseError::InvalidEvent {
                block: self.block,
                transaction: self.transaction
            });
        };

        self.event = Some(event);

        Ok(self)
    }

    fn query_event(&self) -> Result<Cow<'_, CreateCommunityEvent>, QueryDatabaseError<S>> {
        match &self.event {
            Some(event) => Ok(Cow::Borrowed(event)),
            None => {
                let transaction = self.database.query_transaction(
                    self.block,
                    self.transaction
                )?;

                let Ok(Events::CreateCommunity(event)) = Events::from_bytes(transaction.data()) else {
                    return Err(QueryDatabaseError::InvalidEvent {
                        block: self.block,
                        transaction: self.transaction
                    });
                };

                Ok(Cow::Owned(event))
            }
        }
    }

    /// Try to query name of the community from the blockchain storage.
    pub fn query_name(&self) -> Result<Name, QueryDatabaseError<S>> {
        Ok(self.query_event()?.name().clone())
    }

    /// Get iterator over all the stored community posts.
    pub fn posts(&self) -> CommunityPostsIter<S> {
        CommunityPostsIter {
            database: self.database.clone(),
            community_block: self.block,
            community_transaction: self.transaction,
            last_rowid: 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommunityIter<S: Storage> {
    pub(crate) database: Database<S>,
    pub(crate) last_rowid: u64
}

impl<S: Storage> Iterator for CommunityIter<S> {
    type Item = Community<S>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: respect deleted communities once they will be supported.

        loop {
            let (rowid, block, transaction, author, timestamp) = self.database.index.lock()
                .prepare_cached("
                    SELECT rowid, block, transaction, author, timestamp
                    FROM create_community
                    WHERE rowid > ?1
                    ORDER BY rowid ASC
                    LIMIT 1
                ").ok()?
                .query_row([self.last_rowid], |row| {
                    Ok((
                        row.get::<_, u64>("rowid")?,
                        row.get::<_, [u8; Hash::SIZE]>("block")?,
                        row.get::<_, [u8; Hash::SIZE]>("transaction")?,
                        row.get::<_, [u8; VerifyingKey::SIZE]>("author")?,
                        row.get::<_, i64>("timestamp")?
                    ))
                }).ok()?;

            self.last_rowid = rowid;

            let block = Hash::from(block);

            if self.database.storage.has_block(&block).ok()? {
                return Some(Community {
                    database: self.database.clone(),
                    block,
                    transaction: Hash::from(transaction),
                    author: VerifyingKey::from_bytes(&author)?,
                    timestamp: UtcDateTime::from_unix_timestamp(timestamp).ok()?,
                    event: None
                })
            }
        }
    }
}
