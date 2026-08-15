use super::DrawCmd;

/// Which NDS screen an overlay layer targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    Top,
    Bottom,
}

/// Draw commands for both screens after one emulated frame.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Overlay {
    pub top: Vec<DrawCmd>,
    pub bottom: Vec<DrawCmd>,
}

impl Overlay {
    pub fn clear(&mut self) {
        self.top.clear();
        self.bottom.clear();
    }

    pub fn push(&mut self, screen: Screen, cmd: DrawCmd) {
        match screen {
            Screen::Top => self.top.push(cmd),
            Screen::Bottom => self.bottom.push(cmd),
        }
    }

    pub fn extend(&mut self, other: Overlay) {
        self.top.extend(other.top);
        self.bottom.extend(other.bottom);
    }

    pub fn is_empty(&self) -> bool {
        self.top.is_empty() && self.bottom.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::{Color, Line, Point, Rect, Text};

    #[test]
    fn push_routes_to_the_right_screen() {
        let mut overlay = Overlay::default();
        overlay.push(
            Screen::Top,
            DrawCmd::Line(Line::new(
                Point::new(0.0, 0.0),
                Point::new(1.0, 1.0),
                Color::rgb(255, 0, 0),
            )),
        );
        overlay.push(
            Screen::Bottom,
            DrawCmd::Text(Text::new(
                Point::new(4.0, 8.0),
                "dmg",
                Color::rgb(255, 255, 255),
            )),
        );

        assert_eq!(overlay.top.len(), 1);
        assert_eq!(overlay.bottom.len(), 1);
        assert!(matches!(overlay.top[0], DrawCmd::Line(_)));
        assert!(matches!(overlay.bottom[0], DrawCmd::Text(_)));
    }

    #[test]
    fn extend_merges_both_layers() {
        let mut a = Overlay::default();
        a.push(
            Screen::Top,
            DrawCmd::Rect(Rect::stroke(
                Point::new(0.0, 0.0),
                16.0,
                16.0,
                Color::rgb(0, 255, 0),
            )),
        );

        let mut b = Overlay::default();
        b.push(
            Screen::Bottom,
            DrawCmd::Rect(Rect::stroke(
                Point::new(8.0, 8.0),
                32.0,
                32.0,
                Color::rgb(0, 0, 255),
            )),
        );

        a.extend(b);
        assert_eq!(a.top.len(), 1);
        assert_eq!(a.bottom.len(), 1);
    }
}
