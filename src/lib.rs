pub use asynchron;
use asynchron::Futurize;
use egui::{Color32, TextureId};
use epi;

extern "Rust" {
    fn _image_from_bytes(bytes: &[u8]) -> Option<Image>;
    fn _svg_from_bytes(bytes: &[u8]) -> Option<Image>;
    fn _tex_id_from_image(image: &Image, frame: &mut epi::Frame<'_>) -> TextureId;
    fn _load_image(path: String) -> Futurize<Image, String>;
    fn _load_svg(path: String) -> Futurize<Image, String>;
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

    /// New image form bytes of SVG v1.1 file specification which fully supported by usvg crate.
    pub fn new_from_svg(bytes: &[u8]) -> Option<Image> {
        unsafe {
            _svg_from_bytes(bytes)
        }
    }

    pub fn texture_id(&self, frame: &mut epi::Frame<'_>) -> TextureId {
        unsafe {
            _tex_id_from_image(self, frame)
        }
    }

    /// Image loader (.png, .gif, .jpg and .etc ) on top of image crate.
    pub fn load_image(path: String) -> Futurize<Image, String> {
        unsafe {
            _load_image(path)
        }
    }

    /// SVG loader usvg, resvg, tiny-skia and image crates under the hood.
    pub fn load_svg(path: String) -> Futurize<Image, String> {
        unsafe {
            _load_svg(path)
        }
    }
}
