use eframe::egui;
use egui::{Color32, Frame, Pos2, Stroke};
use iw::config::default_iw_config;
use iw::start::iw_start;
use iw::web::{WebLoader, load_shareware_data};
use wasm_bindgen_futures::spawn_local;

pub struct IWApp {
    is_expanded: bool,
}

impl IWApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> IWApp {
        IWApp { is_expanded: false }
    }
}

const MENUE_BORDER_WIDTH: f32 = 2.0;
const MENU_BORDER_COLOUR_BOTTOM_RIGHT: Color32 = egui::Color32::from_rgb(0xD4, 0, 0);

impl eframe::App for IWApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let animation_speed = 0.25;
        let t = ctx.animate_bool_with_time(
            egui::Id::new("sidebar_anim"),
            self.is_expanded,
            animation_speed,
        );

        let min_width = 50.0;
        let max_width = 280.0;
        let current_width = min_width + (max_width - min_width) * t;

        egui::SidePanel::right("wolf_sidebar")
            .resizable(false)
            .exact_width(current_width)
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(0x53, 0, 0))
                    .inner_margin(5.0)
                    // 0xD4 for right and down
                    .stroke(Stroke::new(
                        MENUE_BORDER_WIDTH,
                        egui::Color32::from_rgb(0x70, 0, 0),
                    )),
            ) // Dunkler Rahmen
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let icon = if self.is_expanded { "▶" } else { "◀" };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(icon).color(egui::Color32::WHITE),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        self.is_expanded = !self.is_expanded;
                    }
                });

                ui.add_space(20.0);

                render_animated_item(ui, "💾", "SAVE GAME", t);

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
                    .rect_filled(ui.max_rect(), 0.0, egui::Color32::from_rgb(0x8C, 0, 0));
                if ui.button("Start Playing").clicked() {
                    spawn_local(async {
                        let mut shareware_loader = WebLoader::new_shareware();
                        let iw_config = default_iw_config().expect("default config");
                        load_shareware_data(&mut shareware_loader)
                            .await
                            .expect("load shareware data");
                        iw_start(shareware_loader, iw_config).expect("iw start");
                        log::debug!("started!");
                    });
                }
            });
    }
}

fn render_animated_item(ui: &mut egui::Ui, icon: &str, label: &str, t: f32) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        ui.label(
            egui::RichText::new(icon)
                .size(24.0)
                .color(egui::Color32::from_rgb(85, 255, 85)),
        );

        if t > 0.1 {
            ui.scope(|ui| {
                let opacity = (t - 0.1) / 0.9;

                ui.set_clip_rect(ui.available_rect_before_wrap());

                ui.add_space(10.0);

                let text = egui::RichText::new(label)
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
