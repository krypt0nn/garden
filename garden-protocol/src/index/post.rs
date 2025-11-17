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
use flowerpot::crypto::sign::VerifyingKey;
use flowerpot::storage::Storage;

use time::UtcDateTime;

use crate::{Events, Content, Tag};

use super::{Index, IndexReadError};
use super::comment::CommentIndex;

/// Information about a garden post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostInfo {
    /// Hash of the block of the flowerpot blockchain where the post info is
    /// stored.
    pub block_hash: Hash,

    /// Hash of the message of the flowerpot blockchain where the post info is
    /// stored (practically the address of the post).
    pub message_hash: Hash,

    /// Flowerpot verifying key of the post author.
    pub author: VerifyingKey,

    /// Timestamp when, approximately, the post was created. Derived from the
    /// block where the post is stored on the flowerpot blockchain.
    pub timestamp: UtcDateTime,

    /// Content of the post.
    pub content: Content,

    /// List of tags of the post.
    pub tags: Box<[Tag]>
}

/// Index of a garden post stored in flowerpot blockchain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostIndex {
    /// Block hash where the current post is stored.
    pub(super) block_hash: Hash,

    /// Message hash where the current post is stored.
    pub(super) message_hash: Hash
}

impl PostIndex {
    /// Try to read indexed post from provided flowerpot blockchain storage.
    pub fn read(
        &self,
        storage: &dyn Storage
    ) -> Result<PostInfo, IndexReadError> {
        // FIXME: we don't need to read the whole block, only some of its
        //        metadata, but there's currently no logic for it.
        let Some(block) = storage.read_block(&self.block_hash)? else {
            return Err(IndexReadError::NoMessageInStorage(self.block_hash));
        };

        let Some(message) = storage.read_message(&self.message_hash)? else {
            return Err(IndexReadError::NoMessageInStorage(self.message_hash));
        };

        let Events::Post(post) = Events::from_bytes(message.data())? else {
            return Err(IndexReadError::InvalidEventType(self.message_hash));
        };

        let (_, author) = message.verify()?;

        Ok(PostInfo {
            block_hash: self.block_hash,
            message_hash: self.message_hash,
            author,
            timestamp: *block.timestamp(),
            content: post.content().clone(),
            tags: post.tags().to_vec().into_boxed_slice()
        })
    }

    /// Get iterator over all the comments referencing the current post.
    pub fn comments<'index>(
        &self,
        index: &'index Index
    ) -> impl Iterator<Item = &'index CommentIndex> {
        index.comments().filter(|comment| {
            comment.ref_message_hash == self.message_hash
        })
    }
}
