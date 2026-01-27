use eframe::egui;
use egui::Frame;
use iw::config::default_iw_config;
use iw::start::iw_start;
use iw::web::{WebLoader, load_shareware_data};
use wasm_bindgen_futures::spawn_local;

pub struct IWApp {}

impl IWApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> IWApp {
        IWApp {}
    }
}

impl eframe::App for IWApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0.0,
                    egui::Color32::from_rgb(0x8C, 0, 0), // Custom color
                );
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
