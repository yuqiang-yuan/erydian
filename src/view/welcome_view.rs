use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, base::StyledExt, component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
    }, div, prelude::FluentBuilder,
};

use crate::{actions::{FileOpenedAction, NewSchemaAction}, settings::RecentFiles};

pub struct WelcomeView {
    recent_files: RecentFiles,
}

impl WelcomeView {
    pub fn new() -> Self {
        Self {
            recent_files: RecentFiles::load(),
        }
    }

    fn render_recent_files(&self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().rounded_md().bg(cx.theme().secondary).py_2().children(
            self.recent_files.files.iter().map(|f| {
                let path_str = f.path.as_os_str().to_string_lossy().into_owned();
                let path_clone = f.path.clone();
                div()
                    .id(path_str.clone())
                    .px_2()
                    .hover(|e| e.bg(cx.theme().list_hover))
                    .child(f.name.clone())
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .text_ellipsis()
                            .min_w_0()
                            .mt_neg_1()
                            .child(path_str.clone())
                    )
                    .on_click(cx.listener(move |_, _, window, cx| {
                        let cloned = path_clone.clone();
                        window.dispatch_action(Box::new(FileOpenedAction {
                            path: cloned,
                        }), cx);
                    }))
            }),
        )
    }
}

impl Render for WelcomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("welcome-view")
            .size_full()
            .v_flex()
            .pt_8()
            .items_center()
            .gap_4()
            .child(div().text_center().text_2xl().child("Welcom to Erydian"))
            .child(
                div()
                    .h_flex()
                    .justify_center()
                    .gap_4()
                    .child(
                        Button::new("new-schema-button")
                            .primary()
                            .label("New")
                            .on_click(|_, window, cx| {
                                println!(
                                    "button clicked, focused = {:?}",
                                    window.focused(cx).map(|h| h.is_focused(window))
                                );
                                window.dispatch_action(Box::new(NewSchemaAction), cx)
                            }),
                    )
                    .child(Button::new("open-schema-button").label("Open")),
            )
            .child(div().when(self.recent_files.files.len() > 0, |_| {
                div()
                    .w_112()
                    .mt_8()
                    .child(div().px_2().mb_2().child("Recent files"))
                    .child(self.render_recent_files(window, cx))
            }))
    }
}
