use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::Json;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader};
use uuid::Uuid;

use crate::auth::ensure_profile;
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession};

const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_LONG_EDGE: u32 = 1600;
const JPEG_QUALITY: u8 = 85;

pub async fn upload_image(
    State(_state): State<Arc<AppState>>,
    session: AuthSession,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    ensure_profile(&session.user)?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart: {e}")))?
        .ok_or_else(|| AppError::BadRequest("file required".into()))?;

    let content_type = field
        .content_type()
        .map(|m| m.to_string())
        .unwrap_or_default();

    match content_type.as_str() {
        "image/jpeg" | "image/jpg" | "image/png" | "image/webp" => {}
        _ => {
            return Err(AppError::BadRequest(
                "image must be JPEG, PNG, or WebP".into(),
            ));
        }
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to read file: {e}")))?;

    if data.is_empty() {
        return Err(AppError::BadRequest("empty file".into()));
    }
    if data.len() > MAX_IMAGE_BYTES {
        return Err(AppError::BadRequest("image must be at most 2 MB".into()));
    }

    let processed = tokio::task::spawn_blocking(move || process_image(&data))
        .await
        .map_err(|e| AppError::Internal(format!("image processing failed: {e}")))??;

    let filename = format!("{}.jpg", Uuid::new_v4());
    let path = Path::new("uploads").join(&filename);
    tokio::fs::write(&path, &processed)
        .await
        .map_err(|e| AppError::Internal(format!("failed to save upload: {e}")))?;

    let url = format!("/uploads/{filename}");
    Ok(Json(serde_json::json!({ "url": url })))
}

fn process_image(data: &[u8]) -> AppResult<Vec<u8>> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::BadRequest(format!("invalid image: {e}")))?;

    let img = reader
        .decode()
        .map_err(|e| AppError::BadRequest(format!("could not decode image: {e}")))?;

    let resized = resize_to_long_edge(img, MAX_LONG_EDGE);
    let rgb = resized.to_rgb8();

    let mut out = Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| AppError::Internal(format!("failed to encode JPEG: {e}")))?;

    Ok(out.into_inner())
}

fn resize_to_long_edge(img: DynamicImage, max_long_edge: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let long = w.max(h);
    if long <= max_long_edge {
        return img;
    }
    let scale = max_long_edge as f64 / long as f64;
    let nw = ((w as f64) * scale).round().max(1.0) as u32;
    let nh = ((h as f64) * scale).round().max(1.0) as u32;
    img.resize(nw, nh, FilterType::Lanczos3)
}
