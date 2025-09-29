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

use std::sync::Arc;

use rusqlite::Connection;
use spin::Mutex;
use time::UtcDateTime;

use libflowerpot::crypto::hash::Hash;
use libflowerpot::crypto::sign::VerifyingKey;
use libflowerpot::storage::Storage;

#[derive(Debug, Clone)]
pub struct Community<S: Storage> {
    storage: S,
    index: Arc<Mutex<Connection>>,
    block: Hash,
    transaction: Hash,
    author: VerifyingKey,
    timestamp: UtcDateTime
}

impl<S: Storage> Community<S> {
    #[inline(always)]
    pub const fn storage(&self) -> &S {
        &self.storage
    }

    #[inline(always)]
    pub const fn index(&self) -> &Arc<Mutex<Connection>> {
        &self.index
    }

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
}

#[derive(Debug, Clone)]
pub struct CommunityIter<S: Storage> {
    pub(super) storage: S,
    pub(super) index: Arc<Mutex<Connection>>,
    pub(super) last_rowid: u64
}

impl<S: Storage> Iterator for CommunityIter<S> {
    type Item = Community<S>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: respect deleted communities once they will be supported.

        loop {
            let (rowid, block, transaction, author, timestamp) = self.index.lock()
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

            if self.storage.has_block(&block).ok()? {
                return Some(Community {
                    storage: self.storage.clone(),
                    index: self.index.clone(),
                    block,
                    transaction: Hash::from(transaction),
                    author: VerifyingKey::from_bytes(&author)?,
                    timestamp: UtcDateTime::from_unix_timestamp(timestamp).ok()?
                })
            }
        }
    }
}
