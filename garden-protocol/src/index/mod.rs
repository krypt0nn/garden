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

use flowerpot::crypto::hash::Hash;
use flowerpot::crypto::sign::SignatureError;
use flowerpot::storage::{Storage, StorageError};

use crate::{Events, EventDecodeError};

pub mod post;
pub mod comment;

use post::PostIndex;
use comment::CommentIndex;

#[derive(Debug, thiserror::Error)]
pub enum IndexUpdateError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("failed to decode event: {0}")]
    Event(#[from] EventDecodeError)
}

#[derive(Debug, thiserror::Error)]
pub enum IndexReadError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("failed to decode event: {0}")]
    Event(#[from] EventDecodeError),

    #[error("failed to verify message signature: {0}")]
    Signature(#[from] SignatureError),

    #[error("storage has no block with hash '{}'", .0.to_base64())]
    NoBlockInStorage(Hash),

    #[error("storage has no message with hash '{}'", .0.to_base64())]
    NoMessageInStorage(Hash),

    #[error("storage has no block which provides a message with hash '{}'", .0.to_base64())]
    NoBlockWithMessage(Hash),

    #[error("message with hash '{}' contained invalid event type", .0.to_base64())]
    InvalidEventType(Hash)
}

/// Runtime-built in-memory index of the actual garden state.
///
/// Index is built and updated from a flowerpot blockchain storage. It traverses
/// all the blocks and messages from it and maintains a table of all the posts,
/// comments and other information.
///
/// An actual data is kept within the flowerpot blockchain storage and index
/// only keeps references (hashes) to the stored data.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Index {
    /// Hash of the indexed flowerpot blockchain root block.
    root_block: Hash,

    /// Hash of the last indexed flowerpot blockchain block.
    last_block: Hash,

    /// List of indexed posts.
    posts: Vec<PostIndex>,

    /// List of indexed comments.
    comments: Vec<CommentIndex>
}

impl Index {
    /// Update garden index from provided flowerpot blockchain storage.
    pub fn update(
        &mut self,
        storage: &dyn Storage
    ) -> Result<(), IndexUpdateError> {
        let root_block = storage.root_block()?;

        // Can't update the index from empty storage.
        let Some(root_block) = root_block else {
            return Ok(());
        };

        // Drop the index if root block has changed or the last indexed block
        // was removed from the blockchain (re-indexing is required).
        if self.root_block != root_block
            || !storage.has_block(&self.last_block)?
        {
            #[cfg(feature = "tracing")]
            tracing::debug!("blockchain storage was changed, resetting the garden index");

            self.last_block = Hash::ZERO;

            self.posts.clear();
            self.comments.clear();
        }

        // Store indexed blockchain root block hash.
        self.root_block = root_block;

        // Loop over unindexed blocks.
        while let Some(hash) = storage.next_block(&self.last_block)? {
            let Some(block) = storage.read_block(&hash)? else {
                break;
            };

            // TODO: iterate over ref messages.

            // Iterate over stored messages.
            for message in block.inline_messages() {
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    root_block = root_block.to_base64(),
                    block_hash = block.hash().to_base64(),
                    message_hash = message.hash().to_base64(),
                    "update garden index"
                );

                match Events::from_bytes(message.data())? {
                    Events::Post(_) => {
                        self.posts.push(PostIndex {
                            block_hash: *block.hash(),
                            message_hash: *message.hash()
                        });
                    }

                    Events::Comment(comment) => {
                        self.comments.push(CommentIndex {
                            block_hash: *block.hash(),
                            message_hash: *message.hash(),
                            ref_message_hash: *comment.ref_message_hash()
                        });
                    }

                    _ => ()
                }
            }

            // Update last indexed block hash.
            self.last_block = hash;
        }

        Ok(())
    }

    /// Get iterator over all the indexed posts.
    #[inline(always)]
    pub const fn posts(&self) -> IndexedPostsIter<'_> {
        IndexedPostsIter(self, 0)
    }

    /// Get iterator over all the indexed comments.
    ///
    /// Note that this iter goes over *all* the comments. You will need to
    /// filter it manually.
    #[inline(always)]
    pub const fn comments(&self) -> IndexedCommentsIter<'_> {
        IndexedCommentsIter(self, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexedPostsIter<'index>(&'index Index, usize);

impl<'index> Iterator for IndexedPostsIter<'index> {
    type Item = &'index PostIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let post = self.0.posts.get(self.1)?;

        self.1 += 1;

        Some(post)
    }
}

impl ExactSizeIterator for IndexedPostsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.posts.len() - self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexedCommentsIter<'index>(&'index Index, usize);

impl<'index> Iterator for IndexedCommentsIter<'index> {
    type Item = &'index CommentIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let comment = self.0.comments.get(self.1)?;

        self.1 += 1;

        Some(comment)
    }
}

impl ExactSizeIterator for IndexedCommentsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.comments.len() - self.1
    }
}
