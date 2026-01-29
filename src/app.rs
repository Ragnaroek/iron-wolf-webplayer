use eframe::egui;
use egui::{Color32, Frame, Pos2, RichText, Stroke};
use egui_phosphor::regular::ARROW_DOWN;
use iw::config::default_iw_config;
use iw::start::iw_start;
use iw::web::{WebLoader, load_shareware_data};
use wasm_bindgen_futures::spawn_local;
use web_sys::{KeyboardEvent, window};

pub struct IWApp {
    is_expanded: bool,
    playing: bool,
}

impl IWApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> IWApp {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        IWApp {
            is_expanded: false,
            playing: false,
        }
    }
}

const MENUE_MIN_WDITH: f32 = 50.0;
const MENUE_BORDER_WIDTH: f32 = 2.0;

const BACKGROUND_COLOR: Color32 = egui::Color32::from_rgb(0x88, 0, 0);
const MENU_BORDER_COLOUR_BOTTOM_RIGHT: Color32 = egui::Color32::from_rgb(0xD4, 0x00, 0x00);
const ICON_COLOUR: Color32 = egui::Color32::from_rgb(0xFC, 0xFC, 0x54);

impl eframe::App for IWApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.playing {
            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    let input = ctx.input(|i| i.clone());
                    for event in &input.events {
                        if let egui::Event::Key {
                            key,
                            pressed,
                            modifiers,
                            ..
                        } = event
                        {
                            let init = web_sys::KeyboardEventInit::new();
                            init.set_key(egui_key_to_event_key(key));
                            init.set_ctrl_key(modifiers.ctrl);
                            init.set_alt_key(modifiers.alt);
                            init.set_shift_key(modifiers.shift);
                            init.set_bubbles(true);
                            init.set_cancelable(true);

                            let event_type = if *pressed { "keydown" } else { "keyup" };
                            let event =
                                KeyboardEvent::new_with_keyboard_event_init_dict(event_type, &init)
                                    .expect("event");
                            document.dispatch_event(&event).expect("event dispatch");
                        }
                    }
                }
            }
        }

        let animation_speed = 0.25;
        let t = ctx.animate_bool_with_time(
            egui::Id::new("sidebar_anim"),
            self.is_expanded,
            animation_speed,
        );

        let max_width = 280.0;
        let current_width = MENUE_MIN_WDITH + (max_width - MENUE_MIN_WDITH) * t;

        egui::SidePanel::right("wolf_sidebar")
            .resizable(false)
            .exact_width(current_width)
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(0x53, 0, 0))
                    .inner_margin(5.0)
                    .stroke(Stroke::new(
                        MENUE_BORDER_WIDTH,
                        egui::Color32::from_rgb(0x70, 0, 0),
                    )),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let icon = if self.is_expanded { "▶" } else { "◀" };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(icon).color(egui::Color32::WHITE))
                                .frame(false),
                        )
                        .clicked()
                    {
                        self.is_expanded = !self.is_expanded;
                    }
                });

                ui.add_space(20.0);

                render_animated_item(ui, egui_phosphor::regular::FLOPPY_DISK, "SAVE GAME", t);

                let rect = ui.clip_rect();
                let painter = ui.painter();
                // Right stroke
                painter.line_segment(
                    [
                        Pos2::new(rect.right() - MENUE_BORDER_WIDTH, rect.top()),
                        Pos2::new(rect.right() - MENUE_BORDER_WIDTH, rect.bottom()),
                    ],
                    Stroke::new(MENUE_BORDER_WIDTH, MENU_BORDER_COLOUR_BOTTOM_RIGHT),
                );
                // Bottom stroke
                painter.line_segment(
                    [
                        Pos2::new(rect.left(), rect.bottom() - MENUE_BORDER_WIDTH),
                        Pos2::new(rect.right(), rect.bottom() - MENUE_BORDER_WIDTH),
                    ],
                    Stroke::new(MENUE_BORDER_WIDTH, MENU_BORDER_COLOUR_BOTTOM_RIGHT),
                );
            });

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                ui.painter()
                    .rect_filled(ui.max_rect(), 0.0, BACKGROUND_COLOR);

                ui.add_space(ui.max_rect().bottom() / 2.0 + 480.0 / 2.0);
                ui.horizontal_centered(|ui| {
                    ui.add_space(MENUE_MIN_WDITH);
                    ui.vertical_centered(|ui| {
                        let play_response = ui.label(
                            RichText::new(egui_phosphor::regular::PLAY)
                                .size(30.0)
                                .color(ICON_COLOUR),
                        );

                        if play_response.clicked() {
                            self.playing = true; // TODO set this after the game was successfully started!
                            spawn_local(async {
                                let mut shareware_loader = WebLoader::new_shareware();
                                let iw_config = default_iw_config().expect("default config");
                                load_shareware_data(&mut shareware_loader)
                                    .await
                                    .expect("load shareware data");
                                iw_start(shareware_loader, iw_config).expect("iw start");
                            });
                        }
                        let label_response =
                            ui.label(RichText::new("Play Shareware").color(ICON_COLOUR));
                        if play_response.hovered() || label_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                    })
                });
            });
    }
}

// unfortunately egui does not translate with name() to valid
// js event key names. Some of them have to be corrected.
fn egui_key_to_event_key(key: &egui::Key) -> &str {
    match key {
        egui::Key::ArrowDown => "ArrowDown",
        egui::Key::ArrowLeft => "ArrowLeft",
        egui::Key::ArrowRight => "ArrowRight",
        egui::Key::ArrowUp => "ArrowUp",
        egui::Key::Space => " ",
        other => other.name(),
    }
}

fn render_animated_item(ui: &mut egui::Ui, icon: &str, label: &str, t: f32) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        ui.label(RichText::new(icon).size(24.0).color(ICON_COLOUR));

        if t > 0.1 {
            ui.scope(|ui| {
                let opacity = (t - 0.1) / 0.9;

                ui.set_clip_rect(ui.available_rect_before_wrap());

                ui.add_space(10.0);

                let text = RichText::new(label)
                    .strong()
                    .color(egui::Color32::WHITE.linear_multiply(opacity));

                if ui.button(text).clicked() {
                    println!("Clicked {}", label);
                }
            });
        }
    });
    ui.add_space(15.0);
}
