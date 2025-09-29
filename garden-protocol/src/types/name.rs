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

lazy_static::lazy_static! {
    /// Community / user name regex. The rules are:
    ///
    /// 1. Name can contain only latin alphabet (lower and uppper cases) and
    ///    numbers.
    /// 2. Name can contain underscores ("_") in-between the letters or digits.
    /// 3. Name must be at least 1 character (byte) long and cannot be longer
    ///    than 64 characters (bytes).
    ///
    /// The name length must be verified separately from the regex.
    pub static ref NAME_REGEX: Regex = Regex::new(r#"^[a-zA-Z0-9]{1,64}$|^[a-zA-Z0-9]{1,64}[a-zA-Z0-9\_]{0,64}[a-zA-Z0-9]{1,64}$"#)
        .expect("failed to build name regex");
}

/// A highly filtered ASCII-like text suitable for a user or community name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    /// Create new community or user name from provided string.
    ///
    /// This function will return `None` if provided name has invalid format.
    pub fn new(name: impl AsRef<str>) -> Option<Self> {
        let name = name.as_ref()
            .trim()
            .to_string();

        if !(1..=64).contains(&name.len()) || !NAME_REGEX.is_match(&name) {
            return None;
        }

        Some(Self(name))
    }
}

impl From<Name> for String {
    #[inline(always)]
    fn from(value: Name) -> Self {
        value.0
    }
}

impl AsRef<Name> for Name {
    #[inline(always)]
    fn as_ref(&self) -> &Name {
        self
    }
}

impl AsRef<String> for Name {
    #[inline(always)]
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl std::ops::Deref for Name {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
