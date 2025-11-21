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

use std::sync::Arc;

use spin::{RwLock, RwLockReadGuard};

use flowerpot::crypto::sign::{SigningKey, SignatureError};
use flowerpot::address::Address;
use flowerpot::message::Message;
use flowerpot::node::NodeHandler;

use crate::index::{Index, IndexUpdateError, IndexReadError};
use crate::index::post::{PostInfo, PostIndex};
use crate::index::comment::{CommentInfo, CommentIndex};

use super::{Events, PostEvent, CommentEvent};

/// A helper struct that holds reference to background flowerpot node handler,
/// a database indexer, and allows to execute garden protocol related actions
/// and query data.
#[derive(Clone)]
pub struct Handler {
    /// Garden protocol blockchain address.
    address: Arc<Address>,

    /// Flowerpot node handler.
    node: NodeHandler,

    /// Garden protocol indexer.
    index: Arc<RwLock<Index>>
}

impl Handler {
    /// Create new garden handler from provided flowerpot node handler and hash
    /// of the root block of a blockchain where garden protocol is stored.
    pub fn new(address: impl Into<Address>, node: NodeHandler) -> Self {
        Self {
            address: Arc::new(address.into()),
            node,
            index: Arc::new(RwLock::new(Index::default()))
        }
    }

    /// Get reference to the garden protocol blockchain address.
    #[inline]
    pub const fn address(&self) -> &Arc<Address> {
        &self.address
    }

    /// Get reference to the flowerpot node handler.
    #[inline]
    pub const fn node(&self) -> &NodeHandler {
        &self.node
    }

    /// Get reference to the garden protocol index.
    #[inline]
    pub fn index(&self) -> RwLockReadGuard<'_, Index> {
        self.index.read()
    }

    /// Update garden protocol indexer using blockchain tracker.
    pub fn update(&self) -> Result<(), IndexUpdateError> {
        let mut index = self.index.write();

        let result = self.node.map_storage(&self.address, move |storage| {
            index.update(storage)
        });

        match result {
            Some(result) => result,
            None => Ok(())
        }
    }

    /// Try to read indexed garden post info.
    ///
    /// Return `None` if there's no storage for a blockchain with provided
    /// address.
    ///
    /// Otherwise `Some(..)` with post reading result is returned.
    pub fn read_post(
        &self,
        post: &PostIndex
    ) -> Option<Result<PostInfo, IndexReadError>> {
        self.node.map_storage(&self.address, |storage| {
            Some(post.read(storage))
        }).flatten()
    }

    /// Try to read indexed garden comment info.
    ///
    /// Return `None` if there's no storage for a blockchain with provided
    /// address.
    ///
    /// Otherwise `Some(..)` with comment reading result is returned.
    pub fn read_comment(
        &self,
        comment: &CommentIndex
    ) -> Option<Result<CommentInfo, IndexReadError>> {
        self.node.map_storage(&self.address, |storage| {
            Some(comment.read(storage))
        }).flatten()
    }

    /// Create a new flowerpot message from provided event using provided
    /// signing key and send it to the network using underlying node handler.
    fn send_event(
        &self,
        signing_key: &SigningKey,
        event: &Events
    ) -> Result<(), SignatureError> {
        let message = Message::create(signing_key, event.to_bytes())?;

        self.node.send_message(self.address.as_ref().clone(), message);

        Ok(())
    }

    /// Create a new flowerpot message from new post event using provided
    /// signing key and send it to the network using underlying node handler.
    #[inline]
    pub fn send_post(
        &self,
        signing_key: &SigningKey,
        post: PostEvent
    ) -> Result<(), SignatureError> {
        self.send_event(signing_key, &Events::from(post))
    }

    /// Create a new flowerpot message from new comment event using provided
    /// signing key and send it to the network using underlying node handler.
    #[inline]
    pub fn send_comment(
        &self,
        signing_key: &SigningKey,
        comment: CommentEvent
    ) -> Result<(), SignatureError> {
        self.send_event(signing_key, &Events::from(comment))
    }
}

impl std::fmt::Debug for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handler")
            .field("address", &self.address.to_base64())
            .finish()
    }
}
