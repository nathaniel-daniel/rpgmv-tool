#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use eframe::egui;
use egui::Align2;
use egui::Button;
use egui::Color32;
use egui::ColorImage;
use egui::FontFamily;
use egui::FontId;
use egui::TextFormat;
use egui::TextureHandle;
use egui::load::SizedTexture;
use egui::text::LayoutJob;
use egui::viewport::IconData;
use egui_toast::Toast;
use egui_toast::ToastKind;
use egui_toast::ToastOptions;
use egui_toast::Toasts;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

const TITLE: &str = "RPGMaker Image Viewer";

fn load_image(ctx: &egui::Context, path: &Path) -> anyhow::Result<TextureHandle> {
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;

    let mut encrypted_reader = rpgmvp::Reader::new(std::io::BufReader::new(file));
    let mut raw_image = Vec::with_capacity(usize::try_from(metadata.len())?);
    encrypted_reader.read_to_end(&mut raw_image)?;

    let image = image::load_from_memory(&raw_image)?;
    let rgba8_image = image.into_rgba8();

    let image_size = [rgba8_image.width() as _, rgba8_image.height() as _];
    let pixels = rgba8_image.as_flat_samples();
    let color_image = ColorImage::from_rgba_unmultiplied(image_size, pixels.as_slice());

    let texture_handle = ctx.load_texture("current-image", color_image, Default::default());

    anyhow::Ok(texture_handle)
}

enum Message {
    SelectedImageFile {
        path: Option<PathBuf>,
    },
    LoadedImage {
        result: anyhow::Result<egui::TextureHandle>,
    },
}

struct App {
    messages_rx: std::sync::mpsc::Receiver<Message>,
    messages_tx: std::sync::mpsc::Sender<Message>,
    toasts: Toasts,

    loading_image: bool,
    image: Option<(SizedTexture, TextureHandle)>,
}

impl App {
    fn new() -> Self {
        let (messages_tx, messages_rx) = std::sync::mpsc::channel();

        Self {
            messages_rx,
            messages_tx,
            toasts: Toasts::new()
                .anchor(Align2::LEFT_BOTTOM, (20.0, -20.0))
                .direction(egui::Direction::BottomUp),

            loading_image: false,
            image: None,
        }
    }

    fn load_image(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.loading_image = true;

        let ctx = ctx.clone();
        let messages_tx = self.messages_tx.clone();
        rayon::spawn(move || {
            let result = load_image(&ctx, &path)
                .with_context(|| format!("failed to open file \"{}\"", path.display()));

            let _ = messages_tx.send(Message::LoadedImage { result }).is_ok();
            ctx.request_repaint();
        });
    }

    fn process_message(&mut self, ctx: &egui::Context, message: Message) {
        match message {
            Message::SelectedImageFile { path } => {
                let path = match path {
                    Some(path) => path,
                    None => return,
                };

                self.load_image(ctx, path);
            }
            Message::LoadedImage { result } => {
                self.loading_image = false;

                let texture_handle = match result {
                    Ok(texture_handle) => texture_handle,
                    Err(error) => {
                        let mut job = LayoutJob::default();
                        job.append(
                            "Failed to load image\n",
                            0.0,
                            TextFormat {
                                font_id: FontId::new(14.0, FontFamily::Proportional),
                                color: Color32::WHITE,
                                ..Default::default()
                            },
                        );
                        job.append(
                            format!("{error:?}").as_str(),
                            0.0,
                            TextFormat {
                                font_id: FontId::new(15.0, FontFamily::Proportional),
                                color: Color32::WHITE,
                                ..Default::default()
                            },
                        );

                        self.toasts.add(Toast {
                            text: job.into(),
                            kind: ToastKind::Error,
                            options: ToastOptions::default()
                                .duration_in_seconds(5.0)
                                .show_progress(true),
                            ..Default::default()
                        });
                        return;
                    }
                };

                let sized_texture = egui::load::SizedTexture::from_handle(&texture_handle);
                self.image = Some((sized_texture, texture_handle));
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.messages_rx.try_recv() {
            self.process_message(ctx, message);
        }

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.add_enabled_ui(!self.loading_image, |ui| {
                        if ui.add(Button::new("Open")).clicked() {
                            self.loading_image = true;

                            let ctx = ctx.clone();
                            let messages_tx = self.messages_tx.clone();
                            rayon::spawn(move || {
                                let picked_file = rfd::FileDialog::new()
                                    .add_filter("RPGMaker MV Image File", &["rpgmvp"])
                                    .add_filter("All types", &["*"])
                                    .pick_file()
                                    .map(|file| file.as_path().to_path_buf());

                                let _ = messages_tx
                                    .send(Message::SelectedImageFile { path: picked_file })
                                    .is_ok();
                                ctx.request_repaint();
                            });
                        }
                    });
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.image.as_ref() {
            Some((sized_texture, _)) => {
                ui.centered_and_justified(|ui| {
                    ui.image(*sized_texture);
                });
            }
            None => {
                ui.heading(TITLE);
                ui.label("Welcome! Use File > Load to open an image.");
            }
        });

        self.toasts.show(ctx);
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let icon_raw = include_bytes!("../assets/icon.ico");
    let icon_image = image::load_from_memory(icon_raw)?;
    let icon_rgba8 = icon_image.into_rgba8();
    let icon = IconData {
        width: icon_rgba8.width(),
        height: icon_rgba8.height(),
        rgba: icon_rgba8.into_raw(),
    };

    let image_path = std::env::args().nth(1);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(icon),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(|ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);

            let mut app = Box::new(App::new());
            if let Some(image_path) = image_path {
                app.load_image(&ctx.egui_ctx, PathBuf::from(image_path));
            }

            Ok(app)
        }),
    )
    .map_err(|error| anyhow::Error::msg(error.to_string()))?;

    Ok(())
}
