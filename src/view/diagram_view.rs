use gpui_kit::{
    BorderStyle, Bounds, Context, Corners, Edges, Entity, Font, InteractiveElement, IntoElement, PaintQuad, ParentElement, Pixels, Point, Render, Styled, TextAlign, TextRun, Window, canvas, component::ActiveTheme, div, hsla, point, px, size,
};

use crate::model::SchemaDocument;

pub struct DiagramView {
    schema: Entity<SchemaDocument>,
    scale: f32,
    translate_point: Point<Pixels>,
    canvas_origin: Point<Pixels>,

    // for dragging
    last_mouse: Point<Pixels>,
}

impl DiagramView {
    pub fn new(
        schema: Entity<SchemaDocument>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            schema,
            scale: 1.0,
            translate_point: point(px(0.0), px(0.0)),
            canvas_origin: point(px(0.0), px(0.0)),
            last_mouse: point(px(0.0), px(0.0)),
        }
    }
}

impl Render for DiagramView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trans_point = self.translate_point;
        let scale = self.scale;
        let view = cx.entity().clone();
        let schema = self.schema.clone();

        div()
            .id("diagram-canvas-box")
            .size_full()
            .rounded_md()
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, cx| {
                        let _ = view.update(cx, |this, _| this.canvas_origin = bounds.origin);
                        let origin = bounds.origin;

                        let font_size = cx.theme().font_size;
                        let font = Font {
                            family: cx.theme().mono_font_family.clone(),
                            ..Default::default()
                        };
                        let bg_color = cx.theme().background;

                        let tables = schema
                                .read(cx)
                                .tables
                                .iter()
                                .map(|t| t.clone())
                                .collect::<Vec<_>>();

                        for (i, table) in tables.iter().enumerate() {
                            let top_left = point(
                                origin.x + trans_point.x + px(10.0 * scale) + px(i as f32 * 100.0),
                                origin.y + trans_point.y + px(10.0 * scale),
                            );

                            let runs = vec![TextRun {
                                len: table.name.len(),
                                font: font.clone(),
                                ..Default::default()
                            }];

                            let shaped = window.text_system().shape_line(
                                table.name.clone().into(),
                                font_size,
                                &runs,
                                None,
                            );

                            let w = px(shaped.width.as_f32());
                            let sz = size(w, px(100.0 * scale));

                            window.paint_quad(PaintQuad {
                                bounds: Bounds {
                                    origin: top_left,
                                    size: sz,
                                },
                                corner_radii: Corners::default(),
                                background: cx.theme().foreground.into(),
                                border_widths: Edges {
                                    left: px(2.),
                                    top: px(2.),
                                    right: px(2.),
                                    bottom: px(2.),
                                },
                                border_color: hsla(0., 0., 0.2, 1.),
                                border_style: BorderStyle::default(),
                            });

                            let _ = shaped.paint(top_left, font_size, TextAlign::Left, None, window, cx);
                        }
                    },
                )
                .size_full(),
            )
    }
}
