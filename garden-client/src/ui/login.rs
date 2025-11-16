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

use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug, Clone)]
pub enum LoginWindowMsg {

}

pub struct LoginWindow {

}

#[relm4::component(pub)]
impl SimpleComponent for LoginWindow {
    type Init = ();
    type Input = LoginWindowMsg;
    type Output = ();

    view! {
        #[root]
        window = adw::ApplicationWindow {
            set_title: Some("Garden"),

            set_size_request: (800, 600),
            set_hide_on_close: false,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                adw::PreferencesPage {
                    adw::PreferencesGroup {
                        set_title: "Accounts",

                        #[wrap(Some)]
                        set_header_suffix = &gtk::Button {
                            add_css_class: "flat",

                            adw::ButtonContent {
                                set_icon_name: "contact-new-symbolic",
                                set_label: "New"
                            }
                        },

                        adw::ActionRow {
                            set_title: "hello",
                            set_activatable: true,

                            add_prefix = &gtk::Image {
                                set_icon_name: Some("person-symbolic")
                            },

                            add_suffix = &gtk::Button {
                                set_vexpand: false,
                                set_valign: gtk::Align::Center,

                                add_css_class: "flat",
                                add_css_class: "destructive-action",

                                adw::ButtonContent {
                                    set_icon_name: "user-trash-symbolic"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>
    ) -> ComponentParts<Self> {
        let model = Self {

        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>
    ) {
        match message {

        }
    }
}
