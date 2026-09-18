use erydian::{assets::AppAssets, globals::APP_ID, settings::AppSettings, view::MainView};
#[cfg(target_os = "linux")]
use gpui_kit::WindowDecorations;
use gpui_kit::{
    AppContext, WindowBounds, WindowKind, WindowOptions,
    component::{Root, Theme, ThemeMode, TitleBar},
    px, size,
};

fn main() {
    let app = gpui_kit::application().with_assets(AppAssets);
    app.run(|cx| {
        gpui_kit::init(cx);
        cx.set_app_identity(APP_ID, "Erydian");

        let settings = AppSettings::load();

        if let Some(true) = settings.is_dark {
            Theme::change(ThemeMode::Dark, None, cx);
        } else {
            Theme::change(ThemeMode::Light, None, cx);
        }

        if let Some(font_size) = settings.font_size {
            Theme::global_mut(cx).font_size = px(font_size);
        }

        cx.set_global(settings);

        cx.on_app_quit(|cx| {
            let snapshot = cx.global::<AppSettings>();
            println!("saving settings before quit");
            snapshot.save();
            async {}
        }).detach();

        cx.spawn(async move |cx| {
            cx.update(move |cx| {
                let is_max = settings.window_maximized.unwrap_or(false);

                // TODO: Test if the window bounds over the screen's bounds.
                let bounds = WindowBounds::centered(size(
                    px(settings.window_width.unwrap_or(1200.0)),
                    px(settings.window_height.unwrap_or(800.0)),
                ), cx);

                let options = WindowOptions {
                    window_bounds: if is_max {
                        Some(WindowBounds::Maximized(bounds.get_bounds()))
                    } else {
                        Some(bounds)
                    },
                    kind: WindowKind::Normal,

                    #[cfg(target_os = "linux")]
                    window_decorations: Some(WindowDecorations::Client),
                    ..TitleBar::window_options()
                };

                cx.open_window(options, |window, cx| {
                    let main_view = cx.new(|cx| MainView::new(window, cx));
                    cx.new(|cx| Root::new(main_view, window, cx))
                })
                .expect("Launch application failed");

                cx.activate(true);
            });
        })
        .detach();
    });
}
