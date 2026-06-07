#![cfg_attr(
    all(
        not(debug_assertions),
        windows,
        not(feature = "force-console-subsystem")
    ),
    windows_subsystem = "windows"
)]

use anyhow::Context;
use eframe::egui;
use eframe::egui_wgpu::WgpuConfiguration;
use eframe::egui_wgpu::WgpuSetup;
use eframe::egui_wgpu::WgpuSetupCreateNew;
use eframe::egui_wgpu::wgpu::wgt::PowerPreference;
use egui::Align2;
use egui::Button;
use egui::Color32;
use egui::ColorImage;
use egui::FontFamily;
use egui::FontId;
use egui::Rect;
use egui::TextFormat;
use egui::TextureHandle;
use egui::Ui;
use egui::containers::Scene;
use egui::load::SizedTexture;
use egui::text::LayoutJob;
use egui::viewport::IconData;
use egui_toast::Toast;
use egui_toast::ToastKind;
use egui_toast::ToastOptions;
use egui_toast::Toasts;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

const TITLE: &str = "RPGMaker Image Viewer";
const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const JPEG_MAGIC: &[u8] = &[0xff, 0xd8, 0xff];
const ENCRYPTERATOR_MAGIC: &[u8] = b"ART\0ENCRYPTER100FREE\0VERSION\0\0\0\0";

struct Image {
    sized_texture: SizedTexture,
    // This needs to be kept alive while we use the sized_texture.
    _texture_handle: TextureHandle,

    /// The previous image path, if it exists.
    prev_image_path: Option<PathBuf>,

    /// The next image path, if it exists.
    next_image_path: Option<PathBuf>,
}

fn load_image(ctx: &egui::Context, path: &Path) -> anyhow::Result<Image> {
    let file_name_os_str = path.file_name().context("missing file name")?;
    let file_name = file_name_os_str.to_string_lossy().to_lowercase();

    let mut file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;

    let mut magic = [0; 32];
    file.read_exact(&mut magic)?;
    file.seek(SeekFrom::Start(0))?;

    let mut file = std::io::BufReader::new(file);
    let rgba8_image = if magic.starts_with(PNG_MAGIC) || magic.starts_with(JPEG_MAGIC) {
        let mut raw_image = Vec::with_capacity(usize::try_from(metadata.len())?);
        file.read_to_end(&mut raw_image)?;

        let image = image::load_from_memory(&raw_image)?;
        image.into_rgba8()
    } else if magic.starts_with(ENCRYPTERATOR_MAGIC) {
        let mut encrypted_reader = encrypterator::Reader::new(file);
        encrypted_reader.guess_key()?;

        let mut raw_image = Vec::with_capacity(usize::try_from(metadata.len())?);
        encrypted_reader.read_to_end(&mut raw_image)?;

        let image = image::load_from_memory(&raw_image)?;
        image.into_rgba8()
    } else {
        let mut encrypted_reader = rpgmvp::Reader::new(file);
        let mut raw_image = Vec::with_capacity(usize::try_from(metadata.len())?);
        encrypted_reader.read_to_end(&mut raw_image)?;

        let image = image::load_from_memory(&raw_image)?;
        image.into_rgba8()
    };

    let image_size = [rgba8_image.width() as _, rgba8_image.height() as _];
    let pixels = rgba8_image.as_flat_samples();
    let color_image = ColorImage::from_rgba_unmultiplied(image_size, pixels.as_slice());

    let texture_handle = ctx.load_texture(
        format!("image-{file_name}"),
        color_image,
        Default::default(),
    );
    let sized_texture = SizedTexture::from_handle(&texture_handle);

    // If this fails, just don't populate the prev/next image fields.
    let (prev_image_path, next_image_path) = (|| {
        let parent_dir = match path.parent() {
            Some(parent_dir) => parent_dir,
            None => return Ok((None, None)),
        };

        let mut prev_path = None;
        let mut next_path = None;
        for dir_entry in std::fs::read_dir(parent_dir)? {
            let dir_entry = dir_entry?;
            let dir_entry_file_name = dir_entry.file_name();

            if dir_entry_file_name == file_name_os_str {
                continue;
            }

            if dir_entry_file_name < file_name_os_str
                && prev_path.as_ref().map_or(true, |(prev_file_name, _)| {
                    *prev_file_name < dir_entry_file_name
                })
            {
                prev_path = Some((dir_entry_file_name, dir_entry.path()));
            } else if dir_entry_file_name > file_name_os_str
                && next_path.as_ref().map_or(true, |(next_file_name, _)| {
                    *next_file_name > dir_entry_file_name
                })
            {
                next_path = Some((dir_entry_file_name, dir_entry.path()));
            }
        }

        anyhow::Ok((
            prev_path.map(|(_, path)| path.to_path_buf()),
            next_path.map(|(_, path)| path.to_path_buf()),
        ))
    })()
    .unwrap_or((None, None));

    let image = Image {
        sized_texture,
        _texture_handle: texture_handle,

        prev_image_path,
        next_image_path,
    };

    anyhow::Ok(image)
}

enum Message {
    SelectedImageFile { path: Option<PathBuf> },
    LoadedImage { result: anyhow::Result<Image> },
}

struct App {
    messages_rx: std::sync::mpsc::Receiver<Message>,
    messages_tx: std::sync::mpsc::Sender<Message>,
    toasts: Toasts,

    loading_image: bool,
    image: Option<Image>,
    scene_rect: Rect,
}

impl App {
    fn new() -> Self {
        let (messages_tx, messages_rx) = std::sync::mpsc::channel();
        let toasts = Toasts::new()
            .anchor(Align2::LEFT_BOTTOM, (20.0, -20.0))
            .direction(egui::Direction::BottomUp);

        Self {
            messages_rx,
            messages_tx,
            toasts,

            loading_image: false,
            image: None,
            scene_rect: Rect::ZERO,
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
                    None => {
                        self.loading_image = false;
                        return;
                    },
                };

                self.load_image(ctx, path);
            }
            Message::LoadedImage { result } => {
                self.loading_image = false;

                let image = match result {
                    Ok(texture_handle) => texture_handle,
                    Err(error) => {
                        let mut job = LayoutJob::default();
                        job.append(
                            "Failed to load image\n",
                            0.0,
                            TextFormat {
                                font_id: FontId::new(15.0, FontFamily::Proportional),
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

                self.image = Some(image);
                self.scene_rect = Rect::ZERO;
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.messages_rx.try_recv() {
            self.process_message(ui, message);
        }

        if let Some(image) = self.image.as_ref() {
            let (left_arrow_pressed, right_arrow_pressed) = ui.input_mut(|i| {
                (
                    i.key_pressed(egui::Key::ArrowLeft),
                    i.key_pressed(egui::Key::ArrowRight),
                )
            });
            if left_arrow_pressed && let Some(prev_image_path) = image.prev_image_path.clone() {
                self.load_image(ui, prev_image_path);
            } else if right_arrow_pressed
                && let Some(next_image_path) = image.next_image_path.clone()
            {
                self.load_image(ui, next_image_path);
            }
        }

        if !self.loading_image {
            let dropped_file = ui.input(|input| {
                // Only handle the first file sice we can only open one at a time right now.
                // TODO: Add tabs for multiple images or ignore multiple files?
                let dropped_file = input.raw.dropped_files.first()?;

                dropped_file.path.clone()
            });
            if let Some(dropped_file) = dropped_file {
                self.load_image(ui, dropped_file);
            }
        }

        egui::Panel::top("my_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.add_enabled_ui(!self.loading_image, |ui| {
                        if ui.add(Button::new("Open")).clicked() {
                            ui.close();

                            self.loading_image = true;

                            let ctx = ui.ctx().clone();
                            let messages_tx = self.messages_tx.clone();
                            rayon::spawn(move || {
                                let picked_file = rfd::FileDialog::new()
                                    .add_filter("RPGMaker Image Files", &["rpgmvp", "png_"])
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

                    ui.add_enabled_ui(!self.loading_image && self.image.is_some(), |ui| {
                        if ui.add(Button::new("Close")).clicked() {
                            self.image = None;
                        }
                    });
                });
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.image.as_ref() {
            Some(image) => {
                let response = Scene::new().zoom_range(f32::EPSILON..=50.0_f32).show(
                    ui,
                    &mut self.scene_rect,
                    |ui| ui.add(egui::Image::new(image.sized_texture)),
                );
                if response.response.double_clicked() {
                    self.scene_rect = Rect::ZERO;
                }
            }
            None => {
                ui.heading(TITLE);
                ui.label("Welcome!");
                ui.label("Use File > Load to open an image.");
                ui.label("You can also drag and drop an image onto this window to load it.");
                ui.label("This program was created by Nathaniel Daniel.");
            }
        });

        self.toasts.show(ui);
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
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_drag_and_drop(true),
        centered: true,
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                // I tried to switch to `LowPower` (from its default of `HighPerformance`) to get the Nvidia app to not detect this as a game.
                // Not only did this not work, but this also created a lot of stuttering when moving or resizing the window.
                // As a result, force `HighPerformance`.
                power_preference: PowerPreference::HighPerformance,
                ..WgpuSetupCreateNew::without_display_handle()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut app = Box::new(App::new());
            if let Some(image_path) = image_path {
                let image_path = PathBuf::from(image_path);
                app.load_image(&cc.egui_ctx, image_path);
            }

            Ok(app)
        }),
    )
    .map_err(|error| anyhow::Error::msg(error.to_string()))?;

    Ok(())
}
