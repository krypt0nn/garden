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

use libflowerpot::crypto::base64;
use libflowerpot::crypto::hash::Hash;

/// Address of some entity on the flowerpot blockchain.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockchainAddress {
    block: Hash,
    transaction: Hash
}

impl BlockchainAddress {
    /// Bytes length of the blockchain entity address.
    pub const SIZE: usize = Hash::SIZE * 2;

    pub fn new(block: impl Into<Hash>, transaction: impl Into<Hash>) -> Self {
        Self {
            block: block.into(),
            transaction: transaction.into()
        }
    }

    #[inline(always)]
    pub const fn block(&self) -> &Hash {
        &self.block
    }

    #[inline(always)]
    pub const fn transaction(&self) -> &Hash {
        &self.transaction
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut address = [0; Self::SIZE];

        address[..Hash::SIZE].copy_from_slice(self.block.as_bytes());
        address[Hash::SIZE..].copy_from_slice(self.transaction.as_bytes());

        address
    }

    pub fn from_bytes(address: &[u8; Self::SIZE]) -> Self {
        let mut block = [0; Hash::SIZE];
        let mut transaction = [0; Hash::SIZE];

        block.copy_from_slice(&address[..Hash::SIZE]);
        transaction.copy_from_slice(&address[Hash::SIZE..]);

        Self {
            block: Hash::from(block),
            transaction: Hash::from(transaction)
        }
    }

    #[inline]
    pub fn to_base64(&self) -> String {
        base64::encode(self.to_bytes())
    }

    pub fn from_base64(address: impl AsRef<[u8]>) -> Option<Self> {
        let address = base64::decode(address).ok()?;

        if address.len() != Self::SIZE {
            return None;
        }

        let mut buf = [0; Self::SIZE];

        buf.copy_from_slice(&address);

        Some(Self::from_bytes(&buf))
    }
}
