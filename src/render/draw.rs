use egui::{
    Align2, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2,
};

use crate::app::SCREEN_WIDTH;
use crate::overlay::{Color, DrawCmd, EguiText, Line, Point, Rect as OverlayRect, TextAlign};

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
        DrawCmd::EguiText(text) => draw_egui_text(painter, mapper, text),
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

fn draw_egui_text(painter: &Painter, mapper: &ConsoleMapper, text: &EguiText) {
    let width = text.width();
    let origin = aligned_origin(text.pos, text.align, width, text.cell_height);

    if let Some(background) = text.background {
        painter.rect_filled(
            mapper.console_rect(origin.x, origin.y, width, text.cell_height),
            0.0,
            to_color32(background),
        );
    }

    let font = FontId::monospace(mapper.length(text.font_size).max(1.0));
    let color = to_color32(text.color);
    for (index, ch) in text.text.chars().enumerate() {
        let cell = mapper.console_rect(
            origin.x + index as f32 * text.advance,
            origin.y,
            text.cell_width,
            text.cell_height,
        );
        painter.text(
            cell.center(),
            Align2::CENTER_CENTER,
            ch,
            font.clone(),
            color,
        );
    }
}

fn aligned_origin(pos: Point, align: TextAlign, width: f32, height: f32) -> Point {
    match align {
        TextAlign::LeftTop => pos,
        TextAlign::CenterTop => Point::new(pos.x - width * 0.5, pos.y),
        TextAlign::RightTop => Point::new(pos.x - width, pos.y),
        TextAlign::LeftCenter => Point::new(pos.x, pos.y - height * 0.5),
        TextAlign::Center => Point::new(pos.x - width * 0.5, pos.y - height * 0.5),
        TextAlign::RightCenter => Point::new(pos.x - width, pos.y - height * 0.5),
        TextAlign::LeftBottom => Point::new(pos.x, pos.y - height),
        TextAlign::CenterBottom => Point::new(pos.x - width * 0.5, pos.y - height),
        TextAlign::RightBottom => Point::new(pos.x - width, pos.y - height),
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

    /// Maps one console-space glyph bitmap without expanding neighboring quads.
    fn console_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Rect {
        let left = self.origin.x + x * self.scale;
        let top = self.origin.y + y * self.scale;
        let right = left + width * self.scale;
        let bottom = top + height * self.scale;
        Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, bottom))
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
        let small = ConsoleMapper::new(Rect::from_min_size(Pos2::ZERO, Vec2::new(256.0, 192.0)));
        let large = ConsoleMapper::new(Rect::from_min_size(Pos2::ZERO, Vec2::new(512.0, 384.0)));

        assert_eq!(small.length(16.0), 16.0);
        assert_eq!(large.length(16.0), 32.0);
    }

    #[test]
    fn opaque_colors_convert() {
        let color = to_color32(Color::rgb(255, 128, 64));
        assert_eq!(color, egui::Color32::from_rgb(255, 128, 64));
    }
}
