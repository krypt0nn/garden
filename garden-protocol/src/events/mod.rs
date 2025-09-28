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

mod create_community;
mod create_community_post;

pub use create_community::{CreateCommunityEvent, CreateCommunityEventError};
pub use create_community_post::{
    CreateCommunityPostEvent, CreateCommunityPostEventError
};

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

/// Event is the main component of the garden protocol. It encodes some action
/// performed in the network, stored as flowerpot blockchain transaction.
#[derive(Debug, Clone)]
pub enum Events {
    CreateCommunity(CreateCommunityEvent),

    /// Create new community post.
    CreateCommunityPost(CreateCommunityPostEvent)
}

impl Events {
    pub const V1_CREATE_COMMUNITY: u16      = 0;
    pub const V1_CREATE_COMMUNITY_POST: u16 = 1;

    pub fn to_bytes(&self) -> Box<[u8]> {
        fn alloc(event: &impl Event) -> Vec<u8> {
            match event.size_hint() {
                Some(size) => Vec::with_capacity(size + 2),
                None => Vec::new()
            }
        }

        match self {
            Self::CreateCommunity(event) => {
                let mut buf = alloc(event);

                buf.extend(Self::V1_CREATE_COMMUNITY.to_le_bytes());
                buf.extend(event.to_bytes());

                buf.into_boxed_slice()
            }

            Self::CreateCommunityPost(event) => {
                let mut buf = alloc(event);

                buf.extend(Self::V1_CREATE_COMMUNITY_POST.to_le_bytes());
                buf.extend(event.to_bytes());

                buf.into_boxed_slice()
            }
        }
    }
}
