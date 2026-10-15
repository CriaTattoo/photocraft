//! Artist-oriented Color panel. The wheel and triangle use sRGB HSV, like the brush colours.
//! Geometry is local to this panel: no dependency on any particular docking implementation.
use crate::{PhotocraftApp, panels, theme::Tokens};
use egui::{Color32, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, vec2};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Picker {
    #[default]
    Wheel,
    Rectangle,
}

fn active(app: &PhotocraftApp) -> [f32; 4] {
    if app.ui.color_panel.background { app.session.tools.background } else { app.session.tools.foreground }
}

fn set_color(app: &mut PhotocraftApp, color: [f32; 4]) {
    if !color.iter().all(|v| v.is_finite()) {
        return;
    }
    let color = color.map(|v| v.clamp(0.0, 1.0));
    if app.ui.color_panel.background {
        app.session.tools.background = color;
    } else {
        app.session.tools.foreground = color;
        crate::type_tool::foreground_changed(app);
    }
}

fn remember(app: &mut PhotocraftApp, color: [f32; 4]) {
    if !color.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
        return;
    }
    let mut recent = [None; 12];
    if let Some(first) = recent.first_mut() {
        *first = Some(color);
    }
    let mut out = recent.iter_mut().skip(1);
    for old in app.ui.color_panel.recent.iter().flatten() {
        if old.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) && panels::srgb_bytes(*old) != panels::srgb_bytes(color) {
            let Some(slot) = out.next() else { break };
            *slot = Some(*old);
        }
    }
    app.ui.color_panel.recent = recent;
}

pub(crate) fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    // The clip rectangle describes the visible panel even inside a scrolling dock body.
    // A tall panel grows the wheel up to the available width; a short one scrolls its controls.
    let visible_height = ui.clip_rect().height().min(ui.available_height()).clamp(0.0, 4096.0);
    ui.spacing_mut().item_spacing.y = 6.0;
    header(app, ui);
    let background = app.ui.color_panel.background;
    let before = active(app);
    let observed_id = ui.id().with(("artist-observed-color", background));
    let observed: Option<[f32; 4]> = ui.data(|d| d.get_temp(observed_id));
    let hue_key = Id::new(("color-field-hue", background));
    if app.ui.color_panel.picker == Picker::Wheel {
        let color = active(app);
        let mut hsv = panels::srgb_hsva(color);
        let last: (f32, f32) = ui.data(|d| d.get_temp(hue_key)).unwrap_or((hsv.h, hsv.s));
        if hsv.v < 0.01 {
            hsv.h = last.0;
            hsv.s = last.1;
        } else if hsv.s < 0.01 {
            hsv.h = last.0;
        }
        let footer = if app.ui.color_panel.sliders { 174.0 } else { 90.0 };
        let width = ui.available_width().max(1.0);
        let diameter = (visible_height - footer).max(136.0).min(width);
        if wheel(ui, &mut hsv, diameter) {
            let mut color = panels::hsva_srgb(hsv);
            color[3] = active(app)[3];
            set_color(app, color);
            ui.data_mut(|d| {
                d.insert_temp(hue_key, (hsv.h, hsv.s));
                d.remove::<egui::ecolor::Hsva>(Id::new(("color-picker-hsva", background)));
            });
        }
        ui.horizontal_wrapped(|ui| {
            let mut color = active(app);
            if panels::color_readout(ui, ui.id().with(("color-field", background)), &mut color) {
                set_color(app, color);
            }
        });
    } else {
        // The rectangular option also uses the live body height in floating and custom docks.
        let reserve = if app.ui.color_panel.sliders { 152.0 } else { 68.0 };
        let height = (visible_height - reserve).max(120.0);
        ui.data_mut(|d| d.insert_temp(panels::color_field_fill_id(), height));
        panels::rectangular_color_field(app, ui);
        ui.data_mut(|d| d.remove::<f32>(panels::color_field_fill_id()));
    }
    if app.ui.color_panel.sliders {
        ui.add_space(2.0);
        panels::hsb_sliders(app, ui);
    }
    ui.separator();
    history(app, ui);
    let after = active(app);
    let pending = ui.id().with(("recent-color-pending", background));
    if before != after || observed.is_some_and(|color| color != before) {
        ui.data_mut(|d| d.insert_temp(pending, true));
    }
    // Commit a drag once it ends, instead of filling history with every intermediate colour.
    if !ui.input(|i| i.pointer.primary_down()) && ui.data(|d| d.get_temp::<bool>(pending)).unwrap_or(false) {
        remember(app, after);
        ui.data_mut(|d| d.remove::<bool>(pending));
        ui.ctx().request_repaint();
    }
    ui.data_mut(|d| d.insert_temp(observed_id, after));
}

fn header(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let tokens = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        if app.ui.color_panel.picker == Picker::Wheel {
            let (rect, _) = ui.allocate_exact_size(vec2(57.0, 43.0), Sense::hover());
            let bg = Rect::from_min_size(rect.min + vec2(19.0, 12.0), vec2(30.0, 30.0));
            let fg = Rect::from_min_size(rect.min + vec2(2.0, 1.0), vec2(34.0, 34.0));
            for (r, color, selected) in
                [(bg, app.session.tools.background, app.ui.color_panel.background), (fg, app.session.tools.foreground, !app.ui.color_panel.background)]
            {
                let [red, green, blue] = panels::srgb_bytes(color);
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(red, green, blue));
                ui.painter().rect_stroke(
                    r,
                    3.0,
                    Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { tokens.accent } else { tokens.field_border }),
                    StrokeKind::Outside,
                );
            }
            let bg_response = ui.interact(bg, ui.id().with("artist-bg"), Sense::click()).on_hover_text(tl!("Background Color"));
            let fg_response = ui.interact(fg, ui.id().with("artist-fg"), Sense::click()).on_hover_text(tl!("Foreground Color"));
            fg_response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Foreground Color")));
            bg_response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Background Color")));
            if fg_response.clicked() {
                app.ui.color_panel.background = false;
            } else if bg_response.clicked() {
                app.ui.color_panel.background = true;
            }
            if fg_response.double_clicked() || bg_response.double_clicked() {
                crate::color_picker_ui::open(app, if app.ui.color_panel.background { "background" } else { "foreground" });
            }
        }
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.ui.color_panel.picker, Picker::Wheel, tl!("Wheel"));
                ui.selectable_value(&mut app.ui.color_panel.picker, Picker::Rectangle, tl!("Field"));
            });
            ui.horizontal(|ui| {
                if ui.small_button("⇄").on_hover_text(tl!("Swap Foreground and Background Colors")).clicked() {
                    let _ = app.run("tools.swapColors", serde_json::json!({}));
                }
                ui.checkbox(&mut app.ui.color_panel.sliders, tl!("HSB Sliders"));
            });
        });
    });
}

fn marker(ui: &egui::Ui, position: Pos2, radius: f32) {
    ui.painter().circle_stroke(position, radius + 1.5, Stroke::new(2.0, Color32::from_black_alpha(210)));
    ui.painter().circle_stroke(position, radius, Stroke::new(1.5, Color32::WHITE));
}

fn direction(hue: f32) -> Vec2 {
    // Red starts at the top, matching a traditional painting wheel's orientation.
    let angle = hue * TAU - TAU * 0.25;
    vec2(angle.cos(), angle.sin())
}

fn triangle(center: Pos2, radius: f32) -> [Pos2; 3] {
    // Pure hue, white and black. Keeping orientation fixed makes pen gestures predictable.
    [center + vec2(radius, 0.0), center + vec2(-radius * 0.5, -radius * 0.8660254), center + vec2(-radius * 0.5, radius * 0.8660254)]
}

fn weights(point: Pos2, [a, b, c]: [Pos2; 3]) -> [f32; 3] {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = point - a;
    let denominator = v0.x * v1.y - v1.x * v0.y;
    if denominator.abs() < 0.001 {
        return [1.0, 0.0, 0.0];
    }
    let white = (v2.x * v1.y - v1.x * v2.y) / denominator;
    let black = (v0.x * v2.y - v2.x * v0.y) / denominator;
    [1.0 - white - black, white, black]
}

fn nearest_segment(point: Pos2, start: Pos2, end: Pos2) -> Pos2 {
    let delta = end - start;
    let fraction = ((point - start).dot(delta) / delta.length_sq().max(0.001)).clamp(0.0, 1.0);
    start + delta * fraction
}

fn projected_weights(point: Pos2, vertices: [Pos2; 3]) -> [f32; 3] {
    let values = weights(point, vertices);
    if values.iter().all(|v| *v >= 0.0) {
        return values;
    }
    let [a, b, c] = vertices;
    let mut nearest = a;
    let mut distance = f32::INFINITY;
    for (start, end) in [(a, b), (b, c), (c, a)] {
        let candidate = nearest_segment(point, start, end);
        let d = candidate.distance_sq(point);
        if d < distance {
            nearest = candidate;
            distance = d;
        }
    }
    weights(nearest, vertices).map(|v| v.clamp(0.0, 1.0))
}

fn wheel(ui: &mut egui::Ui, hsv: &mut egui::ecolor::Hsva, diameter: f32) -> bool {
    let width = ui.available_width();
    let (area, response) = ui.allocate_exact_size(vec2(width, diameter), Sense::click_and_drag());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, tl!("Color Wheel")));
    let center = area.center();
    let outer = (diameter * 0.5 - 4.0).max(1.0);
    let inner = (outer - (diameter * 0.075).clamp(10.0, 25.0)).max(1.0);
    let vertices = triangle(center, (inner - 7.0).max(1.0));
    let drag_id = response.id.with("component");
    let mut changed = false;
    // Fix the component at pointer-down. Crossing the ring during a triangle drag must not
    // unexpectedly change the hue (and vice versa). Starting in the gap does nothing.
    if ui.input(|i| i.pointer.primary_pressed()) && response.contains_pointer() {
        if let Some(point) = response.interact_pointer_pos().filter(|p| p.is_finite()) {
            let distance = point.distance(center);
            let component = if distance >= inner && distance <= outer + 3.0 {
                1_u8
            } else if weights(point, vertices).iter().all(|v| *v >= 0.0) {
                2
            } else {
                0
            };
            ui.data_mut(|d| d.insert_temp(drag_id, component));
        }
    }
    if (response.dragged() || response.clicked() || ui.input(|i| i.pointer.primary_pressed()) && response.contains_pointer())
        && let Some(point) = response.interact_pointer_pos().filter(|p| p.is_finite())
    {
        match ui.data(|d| d.get_temp::<u8>(drag_id)).unwrap_or(0) {
            1 => {
                let delta = point - center;
                if delta.length_sq() > 0.01 {
                    hsv.h = (delta.y.atan2(delta.x) / TAU + 0.25).rem_euclid(1.0);
                    changed = true;
                }
            }
            2 => {
                let [hue, white, _] = projected_weights(point, vertices);
                hsv.v = (hue + white).clamp(0.0, 1.0);
                if hsv.v > 0.0001 {
                    hsv.s = (hue / hsv.v).clamp(0.0, 1.0);
                }
                changed = true;
            }
            _ => {}
        }
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        ui.data_mut(|d| d.remove::<u8>(drag_id));
    }
    let mut ring = egui::Mesh::default();
    for step in 0..=144 {
        let hue = step as f32 / 144.0;
        let color = panels::hsva_color32(egui::ecolor::Hsva::new(hue, 1.0, 1.0, 1.0));
        ring.colored_vertex(center + direction(hue) * inner, color);
        ring.colored_vertex(center + direction(hue) * outer, color);
    }
    for step in 0..144_u32 {
        let a = step * 2;
        ring.add_triangle(a, a + 1, a + 3);
        ring.add_triangle(a, a + 3, a + 2);
    }
    ui.painter().add(ring);
    let [hue_vertex, white_vertex, black_vertex] = vertices;
    let at = |hue: f32, white: f32| black_vertex + (hue_vertex - black_vertex) * hue + (white_vertex - black_vertex) * white;
    let mut mesh = egui::Mesh::default();
    // Separate small triangles interpolate encoded RGB (the same space as the readouts).
    // Fixed subdivision bounds both allocations and per-frame rendering cost.
    for row in 0..32 {
        for column in 0..(32 - row) {
            let h = row as f32 / 32.0;
            let w = column as f32 / 32.0;
            let step = 1.0 / 32.0;
            let mut draw = |points: [(f32, f32); 3]| {
                let offset = mesh.vertices.len() as u32;
                for (hue, white) in points {
                    let value = (hue + white).clamp(0.0, 1.0);
                    let saturation = if value > 0.0001 { hue / value } else { hsv.s };
                    mesh.colored_vertex(at(hue, white), panels::hsva_color32(egui::ecolor::Hsva::new(hsv.h, saturation, value, 1.0)));
                }
                mesh.add_triangle(offset, offset + 1, offset + 2);
            };
            draw([(h, w), (h + step, w), (h, w + step)]);
            if row + column < 31 {
                draw([(h + step, w), (h + step, w + step), (h, w + step)]);
            }
        }
    }
    ui.painter().add(mesh);
    let tokens = Tokens::get(ui.ctx());
    ui.painter().add(egui::Shape::closed_line(vertices.to_vec(), Stroke::new(1.0, tokens.separator)));
    marker(ui, center + direction(hsv.h) * ((inner + outer) * 0.5), ((outer - inner) * 0.34).clamp(3.0, 7.0));
    marker(ui, at(hsv.s * hsv.v, (1.0 - hsv.s) * hsv.v), 4.5);
    changed
}

fn history(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(tl!("Recent Colors"));
        if ui.small_button("+").on_hover_text(tl!("Keep Current Color")).clicked() {
            remember(app, active(app));
        }
    });
    let tokens = Tokens::get(ui.ctx());
    let size = ((ui.available_width() - 22.0) / 12.0).clamp(8.0, 38.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for color in app.ui.color_panel.recent.into_iter() {
            let (rect, response) = ui.allocate_exact_size(vec2(size, 23.0), Sense::click());
            let valid = color.filter(|c| c.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
            let fill = valid
                .map(|c| {
                    let [r, g, b] = panels::srgb_bytes(c);
                    Color32::from_rgb(r, g, b)
                })
                .unwrap_or(tokens.field);
            ui.painter().rect_filled(rect, 3.0, fill);
            ui.painter().rect_stroke(rect, 3.0, Stroke::new(1.0, tokens.field_border), StrokeKind::Inside);
            if let Some(color) = valid {
                let [r, g, b] = panels::srgb_bytes(color);
                let response = response.on_hover_text(format!("#{r:02X}{g:02X}{b:02X}"));
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("#{r:02X}{g:02X}{b:02X}")));
                if response.clicked() {
                    set_color(app, color);
                }
            }
        }
    });
}
