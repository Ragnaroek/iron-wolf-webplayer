mod app;

use app::{IWApp, UploadState};

#[cfg(not(feature = "web"))]
fn main() -> eframe::Result {
    use crate::app::load_upload_state;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    let _upload_state = load_upload_state();

    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(IWApp::new(cc, UploadState::no_upload())))),
    )
}

#[cfg(feature = "web")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        use crate::app::load_upload_state;

        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("iw_player_canvas")
            .expect("Failed to find iw_player_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("iw_player_canvas was not a HtmlCanvasElement");

        let upload_state = load_upload_state().await;

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(IWApp::new(cc, upload_state)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
