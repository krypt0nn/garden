// SPDX-License-Identifier: GPL-3.0-or-later
//
// garden-client
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

use flowerpot::crypto::base64;
use flowerpot::crypto::sign::SigningKey;

use anyhow::Context;
use time::UtcDateTime;
use serde_json::{json, Value as Json};

use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use chacha20poly1305::aead::{KeyInit, AeadMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account {
    /// Account name.
    name: String,

    /// Account creation time.
    created_at: UtcDateTime,

    /// Base64-encoded chacha20poly1305 encrypted signing key of account.
    signing_key: String
}

impl Account {
    pub const CONTEXT: &str = "garden client account encryption key context";
    pub const NONCE: [u8; 12] = [73, 144, 0, 139, 49, 38, 122, 43, 159, 112, 212, 48];

    /// Create new account from provided name, signing key and key encryption
    /// password.
    pub fn new(
        name: impl ToString,
        signing_key: impl Into<SigningKey>,
        password: &[u8]
    ) -> anyhow::Result<Self> {
        let password = blake3::derive_key(Self::CONTEXT, password);
        let nonce = Nonce::from_slice(&Self::NONCE);

        let mut encryptor = ChaCha20Poly1305::new_from_slice(&password)
            .map_err(|err| {
                anyhow::anyhow!("failed to create chacha20poly1305 encryptor")
                    .context(err)
            })?;

        let signing_key: SigningKey = signing_key.into();
        let signing_key = signing_key.to_bytes();

        let signing_key = encryptor.encrypt(nonce, signing_key.as_slice())
            .map_err(|err| {
                anyhow::anyhow!("failed to encrypt account signing key")
                    .context(err)
            })?;

        Ok(Self {
            name: name.to_string(),
            created_at: UtcDateTime::now(),
            signing_key: base64::encode(signing_key)
        })
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn created_at(&self) -> &UtcDateTime {
        &self.created_at
    }

    /// Try to decrypt account signing key using provided password.
    pub fn signing_key(&self, password: &[u8]) -> anyhow::Result<SigningKey> {
        let password = blake3::derive_key(Self::CONTEXT, password);
        let nonce = Nonce::from_slice(&Self::NONCE);

        let mut decryptor = ChaCha20Poly1305::new_from_slice(&password)
            .map_err(|err| {
                anyhow::anyhow!("failed to create chacha20poly1305 decryptor")
                    .context(err)
            })?;

        let signing_key = base64::decode(&self.signing_key)
            .map_err(|err| {
                anyhow::anyhow!("failed to decode account signing key from base64")
                    .context(err)
            })?;

        let signing_key = decryptor.decrypt(nonce, signing_key.as_slice())
            .map_err(|err| {
                anyhow::anyhow!("failed to decrypt account signing key")
                    .context(err)
            })?;

        if signing_key.len() != SigningKey::SIZE {
            anyhow::bail!("invalid signing key size");
        }

        let mut buf = [0; SigningKey::SIZE];

        buf.copy_from_slice(&signing_key);

        let signing_key = SigningKey::from_bytes(&buf)
            .ok_or_else(|| {
                anyhow::anyhow!("failed to decode account signing key from decrypted binary data")
            })?;

        Ok(signing_key)
    }

    pub fn to_json(&self) -> Json {
        json!({
            "name": self.name,
            "created_at": self.created_at.unix_timestamp(),
            "signing_key": self.signing_key
        })
    }

    pub fn from_json(json: &Json) -> Option<Self> {
        Some(Self {
            name: json.get("name")
                .and_then(Json::as_str)
                .map(String::from)?,

            created_at: json.get("created_at")
                .and_then(Json::as_i64)
                .and_then(|created_at| {
                    UtcDateTime::from_unix_timestamp(created_at).ok()
                })?,

            signing_key: json.get("signing_key")
                .and_then(Json::as_str)
                .map(String::from)?
        })
    }
}

/// Try to read accounts file.
pub fn read() -> anyhow::Result<Box<[Account]>> {
    if !crate::ACCOUNTS_FILE_PATH.is_file() {
        return Ok(Box::new([]));
    }

    let accounts = std::fs::read(crate::ACCOUNTS_FILE_PATH.as_path())
        .context("failed to read accounts file")?;

    let accounts = serde_json::from_slice::<Box<[Json]>>(&accounts)
        .context("failed to deserialize accounts array")?
        .into_iter()
        .map(|account| Account::from_json(&account))
        .collect::<Option<Box<[Account]>>>()
        .ok_or_else(|| {
            anyhow::anyhow!("failed to deserialize account")
        })?;

    Ok(accounts)
}

/// Try to write accounts file.
pub fn write(
    accounts: impl IntoIterator<Item = Account>
) -> anyhow::Result<()> {
    let accounts = accounts.into_iter()
        .map(|account| account.to_json())
        .collect::<Box<[Json]>>();

    let accounts = serde_json::to_vec_pretty(&json!(accounts))
        .context("failed to serialize accounts array")?;

    std::fs::write(crate::ACCOUNTS_FILE_PATH.as_path(), accounts)
        .context("failed to write accounts file")?;

    Ok(())
}
