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

/// A unicode text which doesn't contain any control or special characters which
/// can glitch the visual representation of the text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrintableText(String);

impl PrintableText {
    /// Create new printable text from provided string.
    ///
    /// This function will return `None` if provided message has invalid format.
    pub fn new(content: impl AsRef<str>) -> Option<Self> {
        let content = content.as_ref()
            .trim()
            .to_string();

        if !(1..=1024).contains(&content.len()) {
            return None;
        }

        // TODO: more restrictions
        if content.chars().any(|c| c.is_ascii_control()) {
            return None;
        }

        Some(Self(content))
    }
}

impl From<PrintableText> for String {
    #[inline(always)]
    fn from(value: PrintableText) -> Self {
        value.0
    }
}

impl AsRef<PrintableText> for PrintableText {
    #[inline(always)]
    fn as_ref(&self) -> &PrintableText {
        self
    }
}

impl AsRef<String> for PrintableText {
    #[inline(always)]
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl std::ops::Deref for PrintableText {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for PrintableText {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
