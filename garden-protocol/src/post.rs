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

use regex::Regex;

use super::Event;

lazy_static::lazy_static! {
    /// Post tag regex. The rules are:
    ///
    /// 1. Tag can contain only lowercase latin alphabet and numbers.
    /// 2. Tag can contain dashes ("-") in-between the letters or digits.
    /// 3. Tag must be at least 1 character (byte) long and cannot be longer
    ///    than 255 characters (bytes).
    ///
    /// The name length must be verified separately from the regex.
    pub static ref TAG_REGEX: Regex = Regex::new(r#"^[a-z0-9]{1,255}$|^[a-z0-9]{1,255}[a-z0-9\-]{0,255}[a-z0-9]{1,255}$"#)
        .expect("failed to build tag regex");
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Content(String);

impl Content {
    /// Create new content string, return `None` if its length exceeds max
    /// allowed size (65,535 bytes).
    pub fn new(content: impl ToString) -> Option<Self> {
        let content = content.to_string();

        if content.len() > u16::MAX as usize {
            return None;
        }

        Some(Self(content))
    }
}

impl From<Content> for String {
    #[inline(always)]
    fn from(value: Content) -> Self {
        value.0
    }
}

impl std::ops::Deref for Content {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(String);

impl Tag {
    /// Create new tag string, return `None` if its length exceeds max allowed
    /// size (255 bytes).
    pub fn new(tag: impl ToString) -> Option<Self> {
        let tag = tag.to_string();

        if !(1..=u8::MAX as usize).contains(&tag.len())
            || !TAG_REGEX.is_match(&tag)
        {
            return None;
        }

        Some(Self(tag))
    }
}

impl From<Tag> for String {
    #[inline(always)]
    fn from(value: Tag) -> Self {
        value.0
    }
}

impl std::ops::Deref for Tag {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PostEventError {
    #[error("invalid unicode sequence: {0}")]
    InvalidUnicode(#[from] std::string::FromUtf8Error),

    #[error("provided post event bytes slice is too short")]
    SliceTooShort,

    #[error("invalid content")]
    InvalidContent,

    #[error("invalid tag")]
    InvalidTag
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostEvent {
    content: Content,
    tags: Box<[Tag]>
}

impl PostEvent {
    /// Create new post event. Return `None` if provided tags len exceeds max
    /// allowed amount (255 items).
    pub fn new(
        content: Content,
        tags: impl IntoIterator<Item = Tag>
    ) -> Option<Self> {
        let tags = tags.into_iter()
            .collect::<Box<[Tag]>>();

        if tags.len() > u8::MAX as usize {
            return None;
        }

        Some(Self {
            content,
            tags
        })
    }

    #[inline(always)]
    pub const fn content(&self) -> &Content {
        &self.content
    }

    #[inline(always)]
    pub const fn tags(&self) -> &[Tag] {
        &self.tags
    }
}

impl Event for PostEvent {
    type Error = PostEventError;

    fn to_bytes(&self) -> Box<[u8]> {
        let content_len = self.content.len();
        let tags_amount = self.tags.len();

        assert!(content_len <= u16::MAX as usize);
        assert!(tags_amount <= u8::MAX as usize);

        let mut buf = Vec::new();

        buf.extend((content_len as u16).to_le_bytes());
        buf.extend(self.content.as_bytes());
        buf.push(tags_amount as u8);

        for tag in &self.tags {
            let tag_len = tag.len();

            assert!(tag_len <= u8::MAX as usize);

            buf.push(tag_len as u8);
            buf.extend(tag.as_bytes());
        }

        buf.into_boxed_slice()
    }

    fn from_bytes(event: &[u8]) -> Result<Self, Self::Error> where Self: Sized {
        let n = event.len();

        if n < 3 {
            return Err(PostEventError::SliceTooShort);
        }

        let content_len = u16::from_le_bytes([event[0], event[1]]) as usize;

        if n < content_len + 2 {
            return Err(PostEventError::SliceTooShort);
        }

        let tags_amount = event[content_len + 2] as usize;

        let content = String::from_utf8(event[2..content_len + 2].to_vec())?;

        let Some(content) = Content::new(content) else {
            return Err(PostEventError::InvalidContent);
        };

        let mut tags = Vec::with_capacity(tags_amount);

        let mut tags_offset = content_len + 3;

        // TODO: more length checks

        for _ in 0..tags_amount {
            let tag_len = event[tags_offset] as usize;

            tags_offset += 1;

            let tag = &event[tags_offset..tags_offset + tag_len];

            tags_offset += tag_len;

            let tag = String::from_utf8(tag.to_vec())?;

            let Some(tag) = Tag::new(tag) else {
                return Err(PostEventError::InvalidTag);
            };

            tags.push(tag);
        }

        Ok(Self {
            content,
            tags: tags.into_boxed_slice()
        })
    }
}
