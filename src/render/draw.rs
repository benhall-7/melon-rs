use egui::{Align2, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};

use crate::app::SCREEN_WIDTH;
use crate::overlay::{
    Color, DrawCmd, Line, Outline, Point, Rect as OverlayRect, Text, TextAlign, TextFont,
};

/// Draws console-space overlay commands on top of one screen.
pub fn draw_screen(ui: &Ui, screen: Rect, cmds: &[DrawCmd]) {
    if cmds.is_empty() {
        return;
    }

    let painter = ui.painter_at(screen);
    let mapper = ConsoleMapper::new(screen);

    for cmd in cmds {
        draw_cmd(&painter, &mapper, cmd);
    }
}

fn draw_cmd(painter: &Painter, mapper: &ConsoleMapper, cmd: &DrawCmd) {
    match cmd {
        DrawCmd::Rect(rect) => draw_rect(painter, mapper, rect),
        DrawCmd::Line(line) => draw_line(painter, mapper, line),
        DrawCmd::Text(text) => draw_text(painter, mapper, text),
    }
}

fn draw_rect(painter: &Painter, mapper: &ConsoleMapper, rect: &OverlayRect) {
    let egui_rect = Rect::from_min_size(
        mapper.point(rect.origin),
        Vec2::new(mapper.length(rect.width), mapper.length(rect.height)),
    );

    if let Some(fill) = rect.fill {
        painter.rect_filled(egui_rect, 0.0, to_color32(fill));
    }

    painter.rect_stroke(
        egui_rect,
        0.0,
        Stroke::new(mapper.length(rect.stroke_width), to_color32(rect.stroke)),
        egui::StrokeKind::Outside,
    );
}

fn draw_line(painter: &Painter, mapper: &ConsoleMapper, line: &Line) {
    painter.line_segment(
        [mapper.point(line.from), mapper.point(line.to)],
        Stroke::new(mapper.length(line.width), to_color32(line.color)),
    );
}

fn draw_text(painter: &Painter, mapper: &ConsoleMapper, text: &Text) {
    let pos = mapper.point(text.pos);
    let font = font_id(text.font, mapper.length(text.size));
    let align = to_align2(text.align);

    if let Some(outline) = text.outline {
        draw_text_outline(painter, pos, align, &text.text, &font, outline, mapper);
    }

    painter.text(
        pos,
        align,
        &text.text,
        font,
        to_color32(text.color),
    );
}

fn draw_text_outline(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    text: &str,
    font: &FontId,
    outline: Outline,
    mapper: &ConsoleMapper,
) {
    let step = mapper.length(outline.width);
    let color = to_color32(outline.color);

    for (dx, dy) in [
        (-1.0, 0.0),
        (1.0, 0.0),
        (0.0, -1.0),
        (0.0, 1.0),
        (-1.0, -1.0),
        (1.0, -1.0),
        (-1.0, 1.0),
        (1.0, 1.0),
    ] {
        painter.text(
            pos + Vec2::new(dx * step, dy * step),
            align,
            text,
            font.clone(),
            color,
        );
    }
}

fn font_id(font: TextFont, size: f32) -> FontId {
    match font {
        TextFont::Monospace => FontId::monospace(size),
        TextFont::Proportional => FontId::proportional(size),
    }
}

fn to_align2(align: TextAlign) -> Align2 {
    match align {
        TextAlign::LeftTop => Align2::LEFT_TOP,
        TextAlign::CenterTop => Align2::CENTER_TOP,
        TextAlign::RightTop => Align2::RIGHT_TOP,
        TextAlign::LeftCenter => Align2::LEFT_CENTER,
        TextAlign::Center => Align2::CENTER_CENTER,
        TextAlign::RightCenter => Align2::RIGHT_CENTER,
        TextAlign::LeftBottom => Align2::LEFT_BOTTOM,
        TextAlign::CenterBottom => Align2::CENTER_BOTTOM,
        TextAlign::RightBottom => Align2::RIGHT_BOTTOM,
    }
}

fn to_color32(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

/// Maps console screen space onto a drawn screen rect.
struct ConsoleMapper {
    origin: Pos2,
    scale: f32,
}

impl ConsoleMapper {
    fn new(screen: Rect) -> Self {
        Self {
            origin: screen.min,
            scale: screen.width() / SCREEN_WIDTH as f32,
        }
    }

    fn point(&self, point: Point) -> Pos2 {
        Pos2::new(
            self.origin.x + point.x * self.scale,
            self.origin.y + point.y * self.scale,
        )
    }

    fn length(&self, console_pixels: f32) -> f32 {
        console_pixels * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::Color;

    fn mapper() -> ConsoleMapper {
        ConsoleMapper::new(Rect::from_min_size(
            Pos2::new(10.0, 20.0),
            Vec2::new(256.0, 192.0),
        ))
    }

    #[test]
    fn console_corners_map_to_the_screen_rect() {
        let mapper = mapper();

        assert_eq!(mapper.point(Point::new(0.0, 0.0)), Pos2::new(10.0, 20.0));
        assert_eq!(
            mapper.point(Point::new(255.0, 191.0)),
            Pos2::new(265.0, 211.0)
        );
    }

    #[test]
    fn lengths_scale_with_the_screen() {
        let small = ConsoleMapper::new(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(256.0, 192.0),
        ));
        let large = ConsoleMapper::new(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(512.0, 384.0),
        ));

        assert_eq!(small.length(16.0), 16.0);
        assert_eq!(large.length(16.0), 32.0);
    }

    #[test]
    fn opaque_colors_convert() {
        let color = to_color32(Color::rgb(255, 128, 64));
        assert_eq!(color, egui::Color32::from_rgb(255, 128, 64));
    }
}
