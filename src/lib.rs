pub use asynchron;
use asynchron::{Futurized};
use egui::{Color32, TextureId};
use epi;

extern "Rust" {
    fn _image_from_bytes(bytes: &[u8]) -> Option<Image>;
    fn _svg_from_bytes(bytes: &[u8]) -> Option<Image>;
    fn _tex_id_from_image(image: &Image, frame: &mut epi::Frame<'_>) -> TextureId;
    fn _load_image(path: String) -> Futurized<(),Image>;
    fn _load_svg(path: String) -> Futurized<(),Image>;
}

/// Example available on repository: https://github.com/Ar37-rs/egui-extras-lib
#[derive(Clone, Default)]
pub struct Image {
    pub size: (f32, f32),
    pub pixels: Vec<Color32>,
}

impl Image {
    /// New image form bytes of .png, .gif, .jpg and .etc which supported by image crate.
    pub fn new(bytes: &[u8]) -> Option<Image> {
        unsafe {
            _image_from_bytes(bytes)
        }
    }

    /// if task id == 0 it means loading image (png, jpg, gif .etc)
    ///
    /// else if task_id == 1 loading svg image
    pub fn type_id(t: usize) -> usize {
        t
    }

    /// New image form bytes of SVG v1.1 file specification which fully supported by usvg crate.
    pub fn new_from_svg(bytes: &[u8]) -> Option<Image> {
        unsafe {
            _svg_from_bytes(bytes)
        }
    }

    /// Image texture id.
    pub fn texture_id(&self, frame: &mut epi::Frame<'_>) -> TextureId {
        unsafe {
            _tex_id_from_image(self, frame)
        }
    }

    /// Image loader (.png, .gif, .jpg and .etc ) on top of image crate.
    pub fn load_image(path: String) -> Futurized<(),Image> {
        unsafe {
            _load_image(path)
        }
    }

    /// SVG loader usvg, resvg, tiny-skia and image crates under the hood.
    pub fn load_svg(path: String) -> Futurized<(),Image> {
        unsafe {
            _load_svg(path)
        }
    }
}
