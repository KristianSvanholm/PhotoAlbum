use leptos::*;
use std::{fs, time::{Duration, Instant}};

use image::{DynamicImage, GrayImage, Rgba};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;

use rustface::{Detector, FaceInfo, ImageData};

#[server(Run, "/api", "Cbor")]
pub async fn run(encoded_string: String) -> Result<String, ServerFnError>{
    let mut detector = match rustface::create_detector("model.bin") {
        Ok(detector) => detector,
        Err(error) => {
            println!("Failed to create detector: {}", error);
            return Err(ServerFnError::new("Failed to create detector".to_string()));
        }
    };

    logging::log!("Running face detection");

    detector.set_min_face_size(20);
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    let image = decode_image(encoded_string);

    let gray = image.to_luma8();

    let faces = detect_faces(&mut *detector, &gray);

    let mut image = DynamicImage::ImageLuma8(gray);

    for face in faces {
        let rect = Rect::at(face.bbox().x() as i32, face.bbox().y() as i32)
            .of_size(face.bbox().width() as u32, face.bbox().height() as u32);
        
        let color: Rgba<u8> = Rgba([0, 255, 0, 0]);

        draw_hollow_rect_mut(&mut image, rect, color);
    }

    let buf = image.to_rgba8().into_raw();
    
    fs::write("./album/test.png", buf).expect("Failed to write image");
    Ok("Success".to_string())
}


fn decode_image(encoded_string: String) -> DynamicImage {
    let bytes = base64::decode(encoded_string).expect("Failed to decode image");
    image::load_from_memory(&bytes).expect("Failed to load image")
}

fn detect_faces(detector: &mut dyn Detector, gray: &GrayImage) -> Vec<FaceInfo> {
    let (width, height) = gray.dimensions();
    let image = ImageData::new(gray, width, height);
    let now = Instant::now();
    let faces = detector.detect(&image);
    println!(
        "Found {} faces in {} ms",
        faces.len(),
        get_millis(now.elapsed())
    );
    faces
}

fn get_millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}


