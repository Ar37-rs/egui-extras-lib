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
            let res = if let Ok(res) = ureq::get(&url).call() {
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

fn save_image(image_content: &[u8], image_url: &str) -> String {
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
        _image.write(image_content).unwrap();
        return format!("image saved to:\n{}", pth_buf.display());
    }
}

struct MyApp {
    total_image: u32,
    seed: Vec<u32>,
    counter: Vec<u32>,
    image_content: Vec<Vec<u8>>,
    raw_image: Vec<(TextureId, (f32, f32))>,
    network_image_loader: Vec<Option<Futurize<(Vec<u8>, NetworkImageInfo), String>>>,
    image_clicked: Vec<bool>,
    image_saved_info: Vec<String>,
    image_counter: Vec<u32>,
    label_info: Vec<String>,
    image_url: Vec<String>,
    cancel_image: Vec<bool>,
}

impl Default for MyApp {
    fn default() -> Self {
        let total_image: usize = 4;
        let mut seed = Vec::new();
        let mut counter = Vec::new();
        let mut image_content = Vec::new();
        let mut raw_image = Vec::new();
        let mut network_image_loader =
            Vec::<Option<Futurize<(Vec<u8>, NetworkImageInfo), String>>>::new();
        let mut image_clicked = Vec::new();
        let mut image_saved_info = Vec::new();
        let mut image_counter = Vec::new();
        let mut label_info = Vec::new();
        let mut image_url = Vec::new();
        let mut cancel_image = Vec::new();

        for mut i in 0..total_image {
            i += 1;
            seed.push(i as u32);
            counter.push(0);
            image_content.push(Vec::<u8>::new());
            raw_image.push((TextureId::default(), (0.0, 0.0)));
            network_image_loader.push(None);
            image_clicked.push(false);
            image_saved_info.push("".to_string());
            image_counter.push(0);
            label_info.push(
                "Image uninitialized, click 'next image' to init or load other network image."
                    .to_string(),
            );
            image_url.push("".to_string());
            cancel_image.push(false)
        }

        Self {
            total_image: total_image as u32,
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
            cancel_image,
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            total_image,
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
            cancel_image,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Network image quick demo");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("prev image").clicked() {
                    // prevent changing if the other task_network_image_loader is still running
                    for i in 0..*total_image as usize {
                        if !label_info[i].contains("Loading...") {
                            if seed[i] > *total_image {
                                seed[i] -= *total_image;
                                let width = 640;
                                let height = 480;
                                let url = format!(
                                    "https://picsum.photos/seed/{}/{}/{}",
                                    seed[i], width, height
                                );
                                network_image_loader[i] = Some(network_image(url))
                            } else {
                                label_info[i] =
                                    "image index is almost out of bound, try click 'next image'"
                                        .to_string()
                            }
                        }
                    }
                }

                if ui.button("next image").clicked() {
                    for i in 0..*total_image as usize {
                        // prevent changing if the other task_network_image_loader is still running
                        if !label_info[i].contains("Loading...") {
                            seed[i] += *total_image;
                            let width = 640;
                            let height = 480;
                            let url = format!(
                                "https://picsum.photos/seed/{}/{}/{}",
                                seed[i], width, height
                            );
                            network_image_loader[i] = Some(network_image(url))
                        }
                    }
                }
            });

            for i in 0..*total_image as usize {
                if let Some(task_network_image_loader) = &network_image_loader[i] {
                    if task_network_image_loader.is_in_progress() {
                        match task_network_image_loader.try_get() {
                            Progress::Current => {
                                counter[i] += 1;
                                label_info[i] = format!("Loading... {}\n", counter[i]);
                                if cancel_image[i] {
                                    task_network_image_loader.cancel()
                                }
                            }
                            Progress::Completed((bytes, image_info)) => {
                                // restore counter to default
                                counter[i] = 0;
                                if let Some(_image) = Image::new(&bytes) {
                                    label_info[i] = format!(
                                        "URL: {}\nContent-type: {}",
                                        image_info.url, image_info.content_type
                                    );
                                    image_url[i] = image_info.url;
                                    frame.tex_allocator().free(raw_image[i].0);
                                    image_content[i] = bytes;
                                    raw_image[i] = (_image.texture_id(frame), _image.size)
                                } else {
                                    label_info[i] = "Unable to read image content.".to_string()
                                }

                                network_image_loader[i] = None
                            }
                            Progress::Canceled => {
                                // restore counter to default
                                counter[i] = 0;
                                label_info[i] = "Loading image canceled!".to_string();
                                cancel_image[i] = false
                            }
                            Progress::Error(err_name) => {
                                // and restore counter to default here also.
                                counter[i] = 0;
                                label_info[i] = err_name;
                            }
                        }
                    }
                }
            }

            ui.vertical(|ui| {
                for i in 0..*total_image as usize {
                    ui.separator();
                    let label = Label::new(&*label_info[i].clone());
                    ui.add(label);
                    // Original image size
                    // let size: (f32, f32) = raw_image.1;
                    //
                    // just resize here for smaller image, 0.66x actual size
                    let size: (f32, f32) = (raw_image[i].1 .0 / 4.0, raw_image[i].1 .1 / 4.0);
                    ui.horizontal(|ui| {
                        let clickable_image = ui
                            .image(raw_image[i].0, size)
                            .interact(Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text("Image is clickable!, click to save the image.");

                        if clickable_image.clicked() {
                            // prevent changing if the other task_network_image_loader is still running and if image save info is still showing
                            if !image_clicked[i] && !label_info[i].contains("Loading...") {
                                image_saved_info[i] = save_image(&image_content[i], &image_url[i]);
                                image_clicked[i] = true
                            }
                        }

                        if clickable_image.hovered() {
                            if image_clicked[i] {
                                ui.label(image_saved_info[i].clone());
                                image_counter[i] += 1;
                                // show image save info until:
                                if image_counter[i] > 50 {
                                    image_counter[i] = 0;
                                    image_clicked[i] = false
                                }
                            }
                        }
                    });

                    if counter[i] > 0 {
                        if ui.button("cancel?").clicked() {
                            cancel_image[i] = true
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
