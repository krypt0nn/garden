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

use std::sync::Arc;

use spin::RwLock;

use flowerpot::node::NodeHandler;

use garden_protocol::index::Index;

/// A helper struct that holds reference to background flowerpot node handler,
/// a database indexer, and allows to execute garden protocol related actions
/// and query data.
#[derive(Clone)]
pub struct Handler {
    node: NodeHandler,
    index: Arc<RwLock<Index>>
}

impl Handler {
    /// Create new garden handler from provided flowerpot node handler.
    pub fn new(node: NodeHandler) -> Self {
        Self {
            node,
            index: Arc::new(RwLock::new(Index::default()))
        }
    }

    pub fn posts(&self) {

    }
}
