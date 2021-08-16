use eframe::{
    egui::{self, FontDefinitions, FontFamily, Label, Sense, TextStyle, TextureId},
    epi,
};
use egui_extras_lib::{
    asynchron::{Futurize, Futurized, InnerTaskHandle, Progress},
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

fn network_image(url: String) -> Futurized<(), (Vec<u8>, NetworkImageInfo)> {
    Futurize::task(
        0,
        move |_task: InnerTaskHandle| -> Progress<(), (Vec<u8>, NetworkImageInfo)> {
            let res = if let Ok(res) = ureq::get(&url).call() {
                res
            } else {
                return Progress::Error(format!(
                    "Network problem, unable to request url: {}",
                    &url
                ).into());
            };

            // check if progress is canceled
            if _task.is_canceled() {
                return Progress::Canceled;
            }

            if res.status() == 200 {
                let img_info = NetworkImageInfo {
                    url: res.get_url().to_string(),
                    content_type: res.content_type().to_string(),
                };

                let mut buf = Vec::new();
                if let Err(_) = res.into_reader().read_to_end(&mut buf) {
                    return Progress::Error("Unable read image content".to_string().into());
                };

                // and check here also.
                if _task.is_canceled() {
                    Progress::Canceled
                } else {
                    Progress::Completed((buf, img_info))
                }
            } else {
                Progress::Error(format!("Network error, status: {}", res.status()).into())
            }
        },
    )
}

fn save_image(image_content: &[u8], image_url: &str) -> String {
    let mut pth_buf = match current_dir() {
        Ok(pth_buf) => pth_buf,
        Err(e) => return e.to_string(),
    };
    let name: Vec<&str> = image_url.split("hmac=").collect();
    pth_buf.push(format!("{}.jpg", name[1][1..7].to_string()));
    if pth_buf.is_file() {
        return format!("{}\nimage name already exist!", pth_buf.display());
    } else {
        let mut _image = match OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&pth_buf)
        {
            Ok(_image) => _image,
            Err(e) => return format!("{}: {}", e.to_string(), &pth_buf.display()),
        };
        if let Err(e) = _image.write(image_content) {
            return format!("{}: {}", e.to_string(), &pth_buf.display());
        }
        return format!("image saved to:\n{}", pth_buf.display());
    }
}

struct MyApp {
    next: i32,
    total_image: u32,
    total_current_tasks: u32,
    seed: Vec<i32>,
    counter: Vec<u32>,
    image_content: Vec<Vec<u8>>,
    raw_image: Vec<(TextureId, (f32, f32))>,
    network_image_loader: Vec<Option<Futurized<(), (Vec<u8>, NetworkImageInfo)>>>,
    image_clicked: Vec<bool>,
    image_saved_info: Vec<String>,
    image_counter: Vec<u32>,
    label_info: Vec<String>,
    image_url: Vec<String>,
    cancel_image: Vec<bool>,
}

impl Default for MyApp {
    fn default() -> Self {
        let total_image: usize = 12;
        let mut seed = Vec::with_capacity(total_image);
        let mut counter = Vec::with_capacity(total_image);
        let mut image_content = Vec::with_capacity(total_image);
        let mut raw_image = Vec::with_capacity(total_image);
        let mut image_clicked = Vec::with_capacity(total_image);
        let mut image_saved_info = Vec::with_capacity(total_image);
        let mut image_counter = Vec::with_capacity(total_image);
        let mut label_info = Vec::with_capacity(total_image);
        let mut image_url = Vec::with_capacity(total_image);
        let mut cancel_image = Vec::with_capacity(total_image);

        for i in 0..total_image {
            seed.push(i as i32);
            counter.push(0);
            image_content.push(Vec::<u8>::with_capacity(1));
            raw_image.push((TextureId::default(), (0.0, 0.0)));
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
            next: 0,
            total_image: total_image as u32,
            total_current_tasks: 0,
            seed,
            counter,
            image_content,
            raw_image,
            network_image_loader: Vec::with_capacity(total_image),
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
            next,
            total_image,
            total_current_tasks,
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
            ui.heading("Egui network image quick demo");
            ui.separator();
            ui.add(
                egui::Slider::new(total_image, 0..=12)
                    .clamp_to_range(true)
                    .text("images"),
            );
            ui.horizontal(|ui| {
                if ui.button("prev image").clicked() {
                    *next -= *total_image as i32;
                    let mut task_len = network_image_loader.len();
                    let current_total_image = *total_image as usize;
                    if (task_len < current_total_image) || (task_len > current_total_image) {
                        *network_image_loader = Vec::with_capacity(current_total_image);
                        task_len = network_image_loader.len()
                    } else {
                        for i in 0..current_total_image {
                            counter[i] = 0;
                            network_image_loader[i] = None
                        }
                    }

                    let width = 640;
                    let height = 480;

                    if *next < 0 {
                        for i in 0..current_total_image {
                            label_info[i] =
                                "index out of bound, try click 'next image' button.".into()
                        }
                        *total_current_tasks = 0
                    } else {
                        for i in 0..current_total_image {
                            seed[i] = ((*next as i32) + (i as i32)) + 1;
                            let url = if *next >= *total_image as i32 {
                                format!(
                                    "https://picsum.photos/seed/{}/{}/{}",
                                    seed[i], width, height
                                )
                            } else {
                                format!(
                                    "https://picsum.photos/seed/{}/{}/{}",
                                    i + 1,
                                    width,
                                    height
                                )
                            };

                            let task = network_image(url);
                            task.try_do();

                            if task_len < current_total_image {
                                network_image_loader.push(Some(task))
                            } else {
                                network_image_loader[i] = Some(task)
                            }
                        }
                        *total_current_tasks = current_total_image as u32
                    }
                }

                if ui.button("next image").clicked() {
                    if *next < 0 {
                        *next = 0
                    } else {
                        *next += *total_image as i32
                    }

                    let mut task_len = network_image_loader.len();
                    let current_total_image = *total_image as usize;
                    if (task_len < current_total_image) || (task_len > current_total_image) {
                        *network_image_loader = Vec::with_capacity(current_total_image);
                        task_len = network_image_loader.len()
                    } else {
                        for i in 0..current_total_image {
                            counter[i] = 0;
                            network_image_loader[i] = None
                        }
                    }

                    let width = 640;
                    let height = 480;

                    for i in 0..current_total_image {
                        seed[i] = ((*next as i32) + (i as i32)) + 1;
                        let url = format!(
                            "https://picsum.photos/seed/{}/{}/{}",
                            seed[i], width, height
                        );

                        let task = network_image(url);
                        task.try_do();

                        if task_len < current_total_image {
                            network_image_loader.push(Some(task))
                        } else {
                            network_image_loader[i] = Some(task)
                        }
                    }
                    *total_current_tasks = current_total_image as u32
                }
            });

            // Don't iterate if total_tasks == 0 (to reduce resource usage);
            if *total_current_tasks > 0 {
                for i in 0..network_image_loader.len() {
                    if let Some(task) = &network_image_loader[i] {
                        task.try_resolve(|progress, _| match progress {
                            Progress::Current(_) => {
                                counter[i] += 1;
                                label_info[i] = format!("Loading... {}\n", counter[i]);
                                if cancel_image[i] {
                                    task.cancel()
                                }
                                // reqwest redraw to the context
                                ctx.request_repaint()
                            }
                            Progress::Completed((bytes, image_info)) => {
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
                            }
                            Progress::Canceled => {
                                label_info[i] = "Loading image canceled!".to_string();
                                cancel_image[i] = false
                            }
                            Progress::Error(err_name) => label_info[i] = err_name.into(),
                        });

                        // Restore some states to default
                        if task.is_done() {
                            network_image_loader[i] = None;
                            counter[i] = 0;
                            *total_current_tasks -= 1
                        }
                    }
                }
            }

            // println!("{}", *total_current_tasks);

            ui.separator();
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.vertical(|ui| {
                    for i in 0..*total_image as usize {
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
                                    image_saved_info[i] =
                                        save_image(&image_content[i], &image_url[i]);
                                    image_clicked[i] = true
                                }
                            }

                            if clickable_image.hovered() {
                                if image_clicked[i] {
                                    ui.label(image_saved_info[i].clone());
                                    image_counter[i] += 1;
                                    // show image save info until:
                                    if image_counter[i] > 20 {
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
                        ui.separator();
                    }
                });
            });
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
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
