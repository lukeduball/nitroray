use std::rc::Rc;

use image::{ImageReader, Rgb32FImage};
use xenofrost::core::math::Vec3;

pub(crate) struct ImageLoader {
    missing_image: Rc<Rgb32FImage>
}

impl ImageLoader {
    pub(crate) fn new() -> Self {
        let missing_image_stream = ImageReader::open("res/images/missing_image.png").unwrap();
        let missing_image = missing_image_stream.decode().unwrap();

        Self {
            missing_image: Rc::new(missing_image.into_rgb32f())
        }
    }

    pub(crate) fn load_image(&self, path: &str) -> Rc<Rgb32FImage> {
        let image_result = ImageReader::open(path);
        match image_result {
            Ok(image) => {
                let decoded_image_result = image.decode();
                match decoded_image_result {
                    Ok(decoded_image) => return Rc::new(decoded_image.into_rgb32f()),
                    Err(e) => eprintln!("Failed to decode image: {}", e),
                };
            },
            Err(e) => eprintln!("Failed to open file: {}", e),
        };

        self.missing_image.clone()
    }
}

pub(crate) fn get_color_at_image_uv(image: Rc<Rgb32FImage>, u: f32, v: f32) -> Vec3 {
    let x_coordinate = (image.width() as f32 * u) as u32;
    let y_coordinate = (image.height() as f32 * v) as u32;
    
    let result = image.get_pixel(x_coordinate.clamp(0, image.width()-1), y_coordinate.clamp(0, image.height()-1));
    
    Vec3::new(result[0], result[1], result[2])
}