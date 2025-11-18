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

use flowerpot::crypto::sign::SigningKey;

use garden_protocol::index::post::PostInfo;
use garden_protocol::handler::Handler;

use crate::node::Progress as StartNodeProgress;

use crate::ui::create_post_dialog::CreatePostDialog;

#[derive(Debug)]
struct MainWindowPostFactory {
    post: PostInfo,
    index: DynamicIndex
}

#[relm4::factory]
impl FactoryComponent for MainWindowPostFactory {
    type Init = PostInfo;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        adw::Bin {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                set_margin_all: 8,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Label {
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,

                        set_label: &format!("@{}", self.post.author.to_base64())
                    },

                    gtk::Label {
                        set_hexpand: true,
                        set_halign: gtk::Align::End,

                        set_label: &self.post.timestamp.to_string()
                    },
                },

                gtk::Label {
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    set_justify: gtk::Justification::Fill,

                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::WordChar,

                    set_label: &self.post.content
                },
            }
        }
    }

    #[inline]
    fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        _sender: FactorySender<Self>
    ) -> Self {
        Self {
            post: init,
            index: index.clone()
        }
    }
}

pub enum HandlerStatus {
    /// Flowerpot node is not started and handler is not available.
    None,

    /// Starting the flowerpot node.
    StartNode(StartNodeProgress),

    /// Handler is available.
    Handler(Handler)
}

#[derive(Debug, Clone)]
pub enum MainWindowMsg {
    StartHandler,
    SetSigningKey(SigningKey),
    OpenCreatePostDialog
}

pub struct MainWindow {
    handler: HandlerStatus,
    signing_key: Option<SigningKey>,

    window: adw::ApplicationWindow,
    posts_factory: FactoryVecDeque<MainWindowPostFactory>,
    create_post_dialog: Controller<CreatePostDialog>
}

#[relm4::component(pub)]
impl SimpleComponent for MainWindow {
    type Init = ();
    type Input = MainWindowMsg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            #[watch]
            set_title: Some(&{
                model.signing_key.as_ref()
                    .map(|signing_key| {
                        let verifying_key = signing_key.verifying_key();

                        format!("@{}", verifying_key.to_base64())
                    })
                    .unwrap_or_else(|| String::from("Garden"))
            }),

            set_size_request: (1000, 700),
            set_hide_on_close: false,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[watch]
                set_visible: !matches!(model.handler, HandlerStatus::Handler(_)),

                adw::HeaderBar,

                adw::StatusPage {
                    set_title: "Starting flowerpot node",

                    #[watch]
                    set_description: match &model.handler {
                        HandlerStatus::None |
                        HandlerStatus::Handler(_) => Some(String::new()),

                        HandlerStatus::StartNode(progress) => {
                            match progress {
                                StartNodeProgress::CreateTracker(path)
                                    => Some(format!("Open blockchain storage at {path:?}")),

                                StartNodeProgress::EstablishConnection(addr)
                                    => Some(format!("Connecting to {addr}")),

                                StartNodeProgress::SynchronizeBlockchain
                                    => Some(String::from("Synchronizing blockchain")),

                                StartNodeProgress::StartNode
                                    => Some(String::from("Starting flowerpot node")),

                                StartNodeProgress::StartListener(addr)
                                    => Some(format!("Starting listener at {addr}"))
                            }
                        }
                    }.as_deref(),

                    set_icon_name: Some("com.github.krypt0nn.garden"),

                    set_vexpand: true,
                    set_valign: gtk::Align::Center
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[watch]
                set_visible: matches!(model.handler, HandlerStatus::Handler(_)),

                adw::HeaderBar {
                    pack_end = &gtk::Button {
                        adw::ButtonContent {
                            set_label: "Create post",
                            set_icon_name: "chat-message-new-symbolic"
                        },

                        connect_clicked => MainWindowMsg::OpenCreatePostDialog
                    }
                },

                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,

                    set_margin_top: 16,
                    set_margin_bottom: 16,

                    adw::Clamp {
                        #[local_ref]
                        posts_factory -> gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,

                            add_css_class: "boxed-list-separate"
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
            handler: HandlerStatus::None,
            signing_key: None,

            window: root.clone(),

            posts_factory: FactoryVecDeque::builder()
                .launch_default()
                .detach(),

            create_post_dialog: CreatePostDialog::builder()
                .launch(())
                .detach()
        };

        let posts_factory = model.posts_factory.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>
    ) {
        match message {
            MainWindowMsg::StartHandler => {
                let (send, recv) = std::sync::mpsc::channel();

                // TODO: error handling.

                let config = crate::config::read()
                    .expect("failed to read config");

                let root_block = config.blockchain_root_block;

                let handle = std::thread::spawn(move || {
                    crate::node::start(&config, |progress| {
                        let _ = send.send(HandlerStatus::StartNode(progress));
                    })
                });

                while let Ok(status) = recv.recv() {
                    self.handler = status;
                }

                let handler = handle.join()
                    .expect("failed to join flowerpot node starting thread")
                    .expect("failed to start flowerpot node");

                let handler = Handler::new(root_block, handler);

                {
                    let handler = handler.clone();

                    std::thread::spawn(move || {
                        loop {
                            if let Err(err) = handler.update() {
                                panic!("failed to update indexer: {err}");
                            }

                            std::thread::sleep(std::time::Duration::from_secs(5));
                        }
                    });
                }

                self.handler = HandlerStatus::Handler(handler);
            }

            MainWindowMsg::SetSigningKey(signing_key) => {
                self.signing_key = Some(signing_key);
            }

            MainWindowMsg::OpenCreatePostDialog => {
                self.create_post_dialog.widget()
                    .present(Some(&self.window));
            }
        }
    }
}
