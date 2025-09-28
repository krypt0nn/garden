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

use crate::types::{PrintableText, BlockchainAddress};

use super::Event;

#[derive(Debug, thiserror::Error)]
pub enum CreateCommunityPostEventError {
    #[error("provided event bytes slice is too short")]
    SliceTooShort,

    #[error("invalid unicode sequence: {0}")]
    InvalidUnicode(#[from] std::string::FromUtf8Error),

    #[error("post title uses invalid format")]
    InvalidTitleFormat,

    #[error("post body uses invalid format")]
    InvalidBodyFormat
}

/// Create new community post.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateCommunityPostEvent {
    /// Blockchain address of the community to create this post in.
    community_address: BlockchainAddress,

    /// Title of the post.
    title: PrintableText,

    /// Body of the post.
    body: PrintableText
}

impl CreateCommunityPostEvent {
    pub fn new(
        community_address: impl Into<BlockchainAddress>,
        title: impl Into<PrintableText>,
        body: impl Into<PrintableText>
    ) -> Self {
        // TODO: ensure title length limit

        Self {
            community_address: community_address.into(),
            title: title.into(),
            body: body.into()
        }
    }

    #[inline(always)]
    pub const fn community_address(&self) -> &BlockchainAddress {
        &self.community_address
    }

    #[inline(always)]
    pub const fn title(&self) -> &PrintableText {
        &self.title
    }

    #[inline(always)]
    pub const fn body(&self) -> &PrintableText {
        &self.body
    }

    fn size(&self) -> usize {
        BlockchainAddress::SIZE +
            2 + self.title.len() +
            self.body.len()
    }
}

impl Event for CreateCommunityPostEvent {
    type Error = CreateCommunityPostEventError;

    fn to_bytes(&self) -> Box<[u8]> {
        assert!(self.title.len() < u16::MAX as usize);

        let mut buf = Vec::with_capacity(self.size());

        let title_len = self.title.len() as u16;

        buf.extend(self.community_address.to_bytes());
        buf.extend(title_len.to_le_bytes());
        buf.extend(self.title.as_bytes());
        buf.extend(self.body.as_bytes());

        buf.into_boxed_slice()
    }

    fn from_bytes(event: &[u8]) -> Result<Self, Self::Error> where Self: Sized {
        if event.len() < BlockchainAddress::SIZE + 2 {
            return Err(CreateCommunityPostEventError::SliceTooShort);
        }

        let mut address = [0; BlockchainAddress::SIZE];
        let mut title_len = [0; 2];

        const TITLE_OFFSET: usize = BlockchainAddress::SIZE + 2;

        address.copy_from_slice(&event[..BlockchainAddress::SIZE]);
        title_len.copy_from_slice(&event[BlockchainAddress::SIZE..TITLE_OFFSET]);

        let title_len = u16::from_le_bytes(title_len) as usize;

        let body_offset = TITLE_OFFSET + title_len;

        if event.len() < body_offset {
            return Err(CreateCommunityPostEventError::SliceTooShort);
        }

        let title = String::from_utf8(event[TITLE_OFFSET..body_offset].to_vec())?;
        let body = String::from_utf8(event[body_offset..].to_vec())?;

        let Some(title) = PrintableText::new(title) else {
            return Err(CreateCommunityPostEventError::InvalidTitleFormat);
        };

        let Some(body) = PrintableText::new(body) else {
            return Err(CreateCommunityPostEventError::InvalidBodyFormat);
        };

        Ok(Self {
            community_address: BlockchainAddress::from_bytes(&address),
            title,
            body
        })
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.size())
    }
}
