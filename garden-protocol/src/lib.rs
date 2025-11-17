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
use flowerpot::message::Message;

mod post;
mod comment;
mod reaction;

pub mod index;

pub use post::{Content, Tag, PostEvent, PostEventError};
pub use comment::{CommentEvent, CommentEventError};
pub use reaction::{Reaction, ReactionEvent, ReactionEventError};

pub trait Event {
    type Error: std::error::Error;

    /// Convert event to the binary representation.
    fn to_bytes(&self) -> Box<[u8]>;

    /// Try to convert bytes slice into the event.
    fn from_bytes(event: &[u8]) -> Result<Self, Self::Error> where Self: Sized;

    /// Get size hint of the current event's binary representation.
    ///
    /// Can be used to efficiently allocate memory buffers.
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventDecodeError {
    #[error("provided event bytes slice is too short")]
    SliceTooShort,

    #[error("unknown event: {0}")]
    UnknownEvent(u16),

    #[error(transparent)]
    Post(#[from] PostEventError),

    #[error(transparent)]
    Comment(#[from] CommentEventError),

    #[error(transparent)]
    Reaction(#[from] ReactionEventError)
}

/// Event is the main component of the garden protocol. It encodes some action
/// performed in the network, stored as flowerpot blockchain transaction.
#[derive(Debug, Clone)]
pub enum Events {
    Post(PostEvent),
    Comment(CommentEvent),
    Reaction(ReactionEvent)
}

impl Events {
    pub const V1_POST: u16     = 0;
    pub const V1_COMMENT: u16  = 1;
    pub const V1_REACTION: u16 = 2;

    pub fn to_bytes(&self) -> Box<[u8]> {
        fn alloc(event: &impl Event) -> Vec<u8> {
            match event.size_hint() {
                Some(size) => Vec::with_capacity(size + 2),
                None => Vec::new()
            }
        }

        match self {
            Self::Post(event) => {
                let mut buf = alloc(event);

                buf.extend(Self::V1_POST.to_le_bytes());
                buf.extend(event.to_bytes());

                buf.into_boxed_slice()
            }

            Self::Comment(event) => {
                let mut buf = alloc(event);

                buf.extend(Self::V1_COMMENT.to_le_bytes());
                buf.extend(event.to_bytes());

                buf.into_boxed_slice()
            }

            Self::Reaction(event) => {
                let mut buf = alloc(event);

                buf.extend(Self::V1_REACTION.to_le_bytes());
                buf.extend(event.to_bytes());

                buf.into_boxed_slice()
            }
        }
    }

    pub fn from_bytes(event: impl AsRef<[u8]>) -> Result<Self, EventDecodeError> {
        let event = event.as_ref();

        if event.len() < 2 {
            return Err(EventDecodeError::SliceTooShort);
        }

        let id = u16::from_le_bytes([event[0], event[1]]);

        match id {
            Self::V1_POST => {
                Ok(Self::Post(
                    PostEvent::from_bytes(&event[2..])?
                ))
            }

            Self::V1_COMMENT => {
                Ok(Self::Comment(
                    CommentEvent::from_bytes(&event[2..])?
                ))
            }

            Self::V1_REACTION => {
                Ok(Self::Reaction(
                    ReactionEvent::from_bytes(&event[2..])?
                ))
            }

            _ => Err(EventDecodeError::UnknownEvent(id))
        }
    }
}

impl From<PostEvent> for Events {
    #[inline(always)]
    fn from(value: PostEvent) -> Self {
        Self::Post(value)
    }
}

impl From<CommentEvent> for Events {
    #[inline(always)]
    fn from(value: CommentEvent) -> Self {
        Self::Comment(value)
    }
}

impl From<ReactionEvent> for Events {
    #[inline(always)]
    fn from(value: ReactionEvent) -> Self {
        Self::Reaction(value)
    }
}

/// Filter function for garden protocol related flowerpot messages. This
/// function will try to decode the message into a garden protocol event and
/// return `true` on success.
#[inline]
pub fn messages_filter(
    _root_block: &Hash,
    message: &Message,
    _author: &VerifyingKey
) -> bool {
    Events::from_bytes(message.data()).is_ok()
}
