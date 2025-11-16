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

use std::str::FromStr;

use flowerpot::crypto::hash::Hash;

use super::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reaction {
    /// `thumb_up` = 👍
    ThumbUp,

    /// `thumb_down` = 👎
    ThumbDown
}

impl Reaction {
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::ThumbUp   => "thumb_up",
            Self::ThumbDown => "thumb_down"
        }
    }

    pub const fn to_emoji(&self) -> char {
        match self {
            Self::ThumbUp   => '👍',
            Self::ThumbDown => '👎'
        }
    }
}

impl std::str::FromStr for Reaction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "thumb_up"   => Ok(Self::ThumbUp),
            "thumb_down" => Ok(Self::ThumbDown),

            _ => Err(())
        }
    }
}

impl std::fmt::Display for Reaction {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_name())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReactionEventError {
    #[error("invalid unicode sequence: {0}")]
    InvalidUnicode(#[from] std::string::FromUtf8Error),

    #[error("provided comment event bytes slice is too short")]
    SliceTooShort,

    #[error("invalid reaction name")]
    InvalidReactionName
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactionEvent {
    ref_address: Hash,
    reaction: Reaction
}

impl ReactionEvent {
    /// Create new reaction event. Reference address is a transaction hash of a
    /// comment or a post.
    pub fn new(
        ref_address: impl Into<Hash>,
        reaction: Reaction
    ) -> Self {
        Self {
            ref_address: ref_address.into(),
            reaction
        }
    }

    #[inline(always)]
    pub const fn ref_address(&self) -> &Hash {
        &self.ref_address
    }

    #[inline(always)]
    pub const fn reaction(&self) -> &Reaction {
        &self.reaction
    }
}

impl Event for ReactionEvent {
    type Error = ReactionEventError;

    fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Hash::SIZE + self.reaction.to_name().len());

        buf.extend(self.ref_address.as_bytes());
        buf.extend(self.reaction.to_name().as_bytes());

        buf.into_boxed_slice()
    }

    fn from_bytes(event: &[u8]) -> Result<Self, Self::Error> where Self: Sized {
        if event.len() < Hash::SIZE {
            return Err(ReactionEventError::SliceTooShort);
        }

        let mut ref_address = [0; Hash::SIZE];

        ref_address.copy_from_slice(&event[..Hash::SIZE]);

        let reaction_name = String::from_utf8(event[Hash::SIZE..].to_vec())?;

        let Ok(reaction) = Reaction::from_str(&reaction_name) else {
            return Err(ReactionEventError::InvalidReactionName);
        };

        Ok(Self {
            ref_address: Hash::from(ref_address),
            reaction
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(Hash::SIZE + self.reaction.to_name().len())
    }
}
