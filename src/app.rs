use eframe::egui;
use egui::{Color32, Frame, Pos2, Rect, RichText, Stroke};
use iw::config::default_iw_config;
use iw::loader::Loader;
use iw::start::iw_start;
use iw::web::load_shareware_data;
use js_sys::Uint8Array;
use poll_promise::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, KeyboardEvent, window};

const KEYDOWN_EVENT: &str = "keydown";
const KEYUP_EVENT: &str = "keyup";
const CONTROL_KEY: &str = "Control";
const KEYUP_DELAY_MS: i32 = 15;
const PLAYER_DB_NAME: &str = "iron-wolf-player";
const PLAYER_STORE: &str = "files";

// files
const AUDIOHED_PREFIX: &str = "AUDIOHED.WL";
const AUDIOT_PREFIX: &str = "AUDIOT.WL";
const CONFIG_PREFIX: &str = "CONFIG.WL";
const GAMEMAPS_PREFIX: &str = "GAMEMAPS.WL";
const MAPHEAD_PREFIX: &str = "MAPHEAD.WL";
const VGADICT_PREFIX: &str = "VGADICT.WL";
const VGAGRAPH_PREFIX: &str = "VGAGRAPH.WL";
const VGAHEAD_PREFIX: &str = "VGAHEAD.WL";
const VSWAP_PREFIX: &str = "VSWAP.WL";

// files None means use the shareware version
pub struct UploadState {
    files: Option<UploadStateFiles>,
}

impl UploadState {
    pub fn no_upload() -> UploadState {
        UploadState { files: None }
    }

    fn with_files(files: UploadStateFiles) -> UploadState {
        UploadState { files: Some(files) }
    }

    fn is_complete(&self) -> bool {
        self.files.as_ref().is_some_and(|f| f.is_complete())
    }

    fn create_loader(&self) -> Loader {
        if let Some(files) = &self.files {
            let variant = match files.version {
                3 => &iw::assets::W3D3,
                6 => &iw::assets::W3D6,
                _ => panic!("unknow version: {}", files.version),
            };
            let mut loader = Loader::new_empty(variant);

            if let Some(data) = &files.vgadict {
                let file_name = file_name(VGADICT_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.vgahead {
                let file_name = file_name(VGAHEAD_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.vgagraph {
                let file_name = file_name(VGAGRAPH_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.maphead {
                let file_name = file_name(MAPHEAD_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.gamemaps {
                let file_name = file_name(GAMEMAPS_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.vswap {
                let file_name = file_name(VSWAP_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.config {
                let file_name = file_name(CONFIG_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.audiohed {
                let file_name = file_name(AUDIOHED_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            if let Some(data) = &files.audiot {
                let file_name = file_name(AUDIOT_PREFIX, files.version);
                loader.load(file_name, data.to_vec());
            }

            loader
        } else {
            Loader::new_shareware()
        }
    }
}

struct UploadStateFiles {
    version: usize, // 3 or 6
    audiohed: Option<Uint8Array>,
    gamemaps: Option<Uint8Array>,
    audiot: Option<Uint8Array>,
    config: Option<Uint8Array>,
    maphead: Option<Uint8Array>,
    vgadict: Option<Uint8Array>,
    vgagraph: Option<Uint8Array>,
    vgahead: Option<Uint8Array>,
    vswap: Option<Uint8Array>,
}

impl UploadStateFiles {
    pub fn new_empty(version: usize) -> UploadStateFiles {
        UploadStateFiles {
            version: version,
            audiohed: None,
            gamemaps: None,
            audiot: None,
            config: None,
            maphead: None,
            vgadict: None,
            vgagraph: None,
            vgahead: None,
            vswap: None,
        }
    }

    pub fn reset(&mut self, version: usize) {
        self.audiohed = None;
        self.gamemaps = None;
        self.audiot = None;
        self.config = None;
        self.maphead = None;
        self.vgadict = None;
        self.vgagraph = None;
        self.vgahead = None;
        self.vswap = None;
        self.version = version;
    }

    pub fn is_complete(&self) -> bool {
        self.audiohed.is_some()
            && self.gamemaps.is_some()
            && self.audiot.is_some()
            && self.config.is_some()
            && self.maphead.is_some()
            && self.vgadict.is_some()
            && self.vgagraph.is_some()
            && self.vgahead.is_some()
            && self.vswap.is_some()
    }
}

pub struct FileUpload {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub struct IWApp {
    is_expanded: bool,
    playing: bool,

    file_upload_promise: Option<Promise<Vec<FileUpload>>>,
    upload: UploadState,

    confirm_reset: Option<Rect>,

    //settings
    show_frame_rate: bool,
}

impl eframe::App for IWApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_file_upload();
        self.forward_key_events(ctx);

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
                    let icon = if self.is_expanded { "▶" } else { "◀   " };
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

                render_savegame_download(ui, t);
                self.render_file_upload(ui, t);
                self.render_settings(ui);

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
                    ui.add_space(current_width); // to keep the play button centred
                    ui.vertical_centered(|ui| {
                        if !self.playing {
                            let play_response = ui.label(
                                RichText::new(egui_phosphor::regular::PLAY)
                                    .size(30.0)
                                    .color(ICON_COLOUR),
                            );

                            if play_response.clicked() {
                                self.playing = true;
                                let window = window().expect("No window object found");
                                let document = window.document().expect("No document object found");

                                let element = document
                                    .get_element_by_id("vga")
                                    .expect("Element not found");
                                if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
                                    html_element.focus().expect("Failed to focus element");
                                }

                                let mut loader = self.upload.create_loader();
                                let load_shareware = !self.upload.is_complete();

                                let show_frame_rate = self.show_frame_rate;
                                spawn_local(async move {
                                    let mut iw_config =
                                        default_iw_config().expect("default config");
                                    iw_config.options.show_frame_rate = show_frame_rate;
                                    if load_shareware {
                                        load_shareware_data(&mut loader)
                                            .await
                                            .expect("load shareware data");
                                    }
                                    iw_start(loader, iw_config).expect("iw start");
                                });
                            }

                            if play_response.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            if self.upload.is_complete() {
                                ui.label(
                                    RichText::new("(Uploaded Version)")
                                        .size(10.0)
                                        .color(ICON_COLOUR),
                                );
                            } else {
                                ui.label(
                                    RichText::new("(Shareware Version)")
                                        .size(10.0)
                                        .color(ICON_COLOUR),
                                );
                            }
                        }
                    })
                });
            });

        if let Some(pos) = self.confirm_reset {
            let dialog_pos = egui::pos2(pos.left(), pos.bottom() + 50.0);
            egui::Window::new("Confirm reset")
                .collapsible(false)
                .resizable(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(dialog_pos)
                .show(ctx, |ui| {
                    ui.label("Delete all uploaded data and reset to shareware?");

                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            spawn_local(async {
                                reset_files_indexeddb().await.expect("file reset");
                            });
                            self.confirm_reset = None;
                            self.upload.files = None;
                        }
                        if ui.button("No").clicked() {
                            self.confirm_reset = None;
                        }
                    });
                });
        }
    }
}

impl IWApp {
    pub fn new(cc: &eframe::CreationContext<'_>, upload_state: UploadState) -> IWApp {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        register_ctrl_handler();

        IWApp {
            is_expanded: false,
            playing: false,

            file_upload_promise: None,
            upload: upload_state,

            confirm_reset: None,

            show_frame_rate: false,
        }
    }

    fn handle_file_upload(&mut self) {
        if let Some(upload_promise) = &self.file_upload_promise {
            if let Some(file_uploads) = upload_promise.ready() {
                if file_uploads.is_empty() {
                    return;
                }

                let version = if file_uploads[0].name.ends_with("WL6") {
                    6
                } else if file_uploads[0].name.ends_with("WL3") {
                    3
                } else {
                    return;
                };

                if let Some(files) = &mut self.upload.files {
                    if files.version != version {
                        files.reset(version);
                    }
                } else {
                    self.upload.files = Some(UploadStateFiles::new_empty(version));
                }

                for file_upload in file_uploads {
                    let data = Uint8Array::from(file_upload.bytes.as_slice());

                    if file_upload.name == file_name(GAMEMAPS_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().gamemaps = Some(data);
                    } else if file_upload.name == file_name(AUDIOHED_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().audiohed = Some(data);
                    } else if file_upload.name == file_name(AUDIOT_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().audiot = Some(data);
                    } else if file_upload.name == file_name(CONFIG_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().config = Some(data);
                    } else if file_upload.name == file_name(MAPHEAD_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().maphead = Some(data);
                    } else if file_upload.name == file_name(VGADICT_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().vgadict = Some(data);
                    } else if file_upload.name == file_name(VGAGRAPH_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().vgagraph = Some(data);
                    } else if file_upload.name == file_name(VGAHEAD_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().vgahead = Some(data);
                    } else if file_upload.name == file_name(VSWAP_PREFIX, version) {
                        self.upload.files.as_mut().unwrap().vswap = Some(data);
                    } else {
                        continue; // ignore all other files
                    }

                    let file_name_db = file_upload.name.clone(); // clone for async move
                    let data_db = Uint8Array::from(file_upload.bytes.as_slice());
                    spawn_local(async move {
                        store_file_indexeddb(&file_name_db, data_db)
                            .await
                            .expect("file store");
                    });
                }

                self.file_upload_promise = None;
            }
        }
    }

    fn render_file_upload(&mut self, ui: &mut egui::Ui, t: f32) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.label(
                RichText::new(egui_phosphor::regular::UPLOAD_SIMPLE)
                    .size(24.0)
                    .color(ICON_COLOUR),
            );

            if t > 0.1 {
                ui.scope(|ui| {
                    let opacity = (t - 0.1) / 0.9;
                    ui.set_clip_rect(ui.available_rect_before_wrap());
                    ui.add_space(10.0);

                    let text = RichText::new("UPLOAD")
                        .strong()
                        .color(egui::Color32::WHITE.linear_multiply(opacity));

                    if ui.button(text).clicked() {
                        let egui_ctx = ui.ctx().clone();
                        self.file_upload_promise =
                            Some(poll_promise::Promise::spawn_local(async move {
                                let file_uploads = open_files().await;
                                egui_ctx.request_repaint(); // Wake ui thread
                                file_uploads
                            }));
                    }

                    let reset_button = ui.button("RESET");
                    if reset_button.clicked() {
                        self.confirm_reset = Some(reset_button.rect);
                    };
                });
            }
        });

        if self.is_expanded {
            if let Some(files) = &self.upload.files {
                file_upload_status(
                    ui,
                    &file_name(AUDIOHED_PREFIX, files.version),
                    files.audiohed.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(AUDIOT_PREFIX, files.version),
                    files.audiot.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(CONFIG_PREFIX, files.version),
                    files.config.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(GAMEMAPS_PREFIX, files.version),
                    files.gamemaps.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(MAPHEAD_PREFIX, files.version),
                    files.maphead.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(VGADICT_PREFIX, files.version),
                    files.vgadict.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(VGAGRAPH_PREFIX, files.version),
                    files.vgagraph.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(VGAHEAD_PREFIX, files.version),
                    files.vgahead.is_some(),
                );
                file_upload_status(
                    ui,
                    &file_name(VSWAP_PREFIX, files.version),
                    files.vswap.is_some(),
                );
            } else {
                // shareware is always available
                file_upload_status(ui, &file_name(AUDIOHED_PREFIX, 1), true);
                file_upload_status(ui, &file_name(AUDIOT_PREFIX, 1), true);
                file_upload_status(ui, &file_name(CONFIG_PREFIX, 1), true);
                file_upload_status(ui, &file_name(GAMEMAPS_PREFIX, 1), true);
                file_upload_status(ui, &file_name(MAPHEAD_PREFIX, 1), true);
                file_upload_status(ui, &file_name(VGADICT_PREFIX, 1), true);
                file_upload_status(ui, &file_name(VGAGRAPH_PREFIX, 1), true);
                file_upload_status(ui, &file_name(VGAHEAD_PREFIX, 1), true);
                file_upload_status(ui, &file_name(VSWAP_PREFIX, 1), true);
            };
        }

        ui.add_space(15.0);
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.label(
                RichText::new(egui_phosphor::regular::GEAR)
                    .size(24.0)
                    .color(ICON_COLOUR),
            );
            if self.is_expanded {
                ui.label(RichText::new("SETTINGS").size(16.0).color(ICON_COLOUR));
            }
        });

        if self.is_expanded {
            ui.horizontal(|ui| {
                ui.add_space(25.0);
                ui.checkbox(
                    &mut self.show_frame_rate,
                    RichText::new("Show framerate").color(ICON_COLOUR),
                );
            });
        }

        ui.add_space(15.0);
    }

    fn forward_key_events(&self, ctx: &egui::Context) {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                let input = ctx.input(|i| i.clone());
                for event in &input.events {
                    if let egui::Event::Key { key, pressed, .. } = event {
                        let init = web_sys::KeyboardEventInit::new();
                        init.set_key(egui_key_to_event_key(key));
                        init.set_bubbles(true);
                        init.set_cancelable(true);

                        let event_type = if *pressed { KEYDOWN_EVENT } else { KEYUP_EVENT };
                        let event =
                            KeyboardEvent::new_with_keyboard_event_init_dict(event_type, &init)
                                .expect("event");

                        let vga = document
                            .get_element_by_id("vga")
                            .expect("Element not found");

                        vga.dispatch_event(&event).expect("event dispatch");
                    }
                }
            }
        }
    }
}

pub async fn open_files() -> Vec<FileUpload> {
    let files = rfd::AsyncFileDialog::new().pick_files().await.unwrap();

    let mut result = Vec::with_capacity(files.len());
    for file in files {
        let bytes = file.read().await;
        result.push(FileUpload {
            name: file.file_name(),
            bytes,
        })
    }
    result
}

pub async fn load_upload_state() -> UploadState {
    let mut state = UploadStateFiles::new_empty(0);
    let mut v = 1;
    let (audiohed, v_l) = load_file(AUDIOHED_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.audiohed = audiohed;
    }
    let (gamemaps, v_l) = load_file(GAMEMAPS_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.gamemaps = gamemaps;
    }
    let (audiot, v_l) = load_file(AUDIOT_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.audiot = audiot;
    }
    let (config, v_l) = load_file(CONFIG_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.config = config;
    }
    let (maphead, v_l) = load_file(MAPHEAD_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.maphead = maphead;
    }
    let (vgadict, v_l) = load_file(VGADICT_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.vgadict = vgadict;
    }
    let (vgagraph, v_l) = load_file(VGAGRAPH_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.vgagraph = vgagraph;
    }
    let (vgahead, v_l) = load_file(VGAHEAD_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.vgahead = vgahead;
    }
    let (vswap, v_l) = load_file(VSWAP_PREFIX).await;
    if v_l >= v {
        v = v_l;
        state.vswap = vswap;
    }

    state.version = v;
    if v > 1 {
        return UploadState::with_files(state);
    }

    // fall back to shareware version
    UploadState::no_upload()
}

async fn load_file(file_prefix: &str) -> (Option<Uint8Array>, usize) {
    let file = &file_name(file_prefix, 6);
    let file_6 = load_file_indexeddb(&file).await;
    if file_6.is_ok() {
        return (Some(file_6.unwrap()), 6);
    }

    // try WL3
    let file = &file_name(file_prefix, 3);
    let file_3 = load_file_indexeddb(&file).await;
    if file_3.is_ok() {
        return (Some(file_3.unwrap()), 3);
    }

    (None, 0)
}

fn file_name(prefix: &str, version: usize) -> String {
    format!("{}{}", prefix, version)
}

fn register_ctrl_handler() {
    {
        let closure_keydown = Closure::wrap(Box::new(|e: web_sys::KeyboardEvent| {
            if e.ctrl_key() && !e.shift_key() && !e.alt_key() && !e.meta_key() {
                {
                    let init = web_sys::KeyboardEventInit::new();
                    init.set_ctrl_key(true);
                    init.set_key(CONTROL_KEY);
                    let event =
                        KeyboardEvent::new_with_keyboard_event_init_dict(KEYDOWN_EVENT, &init)
                            .expect("event");
                    window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("vga")
                        .expect("Element not found")
                        .dispatch_event(&event)
                        .expect("event dispatch");
                }

                let closure = Closure::once(move || {
                    let init = web_sys::KeyboardEventInit::new();
                    init.set_ctrl_key(true);
                    init.set_key(CONTROL_KEY);
                    let event =
                        KeyboardEvent::new_with_keyboard_event_init_dict(KEYUP_EVENT, &init)
                            .expect("event");
                    window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("vga")
                        .expect("Element not found")
                        .dispatch_event(&event)
                        .expect("event dispatch");
                });
                window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        KEYUP_DELAY_MS,
                    )
                    .unwrap();
                closure.forget();
            }
        }) as Box<dyn FnMut(_)>);
        window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback(
                KEYDOWN_EVENT,
                closure_keydown.as_ref().unchecked_ref(),
            )
            .unwrap();
        closure_keydown.forget();
    }
}

const MENUE_MIN_WDITH: f32 = 50.0;
const MENUE_BORDER_WIDTH: f32 = 2.0;

const BACKGROUND_COLOR: Color32 = egui::Color32::from_rgb(0x88, 0, 0);
const MENU_BORDER_COLOUR_BOTTOM_RIGHT: Color32 = egui::Color32::from_rgb(0xD4, 0x00, 0x00);
const ICON_COLOUR: Color32 = egui::Color32::from_rgb(0xFC, 0xFC, 0x54);

fn file_upload_status(ui: &mut egui::Ui, file_name: &str, present: bool) {
    ui.horizontal(|ui| {
        ui.add_space(25.0);
        if present {
            ui.label(
                RichText::new(egui_phosphor::regular::CHECK_FAT)
                    .size(16.0)
                    .color(ICON_COLOUR),
            );
        } else {
            ui.label(
                RichText::new(egui_phosphor::regular::X)
                    .size(16.0)
                    .color(BACKGROUND_COLOR),
            );
        }
        ui.label(RichText::new(file_name).color(ICON_COLOUR));
    });
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

fn render_savegame_download(ui: &mut egui::Ui, t: f32) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        ui.label(
            RichText::new(egui_phosphor::regular::FLOPPY_DISK)
                .size(24.0)
                .color(ICON_COLOUR),
        );

        if t > 0.1 {
            ui.scope(|ui| {
                // TODO render save-game files for download
            });
        }
    });
    ui.add_space(15.0);
}

async fn store_file_indexeddb(file_name: &str, data: Uint8Array) -> Result<(), JsValue> {
    let db = open_db().await?;
    let transaction =
        db.transaction_with_str_and_mode(PLAYER_STORE, web_sys::IdbTransactionMode::Readwrite)?;

    let store = transaction.object_store(PLAYER_STORE)?;
    idb_request_await(&store.put_with_key(&data, &file_name.into())?)
        .await
        .map_err(|_| "idb store failed")?;
    Ok(())
}

async fn reset_files_indexeddb() -> Result<(), JsValue> {
    let db = open_db().await?;
    let transaction =
        db.transaction_with_str_and_mode(PLAYER_STORE, web_sys::IdbTransactionMode::Readwrite)?;

    let store = transaction.object_store(PLAYER_STORE)?;
    store.clear()?;
    Ok(())
}

async fn load_file_indexeddb(file_name: &str) -> Result<Uint8Array, JsValue> {
    let db = open_db().await?;
    let transaction =
        db.transaction_with_str_and_mode(PLAYER_STORE, web_sys::IdbTransactionMode::Readwrite)?;

    let store = transaction.object_store(PLAYER_STORE)?;
    let value = idb_request_await(&store.get(&file_name.into())?)
        .await
        .map_err(|_| "idb load failed")?;
    if value.is_undefined() {
        Err(JsValue::NULL)
    } else {
        let uint8_array = Uint8Array::new(&value);
        Ok(uint8_array)
    }
}

async fn open_db() -> Result<web_sys::IdbDatabase, JsValue> {
    let window = web_sys::window().expect("global window access");
    let factory = window.indexed_db().map_err(|e| e)?;
    if let Some(factory) = factory {
        let open_request = factory.open_with_u32(PLAYER_DB_NAME, 2)?;

        let db_promise = js_sys::Promise::new(&mut |resolve, reject| {
            let onsuccess = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let db = web_sys::IdbDatabase::from(
                    js_sys::Reflect::get(&event, &JsValue::from_str("target"))
                        .unwrap()
                        .dyn_into::<web_sys::IdbOpenDbRequest>()
                        .unwrap()
                        .result()
                        .unwrap(),
                );
                resolve.call1(&JsValue::NULL, &db).unwrap();
            }) as Box<dyn FnMut(_)>);

            let onerror = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let error = "opening IndexDB failed".into();
                reject.call1(&JsValue::NULL, &error).unwrap();
            }) as Box<dyn FnMut(_)>);

            let onupgradeneeded = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let db = web_sys::IdbDatabase::from(
                    js_sys::Reflect::get(&event, &JsValue::from_str("target"))
                        .unwrap()
                        .dyn_into::<web_sys::IdbOpenDbRequest>()
                        .unwrap()
                        .result()
                        .unwrap(),
                );

                if !db.object_store_names().contains(PLAYER_STORE) {
                    db.create_object_store(PLAYER_STORE)
                        .expect("created save store");
                }
            }) as Box<dyn FnMut(_)>);

            open_request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
            open_request.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            open_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));
            onsuccess.forget();
            onerror.forget();
            onupgradeneeded.forget();
        });

        let db = JsFuture::from(db_promise).await?;
        let db = web_sys::IdbDatabase::from(db);
        Ok(db)
    } else {
        Err("could not access IndexDB".into())
    }
}

async fn idb_request_await(request: &web_sys::IdbRequest) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let on_success = Closure::once(move |_: web_sys::Event| {
            resolve.call0(&JsValue::NULL).unwrap();
        });
        let on_error = Closure::once(move |e: JsValue| {
            reject.call1(&JsValue::NULL, &e).unwrap();
        });

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        on_success.forget();
        on_error.forget();
    });
    JsFuture::from(promise).await?;
    request.result()
}
