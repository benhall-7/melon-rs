use std::sync::{Arc, OnceLock};

use egui::{
    Color32, ColorImage, Context, Pos2, Rect, TextureHandle, TextureOptions, Ui, Vec2,
    ViewportBuilder,
};
use tokio::sync::watch;

use crate::frontend::Frames;
use crate::input::{InputBridge, InputEvent, Modifiers, TouchPoint};
use crate::render::{draw_screen, RenderContext, RenderHook, RenderStatus, ScreenRect};
use crate::EmuStateChange;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 192;

/// Handle the emulator uses to wake the UI once a frame is ready.
///
/// eframe creates the [`Context`], so the emulator task cannot be handed one
/// when it is spawned; it takes this and waits for the first repaint to fill it.
pub type RepaintHandle = Arc<OnceLock<Context>>;

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([
            SCREEN_WIDTH as f32,
            (2 * SCREEN_HEIGHT) as f32,
        ]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    }
}

/// Draws the two screens and forwards host input.
///
/// Deliberately an observer: it reads the latest frame and pushes events into
/// the input channel, and never advances emulation. Emulation timing belongs to
/// the emulator task, which owns its own clock, because eframe throttles
/// repaints when the window is unfocused or occluded.
pub struct App {
    frames: watch::Receiver<Arc<Frames>>,
    status: watch::Receiver<RenderStatus>,
    render_hooks: Vec<Box<dyn RenderHook>>,
    bridge: InputBridge,
    state_tx: watch::Sender<Option<EmuStateChange>>,
    top: TextureHandle,
    bottom: TextureHandle,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        frames: watch::Receiver<Arc<Frames>>,
        status: watch::Receiver<RenderStatus>,
        render_hooks: Vec<Box<dyn RenderHook>>,
        bridge: InputBridge,
        state_tx: watch::Sender<Option<EmuStateChange>>,
        repaint: &RepaintHandle,
    ) -> Self {
        let _ = repaint.set(cc.egui_ctx.clone());

        let blank = ColorImage::new(
            [SCREEN_WIDTH, SCREEN_HEIGHT],
            vec![Color32::BLACK; SCREEN_WIDTH * SCREEN_HEIGHT],
        );

        App {
            frames,
            status,
            render_hooks,
            bridge,
            state_tx,
            top: cc
                .egui_ctx
                .load_texture("top_screen", blank.clone(), TextureOptions::NEAREST),
            bottom: cc
                .egui_ctx
                .load_texture("bottom_screen", blank, TextureOptions::NEAREST),
        }
    }

    /// Uploads the latest pair of screens, if the emulator has drawn any since
    /// the last repaint. Resizing the window should not cost two conversions.
    fn upload_frames(&mut self) {
        if !self.frames.has_changed().unwrap_or(false) {
            return;
        }

        // Cloning the handle rather than reading through it keeps the emulator
        // from waiting on the conversion below.
        let frames = self.frames.borrow_and_update().clone();

        self.top
            .set(screen_image(&frames.top), TextureOptions::NEAREST);
        self.bottom
            .set(screen_image(&frames.bottom), TextureOptions::NEAREST);
    }

    fn invoke_render_hooks(&mut self, ui: &Ui, top_screen: Rect, bottom_screen: Rect) {
        let status = *self.status.borrow();
        let ctx = RenderContext {
            frame: status.frame,
            paused: status.paused,
            top_screen: screen_rect(top_screen),
            bottom_screen: screen_rect(bottom_screen),
        };

        for hook in &mut self.render_hooks {
            let overlay = hook.on_render(ctx);
            draw_screen(ui, top_screen, &overlay.top);
            draw_screen(ui, bottom_screen, &overlay.bottom);
        }
    }

    /// Draws both screens and reports where the bottom one landed.
    fn draw_screens(&mut self, ui: &mut Ui) -> (Rect, Rect) {
        ui.spacing_mut().item_spacing = Vec2::ZERO;

        let available = ui.available_size();
        let size = screen_size(available);
        let origin = ui.available_rect_before_wrap().min;
        let inset = Vec2::new(
            (available.x - size.x).max(0.0) / 2.0,
            (available.y - 2.0 * size.y).max(0.0) / 2.0,
        );

        let top_screen = Rect::from_min_size(origin + inset, size);
        let bottom_screen =
            Rect::from_min_size(origin + inset + Vec2::new(0.0, size.y), size);

        ui.vertical_centered(|ui| {
            ui.add_space(inset.y);
            ui.add(screen_widget(&self.top, size));
            ui.add(screen_widget(&self.bottom, size));
        });

        (top_screen, bottom_screen)
    }

    fn touch_screen(&self, raw: &egui::RawInput) -> Rect {
        raw.screen_rect
            .map(bottom_screen_rect)
            .or_else(|| *self.bridge.bottom_screen.lock().unwrap())
            .unwrap_or_else(|| {
                bottom_screen_rect(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(SCREEN_WIDTH as f32, 2.0 * SCREEN_HEIGHT as f32),
                ))
            })
    }

    fn forward_input(&self, ctx: &Context, screen: Rect, raw_events: &[egui::Event]) {
        // While egui owns the keyboard, keystrokes belong to whatever is focused
        // rather than to the console.
        let typing = ctx.egui_wants_keyboard_input();

        let mut events = Vec::new();
        for event in raw_events {
            if typing && matches!(event, egui::Event::Key { .. }) {
                continue;
            }
            host_events(event, screen, &mut events);
        }

        self.bridge.forward(events);
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let screen = self.touch_screen(raw);
        self.forward_input(ctx, screen, &raw.events);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.upload_frames();
        let (top_screen, bottom_screen) = self.draw_screens(ui);
        self.invoke_render_hooks(ui, top_screen, bottom_screen);
        *self.bridge.bottom_screen.lock().unwrap() = Some(bottom_screen);
    }

    /// Letterboxing around the screens, rather than egui's window colour.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::BLACK.to_normalized_gamma_f32()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.state_tx.send(Some(EmuStateChange::Stop));
    }
}

fn screen_rect(rect: Rect) -> ScreenRect {
    ScreenRect {
        min_x: rect.min.x,
        min_y: rect.min.y,
        width: rect.width(),
        height: rect.height(),
    }
}

fn screen_widget(texture: &TextureHandle, size: Vec2) -> egui::Image<'static> {
    egui::Image::new(egui::load::SizedTexture::from(texture)).fit_to_exact_size(size)
}

/// melonDS writes BGRA and does not promise a meaningful alpha channel; egui
/// blends what it is given, so the screens have to be made opaque here.
fn screen_image(frame: &[u8]) -> ColorImage {
    let pixels = frame
        .chunks_exact(4)
        .map(|pixel| Color32::from_rgb(pixel[2], pixel[1], pixel[0]))
        .collect();

    ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], pixels)
}

/// Largest aspect-correct size for one screen that leaves room for both.
fn screen_size(available: Vec2) -> Vec2 {
    let scale = (available.x / SCREEN_WIDTH as f32)
        .min(available.y / (2 * SCREEN_HEIGHT) as f32)
        .max(0.0);

    Vec2::new(SCREEN_WIDTH as f32 * scale, SCREEN_HEIGHT as f32 * scale)
}

/// Where the bottom screen lands inside `content`, matching [`App::draw_screens`].
fn bottom_screen_rect(content: Rect) -> Rect {
    let available = content.size();
    let size = screen_size(available);
    let origin = content.min;

    Rect::from_min_size(
        Pos2::new(
            origin.x + (available.x - size.x).max(0.0) / 2.0,
            origin.y + (available.y - 2.0 * size.y).max(0.0) / 2.0 + size.y,
        ),
        size,
    )
}

/// Maps a pointer position onto the console's touch grid.
///
/// A function of where the bottom screen was drawn rather than of the window,
/// so it holds at any size and scale factor, and answers "is the pointer on the
/// touch screen at all" instead of leaving that to be inferred.
pub fn screen_point(screen: Rect, pos: Pos2) -> Option<TouchPoint> {
    if !screen.contains(pos) || screen.width() <= 0.0 || screen.height() <= 0.0 {
        return None;
    }

    let x = (pos.x - screen.left()) / screen.width() * SCREEN_WIDTH as f32;
    let y = (pos.y - screen.top()) / screen.height() * SCREEN_HEIGHT as f32;

    // The rect is inclusive of its far edge, which maps to one past the last
    // pixel.
    TouchPoint::new(
        (x as usize).min(SCREEN_WIDTH - 1) as u8,
        (y as usize).min(SCREEN_HEIGHT - 1) as u8,
    )
}

/// Translates one egui event into host events for the binding layer.
///
/// `screen` is where the bottom screen was drawn this repaint.
pub fn host_events(event: &egui::Event, screen: Rect, out: &mut Vec<InputEvent>) {
    match event {
        egui::Event::Key {
            key,
            physical_key,
            pressed,
            repeat,
            modifiers,
        } => {
            if *repeat {
                return;
            }

            // A binding names a position on the keyboard, not the letter the
            // host keymap produced there.
            let key = physical_key.unwrap_or(*key);

            out.push(InputEvent::KeyModifierChange(Modifiers::from(*modifiers)));
            out.push(match pressed {
                true => InputEvent::KeyDown(key),
                false => InputEvent::KeyUp(key),
            });
        }
        egui::Event::PointerMoved(pos) => {
            if let Some(point) = screen_point(screen, *pos) {
                out.push(InputEvent::CursorMove(point.x, point.y));
            }
        }
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            ..
        } => {
            if !pressed {
                out.push(InputEvent::MouseUp);
                return;
            }

            // A press that missed the touch screen is not a stylus press, so it
            // must not touch wherever the cursor was last seen.
            if let Some(point) = screen_point(screen, *pos) {
                out.push(InputEvent::CursorMove(point.x, point.y));
                out.push(InputEvent::MouseDown);
            }
        }
        // A pointer that left the window cannot still be holding the stylus.
        egui::Event::PointerGone => out.push(InputEvent::MouseUp),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use egui::Key;

    /// An arbitrary scaled and offset placement, to keep the tests from passing
    /// only for a screen that happens to sit at the origin at 1x.
    fn screen() -> Rect {
        Rect::from_min_size(Pos2::new(37.0, 512.0), Vec2::new(768.0, 576.0))
    }

    fn key(key: Key, physical: Option<Key>) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: physical,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }
    }

    fn translate(event: &egui::Event) -> Vec<InputEvent> {
        let mut out = Vec::new();
        host_events(event, screen(), &mut out);
        out
    }

    #[test]
    fn the_screen_corners_map_to_the_console_corners() {
        let screen = screen();

        assert_eq!(screen_point(screen, screen.min), TouchPoint::new(0, 0));
        assert_eq!(
            screen_point(screen, Pos2::new(screen.right(), screen.top())),
            TouchPoint::new(255, 0)
        );
        assert_eq!(
            screen_point(screen, Pos2::new(screen.left(), screen.bottom())),
            TouchPoint::new(0, 191)
        );
        assert_eq!(screen_point(screen, screen.max), TouchPoint::new(255, 191));
    }

    #[test]
    fn a_position_off_the_screen_touches_nothing() {
        let screen = screen();

        assert_eq!(screen_point(screen, Pos2::ZERO), None);
        assert_eq!(
            screen_point(screen, Pos2::new(screen.center().x, screen.top() - 1.0)),
            None
        );
        assert_eq!(
            screen_point(screen, Pos2::new(screen.right() + 1.0, screen.center().y)),
            None
        );
    }

    /// The same place on the screen has to mean the same place on the console
    /// whatever the window size is, which is exactly what the replaced
    /// window-space arithmetic could not promise.
    #[test]
    fn the_mapping_does_not_depend_on_the_screen_size() {
        let small = Rect::from_min_size(Pos2::ZERO, Vec2::new(256.0, 192.0));
        let large = Rect::from_min_size(Pos2::new(11.0, 23.0), Vec2::new(1024.0, 768.0));

        for fraction in [0.0, 0.25, 0.5, 0.75] {
            let at = |rect: Rect| {
                screen_point(
                    rect,
                    rect.min + Vec2::new(rect.width() * fraction, rect.height() * fraction),
                )
            };

            assert_eq!(at(small), at(large), "at {fraction} of the screen");
        }
    }

    #[test]
    fn a_binding_follows_the_physical_key() {
        assert_eq!(
            translate(&key(Key::Semicolon, Some(Key::S))),
            vec![
                InputEvent::KeyModifierChange(Modifiers::empty()),
                InputEvent::KeyDown(Key::S),
            ]
        );
    }

    #[test]
    fn a_key_without_a_physical_spelling_falls_back_to_the_logical_one() {
        assert_eq!(
            translate(&key(Key::S, None)),
            vec![
                InputEvent::KeyModifierChange(Modifiers::empty()),
                InputEvent::KeyDown(Key::S),
            ]
        );
    }

    #[test]
    fn host_key_repeat_is_dropped() {
        let repeat = egui::Event::Key {
            key: Key::S,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::default(),
        };

        assert_eq!(translate(&repeat), vec![]);
    }

    #[test]
    fn modifiers_arrive_with_the_key_that_carried_them() {
        let event = egui::Event::Key {
            key: Key::S,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        };

        assert_eq!(
            translate(&event),
            vec![
                InputEvent::KeyModifierChange(Modifiers::CTRL),
                InputEvent::KeyDown(Key::S),
            ]
        );
    }

    #[test]
    fn a_click_on_the_screen_touches_where_it_landed() {
        let screen = screen();
        let event = egui::Event::PointerButton {
            pos: screen.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        };

        assert_eq!(
            translate(&event),
            vec![InputEvent::CursorMove(128, 96), InputEvent::MouseDown]
        );
    }

    /// Otherwise pressing a menu item would touch wherever the stylus was last
    /// seen on the screen.
    #[test]
    fn a_click_off_the_screen_touches_nothing() {
        let event = egui::Event::PointerButton {
            pos: Pos2::ZERO,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        };

        assert_eq!(translate(&event), vec![]);
    }

    /// A release has to be honoured wherever it happens, or dragging off the
    /// screen and letting go would leave the stylus down forever.
    #[test]
    fn a_release_anywhere_releases_the_touch() {
        let event = egui::Event::PointerButton {
            pos: Pos2::ZERO,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        };

        assert_eq!(translate(&event), vec![InputEvent::MouseUp]);
        assert_eq!(
            translate(&egui::Event::PointerGone),
            vec![InputEvent::MouseUp]
        );
    }

    #[test]
    fn one_screen_fits_half_the_available_height() {
        assert_eq!(
            screen_size(Vec2::new(256.0, 384.0)),
            Vec2::new(256.0, 192.0)
        );
        // Width is the constraint here, so both screens must still fit.
        assert_eq!(
            screen_size(Vec2::new(256.0, 1000.0)),
            Vec2::new(256.0, 192.0)
        );
        assert_eq!(
            screen_size(Vec2::new(1000.0, 384.0)),
            Vec2::new(256.0, 192.0)
        );
    }
}
