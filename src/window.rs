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

use std::time::Duration;

use async_std::task::sleep;
use gpui::*;

use crate::{
    platform::{get_frontmost_application_data, AppData},
    state::StateModel,
};

pub static WIDTH: f64 = 800.0;
pub static HEIGHT: f64 = 450.0;

pub enum WindowStyle {
    Main,
    Toast { width: f64, height: f64 },
    Settings,
}

impl WindowStyle {
    pub fn options(&self, bounds: Bounds<GlobalPixels>) -> WindowOptions {
        let mut options = WindowOptions::default();
        let center = bounds.center();

        let (width, height, x, y) = match self {
            WindowStyle::Main => {
                options.focus = true;
                let width = GlobalPixels::from(WIDTH);
                let height = GlobalPixels::from(HEIGHT);
                let x: GlobalPixels = center.x - width / 2.0;
                let y: GlobalPixels = center.y - height / 2.0;
                (width, height, x, y)
            }
            WindowStyle::Toast { width, height } => {
                options.focus = false;
                let width = GlobalPixels::from(*width);
                let height = GlobalPixels::from(*height);
                let x: GlobalPixels = center.x - width / 2.0;
                let y: GlobalPixels = bounds.bottom() - height - GlobalPixels::from(200.0);
                (width, height, x, y)
            }
            WindowStyle::Settings => {
                return options;
            }
        };
        options.bounds = Some(Bounds::new(Point { x, y }, Size { width, height }));
        options.titlebar = None;
        options.is_movable = false;
        options.kind = WindowKind::PopUp;
        options
    }
}

pub struct Window {
    //inner: View<Workspace>,
    hidden: bool,
}

#[allow(dead_code)]
impl Window {
    pub fn init(cx: &mut AppContext) {
        cx.set_global::<Self>(Self {
            //inner: view.clone(),
            hidden: false,
        });
    }
    pub fn is_open(cx: &AsyncAppContext) -> bool {
        cx.read_global::<Self, _>(|w, _| !w.hidden).unwrap_or(false)
    }
    pub fn open(cx: &mut WindowContext) {
        cx.update_global::<Self, _>(|this, cx| {
            if this.hidden {
                cx.activate_window();
                this.hidden = false;
            }
        });
    }
    pub fn close(cx: &mut WindowContext) {
        cx.update_global::<Self, _>(|this, cx| {
            this.hidden = true;
            cx.hide();
        });
        // After 90 seconds, reset the state
        cx.spawn(|mut cx| async move {
            sleep(Duration::from_secs(90)).await;
            // cx.background_executor()
            //     .timer(Duration::from_secs(90))
            //     .await;
            let _ = cx.update_global::<Self, _>(|window, cx| {
                if window.hidden {
                    StateModel::update(|this, cx| this.reset(cx), cx);
                }
            });
        })
        .detach();
    }
    pub async fn wait_for_close(cx: &mut AsyncWindowContext) {
        while let Ok(active) =
            cx.update_window::<bool, _>(cx.window_handle(), |_, cx| cx.is_window_active())
        {
            if !active {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    }
}

impl Global for Window {}

pub struct Frontmost {
    inner: Model<Option<AppData>>,
}

#[allow(dead_code)]
impl Frontmost {
    pub fn init(cx: &mut AppContext) {
        let model = cx.new_model(|cx| {
            cx.spawn(|this, mut cx| async move {
                loop {
                    let result = this.update(&mut cx, |this: &mut Option<AppData>, cx| {
                        if let Some(app) = get_frontmost_application_data() {
                            if !app
                                .id
                                .eq(this.as_ref().map(|a| &a.id).unwrap_or(&"".to_string()))
                            {
                                *this = Some(app);
                                cx.notify();
                            }
                        };
                    });
                    if result.is_err() {
                        break;
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            })
            .detach();
            get_frontmost_application_data()
        });
        cx.set_global::<Self>(Self { inner: model });
    }
    pub fn get_async(cx: &AsyncAppContext) -> Option<AppData> {
        cx.read_global::<Self, Option<AppData>>(|this, cx| {
            cx.read_model(&this.inner, |this, _| this.clone())
        })
        .unwrap_or(None)
    }
    pub fn get(cx: &AppContext) -> Option<AppData> {
        let model = cx.global::<Self>();
        cx.read_model(&model.inner, |this, _| this.clone())
    }
    pub fn inner(cx: &AppContext) -> Model<Option<AppData>> {
        cx.global::<Self>().inner.clone()
    }
}

impl Global for Frontmost {}
