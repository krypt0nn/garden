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

use garden_protocol::PostEvent;

#[derive(Debug, Clone)]
pub enum CreatePostDialogMsg {

}

pub struct CreatePostDialog;

#[relm4::component(pub)]
impl SimpleComponent for CreatePostDialog {
    type Init = ();
    type Input = CreatePostDialogMsg;
    type Output = PostEvent;

    view! {
        adw::Dialog {
            set_title: "Create post",

            set_size_request: (600, 400),

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat",

                    pack_end = &gtk::Button {
                        add_css_class: "suggested-action",

                        adw::ButtonContent {
                            set_label: "Publish",
                            set_icon_name: "chat-message-new-symbolic"
                        }
                    }
                },

                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,

                    set_margin_top: 16,
                    set_margin_bottom: 32,

                    adw::Clamp {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                add_css_class: "heading",

                                set_text: "Post content"
                            },

                            adw::Bin {
                                set_margin_top: 8,

                                add_css_class: "card",

                                gtk::TextView {
                                    set_vexpand: true,
                                    set_hexpand: true,

                                    set_margin_all: 8,

                                    add_css_class: "inline",

                                    set_wrap_mode: gtk::WrapMode::WordChar
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
        let model = Self;

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
