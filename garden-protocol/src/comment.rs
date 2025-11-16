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

use super::post::Content;
use super::Event;

#[derive(Debug, thiserror::Error)]
pub enum CommentEventError {
    #[error("invalid unicode sequence: {0}")]
    InvalidUnicode(#[from] std::string::FromUtf8Error),

    #[error("provided comment event bytes slice is too short")]
    SliceTooShort,

    #[error("invalid content")]
    InvalidContent
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommentEvent {
    ref_message_hash: Hash,
    content: Content
}

impl CommentEvent {
    /// Create new comment event. Reference address is a flowerpot message hash
    /// of another comment or a post.
    pub fn new(
        ref_message_hash: impl Into<Hash>,
        content: Content
    ) -> Self {
        Self {
            ref_message_hash: ref_message_hash.into(),
            content
        }
    }

    #[inline(always)]
    pub const fn ref_message_hash(&self) -> &Hash {
        &self.ref_message_hash
    }

    #[inline(always)]
    pub const fn content(&self) -> &Content {
        &self.content
    }
}

impl Event for CommentEvent {
    type Error = CommentEventError;

    fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Hash::SIZE + self.content.len());

        buf.extend(self.ref_message_hash.as_bytes());
        buf.extend(self.content.as_bytes());

        buf.into_boxed_slice()
    }

    fn from_bytes(event: &[u8]) -> Result<Self, Self::Error> where Self: Sized {
        if event.len() < Hash::SIZE {
            return Err(CommentEventError::SliceTooShort);
        }

        let mut ref_message_hash = [0; Hash::SIZE];

        ref_message_hash.copy_from_slice(&event[..Hash::SIZE]);

        let content = String::from_utf8(event[Hash::SIZE..].to_vec())?;

        let Some(content) = Content::new(content) else {
            return Err(CommentEventError::InvalidContent);
        };

        Ok(Self {
            ref_message_hash: Hash::from(ref_message_hash),
            content
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(Hash::SIZE + self.content.len())
    }
}
