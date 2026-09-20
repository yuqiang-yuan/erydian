use gpui_kit::{
    InteractiveElement, IntoElement, ParentElement, Render, Styled, base::StyledExt, component::button::{Button, ButtonVariants}, div,
};

use crate::actions::NewSchemaAction;

pub struct WelcomeView {
    // pub on_new: Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
}

impl Render for WelcomeView {
    fn render(
        &mut self,
        _: &mut gpui_kit::Window,
        _: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("welcome-view")
            .size_full()
            .v_flex()
            .justify_center()
            .items_center()
            .gap_8()
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
                                println!("button clicked, focused = {:?}", window.focused(cx).map(|h| h.is_focused(window)));
                                window.dispatch_action(Box::new(NewSchemaAction), cx)
                            }),
                    )
                    .child(Button::new("open-schema-button").label("Open")),
            )
    }
}
