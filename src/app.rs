/*
 *
 *  This source file is part of the Loungy open source project
 *
 *  Copyright (c) 2024 Loungy, Matthias Grandl and the Loungy project contributors
 *  Licensed under MIT License
 *
 *  See https://github.com/MatthiasGrandl/Loungy/blob/main/LICENSE.md for license information
 *
 */

use gpui::*;

use crate::{
    assets::Assets,
    commands::RootCommands,
    hotkey::HotkeyManager,
    theme::Theme,
    window::{Frontmost, Window, WindowStyle},
    workspace::Workspace,
};

pub fn run_app(app: gpui::App) {
    app.with_assets(Assets).run(move |cx: &mut AppContext| {
        Theme::init(cx);
        // TODO: This still only works for a single display
        let bounds = cx.displays().first().map(|d| d.bounds()).unwrap_or(Bounds {
            origin: Point::new(GlobalPixels::from(0.0), GlobalPixels::from(0.0)),
            size: Size {
                width: GlobalPixels::from(1920.0),
                height: GlobalPixels::from(1080.0),
            },
        });
        cx.open_window(WindowStyle::Main.options(bounds), |cx| {
            Frontmost::init(cx);
            RootCommands::init(cx);
            HotkeyManager::init(cx);
            let view = Workspace::build(cx);
            Window::init(cx);

            view
        });
    });
}
