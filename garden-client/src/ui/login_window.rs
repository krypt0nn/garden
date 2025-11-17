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

use crate::ui::new_account_dialog::{NewAccountDialog, NewAccountDialogMsg};
use crate::ui::login_account_dialog::{LoginAccountDialog, LoginAccountDialogMsg};

#[derive(Debug, Clone)]
pub enum LoginWindowAccountFactoryMsg {
    Delete,
    Login
}

#[derive(Debug)]
struct LoginWindowAccountFactory {
    account: Account,
    index: DynamicIndex
}

#[relm4::factory]
impl FactoryComponent for LoginWindowAccountFactory {
    type Init = Account;
    type Input = LoginWindowAccountFactoryMsg;
    type Output = LoginWindowMsg;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            set_title: self.account.name(),
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
                },

                connect_clicked => LoginWindowAccountFactoryMsg::Delete
            },

            connect_activated => LoginWindowAccountFactoryMsg::Login
        }
    }

    #[inline]
    fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        _sender: FactorySender<Self>
    ) -> Self {
        Self {
            account: init,
            index: index.clone()
        }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: FactorySender<Self>
    ) {
        match msg {
            LoginWindowAccountFactoryMsg::Delete => {
                let _ = sender.output(LoginWindowMsg::DeleteAccount(self.index.current_index()));
            }

            LoginWindowAccountFactoryMsg::Login => {
                let _ = sender.output(LoginWindowMsg::LoginIntoAccount(self.index.current_index()));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoginWindowMsg {
    New,
    AddAccount(Account),
    DeleteAccount(usize),
    LoginIntoAccount(usize),
    Login(SigningKey)
}

pub struct LoginWindow {
    window: adw::ApplicationWindow,
    accounts_factory: FactoryVecDeque<LoginWindowAccountFactory>,
    new_account_dialog: Controller<NewAccountDialog>,
    login_account_dialog: Controller<LoginAccountDialog>
}

#[relm4::component(pub)]
impl SimpleComponent for LoginWindow {
    type Init = Vec<Account>;
    type Input = LoginWindowMsg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some("Garden"),

            set_size_request: (800, 600),
            set_hide_on_close: false,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                adw::StatusPage {
                    set_title: "Create account",
                    set_description: Some("You don't have any account yet, so you need to create one"),

                    set_icon_name: Some("com.github.krypt0nn.garden"),

                    set_vexpand: true,
                    set_valign: gtk::Align::Center,

                    #[watch]
                    set_visible: model.accounts_factory.is_empty(),

                    gtk::Button {
                        set_hexpand: false,
                        set_halign: gtk::Align::Center,

                        add_css_class: "suggested-action",
                        add_css_class: "pill",

                        adw::ButtonContent {
                            set_icon_name: "contact-new-symbolic",
                            set_label: "New account"
                        },

                        connect_clicked => LoginWindowMsg::New
                    }
                },

                adw::PreferencesPage {
                    #[watch]
                    set_visible: !model.accounts_factory.is_empty(),

                    #[local_ref]
                    accounts_factory -> adw::PreferencesGroup {
                        set_title: "Accounts",

                        #[wrap(Some)]
                        set_header_suffix = &gtk::Button {
                            add_css_class: "flat",

                            adw::ButtonContent {
                                set_icon_name: "contact-new-symbolic",
                                set_label: "New"
                            },

                            connect_clicked => LoginWindowMsg::New
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>
    ) -> ComponentParts<Self> {
        let mut model = Self {
            window: root.clone(),

            accounts_factory: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), std::convert::identity),

            new_account_dialog: NewAccountDialog::builder()
                .launch(())
                .forward(sender.input_sender(), LoginWindowMsg::AddAccount),

            login_account_dialog: LoginAccountDialog::builder()
                .launch(())
                .forward(sender.input_sender(), LoginWindowMsg::Login)
        };

        for account in init {
            model.accounts_factory.guard().push_back(account);
        }

        let accounts_factory = model.accounts_factory.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>
    ) {
        match message {
            LoginWindowMsg::New => {
                self.new_account_dialog.emit(NewAccountDialogMsg::Reset);

                self.new_account_dialog.widget()
                    .present(Some(&self.window));
            }

            LoginWindowMsg::AddAccount(account) => {
                let mut guard = self.accounts_factory.guard();

                guard.push_back(account);

                let accounts = guard.iter()
                    .map(|component| &component.account);

                // TODO: error handling dialog
                crate::accounts::write(accounts)
                    .expect("failed to update accounts file");
            }

            LoginWindowMsg::DeleteAccount(index) => {
                let mut guard = self.accounts_factory.guard();

                guard.remove(index);

                let accounts = guard.iter()
                    .map(|component| &component.account);

                // TODO: error handling dialog
                crate::accounts::write(accounts)
                    .expect("failed to update accounts file");
            }

            LoginWindowMsg::LoginIntoAccount(index) => {
                let account = self.accounts_factory.guard()
                    .get(index)
                    .map(|component| component.account.clone());

                if let Some(account) = account {
                    // Try to login into account using empty password.
                    match account.signing_key(b"") {
                        // On success - just login into it.
                        Ok(signing_key) => {
                            sender.input(LoginWindowMsg::Login(signing_key));
                        }

                        // Otherwise open password entry dialog.
                        Err(_) => {
                            self.login_account_dialog.emit(
                                LoginAccountDialogMsg::SetAccount(account)
                            );

                            self.login_account_dialog.widget()
                                .present(Some(&self.window));
                        }
                    }
                }
            }

            LoginWindowMsg::Login(signing_key) => {
                println!("{}", signing_key.to_base64());
            }
        }
    }
}
