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
use relm4::{Worker, WorkerController};

use flowerpot::crypto::hash::Hash;
use flowerpot::crypto::sign::SigningKey;

use garden_protocol::PostEvent;
use garden_protocol::index::post::PostInfo;
use garden_protocol::handler::Handler;

use crate::node::Progress as StartNodeProgress;

use crate::ui::create_post_dialog::CreatePostDialog;

#[derive(Debug, Clone)]
enum MainWindowHandlerWorkerInput {
    /// Set garden protocol handler.
    SetHandler(Handler),

    /// Request worker to update the garden index.
    Update,

    /// Send post to the network.
    PublishPost {
        signing_key: SigningKey,
        event: PostEvent
    },

    /// Query posts since provided message hash.
    QueryPosts {
        since_message: Option<Hash>
    }
}

#[derive(Debug, Clone)]
enum MainWindowHandlerWorkerOutput {
    /// Update main window status.
    UpdateStatus(MainWindowStatus),

    /// Queried post info.
    Post(PostInfo)
}

struct MainWindowHandlerWorker {
    handler: Option<Handler>
}

impl Worker for MainWindowHandlerWorker {
    type Init = ();
    type Input = MainWindowHandlerWorkerInput;
    type Output = MainWindowHandlerWorkerOutput;

    fn init(
        _init: Self::Init,
        sender: ComponentSender<Self>
    ) -> Self {
        // TODO: error handling.

        let config = crate::config::read()
            .expect("failed to read config");

        std::thread::spawn(move || {
            let address = config.blockchain_address.clone();

            let handle = crate::node::start(&config, |progress| {
                let _ = sender.output(
                    MainWindowHandlerWorkerOutput::UpdateStatus(
                        MainWindowStatus::Starting(progress)
                    )
                );
            });

            let handler = handle.expect("failed to start flowerpot node");

            let handler = Handler::new(address, handler);

            sender.input(MainWindowHandlerWorkerInput::SetHandler(handler));

            let _ = sender.output(
                MainWindowHandlerWorkerOutput::UpdateStatus(
                    MainWindowStatus::Running
                )
            );

            loop {
                sender.input(MainWindowHandlerWorkerInput::Update);

                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });

        Self {
            handler: None
        }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>
    ) {
        match message {
            MainWindowHandlerWorkerInput::SetHandler(handler) => {
                self.handler = Some(handler);
            }

            MainWindowHandlerWorkerInput::Update => {
                #[allow(clippy::collapsible_if)]
                if let Some(handler) = &self.handler {
                    if let Err(err) = handler.update() {
                        tracing::error!(?err, "failed to update garden handler");
                    }
                }
            }

            MainWindowHandlerWorkerInput::PublishPost {
                signing_key,
                event
            } => {
                if let Some(handler) = &self.handler {
                    handler.send_post(&signing_key, event)
                        .expect("failed to send post to the flowerpot network");
                }
            }

            MainWindowHandlerWorkerInput::QueryPosts { since_message } => {
                if let Some(handler) = &self.handler {
                    let posts = handler.index()
                        .posts()
                        .skip_while(|post| {
                            match &since_message {
                                Some(since_message) => post.message_hash() != since_message,
                                None => false
                            }
                        })
                        .skip(if since_message.is_some() { 1 } else { 0 })
                        .cloned()
                        .collect::<Vec<_>>();

                    for post in posts {
                        if let Some(post) = handler.read_post(&post) {
                            match post {
                                Ok(post) => {
                                    let _ = sender.output(MainWindowHandlerWorkerOutput::Post(post));
                                }

                                Err(err) => {
                                    // TODO: error handling.

                                    tracing::error!(?err, "failed to read post info");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainWindowStatus {
    /// Flowerpot node is not started and handler is not available.
    None,

    /// Starting the flowerpot node.
    Starting(StartNodeProgress),

    /// Handler is available.
    Running
}

#[derive(Debug, Clone)]
pub enum MainWindowMsg {
    SetStatus(MainWindowStatus),
    SetSigningKey(SigningKey),
    Update,
    OpenCreatePostDialog,
    PublishPost(PostEvent),
    AddPost(PostInfo)
}

pub struct MainWindow {
    status: MainWindowStatus,
    signing_key: Option<SigningKey>,

    handler_worker: WorkerController<MainWindowHandlerWorker>,

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
                set_visible: model.status != MainWindowStatus::Running,

                adw::HeaderBar,

                adw::StatusPage {
                    set_title: "Starting flowerpot node",

                    #[watch]
                    set_description: match &model.status {
                        MainWindowStatus::None |
                        MainWindowStatus::Running => Some(String::new()),

                        MainWindowStatus::Starting(progress) => {
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
                set_visible: model.status == MainWindowStatus::Running,

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
        sender: ComponentSender<Self>
    ) -> ComponentParts<Self> {
        let model = Self {
            status: MainWindowStatus::None,
            signing_key: None,

            handler_worker: MainWindowHandlerWorker::builder()
                .detach_worker(())
                .forward(sender.input_sender(), |message| {
                    match message {
                        MainWindowHandlerWorkerOutput::UpdateStatus(status) =>
                            MainWindowMsg::SetStatus(status),

                        MainWindowHandlerWorkerOutput::Post(post)
                            => MainWindowMsg::AddPost(post)
                    }
                }),

            window: root.clone(),

            posts_factory: FactoryVecDeque::builder()
                .launch_default()
                .detach(),

            create_post_dialog: CreatePostDialog::builder()
                .launch(())
                .forward(sender.input_sender(), MainWindowMsg::PublishPost)
        };

        let posts_factory = model.posts_factory.widget();

        let widgets = view_output!();

        std::thread::spawn(move || {
            loop {
                sender.input(MainWindowMsg::Update);

                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>
    ) {
        match message {
            MainWindowMsg::SetStatus(status) => {
                self.status = status;
            }

            MainWindowMsg::SetSigningKey(signing_key) => {
                self.signing_key = Some(signing_key);
            }

            MainWindowMsg::Update => {
                let last_post = self.posts_factory.guard()
                    .get(0)
                    .map(|post| post.post.message_hash);

                self.handler_worker.emit(MainWindowHandlerWorkerInput::QueryPosts {
                    since_message: last_post
                });
            }

            MainWindowMsg::OpenCreatePostDialog => {
                self.create_post_dialog.widget()
                    .present(Some(&self.window));
            }

            MainWindowMsg::PublishPost(event) => {
                if let Some(signing_key) = self.signing_key.clone() {
                    self.handler_worker.emit(MainWindowHandlerWorkerInput::PublishPost {
                        signing_key,
                        event
                    });
                }
            }

            MainWindowMsg::AddPost(post) => {
                self.posts_factory.guard()
                    .push_front(post);
            }
        }
    }
}
