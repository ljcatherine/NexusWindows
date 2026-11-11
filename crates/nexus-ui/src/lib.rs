//! Midnight Liquid Glass design system for Nexus Desktop.
//!
//! Tokens come from the iOS app (`App/NexusDesignSystem.swift`) and the approved
//! Midnight concept (`docs/design/liquid-glass/src/concepts.html`). Glass is
//! painted — gradient fill, lit top edge, soft shadow — the same technique the
//! iOS cards use, which needs no blur. The code editor surface stays opaque.

use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Painter, Pos2, Rect, Response, Sense, Shadow,
    Stroke, StrokeKind, TextStyle, Ui, Vec2, Visuals, Widget,
};
use serde::{Deserialize, Serialize};

pub const fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub const fn white_a(alpha: f32) -> Color32 {
    let a = (alpha * 255.0) as u8;
    Color32::from_rgba_premultiplied(a, a, a, a)
}

/// Configurable design tokens. Defaults are the Midnight palette.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tokens {
    pub background: [u8; 3],
    pub surface: [u8; 3],
    pub elevated: [u8; 3],
    pub editor: [u8; 3],
    pub border: [u8; 3],
    pub text: [u8; 3],
    pub text_secondary: [u8; 3],
    pub accent: [u8; 3],
    pub purple: [u8; 3],
    pub success: [u8; 3],
    pub warning: [u8; 3],
    pub error: [u8; 3],
    pub glow_a: [u8; 3],
    pub glow_b: [u8; 3],
    pub glow_c: [u8; 3],
    pub glass_fill: f32,
    pub glass_fill_strong: f32,
    pub glass_edge: f32,
    pub radius: u8,
    pub reduce_transparency: bool,
    pub reduce_motion: bool,
    pub ui_scale: f32,
    pub editor_font_size: f32,
}

fn rgb(v: u32) -> [u8; 3] {
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

impl Default for Tokens {
    fn default() -> Self {
        Self {
            background: rgb(0x070C18),
            surface: rgb(0x172137),
            elevated: rgb(0x202B42),
            editor: rgb(0x0A1020),
            border: rgb(0x28354A),
            text: rgb(0xE9EDF7),
            text_secondary: rgb(0xA6AFC2),
            accent: rgb(0x9CBCFF),
            purple: rgb(0xC9ADFF),
            success: rgb(0x7CE8CA),
            warning: rgb(0xF1C07E),
            error: rgb(0xFF93A2),
            glow_a: rgb(0x27457F),
            glow_b: rgb(0x1B3A63),
            glow_c: rgb(0x34276E),
            glass_fill: 0.075,
            glass_fill_strong: 0.13,
            glass_edge: 0.27,
            radius: 14,
            reduce_transparency: false,
            reduce_motion: false,
            ui_scale: 1.0,
            editor_font_size: 14.0,
        }
    }
}

fn c(v: [u8; 3]) -> Color32 {
    Color32::from_rgb(v[0], v[1], v[2])
}

impl Tokens {
    pub fn bg(&self) -> Color32 {
        c(self.background)
    }
    pub fn surface(&self) -> Color32 {
        c(self.surface)
    }
    pub fn elevated(&self) -> Color32 {
        c(self.elevated)
    }
    pub fn editor(&self) -> Color32 {
        c(self.editor)
    }
    pub fn border(&self) -> Color32 {
        c(self.border)
    }
    pub fn text(&self) -> Color32 {
        c(self.text)
    }
    pub fn text2(&self) -> Color32 {
        c(self.text_secondary)
    }
    pub fn accent(&self) -> Color32 {
        c(self.accent)
    }
    pub fn purple(&self) -> Color32 {
        c(self.purple)
    }
    pub fn success(&self) -> Color32 {
        c(self.success)
    }
    pub fn warning(&self) -> Color32 {
        c(self.warning)
    }
    pub fn error(&self) -> Color32 {
        c(self.error)
    }
    pub fn glass(&self) -> Color32 {
        if self.reduce_transparency { self.surface() } else { white_a(self.glass_fill) }
    }
    pub fn glass_strong(&self) -> Color32 {
        if self.reduce_transparency { self.elevated() } else { white_a(self.glass_fill_strong) }
    }
    pub fn edge(&self) -> Color32 {
        if self.reduce_transparency { self.border() } else { white_a(self.glass_edge) }
    }
    pub fn hairline(&self) -> Color32 {
        white_a(0.07)
    }

    /// Apply egui visuals and text styles.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut v = Visuals::dark();
        v.override_text_color = Some(self.text());
        v.panel_fill = Color32::TRANSPARENT;
        v.window_fill = self.surface().gamma_multiply(0.97);
        v.extreme_bg_color = self.editor();
        v.faint_bg_color = white_a(0.03);
        v.code_bg_color = white_a(0.06);
        v.hyperlink_color = self.accent();
        v.selection.bg_fill = self.accent().gamma_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, self.accent());
        v.window_stroke = Stroke::new(1.0, self.edge());
        v.window_corner_radius = CornerRadius::same(self.radius);
        v.menu_corner_radius = CornerRadius::same(10);
        v.window_shadow = Shadow { offset: [0, 14], blur: 34, spread: 0, color: Color32::from_rgba_premultiplied(3, 8, 24, 110) };
        v.popup_shadow = Shadow { offset: [0, 10], blur: 24, spread: 0, color: Color32::from_rgba_premultiplied(3, 8, 24, 100) };
        v.warn_fg_color = self.warning();
        v.error_fg_color = self.error();
        let r = CornerRadius::same(8);
        for (w, fill, stroke) in [
            (&mut v.widgets.noninteractive, Color32::TRANSPARENT, self.hairline()),
            (&mut v.widgets.inactive, white_a(0.06), white_a(0.08)),
            (&mut v.widgets.hovered, white_a(0.11), white_a(0.18)),
            (&mut v.widgets.active, white_a(0.16), self.accent().gamma_multiply(0.7)),
            (&mut v.widgets.open, white_a(0.12), white_a(0.18)),
        ] {
            w.bg_fill = fill;
            w.weak_bg_fill = fill;
            w.bg_stroke = Stroke::new(1.0, stroke);
            w.corner_radius = r;
        }
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text2());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text());
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        ctx.set_visuals(v);
        ctx.global_style_mut(|s| {
            s.spacing.item_spacing = vec2(8.0, 6.0);
            s.spacing.button_padding = vec2(10.0, 5.0);
            s.spacing.interact_size.y = 26.0;
            s.spacing.window_margin = Margin::same(14);
            s.spacing.menu_margin = Margin::same(8);
            s.spacing.scroll.bar_width = 8.0;
            s.spacing.scroll.floating = true;
            s.animation_time = if self.reduce_motion { 0.0 } else { 0.14 };
            s.text_styles = [
                (TextStyle::Heading, FontId::new(22.0, FontFamily::Name("semibold".into()))),
                (TextStyle::Name("title".into()), FontId::new(15.0, FontFamily::Name("semibold".into()))),
                (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
                (TextStyle::Button, FontId::new(13.0, FontFamily::Proportional)),
                (TextStyle::Small, FontId::new(11.5, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
            ]
            .into();
        });
        ctx.set_zoom_factor(self.ui_scale);
    }
}

/// Load Windows system fonts (Segoe UI Variable, Cascadia) when present,
/// falling back to egui's bundled fonts.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let dir = std::env::var("WINDIR").map(|w| std::path::PathBuf::from(w).join("Fonts")).unwrap_or_default();
    let mut load = |name: &str, files: &[&str]| -> bool {
        for f in files {
            if let Ok(bytes) = std::fs::read(dir.join(f)) {
                fonts.font_data.insert(name.into(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
                return true;
            }
        }
        false
    };
    let ui = load("nexus-ui", &["SegUIVar.ttf", "segoeui.ttf"]);
    let semi = load("nexus-semibold", &["seguisb.ttf", "SegUIVar.ttf"]);
    let mono = load("nexus-mono", &["CascadiaMono.ttf", "CascadiaCode.ttf", "consola.ttf"]);
    let icons = load("nexus-icons", &["SegoeIcons.ttf", "segmdl2.ttf"]);
    let symbols = load("nexus-symbols", &["seguisym.ttf"]);
    let emoji = load("nexus-emoji", &["seguiemj.ttf"]);
    let prop = fonts.families.entry(FontFamily::Proportional).or_default();
    if ui {
        prop.insert(0, "nexus-ui".into());
    }
    if symbols {
        prop.push("nexus-symbols".into());
    }
    if emoji {
        prop.push("nexus-emoji".into());
    }
    let prop_list = prop.clone();
    let monos = fonts.families.entry(FontFamily::Monospace).or_default();
    if mono {
        monos.insert(0, "nexus-mono".into());
    }
    if symbols {
        monos.push("nexus-symbols".into());
    }
    let mut semi_list = prop_list;
    if semi {
        semi_list.insert(0, "nexus-semibold".into());
    }
    fonts.families.insert(FontFamily::Name("semibold".into()), semi_list);
    let mut icon_list = Vec::new();
    if icons {
        icon_list.push("nexus-icons".to_string());
    }
    icon_list.extend(fonts.families[&FontFamily::Proportional].iter().cloned());
    fonts.families.insert(FontFamily::Name("icons".into()), icon_list);
    ctx.set_fonts(fonts);
}

/// Segoe Fluent Icons / MDL2 code points.
pub mod icons {
    pub const HOME: char = '\u{E80F}';
    pub const DEVICES: char = '\u{E772}';
    pub const CODE: char = '\u{E943}';
    pub const NETWORK: char = '\u{E968}';
    pub const SETTINGS: char = '\u{E713}';
    pub const SPARKLE: char = '\u{E945}';
    pub const TERMINAL: char = '\u{E756}';
    pub const SEARCH: char = '\u{E721}';
    pub const BRANCH: char = '\u{E8AB}';
    pub const FOLDER: char = '\u{E8B7}';
    pub const FOLDER_OPEN: char = '\u{E838}';
    pub const FILE: char = '\u{E7C3}';
    pub const REFRESH: char = '\u{E72C}';
    pub const ADD: char = '\u{E710}';
    pub const CLOSE: char = '\u{E711}';
    pub const SAVE: char = '\u{E74E}';
    pub const PLAY: char = '\u{E768}';
    pub const STOP: char = '\u{E71A}';
    pub const MORE: char = '\u{E712}';
    pub const CHEVRON_RIGHT: char = '\u{E76C}';
    pub const CHEVRON_DOWN: char = '\u{E70D}';
    pub const LOCK: char = '\u{E72E}';
    pub const SHIELD: char = '\u{EA18}';
    pub const WARNING: char = '\u{E7BA}';
    pub const ERROR: char = '\u{E783}';
    pub const CHECK: char = '\u{E73E}';
    pub const SEND: char = '\u{E724}';
    pub const ATTACH: char = '\u{E723}';
    pub const HISTORY: char = '\u{E81C}';
    pub const BUG: char = '\u{EBE8}';
    pub const COPY: char = '\u{E8C8}';
    pub const DELETE: char = '\u{E74D}';
    pub const SPLIT: char = '\u{E89F}';
    pub const SIDEBAR: char = '\u{E700}';
    pub const PANEL_BOTTOM: char = '\u{E8A0}';
    pub const SERVER: char = '\u{E968}';
    pub const GAME: char = '\u{E7FC}';
    pub const CHIP: char = '\u{E950}';
    pub const EXTENSIONS: char = '\u{EA86}';
    pub const CLOUD: char = '\u{E753}';
}

/// An icon glyph as rich text.
pub fn icon(cp: char, size: f32) -> egui::RichText {
    egui::RichText::new(cp.to_string()).family(FontFamily::Name("icons".into())).size(size)
}

/// Paint the static midnight backdrop: base colour plus three soft glows.
pub fn paint_backdrop(p: &Painter, rect: Rect, t: &Tokens) {
    p.rect_filled(rect, 0.0, t.bg());
    if t.reduce_transparency {
        return;
    }
    let (w, h) = (rect.width(), rect.height());
    glow(p, pos2(rect.left() + w * 0.80, rect.top() + h * 0.04), vec2(w * 0.46, h * 0.38), c(t.glow_a).gamma_multiply(0.55));
    glow(p, pos2(rect.left() + w * 0.04, rect.top() + h * 0.58), vec2(w * 0.53, h * 0.40), c(t.glow_b).gamma_multiply(0.5));
    glow(p, pos2(rect.left() + w * 0.90, rect.top() + h * 1.0), vec2(w * 0.46, h * 0.34), c(t.glow_c).gamma_multiply(0.55));
}

/// Radial gradient ellipse (centre colour → transparent).
pub fn glow(p: &Painter, center: Pos2, radii: Vec2, color: Color32) {
    let mut mesh = Mesh::default();
    let rings = 6;
    let seg = 48;
    mesh.vertices.push(Vertex { pos: center, uv: WHITE_UV, color });
    for r in 1..=rings {
        let f = r as f32 / rings as f32;
        let a = (1.0 - f).powf(1.6);
        let col = color.gamma_multiply(a);
        for s in 0..seg {
            let ang = s as f32 / seg as f32 * std::f32::consts::TAU;
            mesh.vertices.push(Vertex { pos: center + vec2(ang.cos() * radii.x * f, ang.sin() * radii.y * f), uv: WHITE_UV, color: col });
        }
    }
    for s in 0..seg as u32 {
        let n = (s + 1) % seg as u32;
        mesh.add_triangle(0, 1 + s, 1 + n);
    }
    for r in 0..(rings - 1) as u32 {
        let base = 1 + r * seg as u32;
        let next = base + seg as u32;
        for s in 0..seg as u32 {
            let n = (s + 1) % seg as u32;
            mesh.add_triangle(base + s, next + s, next + n);
            mesh.add_triangle(base + s, next + n, base + n);
        }
    }
    p.add(mesh);
}

/// Paint a glass surface: translucent fill with a diagonal sheen and lit edge.
pub fn paint_glass(p: &Painter, rect: Rect, radius: f32, t: &Tokens, strong: bool) {
    let cr = CornerRadius::same(radius as u8);
    if t.reduce_transparency {
        p.rect(rect, cr, if strong { t.elevated() } else { t.surface() }, Stroke::new(1.0, t.border()), StrokeKind::Inside);
        return;
    }
    p.rect_filled(rect, cr, if strong { t.glass_strong() } else { t.glass() });
    // Sheen: brighter top-left, fading by ~26%.
    let sheen = Rect::from_min_size(rect.min, vec2(rect.width(), (rect.height() * 0.45).min(60.0)));
    let mut mesh = Mesh::default();
    let top = white_a(0.06);
    let bottom = Color32::TRANSPARENT;
    let idx = mesh.vertices.len() as u32;
    let inset = radius * 0.3;
    for (pos, col) in [
        (pos2(sheen.left() + inset, sheen.top() + 1.0), top),
        (pos2(sheen.right() - inset, sheen.top() + 1.0), white_a(0.02)),
        (pos2(sheen.right() - inset, sheen.bottom()), bottom),
        (pos2(sheen.left() + inset, sheen.bottom()), bottom),
    ] {
        mesh.vertices.push(Vertex { pos, uv: WHITE_UV, color: col });
    }
    mesh.add_triangle(idx, idx + 1, idx + 2);
    mesh.add_triangle(idx, idx + 2, idx + 3);
    p.add(mesh);
    p.rect_stroke(rect, cr, Stroke::new(1.0, white_a(0.09)), StrokeKind::Inside);
    // Lit top edge.
    let y = rect.top() + 0.5;
    p.line_segment([pos2(rect.left() + radius, y), pos2(rect.right() - radius, y)], Stroke::new(1.0, t.edge()));
}

/// A glass `Frame` for panels.
pub fn glass_frame(t: &Tokens) -> Frame {
    Frame::new()
        .fill(t.glass())
        .stroke(Stroke::new(1.0, white_a(0.08)))
        .corner_radius(CornerRadius::same(t.radius))
        .inner_margin(Margin::same(12))
}

/// Card: glass container used across Dashboard/Devices/Infrastructure.
pub fn card<R>(ui: &mut Ui, t: &Tokens, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    // Reserve a slot below the content, then fill it once the size is known.
    let slot = ui.painter().add(egui::Shape::Noop);
    let inner = Frame::new().inner_margin(Margin::same(14)).show(ui, |ui| ui.vertical(add).inner);
    paint_glass_under(ui, slot, inner.response.rect, t);
    inner
}

/// Fill a reserved shape slot with shadow + glass so content stays on top.
pub fn paint_glass_under(ui: &Ui, idx: egui::layers::ShapeIdx, rect: Rect, t: &Tokens) {
    let shadow = Shadow {
        offset: [0, 10],
        blur: 26,
        spread: 0,
        color: Color32::from_rgba_premultiplied(3, 8, 24, if t.reduce_transparency { 0 } else { 90 }),
    };
    let mut shapes: Vec<egui::Shape> = vec![shadow.as_shape(rect, CornerRadius::same(t.radius)).into()];
    let cr = CornerRadius::same(t.radius);
    if t.reduce_transparency {
        shapes.push(egui::Shape::rect_filled(rect, cr, t.surface()));
        shapes.push(egui::epaint::RectShape::stroke(rect, cr, Stroke::new(1.0, t.border()), StrokeKind::Inside).into());
    } else {
        shapes.push(egui::Shape::rect_filled(rect, cr, t.glass()));
        let sheen_h = (rect.height() * 0.45).min(60.0);
        let mut mesh = Mesh::default();
        let inset = t.radius as f32 * 0.3;
        for (pos, col) in [
            (pos2(rect.left() + inset, rect.top() + 1.0), white_a(0.06)),
            (pos2(rect.right() - inset, rect.top() + 1.0), white_a(0.02)),
            (pos2(rect.right() - inset, rect.top() + sheen_h), Color32::TRANSPARENT),
            (pos2(rect.left() + inset, rect.top() + sheen_h), Color32::TRANSPARENT),
        ] {
            mesh.vertices.push(Vertex { pos, uv: WHITE_UV, color: col });
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        shapes.push(egui::Shape::mesh(mesh));
        shapes.push(egui::epaint::RectShape::stroke(rect, cr, Stroke::new(1.0, white_a(0.09)), StrokeKind::Inside).into());
        let y = rect.top() + 0.5;
        let r = t.radius as f32;
        shapes.push(egui::Shape::line_segment([pos2(rect.left() + r, y), pos2(rect.right() - r, y)], Stroke::new(1.0, t.edge())));
    }
    ui.painter().set(idx, egui::Shape::Vec(shapes));
}

/// Status dot.
pub fn dot(ui: &mut Ui, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
    ui.painter().circle_filled(rect.center(), 6.0, color.gamma_multiply(0.18));
    resp
}

/// Pill-shaped label with tinted background.
pub fn pill(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(11.5), color);
    let size = galley.size() + vec2(14.0, 6.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(255), color.gamma_multiply(0.14));
    ui.painter().rect_stroke(rect, CornerRadius::same(255), Stroke::new(1.0, color.gamma_multiply(0.35)), StrokeKind::Inside);
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, color);
    resp
}

/// Primary accent button (blue→purple gradient like the iOS send button).
pub struct AccentButton<'a> {
    pub text: &'a str,
    pub enabled: bool,
}

impl Widget for AccentButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let galley = ui.painter().layout_no_wrap(self.text.to_string(), FontId::proportional(13.0), hex(0x0A1020));
        let size = vec2(galley.size().x + 26.0, 28.0);
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, resp) = ui.allocate_exact_size(size, sense);
        let p = ui.painter();
        let (a, b) = (hex(0x9CBCFF), hex(0xC9ADFF));
        let k = if !self.enabled { 0.35 } else if resp.hovered() { 1.0 } else { 0.9 };
        // Rounded ends in the two end colours, linear gradient between them.
        p.rect_filled(rect, CornerRadius::same(9), a.gamma_multiply(k));
        let inner = rect.shrink2(vec2(6.0, 0.0));
        let mut m2 = Mesh::default();
        for (pos, col) in [(inner.left_top(), a), (inner.right_top(), b), (inner.right_bottom(), b), (inner.left_bottom(), a)] {
            m2.vertices.push(Vertex { pos, uv: WHITE_UV, color: col.gamma_multiply(k) });
        }
        m2.add_triangle(0, 1, 2);
        m2.add_triangle(0, 2, 3);
        p.add(m2);
        p.rect_filled(Rect::from_min_max(pos2(inner.right(), rect.top()), rect.right_bottom()), CornerRadius { nw: 0, ne: 9, sw: 0, se: 9 }, b.gamma_multiply(k));
        p.line_segment([pos2(rect.left() + 8.0, rect.top() + 0.5), pos2(rect.right() - 8.0, rect.top() + 0.5)], Stroke::new(1.0, white_a(0.55)));
        p.galley(rect.center() - galley.size() / 2.0, galley, hex(0x0A1020));
        resp.on_hover_cursor(if self.enabled { egui::CursorIcon::PointingHand } else { egui::CursorIcon::NotAllowed })
    }
}

/// Section header text.
pub fn section_title(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.label(egui::RichText::new(text).text_style(TextStyle::Name("title".into())).color(t.text()));
}

pub fn secondary(t: &Tokens, text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into()).color(t.text2()).size(12.0)
}

/// Small sparkline for metric history.
pub fn sparkline(ui: &mut Ui, values: &[f64], max: f64, color: Color32, size: Vec2) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    if values.len() < 2 {
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, "collecting samples…", FontId::proportional(11.0), color.gamma_multiply(0.6));
        return resp;
    }
    let n = values.len();
    let max = if max > 0.0 { max } else { values.iter().cloned().fold(1.0, f64::max) };
    let pts: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(i, v)| pos2(rect.left() + rect.width() * i as f32 / (n - 1) as f32, rect.bottom() - rect.height() * (v / max).clamp(0.0, 1.0) as f32))
        .collect();
    let mut fill = Mesh::default();
    for (i, p) in pts.iter().enumerate() {
        fill.vertices.push(Vertex { pos: *p, uv: WHITE_UV, color: color.gamma_multiply(0.22) });
        fill.vertices.push(Vertex { pos: pos2(p.x, rect.bottom()), uv: WHITE_UV, color: Color32::TRANSPARENT });
        if i > 0 {
            let b = (i as u32) * 2;
            fill.add_triangle(b - 2, b - 1, b);
            fill.add_triangle(b - 1, b + 1, b);
        }
    }
    ui.painter().add(fill);
    ui.painter().add(egui::Shape::line(pts, Stroke::new(1.6, color)));
    resp
}

/// Thin usage bar (context meter, disk, memory).
pub fn meter(ui: &mut Ui, fraction: f32, color: Color32, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 5.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(3), white_a(0.08));
    let mut f = rect;
    f.set_width(rect.width() * fraction.clamp(0.0, 1.0));
    ui.painter().rect_filled(f, CornerRadius::same(3), color);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokens_match_ios_midnight() {
        let t = Tokens::default();
        assert_eq!(t.bg(), hex(0x070C18));
        assert_eq!(t.accent(), hex(0x9CBCFF));
        assert!(t.glass().a() > 0 && t.glass().a() < 40);
        let mut r = Tokens::default();
        r.reduce_transparency = true;
        assert_eq!(r.glass(), r.surface());
    }
}
