use egui_extras_lib::{Image, asynchron::{Futurize, Progress}};
use eframe::{egui::{self, FontDefinitions, FontFamily, Sense, TextStyle, TextureId}, epi};

struct MyApp {
    name: String,
    age: u32,
    counter: u32,
    raw_image: (TextureId, (f32, f32)),
    image_loader: Option<Futurize<Image, String>>,
    image_clicked: bool,
    btn2_label: String
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Saprol".to_owned(),
            age: 24,
            counter: 0,
            raw_image: (TextureId::default(), (0.0, 0.0)),
            image_loader: None,
            image_clicked: false,
            btn2_label: "Load SVG".to_string()
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            name,
            age,
            counter,
            raw_image,
            image_loader,
            image_clicked,
            btn2_label
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application Ok()");
            ui.vertical(|ui| {
                ui.separator();
                // ui.label(format!("counter: {}", counter.to_string()));
                ui.label("Your name: ");
                ui.text_edit_singleline(name);
            });
            ui.add(egui::Slider::new(age, 0..=120).text("age"));
            ui.horizontal(|ui| {
                let btn = ui.button("Change image").on_hover_text("if age is odd image will be changed to fractal.png\nelse if age is even image will be back to cargo-crew.png");
                if btn.clicked() {
                    // prevent changing if the other task_image_loader is still running
                    if !btn2_label.contains("Loading") {
                        *age += 1;
                        if *age % 2 == 0 {
                            *image_loader = Some(Image::load_image("images/cargo-crew.png".to_string()))
                        } else {
                            *image_loader = Some(Image::load_image("images/fractal.png".to_string()))
                        }
                    }
                }

                let btn2 = ui.button(&btn2_label).on_hover_text("try load SVG, if age is odd heart.svg file will be loaded (sometimes it takes a few milliseconds to complete, depending on the size and complexity of the SVG file)");
                if btn2.clicked() {
                    if !btn2_label.contains("Loading") {
                        *age += 1;
                        if *age % 2 == 0 {
                            *image_loader = Some(Image::load_svg("images/tiger.svg".to_string()))
                        } else {
                            *image_loader = Some(Image::load_svg("images/heart.svg".to_string()))
                        }
                    }
                }

                if let Some(task_image_loader) = image_loader {
                    if task_image_loader.is_in_progress() {
                        match task_image_loader.try_get() {
                            Progress::Current => {
                                // if task id == 0 it means loading image (png, jpg, gif .etc) else if task_id == 1 loading svg image
                                if task_image_loader.task_id() == 1 {
                                    *counter += 1; 
                                    *btn2_label = format!("Loading... {}", counter)
                                }
                            }
                            Progress::Completed(_image) => {
                                // restore some states to default
                                if task_image_loader.task_id() == 1 {
                                    *counter = 0;
                                    *btn2_label = "Load SVG".to_string();
                                }
                                frame.tex_allocator().free(raw_image.0);
                                *raw_image = (_image.texture_id(frame), _image.size);
                                *image_loader = None;
                            }
                            Progress::Error(img) => {
                                if task_image_loader.task_id() == 1 {
                                    *counter = 0;
                                    *btn2_label = "Load SVG".to_string();
                                }
                                println!("unable to load {}", img);
                            }
                            _ => ()
                        }
                    }
                }
            });

            ui.label(format!("Hello '{}', age {}", name, age));

            if *image_clicked {
                ui.vertical(|ui| {
                    ui.label("Image clicked!");
                });
            }

            //// original image size
            // let size: (f32, f32) = raw_image.1;
            //
            // just resize here for smaller image, 0.66x actual size
            let size: (f32, f32) = (raw_image.1.0/1.5, raw_image.1.1/1.5);
            
            let img = ui
                .image(raw_image.0, size)
                .interact(Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("This image is clickable!");
            if img.clicked() {
                if !*image_clicked {
                    *image_clicked = true
                } else {
                    *image_clicked = false
                }
            }
            
            if img.hovered() {
                ui.separator();
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
        ctx.request_repaint();
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "JetBrainsMonoNL-Regular".to_string(),
            std::borrow::Cow::Borrowed(include_bytes!("../font/JetBrainsMonoNL-Regular.ttf")),
        );

        fonts
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "JetBrainsMonoNL-Regular".to_owned());
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
        let _image = Image::new(include_bytes!("../images/cargo-crew.png"));
        self.raw_image = (_image.texture_id(frame), _image.size)
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::default()), options);
}
