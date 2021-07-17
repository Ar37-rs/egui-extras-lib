use eframe::{
    egui::{self, FontDefinitions, FontFamily, Label, Sense, TextStyle, TextureId},
    epi,
};
use egui_extras_lib::{
    asynchron::{Futurize, Progress},
    Image,
};
use std::{
    env::current_dir,
    fs::OpenOptions,
    io::{Read, Write},
};

#[derive(Clone, Debug)]
struct NetworkImageInfo {
    url: String,
    content_type: String,
}

fn network_image(url: String) -> Futurize<(Vec<u8>, NetworkImageInfo), String> {
    let task = Futurize::task(
        0,
        move |_canceled| -> Progress<(Vec<u8>, NetworkImageInfo), String> {
            let req = ureq::get(&url);
            let res = if let Ok(res) = req.clone().call() {
                res
            } else {
                return Progress::Error("Network problem, unable to request url.".to_string());
            };

            // check if progress is canceled
            if _canceled.load(std::sync::atomic::Ordering::Relaxed) {
                return Progress::Canceled;
            }

            if res.status() == 200 {
                let img_info = NetworkImageInfo {
                    url: res.get_url().to_string(),
                    content_type: res.content_type().to_string(),
                };
                let mut buf = Vec::new();
                if let Err(_) = res.into_reader().read_to_end(&mut buf) {
                    return Progress::Error("Unable read image content".to_string());
                };

                // and check here also.
                if _canceled.load(std::sync::atomic::Ordering::Relaxed) {
                    return Progress::Canceled;
                } else {
                    return Progress::Completed((buf, img_info));
                }
            } else {
                return Progress::Error(format!("Networl error: {}", res.status()));
            }
        },
    );
    task.try_do();
    task
}

fn save_image(image_content: Vec<u8>, image_url: String) -> String {
    let mut pth_buf = current_dir().unwrap();
    let name: Vec<&str> = image_url.split("hmac=").collect();
    pth_buf.push(format!("{}.jpg", name[1][1..7].to_string()));
    if pth_buf.is_file() {
        return format!("{}\nimage name already exist!", pth_buf.display());
    } else {
        let mut _image = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&pth_buf)
            .unwrap();
        _image.write(&image_content).unwrap();
        return format!("image saved to:\n{}", pth_buf.display());
    }
}

struct MyApp {
    seed: u32,
    counter: u32,
    image_content: Vec<u8>,
    raw_image: (TextureId, (f32, f32)),
    network_image_loader: Option<Futurize<(Vec<u8>, NetworkImageInfo), String>>,
    image_clicked: bool,
    image_saved_info: String,
    image_counter: u32,
    label_info: String,
    image_url: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            seed: 1,
            counter: 0,
            image_content: Vec::new(),
            raw_image: (TextureId::default(), (0.0, 0.0)),
            network_image_loader: None,
            image_clicked: false,
            image_saved_info: "".to_string(),
            image_counter: 0,
            label_info:
                "Image uninitialized, click 'next image' to init or load other network image."
                    .to_string(),
            image_url: String::new(),
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            seed,
            counter,
            image_content,
            raw_image,
            network_image_loader,
            image_clicked,
            image_saved_info,
            image_counter,
            label_info,
            image_url,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Network image quick demo");

            ui.separator();
            ui.vertical(|ui| {
                let label = Label::new(&*label_info.clone());
                ui.add(label)
            });

            ui.horizontal(|ui| {
                if ui.button("prev image").clicked() {
                    // prevent changing if the other task_network_image_loader is still running
                    if !label_info.contains("Loading...") {
                        if *seed > 1 {
                            *seed -= 1;
                            let width = 640;
                            let height = 480;
                            let url =
                                format!("https://picsum.photos/seed/{}/{}/{}", seed, width, height);
                            *network_image_loader = Some(network_image(url))
                        } else {
                            *label_info =
                                "image index is almost out of bound, try click 'next image'"
                                    .to_string()
                        }
                    }
                }

                if ui.button("next image").clicked() {
                    // prevent changing if the other task_network_image_loader is still running
                    if !label_info.contains("Loading...") {
                        *seed += 1;
                        let width = 640;
                        let height = 480;
                        let url =
                            format!("https://picsum.photos/seed/{}/{}/{}", seed, width, height);
                        *network_image_loader = Some(network_image(url))
                    }
                }
            });

            if let Some(task_network_image_loader) = network_image_loader {
                if task_network_image_loader.is_in_progress() {
                    match task_network_image_loader.try_get() {
                        Progress::Current => {
                            *counter += 1;
                            *label_info = format!("Loading... {}\n", counter);
                            if ui.button("cancel?").clicked() {
                                task_network_image_loader.cancel()
                            }
                        }
                        Progress::Completed((bytes, image_info)) => {
                            // restore counter to default
                            *counter = 0;

                            if let Some(_image) = Image::new(&bytes) {
                                *label_info = format!(
                                    "URL: {}\nContent-type: {}",
                                    image_info.url, image_info.content_type
                                );
                                *image_url = image_info.url;
                                frame.tex_allocator().free(raw_image.0);
                                *image_content = bytes;
                                *raw_image = (_image.texture_id(frame), _image.size)
                            } else {
                                *label_info = "Unable to read image content.".to_string()
                            }

                            *network_image_loader = None
                        }
                        Progress::Canceled => {
                            // restore counter to default
                            *counter = 0;

                            *label_info = "Loading image canceled!".to_string()
                        }
                        Progress::Error(err_name) => {
                            // and restore counter to default here also.
                            *counter = 0;

                            *label_info = err_name
                        }
                    }
                }
            }

            //// original image size
            // let size: (f32, f32) = raw_image.1;
            //
            // just resize here for smaller image, 0.66x actual size
            let size: (f32, f32) = (raw_image.1 .0 / 1.5, raw_image.1 .1 / 1.5);

            ui.horizontal(|ui| {
                let clickable_image = ui
                    .image(raw_image.0, size)
                    .interact(Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Image is clickable!, click to save the image.");

                if clickable_image.clicked() {
                    // prevent changing if the other task_network_image_loader is still running and if image save info is still showing
                    if !*image_clicked && !label_info.contains("Loading...") {
                        *image_saved_info =
                            save_image(image_content.clone().to_vec(), image_url.clone());
                        *image_clicked = true
                    }
                }

                if clickable_image.hovered() {
                    if *image_clicked {
                        ui.label(image_saved_info.clone());
                        *image_counter += 1;
                        // show image save info until:
                        if *image_counter > 50 {
                            *image_counter = 0;
                            *image_clicked = false
                        }
                    }
                }
            });
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
        ctx.request_repaint()
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let mut fonts = FontDefinitions::default();
        fonts
            .family_and_size
            .insert(TextStyle::Button, (FontFamily::Monospace, 13.0));
        fonts
            .family_and_size
            .insert(TextStyle::Heading, (FontFamily::Proportional, 18.0));
        fonts
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Monospace, 13.0));
        fonts
            .family_and_size
            .insert(TextStyle::Small, (FontFamily::Monospace, 13.0));
        ctx.set_fonts(fonts.clone());
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::default()), options)
}
