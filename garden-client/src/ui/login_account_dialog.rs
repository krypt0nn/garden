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

use crate::accounts::Account;

#[derive(Debug, Clone)]
pub enum LoginAccountDialogMsg {
    SetAccount(Account),
    VerifyPassword,
    Login
}

pub struct LoginAccountDialog {
    account: Option<Account>,

    window: adw::Dialog,
    name_row: adw::ActionRow,
    password_row: adw::PasswordEntryRow,

    is_password_valid: bool
}

#[relm4::component(pub)]
impl SimpleComponent for LoginAccountDialog {
    type Init = ();
    type Input = LoginAccountDialogMsg;
    type Output = SigningKey;

    view! {
        adw::Dialog {
            set_title: "Login",

            set_size_request: (600, 340),

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                adw::PreferencesPage {
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,

                    adw::PreferencesGroup {
                        #[local_ref]
                        name_row -> adw::ActionRow {
                            #[watch]
                            set_title?: model.account.as_ref()
                                .map(|account| account.name()),

                            #[watch]
                            set_subtitle?: model.account.as_ref()
                                .map(|account| {
                                    format!("Created at {}", account.created_at())
                                })
                                .as_deref(),

                            add_prefix = &gtk::Image {
                                set_icon_name: Some("person-symbolic")
                            }
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

                            connect_changed => LoginAccountDialogMsg::VerifyPassword
                        }
                    },

                    adw::PreferencesGroup {
                        gtk::Button {
                            set_hexpand: false,
                            set_halign: gtk::Align::Start,

                            #[watch]
                            set_css_classes: if model.is_password_valid {
                                &["suggested-action", "pill"]
                            } else {
                                &["pill"]
                            },

                            #[watch]
                            set_sensitive: model.is_password_valid,

                            adw::ButtonContent {
                                set_icon_name: "contact-new-symbolic",
                                set_label: "Login"
                            },

                            connect_clicked => LoginAccountDialogMsg::Login
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
            account: None,

            window: root.clone(),
            name_row: adw::ActionRow::new(),
            password_row: adw::PasswordEntryRow::new(),

            is_password_valid: false
        };

        let name_row = &model.name_row;
        let password_row = &model.password_row;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>
    ) {
        match message {
            LoginAccountDialogMsg::SetAccount(account) => {
                self.account = Some(account);
                self.password_row.set_text("");

                sender.input(LoginAccountDialogMsg::VerifyPassword);
            }

            LoginAccountDialogMsg::VerifyPassword => {
                if let Some(account) = &self.account {
                    let password = self.password_row.text();

                    self.is_password_valid = account.signing_key(password.as_bytes()).is_ok();
                }
            }

            LoginAccountDialogMsg::Login => {
                if let Some(account) = &self.account {
                    let password = self.password_row.text();

                    // TODO: error handling dialog

                    let Ok(signing_key) = account.signing_key(password.as_bytes()) else {
                        return;
                    };

                    let _ = sender.output(signing_key);

                    self.window.close();
                }
            }
        }
    }
}
