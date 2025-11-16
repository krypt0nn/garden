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

use relm4::prelude::*;

pub mod ui;

lazy_static::lazy_static! {
    pub static ref APP_DEBUG: bool = cfg!(debug_assertions);
}

fn main() -> anyhow::Result<()> {
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

    // Create the app.
    let app = RelmApp::new("com.github.krypt0nn.garden");

    // Show loading window.
    app.run::<ui::login::LoginWindow>(());

    Ok(())
}
