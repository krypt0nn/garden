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

use garden_protocol::{Content, PostEvent};

#[derive(Debug, Clone)]
pub enum CreatePostDialogMsg {
    Reset,
    VerifyContent,
    Publish
}

pub struct CreatePostDialog {
    window: adw::Dialog,
    text_view: gtk::TextView,

    is_content_valid: bool
}

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
                        #[watch]
                        set_css_classes: if model.is_content_valid {
                            &["suggested-action"]
                        } else {
                            &[]
                        },

                        #[watch]
                        set_sensitive: model.is_content_valid,

                        adw::ButtonContent {
                            set_label: "Publish",
                            set_icon_name: "chat-message-new-symbolic"
                        },

                        connect_clicked => CreatePostDialogMsg::Publish
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
                                set_margin_bottom: 2,

                                add_css_class: "card",

                                #[local_ref]
                                text_view -> gtk::TextView {
                                    set_vexpand: true,
                                    set_hexpand: true,

                                    set_margin_all: 8,

                                    add_css_class: "inline",

                                    set_wrap_mode: gtk::WrapMode::WordChar,

                                    #[wrap(Some)]
                                    set_buffer = &gtk::TextBuffer {
                                        connect_changed => CreatePostDialogMsg::VerifyContent
                                    }
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
            window: root.clone(),
            text_view: gtk::TextView::new(),

            is_content_valid: true
        };

        let text_view = &model.text_view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>
    ) {
        match message {
            CreatePostDialogMsg::Reset => {
                self.text_view.buffer().set_text("");

                self.is_content_valid = true;
            }

            CreatePostDialogMsg::VerifyContent => {
                let content = self.text_view.buffer().text(
                    &self.text_view.buffer().start_iter(),
                    &self.text_view.buffer().end_iter(),
                    true
                );

                self.is_content_valid = Content::new(content).is_some();
            }

            CreatePostDialogMsg::Publish => {
                let content = self.text_view.buffer().text(
                    &self.text_view.buffer().start_iter(),
                    &self.text_view.buffer().end_iter(),
                    true
                );

                let Some(content) = Content::new(content) else {
                    return;
                };

                if let Some(event) = PostEvent::new(content, []) {
                    let _ = sender.output(event);

                    self.window.close();
                }
            }
        }
    }
}
