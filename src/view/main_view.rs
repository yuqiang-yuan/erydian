use std::collections::BTreeMap;

use gpui_fps::fps_monitor;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::notification::NotificationType;
use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::actions::{NewSchemaAction, SchemaCreatedAction};
use crate::dialect::DialectKind;
use crate::model::SchemaDocument;
use crate::settings::AppSettings;
use crate::view::{EditorView, NewSchemaView, WelcomeView};

enum Scene {
    Welcome,
    NewSchema,
    Editor,
}

pub struct MainView {
    focus_handle: FocusHandle,
    schema: Option<SchemaDocument>,
    scene: Scene,
    welcome_view: Entity<WelcomeView>,
    new_schema_view: Entity<NewSchemaView>,
    editor_view: Entity<EditorView>,
    _subscriptions: Vec<Subscription>,
    show_fps: bool,
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        let mut subscriptions = vec![];

        // dark/light subscrition
        subscriptions.push(cx.observe_global::<Theme>(|_, cx| {
            let is_dark = cx.theme().is_dark();
            let font_size = cx.theme().font_size;
            let settings = cx.global_mut::<AppSettings>();
            settings.is_dark = Some(is_dark);
            settings.font_size = Some(font_size.as_f32());
        }));

        // window resized
        subscriptions.push(cx.observe_window_bounds(window, |_, window, cx| {
            let bounds = window.bounds();
            let is_maximum = window.is_maximized();
            let settings = cx.global_mut::<AppSettings>();
            settings.window_width = Some(bounds.size.width.as_f32());
            settings.window_height = Some(bounds.size.height.as_f32());
            settings.window_maximized = Some(is_maximum);
        }));

        Self {
            focus_handle,
            schema: None,
            scene: Scene::Editor,
            welcome_view: cx.new(|_| WelcomeView {}),
            new_schema_view: cx.new(|cx| NewSchemaView::new(DialectKind::MySql, window, cx)),
            editor_view: cx.new(|cx| {
                EditorView::new(
                    SchemaDocument::new(DialectKind::MySql, "MyTest And a very long name".into(), BTreeMap::new()),
                    window,
                    cx,
                )
            }),
            _subscriptions: subscriptions,
            show_fps: false,
        }
    }

    fn on_new_schema_action(
        &mut self,
        _: &NewSchemaAction,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        println!("new scheme action received");
        self.scene = Scene::NewSchema;
        cx.notify();
    }

    fn on_schema_created_action(
        &mut self,
        action: &SchemaCreatedAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Ok(schema) = serde_json::from_str::<SchemaDocument>(&action.schema_json) {
            self.editor_view = cx.new(|cx| EditorView::new(schema, window, cx));
            self.scene = Scene::Editor;

            cx.notify();
        } else {
            window.push_notification((NotificationType::Error, "Parse schema data failed"), cx);
        }
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .id("main-view")
            .relative()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_new_schema_action))
            .on_action(cx.listener(Self::on_schema_created_action))
            .size_full()
            .v_flex()
            .child(
                TitleBar::new().child(
                    div()
                        .size_full()
                        .h_flex()
                        .child(div().flex_grow_1())
                        .child(font_size_button())
                        .child(theme_button(cx)),
                ),
            )
            .child(match self.scene {
                Scene::Welcome => div().size_full().child(self.welcome_view.clone()),
                Scene::NewSchema => div().size_full().child(self.new_schema_view.clone()),
                Scene::Editor => div().size_full().p_1().child(self.editor_view.clone()),
            })
            .when(matches!(self.scene, Scene::Editor), |this| {
                this.child(StatusBar::new().left("Ready"))
            })
            .when(self.show_fps, |this| this.child(fps_monitor(window, cx)))
            .children(dialog_layer)
            .children(notification_layer)
    }
}

/// Add a button to switch theme (dark/light)
fn theme_button(cx: &mut App) -> impl IntoElement {
    Button::new("theme-button")
        .ghost()
        .rounded_none()
        .icon(if cx.theme().is_dark() {
            IconName::Moon
        } else {
            IconName::Sun
        })
        .tooltip(if cx.theme().is_dark() {
            "Swith to light"
        } else {
            "Switch to dark"
        })
        .tooltip_placement(Placement::Bottom)
        .on_click(|_, window, cx| {
            let target_mode = if cx.theme().is_dark() {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            };

            Theme::change(target_mode, Some(window), cx);
            cx.refresh_windows();
        })
}

/// Add a font size button
fn font_size_button() -> impl IntoElement {
    Button::new("font-size-button")
        .ghost()
        .rounded_none()
        .icon(IconName::ALargeSmall)
        .tooltip("Change font size")
        .tooltip_placement(Placement::Bottom)
        .dropdown_menu(|menu, _, cx| {
            let current_font_size = cx.theme().font_size.as_f32().round() as i32;

            menu.item(
                PopupMenuItem::new("Small")
                    .checked(current_font_size == 14)
                    .on_click(|_, _, cx| {
                        Theme::global_mut(cx).font_size = px(14.0);
                        Theme::sync_base(cx);
                        cx.refresh_windows();
                    }),
            )
            .item(
                PopupMenuItem::new("Regular")
                    .checked(current_font_size == 16)
                    .on_click(|_, _, cx| {
                        Theme::global_mut(cx).font_size = px(16.0);
                        Theme::sync_base(cx);
                        cx.refresh_windows();
                    }),
            )
            .item(
                PopupMenuItem::new("Large")
                    .checked(current_font_size == 18)
                    .on_click(|_, _, cx| {
                        Theme::global_mut(cx).font_size = px(18.0);
                        Theme::sync_base(cx);
                        cx.refresh_windows();
                    }),
            )
        })
}
