#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use iced::Center;
use iced::Element;
use iced::Length;
use iced::Task;
use iced::advanced::image::Handle as IcedImageHandle;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::Row;
use iced::widget::button;
use iced::widget::column;
use iced::widget::image::Viewer as ImageViewer;
use iced::widget::row;
use iced::window::Settings as WindowSettings;
use iced_aw::Menu;
use iced_aw::MenuBar;
use iced_aw::menu::Item as MenuItem;
use iced_toasts::ToastContainer;
use iced_toasts::ToastId;
use iced_toasts::ToastLevel;
use iced_toasts::toast;
use iced_toasts::toast_container;
use nd_util::ArcAnyhowError;
use std::io::Read;
use std::path::PathBuf;

async fn open_image_file_picker() -> Option<PathBuf> {
    let picked_file = rfd::AsyncFileDialog::new()
        .add_filter("RPGMaker MV Image File", &["rpgmvp"])
        .add_filter("All types", &["*"])
        .pick_file()
        .await?;

    Some(picked_file.path().into())
}

async fn load_image(path: PathBuf) -> Result<IcedImageHandle, ArcAnyhowError> {
    let path1 = path.clone();
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(path1)?;
        let metadata = file.metadata()?;

        let mut encrypted_reader = rpgmvp::Reader::new(std::io::BufReader::new(file));
        let mut raw_image = Vec::with_capacity(usize::try_from(metadata.len())?);
        encrypted_reader.read_to_end(&mut raw_image)?;

        let image = image::load_from_memory(&raw_image)?;
        let rgba8_image = image.into_rgba8();

        let handle = IcedImageHandle::from_rgba(
            rgba8_image.width(),
            rgba8_image.height(),
            rgba8_image.into_raw(),
        );

        anyhow::Ok(handle)
    })
    .await
    .context("failed to join task")
    .and_then(std::convert::identity)
    .with_context(|| format!("failed to open file \"{}\"", path.display()))
    .map_err(ArcAnyhowError::new)
}

#[derive(Debug, Clone)]
enum Message {
    OpenImageFilePicker,
    SelectedImageFile(Option<PathBuf>),
    ImageLoaded(Result<IcedImageHandle, ArcAnyhowError>),

    DismissToast(ToastId),
}

struct App<'a> {
    loading_image: bool,
    image: Option<IcedImageHandle>,

    toasts: ToastContainer<'a, Message>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            loading_image: false,
            image: None,

            toasts: toast_container(Message::DismissToast)
                .alignment_x(iced_toasts::alignment::Horizontal::Left)
                .alignment_y(iced_toasts::alignment::Vertical::Bottom),
        }
    }
}

impl App<'_> {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenImageFilePicker => {
                if self.loading_image {
                    return Task::none();
                }

                Task::perform(open_image_file_picker(), Message::SelectedImageFile)
            }
            Message::SelectedImageFile(result) => {
                if self.loading_image {
                    return Task::none();
                }
                let path = match result {
                    Some(path) => path,
                    None => return Task::none(),
                };

                self.loading_image = true;
                Task::perform(load_image(path), Message::ImageLoaded)
            }
            Message::ImageLoaded(result) => {
                self.loading_image = false;
                let image = match result {
                    Ok(image) => image,
                    Err(error) => {
                        self.toasts.push(
                            toast(&format!("{error:?}"))
                                .title("Failed to load image")
                                .level(ToastLevel::Error),
                        );
                        return Task::none();
                    }
                };
                self.image = Some(image);

                Task::none()
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss(id);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let main_content = match self.image.as_ref() {
            Some(image) => Container::new(
                ImageViewer::new(image)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
            None => Container::new(Container::new(
                button("Load Image").on_press(Message::OpenImageFilePicker),
            ))
            .center(Length::Fill),
        };

        self.toasts.view(main_content)
        
    }
}

fn main() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("WGPU_POWER_PREF", "low");

        // Iced resizes poorly on the default "vulkan" backend.
        // The "gl" backend works the best.
        // The "dx12" backend shows some flickering while resizing.
        // The "metal" backend is unavailable.
        #[cfg(windows)]
        std::env::set_var("WGPU_BACKEND", "gl");
    }

    let file = std::env::args().nth(1);
    dbg!(file);

    let icon_raw = include_bytes!("../assets/icon.ico");
    let icon = iced::window::icon::from_file_data(icon_raw, None)?;

    let title = "RPGMaker Image Viewer";
    let window_settings = WindowSettings {
        icon: Some(icon),
        ..Default::default()
    };
    iced::application(title, App::update, App::view)
        .window(window_settings)
        .run_with(|| (App::default(), Task::none()))?;

    Ok(())
}
