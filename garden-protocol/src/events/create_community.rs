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

use crate::types::Name;

use super::Event;

#[derive(Debug, thiserror::Error)]
pub enum CreateCommunityEventError {
    #[error("invalid unicode sequence: {0}")]
    InvalidUnicode(#[from] std::string::FromUtf8Error),

    #[error("name uses invalid format")]
    InvalidNameFormat
}

/// Create new community.
///
/// Community is a place where multiple users can create posts, comment them
/// and to other kind of interactions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateCommunityEvent {
    /// Unique name of the community.
    name: Name
}

impl CreateCommunityEvent {
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            name: name.into()
        }
    }
}

impl Event for CreateCommunityEvent {
    type Error = CreateCommunityEventError;

    fn to_bytes(&self) -> Box<[u8]> {
        self.name.as_bytes().to_vec().into_boxed_slice()
    }

    fn from_bytes(event: &[u8]) -> Result<Self, Self::Error> where Self: Sized {
        let name = String::from_utf8(event.to_vec())?;

        let Some(name) = Name::new(name) else {
            return Err(CreateCommunityEventError::InvalidNameFormat);
        };

        Ok(Self {
            name
        })
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.name.len())
    }
}
