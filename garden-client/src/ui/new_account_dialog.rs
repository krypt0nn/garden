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

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

use time::UtcDateTime;

use crate::accounts::Account;

#[derive(Debug, Clone)]
pub enum NewAccountDialogMsg {
    RandSigningKey,
    VerifySigningKey,
    VerifyPassword,
    Create
}

pub struct NewAccountDialog {
    rng: ChaCha20Rng,

    window: adw::Dialog,
    name_row: adw::EntryRow,
    signing_key_row: adw::EntryRow,
    password_row: adw::PasswordEntryRow,
    repeat_password_row: adw::PasswordEntryRow,

    is_signing_key_valid: bool,
    is_password_valid: bool
}

#[relm4::component(pub)]
impl SimpleComponent for NewAccountDialog {
    type Init = ();
    type Input = NewAccountDialogMsg;
    type Output = Account;

    view! {
        adw::Dialog {
            set_title: "New account",

            set_size_request: (600, 410),

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                adw::PreferencesPage {
                    adw::PreferencesGroup {
                        set_title: "New account",

                        #[local_ref]
                        name_row -> adw::EntryRow {
                            set_title: "Name",
                            set_text: "New account",

                            set_show_apply_button: false
                        },

                        #[local_ref]
                        signing_key_row -> adw::EntryRow {
                            set_title: "Signing key",

                            set_show_apply_button: false,

                            #[watch]
                            set_css_classes: if model.is_signing_key_valid {
                                &[]
                            } else {
                                &["error"]
                            },

                            add_suffix = &gtk::Button {
                                set_vexpand: false,
                                set_valign: gtk::Align::Center,

                                add_css_class: "flat",

                                adw::ButtonContent {
                                    set_icon_name: "dice3-symbolic"
                                },

                                connect_clicked => NewAccountDialogMsg::RandSigningKey
                            },

                            connect_changed => NewAccountDialogMsg::VerifySigningKey
                        },

                        #[local_ref]
                        password_row -> adw::PasswordEntryRow {
                            set_title: "Password",

                            set_show_apply_button: false,

                            #[watch]
                            set_css_classes: if model.is_password_valid {
                                &[]
                            } else {
                                &["error"]
                            },

                            connect_changed => NewAccountDialogMsg::VerifyPassword
                        },

                        #[local_ref]
                        repeat_password_row -> adw::PasswordEntryRow {
                            set_title: "Repeat password",

                            set_show_apply_button: false,

                            #[watch]
                            set_css_classes: if model.is_password_valid {
                                &[]
                            } else {
                                &["error"]
                            },

                            connect_changed => NewAccountDialogMsg::VerifyPassword
                        }
                    },

                    adw::PreferencesGroup {
                        gtk::Button {
                            set_hexpand: false,
                            set_halign: gtk::Align::Start,

                            #[watch]
                            set_css_classes: if model.is_signing_key_valid && model.is_password_valid {
                                &["suggested-action", "pill"]
                            } else {
                                &["pill"]
                            },

                            #[watch]
                            set_sensitive: model.is_signing_key_valid && model.is_password_valid,

                            adw::ButtonContent {
                                set_icon_name: "contact-new-symbolic",
                                set_label: "Create"
                            },

                            connect_clicked => NewAccountDialogMsg::Create
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
        let mut rng = ChaCha20Rng::from_entropy();

        let mut rng = ChaCha20Rng::seed_from_u64(
            rng.next_u64() ^
            UtcDateTime::now().unix_timestamp() as u64
        );

        let signing_key = SigningKey::random(&mut rng);

        let model = Self {
            rng,

            window: root.clone(),
            name_row: adw::EntryRow::new(),
            signing_key_row: adw::EntryRow::new(),
            password_row: adw::PasswordEntryRow::new(),
            repeat_password_row: adw::PasswordEntryRow::new(),

            is_signing_key_valid: true,
            is_password_valid: true
        };

        model.signing_key_row.set_text(signing_key.to_base64().as_str());

        let name_row = &model.name_row;
        let signing_key_row = &model.signing_key_row;
        let password_row = &model.password_row;
        let repeat_password_row = &model.repeat_password_row;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>
    ) {
        match message {
            NewAccountDialogMsg::RandSigningKey => {
                let signing_key = SigningKey::random(&mut self.rng);

                self.signing_key_row.set_text(signing_key.to_base64().as_str());
            }

            NewAccountDialogMsg::VerifySigningKey => {
                let signing_key = self.signing_key_row.text();

                self.is_signing_key_valid = SigningKey::from_base64(signing_key).is_some();
            }

            NewAccountDialogMsg::VerifyPassword => {
                let password = self.password_row.text();
                let repeat_password = self.repeat_password_row.text();

                self.is_password_valid = password == repeat_password;
            }

            NewAccountDialogMsg::Create => {
                let name = self.name_row.text();
                let signing_key = self.signing_key_row.text();
                let password = self.password_row.text();

                // TODO: error handling dialog

                let Some(signing_key) = SigningKey::from_base64(signing_key) else {
                    return;
                };

                let account = Account::new(name, signing_key, password.as_bytes())
                    .expect("failed to create account");

                let _ = sender.output(account);

                self.window.close();
            }
        }
    }
}
