use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use gpui_fps::fps_monitor;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{AppMenuBar, DropdownMenu, PopupMenuItem};
use gpui_kit::component::notification::NotificationType;
use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::actions::{AboutAction, FileOpenedAction, FileSavedAction, NewRelationshipAction, NewSchemaAction, NewTableAction, OpenFileAction, QuitAction, SaveFileAction, SchemaCreatedAction};
use crate::dialect::DialectKind;
use crate::model::{SchemaDocument, TableSpec};
use crate::settings::{AppSettings, RecentFiles};
use crate::view::{EditorView, NewSchemaView, WelcomeView};

#[derive(Debug, PartialEq, Eq)]
enum Scene {
    Welcome,
    NewSchema,
    Editor,
}

pub struct MainView {
    focus_handle: FocusHandle,
    menubar: Entity<AppMenuBar>,
    schema: Option<Entity<SchemaDocument>>,
    scene: Scene,
    welcome_view: Entity<WelcomeView>,
    new_schema_view: Entity<NewSchemaView>,
    editor_view: Entity<EditorView>,
    _subscriptions: Vec<Subscription>,
    show_fps: bool,

    // last used path for file dialog
    last_path: Option<PathBuf>,
    // the current opened file's path
    file_path: Option<PathBuf>,
}

fn gen_test_schema(cx: &mut Context<MainView>) -> Entity<SchemaDocument> {
    cx.new(|_| {
        let mut doc = SchemaDocument::new(
            DialectKind::MySql,
            "MyTest And a very long name",
            BTreeMap::new(),
        );

        doc.tables.extend(
            vec![
                TableSpec::new("users"),
                TableSpec::new("posts"),
                TableSpec::new("comments"),
                TableSpec::new("orders"),
            ]
        );

        doc
    })
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        #[cfg(target_os = "macos")]
        {
            cx.set_menus(build_menus());
        }

        #[cfg(not(target_os = "macos"))]
        {
            let menus = build_menus().into_iter().map(|menu| menu.owned()).collect();
            GlobalState::global_mut(cx).set_app_menus(menus);
        }

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

        let temp_schema = cx.new(|_| SchemaDocument::new(DialectKind::MySql, "temp", BTreeMap::new()));

        let focus_handle_clone = focus_handle.clone();
        window.defer(cx, move |window, cx| {
            if window.focused(cx).is_none() {
                focus_handle_clone.focus(window, cx);
            }
        });

        Self {
            focus_handle,
            menubar: AppMenuBar::new(cx),
            schema: Some(temp_schema.clone()),
            scene: Scene::Welcome,
            welcome_view: cx.new(|_| WelcomeView::new()),
            new_schema_view: cx.new(|cx| NewSchemaView::new(DialectKind::MySql, window, cx)),
            editor_view: cx.new(|cx| {
                EditorView::new(
                    temp_schema.clone(),
                    window,
                    cx,
                )
            }),
            _subscriptions: subscriptions,
            show_fps: false,
            last_path: None,
            file_path: None,
        }
    }

    fn on_new_schema_action(
        &mut self,
        _: &NewSchemaAction,
        window: &mut Window,
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
            let ent_schema = cx.new(|_| schema);
            let ent_schema_clone = ent_schema.clone();

            self.schema = Some(ent_schema);
            self.editor_view = cx.new(|cx| EditorView::new(ent_schema_clone, window, cx));
            self.scene = Scene::Editor;

            cx.notify();

            self.focus_handle.focus(window, cx);
        } else {
            window.push_notification((NotificationType::Error, "Parse schema data failed"), cx);
        }
    }

    fn on_table_added_action(
        &mut self,
        action: &NewTableAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(doc) = &self.schema {
            doc.update(cx, |this, cx| {
                this.tables.push(TableSpec::new("table"));
                cx.notify();
            })
        }
    }

    fn on_save_file_action(
        &mut self,
        _: &SaveFileAction,
        window: &mut Window,
        cx: &mut Context<Self>
    ) {
        println!("begin to prompt for saving file");
        if self.schema.is_none() {
            return;
        }

        if self.file_path.is_some() {
            self.save(self.file_path.as_ref().unwrap().clone(), window, cx);
        } else {
            let home_path = dirs::home_dir().unwrap();
            let path = cx.prompt_for_new_path(
                self.last_path.as_ref().map(|p| p.as_path()).unwrap_or(home_path.as_path()),
                Some("Untitled.erj")
            );

            cx.spawn_in(window, async move |this, cx| {
                if let Ok(Ok(Some(file))) = path.await {
                    this.update_in(cx, |this, window, cx| {
                        this.save(file, window, cx);
                    }).ok();
                }
            }).detach();
        }
    }

    fn save(&mut self, file_path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(doc) = &self.schema
            && let Ok(s) = serde_json::to_string(doc.read(cx))
            && let Ok(_) = fs::write(&file_path, s)
        {
            self.file_path = Some(file_path.clone());
            self.last_path = file_path.parent().map(|p| Some(p.to_path_buf())).unwrap_or(None);

            // save recent files
            let mut recent_files = RecentFiles::load();
            recent_files.add(&doc.read(cx).name, &file_path);
            recent_files.save();

            window.push_notification(
                (NotificationType::Success, format!("File saved to: {}", file_path.display())),
                cx
            );
        }
    }

    fn on_file_opened_action(&mut self, action: &FileOpenedAction, window: &mut Window, cx: &mut Context<Self>) {
        if action.path.exists()
            && let Ok(s) = fs::read_to_string(&action.path)
            && let Ok(doc) = serde_json::from_str::<SchemaDocument>(&s)
        {
            self.file_path = Some(action.path.clone());
            self.schema = Some(cx.new(|_| doc));
            self.scene = Scene::Editor;
            cx.notify();
        }
        else
        {
            window.push_notification(
                (NotificationType::Error, "Open file failed"),
                cx
            );
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
            .size_full()
            .v_flex()
            .on_action(cx.listener(Self::on_new_schema_action))
            .on_action(cx.listener(Self::on_schema_created_action))
            .on_action(cx.listener(Self::on_table_added_action))
            .on_action(cx.listener(Self::on_save_file_action))
            .on_action(cx.listener(Self::on_file_opened_action))
            .child(
                TitleBar::new().child(
                    div()
                        .size_full()
                        .h_flex()
                        .child(self.menubar.clone())
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

/// Build the application menu
fn build_menus() -> Vec<Menu> {
    vec![
        #[cfg(target_os = "macos")]
        {
            Menu {
                name: "Erydian".into(),
                items: vec![
                    MenuItem::action("About", AboutAction),
                    MenuItem::separator(),
                    MenuItem::action("Quit", QuitAction),
                ],
                disabled: false,
            }
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New", NewSchemaAction),
                MenuItem::action("Open", OpenFileAction),
                MenuItem::action("Save", SaveFileAction),
                MenuItem::separator(),
                MenuItem::action("Quit", QuitAction),
            ],
            disabled: false,
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::separator(),
                MenuItem::action("New Table", NewTableAction),
                MenuItem::action("New Relationship", NewRelationshipAction),
            ],
            disabled: false,
        },
        Menu {
            name: "Help".into(),

            items: vec![MenuItem::action("About", AboutAction)],
            disabled: false,
        },
    ]
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
