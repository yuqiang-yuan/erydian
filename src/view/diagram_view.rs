use gpui_kit::{
    BorderStyle, Bounds, Context, Corners, Edges, Entity, Font, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, ParentElement, Pixels,
    Point, Render, ScrollDelta, ScrollWheelEvent, Styled, TextAlign, TextRun, Window, canvas,
    component::ActiveTheme, div, hsla, point, px, size,
};

use crate::model::{GraphData, SchemaDocument};

/// What (if anything) a left-button drag is currently operating on.
#[derive(Clone, PartialEq)]
enum DragMode {
    /// Not dragging.
    None,

    /// Dragging a single rectangle by index.
    Table(String),

    /// The click landed on the connection line (recorded, no drag effect).
    Line,

    /// Panning the whole canvas.
    Canvas,
}

pub struct DiagramView {
    schema: Entity<SchemaDocument>,
    scale: f32,

    drag: DragMode,

    /// Screen-space translation of the whole canvas.
    pan: Point<Pixels>,

    /// Top-left of the canvas element in window coordinates, recorded each
    /// paint and read back by event handlers for coordinate conversion.
    canvas_origin: Point<Pixels>,

    // for dragging
    last_mouse: Point<Pixels>,
}

impl DiagramView {
    pub fn new(schema: Entity<SchemaDocument>, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            schema,
            scale: 1.0,
            pan: point(px(0.0), px(0.0)),
            canvas_origin: point(px(0.0), px(0.0)),
            last_mouse: point(px(0.0), px(0.0)),
            drag: DragMode::None,
        }
    }
}

impl DiagramView {
    /// Convert a screen (window pixel) point to world coordinates.
    ///   world = (screen - origin - pan) / scale
    fn screen_to_world(&self, screen: Point<Pixels>) -> Point<f32> {
        point(
            (screen.x - self.canvas_origin.x - self.pan.x) / px(self.scale),
            (screen.y - self.canvas_origin.y - self.pan.y) / px(self.scale),
        )
    }

    /// Pick what a press at `screen` should drag, in priority order:
    /// rect (top-most first) -> connection line -> background pan.
    fn pick_drag(&self, screen: Point<Pixels>, cx: &mut Context<Self>) -> DragMode {
        let world = self.screen_to_world(screen);

        // last added table, first check
        for (_, table) in self.schema.read(cx).tables.iter().enumerate().rev() {
            if let Some(g) = &table.graph
                && g.rect.contains(crate::model::Point::new(world.x, world.y))
            {
                return DragMode::Table(table.id.clone());
            }
        }

        // // Connection line: distance from the click to the segment between the
        // // two rectangle centers, in world units. < ~6px/scale counts as a hit.
        // let a = self.rects[0].center();
        // let b = self.rects[1].center();
        // if point_to_segment_dist(world, a, b) <= 6.0 / self.scale {
        //     return DragMode::Line;
        // }

        DragMode::Canvas
    }

    fn on_mouse_down(&mut self, e: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.last_mouse = e.position;
        self.drag = self.pick_drag(e.position, cx);
        cx.notify();
    }

    fn on_mouse_move(&mut self, e: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.drag == DragMode::None {
            return;
        }

        // Screen-space delta since last move.
        let dx = e.position.x - self.last_mouse.x;
        let dy = e.position.y - self.last_mouse.y;

        match &self.drag {
            DragMode::Table(table_id) => {
                self.schema.update(cx, |this, _| {
                    if let Some(table) = this.tables.iter_mut().find(|t| &t.id == table_id)
                        && let Some(g) = &mut table.graph
                    {
                        g.rect.left += dx / px(self.scale);
                        g.rect.top += dy / px(self.scale);
                        g.is_dirty = true;
                    }
                });
            }
            DragMode::Canvas => {
                self.pan.x += dx;
                self.pan.y += dy;
            }
            DragMode::Line => {
                // Click landed on the line: recorded, but dragging does nothing.
            }
            DragMode::None => {}
        }

        self.last_mouse = e.position;
        cx.notify();
    }

    fn on_mouse_up(&mut self, _e: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.drag = DragMode::None;
        cx.notify();
    }

    fn on_scroll_wheel(
        &mut self,
        e: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mouse = e.position;
        // World point currently under the cursor (with the old scale).
        let world_before = self.screen_to_world(mouse);

        // Prefer exact pixel deltas; fall back to lines at ~100px/line.
        // GPUI's scroll sign is "natural": a positive delta.y scrolls toward the
        // top (wheel up), so we zoom *in* on a positive delta.
        let dy: f32 = match e.delta {
            ScrollDelta::Pixels(p) => p.y / px(1.0),
            ScrollDelta::Lines(l) => l.y * 100.0,
        };
        // Scroll up (dy>0) zooms in, scroll down (dy<0) zooms out.
        let factor = 1.0 + dy * 0.0001;
        self.scale = (self.scale * factor).clamp(0.25, 4.0);

        // Keep the world point under the cursor stationary:
        //   pan = mouse - origin - world_before * scale
        self.pan.x = mouse.x - self.canvas_origin.x - px(world_before.x * self.scale);
        self.pan.y = mouse.y - self.canvas_origin.y - px(world_before.y * self.scale);

        cx.notify();
    }
}

impl Render for DiagramView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trans_point = self.pan;
        let scale = self.scale;
        let view = cx.entity().clone();
        let schema = self.schema.clone();
        let pre_schema = self.schema.clone();

        let text_color = cx.theme().foreground;
        let bg_color = cx.theme().secondary;
        let table_name_bg_color = cx.theme().list_active_border;
        let table_name_color = hsla(0.0, 0.0, 1.0, 1.0);
        let border_color = cx.theme().border;
        let font_size = cx.theme().font_size;

        let padding_y = 8.0f32;
        let padding_x = 12.0f32;

        div()
            .id("diagram-canvas-box")
            .size_full()
            .rounded_md()
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(Self::on_scroll_wheel))
            .child(
                canvas(
                    move |_, window, cx| {
                        let font = Font {
                            family: cx.theme().mono_font_family.clone(),
                            ..Default::default()
                        };

                        pre_schema.update(cx, |this, _| {
                            for table in &mut this.tables {
                                if table.graph.is_some() && !table.graph.as_ref().unwrap().is_dirty
                                {
                                    return;
                                }

                                let old_origin = if let Some(g) = &table.graph {
                                    (g.rect.left, g.rect.top)
                                } else {
                                    (0.0, 0.0)
                                };

                                let runs = vec![TextRun {
                                    len: table.name.len(),
                                    font: font.clone(),
                                    color: text_color,
                                    ..Default::default()
                                }];

                                let shaped = window.text_system().shape_line(
                                    table.name.clone().into(),
                                    font_size,
                                    &runs,
                                    None,
                                );

                                let name_width = shaped.width.as_f32();

                                // 先按照字符的数量来算最大宽度。这个在等宽字体下是成立的。
                                // 但是如果未来允许用户自行设置字体的话，就需要真实的测量每个列的宽度之后再决定哪个是最宽的
                                let col_width =
                                    match table.columns.iter().map(|c| c.name.len()).max() {
                                        Some(u) => table
                                            .columns
                                            .iter()
                                            .find(|c| c.name.len() == u)
                                            .map(|c| {
                                                let runs = vec![TextRun {
                                                    len: c.name.len(),
                                                    font: font.clone(),
                                                    color: text_color,
                                                    ..Default::default()
                                                }];

                                                let shaped = window.text_system().shape_line(
                                                    c.name.clone().into(),
                                                    font_size,
                                                    &runs,
                                                    None,
                                                );

                                                shaped.width.as_f32()
                                            })
                                            .unwrap_or(name_width),
                                        None => name_width,
                                    };

                                let table_width = if name_width > col_width {
                                    name_width
                                } else {
                                    col_width
                                } + padding_x * 2.0;

                                let height = (table.columns.len() + 1) as f32
                                    * (font_size.as_f32() + padding_y * 2.0)
                                    + padding_y * 2.0;

                                table.graph = Some(GraphData {
                                    is_dirty: false,
                                    selected: false,
                                    rect: crate::model::Rect::new(
                                        old_origin.0,
                                        old_origin.1,
                                        table_width,
                                        height,
                                    ),
                                    points: vec![],
                                });
                            }
                        });
                    },
                    move |bounds, _, window, cx| {
                        let _ = view.update(cx, |this, _| this.canvas_origin = bounds.origin);
                        let origin = bounds.origin;

                        let scaled_border_widths = Edges {
                            left: px(1.0 * scale),
                            top: px(1. * scale),
                            right: px(1. * scale),
                            bottom: px(1. * scale),
                        };

                        let scaled_corners = Corners {
                            top_left: px(4.0 * scale),
                            top_right: px(4.0 * scale),
                            bottom_right: px(4.0 * scale),
                            bottom_left: px(4.0 * scale),
                        };

                        let tables = schema
                            .read(cx)
                            .tables
                            .iter()
                            .map(|table| table.clone())
                            .collect::<Vec<_>>();

                        let font = Font {
                            family: cx.theme().mono_font_family.clone(),
                            ..Default::default()
                        };

                        let scaled_padding_x = padding_x * scale;
                        let scaled_padding_y = padding_y * scale;
                        let scaled_font_size = font_size * scale;

                        tables.iter().for_each(|table| {
                            if table.graph.is_none() {
                                return;
                            }

                            let graph = table.graph.as_ref().unwrap();

                            let top_left = point(
                                origin.x + trans_point.x + px(graph.rect.left * scale),
                                origin.y + trans_point.y + px(graph.rect.top * scale),
                            );

                            let scaled_size =
                                size(px(graph.rect.width * scale), px(graph.rect.height * scale));

                            // table rectangle
                            window.paint_quad(PaintQuad {
                                bounds: Bounds {
                                    origin: top_left,
                                    size: scaled_size,
                                },
                                corner_radii: scaled_corners,
                                background: bg_color.into(),
                                border_widths: scaled_border_widths,
                                border_color: border_color,
                                border_style: BorderStyle::default(),
                            });

                            // table title bar bg
                            window.paint_quad(PaintQuad {
                                bounds: Bounds {
                                    origin: top_left,
                                    size: size(
                                        px(graph.rect.width * scale),
                                        px(scaled_font_size.as_f32() + scaled_padding_y * 3.0),
                                    ),
                                },
                                corner_radii: Corners {
                                    top_left: px(4.0 * scale),
                                    top_right: px(4.0 * scale),
                                    bottom_right: px(0.0),
                                    bottom_left: px(0.0),
                                },
                                background: table_name_bg_color.into(),
                                border_widths: Edges {
                                    left: px(0.0),
                                    top: px(0.0),
                                    right: px(0.0),
                                    bottom: px(0.0),
                                },
                                border_color: border_color,
                                border_style: BorderStyle::default(),
                            });

                            // table name
                            let runs = vec![TextRun {
                                len: table.name.len(),
                                font: font.clone(),
                                color: table_name_color,
                                ..Default::default()
                            }];

                            let shaped = window.text_system().shape_line(
                                table.name.clone().into(),
                                scaled_font_size,
                                &runs,
                                None,
                            );

                            let _ = shaped.paint(
                                top_left + point(px(scaled_padding_x), px(scaled_padding_y * 1.5)), // the table rect has padding
                                scaled_font_size,
                                TextAlign::Left,
                                None,
                                window,
                                cx,
                            );

                            // column names
                            for (i, col) in table.columns.iter().enumerate() {
                                let runs = vec![TextRun {
                                    len: col.name.len(),
                                    font: font.clone(),
                                    color: text_color,
                                    ..Default::default()
                                }];

                                let shaped = window.text_system().shape_line(
                                    col.name.clone().into(),
                                    scaled_font_size,
                                    &runs,
                                    None,
                                );

                                let _ = shaped.paint(
                                    top_left
                                        + point(
                                            px(scaled_padding_x),
                                            px((i + 1) as f32
                                                * (scaled_font_size.as_f32()
                                                    + scaled_padding_y * 2.0)
                                                + scaled_padding_y * 2.0),
                                        ),
                                    font_size * scale,
                                    TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                );
                            }
                        });
                    },
                )
                .size_full(),
            )
    }
}
