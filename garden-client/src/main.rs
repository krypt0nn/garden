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

use std::path::PathBuf;

use relm4::prelude::*;

use flowerpot::node::{Node, NodeOptions};

use anyhow::Context;

pub mod config;
pub mod accounts;
pub mod handler;
pub mod ui;

lazy_static::lazy_static! {
    pub static ref APP_DEBUG: bool = cfg!(debug_assertions);

    pub static ref DATA_FOLDER_PATH: PathBuf = {
        if let Ok(path) = std::env::var("GARDEN_DATA_FOLDER") {
            return PathBuf::from(path);
        }

        let path = std::env::var("XDG_DATA_HOME")
            .map(|data| format!("{data}/garden"))
            .or_else(|_| {
                std::env::var("HOME")
                    .map(|home| {
                        format!("{home}/.local/share/garden")
                    })
            })
            .or_else(|_| {
                std::env::var("USER")
                    .or_else(|_| std::env::var("USERNAME"))
                    .map(|username| {
                        format!("/home/{username}/.local/share/garden")
                    })
            })
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::current_dir()
                    .map(|current| current.join("data"))
            })
            .expect("Couldn't locate data directory");

        path.canonicalize().unwrap_or(path)
    };

    pub static ref CONFIG_FILE_PATH: PathBuf = DATA_FOLDER_PATH.join("config.json");

    pub static ref ACCOUNTS_FILE_PATH: PathBuf = DATA_FOLDER_PATH.join("accounts.json");
}

fn main() -> anyhow::Result<()> {
    // Create data folder.
    if !DATA_FOLDER_PATH.exists() {
        std::fs::create_dir_all(DATA_FOLDER_PATH.as_path())
            .context("failed to create garden data folder")?;
    }

    // Read config file and update it immediately (in case it was changed).
    let config = config::read()
        .context("failed to read config file")?;

    config::write(&config)
        .context("failed to update config file")?;

    // Create flowerpot node handler from config options.
    // let options = NodeOptions {
    //     messages_filter: Some(garden_protocol::messages_filter),

    //     ..NodeOptions::default()
    // };

    // let mut node = Node::default();

    // Create garden protocol handler.
    // ...

    // Initialize libadwaita.
    adw::init().expect("Failed to initializa libadwaita");

    // Register and include resources.
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("Failed to register resources");

    // Set icons search path.
    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);

        theme.add_resource_path("/com/github/krypt0nn/garden/icons");
    }

    // Set application title.
    gtk::glib::set_application_name("Garden");
    gtk::glib::set_program_name(Some("Garden"));

    // Run the app.
    let app = RelmApp::new("com.github.krypt0nn.garden");

    app.run::<ui::login_window::LoginWindow>(accounts::read()?.to_vec());

    Ok(())
}
