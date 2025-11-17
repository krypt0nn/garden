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

#[derive(Debug, Clone)]
pub enum MainWindowMsg {

}

pub struct MainWindow {
    signing_key: SigningKey
}

#[relm4::component(pub)]
impl SimpleComponent for MainWindow {
    type Init = SigningKey;
    type Input = MainWindowMsg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some("Garden"),

            set_size_request: (1000, 700),
            set_hide_on_close: false,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,

                    set_margin_top: 16,
                    set_margin_bottom: 16,

                    adw::Clamp {
                        gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,

                            add_css_class: "boxed-list",

                            adw::Bin {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    set_margin_all: 8,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::Start,

                                            set_label: "@amogus"
                                        },

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::End,

                                            set_label: "Nov 17, 00:59"
                                        },
                                    },

                                    gtk::Label {
                                        set_hexpand: true,
                                        set_halign: gtk::Align::Start,
                                        set_justify: gtk::Justification::Fill,

                                        set_wrap: true,

                                        set_label: "Hello, World!"
                                    },
                                }
                            },

                            adw::Bin {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    set_margin_all: 8,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::Start,

                                            set_label: "@amogus"
                                        },

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::End,

                                            set_label: "Nov 17, 00:59"
                                        },
                                    },

                                    gtk::Label {
                                        set_hexpand: true,
                                        set_halign: gtk::Align::Start,
                                        set_justify: gtk::Justification::Fill,

                                        set_wrap: true,

                                        set_label: "Lorem ipsum dolor sit amet consectetur adipiscing elit. Placerat in id cursus mi pretium tellus duis. Urna tempor pulvinar vivamus fringilla lacus nec metus. Integer nunc posuere ut hendrerit semper vel class. Conubia nostra inceptos himenaeos orci varius natoque penatibus. Mus donec rhoncus eros lobortis nulla molestie mattis. Purus est efficitur laoreet mauris pharetra vestibulum fusce. Sodales consequat magna ante condimentum neque at luctus. Ligula congue sollicitudin erat viverra ac tincidunt nam. Lectus commodo augue arcu dignissim velit aliquam imperdiet. Cras eleifend turpis fames primis vulputate ornare sagittis. Libero feugiat tristique accumsan maecenas potenti ultricies habitant. Cubilia curae hac habitasse platea dictumst lorem ipsum. Faucibus ex sapien vitae pellentesque sem placerat in. Tempus leo eu aenean sed diam urna tempor."
                                    },
                                }
                            },

                            adw::Bin {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    set_margin_all: 8,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::Start,

                                            set_label: "@amogus"
                                        },

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::End,

                                            set_label: "Nov 17, 00:59"
                                        },
                                    },

                                    gtk::Label {
                                        set_hexpand: true,
                                        set_halign: gtk::Align::Start,
                                        set_justify: gtk::Justification::Fill,

                                        set_wrap: true,

                                        set_label: "Lorem ipsum dolor sit amet consectetur adipiscing elit. Urna tempor pulvinar vivamus fringilla lacus nec metus. Conubia nostra inceptos himenaeos orci varius natoque penatibus. Purus est efficitur laoreet mauris pharetra vestibulum fusce. Ligula congue sollicitudin erat viverra ac tincidunt nam. Cras eleifend turpis fames primis vulputate ornare sagittis. Cubilia curae hac habitasse platea dictumst lorem ipsum. Tempus leo eu aenean sed diam urna tempor. Taciti sociosqu ad litora torquent per conubia nostra. Maximus eget fermentum odio phasellus non purus est. Finibus facilisis dapibus etiam interdum tortor ligula congue. Nullam volutpat porttitor ullamcorper rutrum gravida cras eleifend. Senectus netus suscipit auctor curabitur facilisi cubilia curae. Cursus mi pretium tellus duis convallis tempus leo. Ut hendrerit semper vel class aptent taciti sociosqu. Eros lobortis nulla molestie mattis scelerisque maximus eget. Ante condimentum neque at luctus nibh finibus facilisis. Arcu dignissim velit aliquam imperdiet mollis nullam volutpat. Accumsan maecenas potenti ultricies habitant morbi senectus netus. Vitae pellentesque sem placerat in id cursus mi. Nisl malesuada lacinia integer nunc posuere ut hendrerit. Montes nascetur ridiculus mus donec rhoncus eros lobortis. Suspendisse aliquet nisi sodales consequat magna ante condimentum. Euismod quam justo lectus commodo augue arcu dignissim. Venenatis ultrices proin libero feugiat tristique accumsan maecenas. Adipiscing elit quisque faucibus ex sapien vitae pellentesque. Nec metus bibendum egestas iaculis massa nisl malesuada. Natoque penatibus et magnis dis parturient montes nascetur. Vestibulum fusce dictum risus blandit quis suspendisse aliquet. Tincidunt nam porta elementum a enim euismod quam."
                                    },
                                }
                            },

                            adw::Bin {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    set_margin_all: 8,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::Start,

                                            set_label: "@amogus"
                                        },

                                        gtk::Label {
                                            set_hexpand: true,
                                            set_halign: gtk::Align::End,

                                            set_label: "Nov 17, 00:59"
                                        },
                                    },

                                    gtk::Label {
                                        set_hexpand: true,
                                        set_halign: gtk::Align::Start,
                                        set_justify: gtk::Justification::Fill,

                                        set_wrap: true,

                                        set_label: "Lorem ipsum dolor sit amet consectetur adipiscing elit. Urna tempor pulvinar vivamus fringilla lacus nec metus. Conubia nostra inceptos himenaeos orci varius natoque penatibus. Purus est efficitur laoreet mauris pharetra vestibulum fusce. Ligula congue sollicitudin erat viverra ac tincidunt nam. Cras eleifend turpis fames primis vulputate ornare sagittis. Cubilia curae hac habitasse platea dictumst lorem ipsum. Tempus leo eu aenean sed diam urna tempor. Taciti sociosqu ad litora torquent per conubia nostra. Maximus eget fermentum odio phasellus non purus est. Finibus facilisis dapibus etiam interdum tortor ligula congue. Nullam volutpat porttitor ullamcorper rutrum gravida cras eleifend. Senectus netus suscipit auctor curabitur facilisi cubilia curae. Cursus mi pretium tellus duis convallis tempus leo. Ut hendrerit semper vel class aptent taciti sociosqu. Eros lobortis nulla molestie mattis scelerisque maximus eget. Ante condimentum neque at luctus nibh finibus facilisis. Arcu dignissim velit aliquam imperdiet mollis nullam volutpat. Accumsan maecenas potenti ultricies habitant morbi senectus netus. Vitae pellentesque sem placerat in id cursus mi. Nisl malesuada lacinia integer nunc posuere ut hendrerit. Montes nascetur ridiculus mus donec rhoncus eros lobortis. Suspendisse aliquet nisi sodales consequat magna ante condimentum. Euismod quam justo lectus commodo augue arcu dignissim. Venenatis ultrices proin libero feugiat tristique accumsan maecenas. Adipiscing elit quisque faucibus ex sapien vitae pellentesque. Nec metus bibendum egestas iaculis massa nisl malesuada. Natoque penatibus et magnis dis parturient montes nascetur. Vestibulum fusce dictum risus blandit quis suspendisse aliquet. Tincidunt nam porta elementum a enim euismod quam."
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>
    ) -> ComponentParts<Self> {
        let model = Self {
            signing_key: init
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
