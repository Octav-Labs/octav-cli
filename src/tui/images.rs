use std::collections::HashSet;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::tui::event::DataEvent;

/// Rasterize SVG bytes into a DynamicImage (64x64).
fn decode_svg(bytes: &[u8]) -> Option<image::DynamicImage> {
    let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default()).ok()?;
    let size = 64u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;
    let sx = size as f32 / tree.size().width();
    let sy = size as f32 / tree.size().height();
    let scale = sx.min(sy);
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    // tiny_skia uses premultiplied RGBA; convert to straight RGBA
    let mut rgba = pixmap.take();
    for pixel in rgba.chunks_exact_mut(4) {
        let a = pixel[3] as u16;
        if a > 0 && a < 255 {
            pixel[0] = ((pixel[0] as u16) * 255 / a).min(255) as u8;
            pixel[1] = ((pixel[1] as u16) * 255 / a).min(255) as u8;
            pixel[2] = ((pixel[2] as u16) * 255 / a).min(255) as u8;
        }
    }
    image::RgbaImage::from_raw(size, size, rgba).map(image::DynamicImage::ImageRgba8)
}

/// Normalize to a consistent square size so halfblocks gets uniform input.
fn normalize(img: image::DynamicImage) -> image::DynamicImage {
    const TARGET: u32 = 128;
    if img.width() == TARGET && img.height() == TARGET {
        return img;
    }
    img.resize_exact(TARGET, TARGET, image::imageops::FilterType::Lanczos3)
}

/// Try to decode bytes as raster first, then SVG fallback.
fn decode_image(bytes: &[u8]) -> Option<image::DynamicImage> {
    image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.decode().ok())
        .or_else(|| decode_svg(bytes))
        .map(normalize)
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("octav-cli/0.1")
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub struct ImageCache {
    pub cache_dir: PathBuf,
    in_flight: HashSet<String>,
}

impl ImageCache {
    pub fn new() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".octav")
            .join("cache")
            .join("images");
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            in_flight: HashSet::new(),
        }
    }

    pub fn fetch_images(&mut self, urls: Vec<String>, tx: mpsc::Sender<DataEvent>) {
        let new_urls: Vec<String> = urls
            .into_iter()
            .filter(|u| !u.is_empty() && self.in_flight.insert(u.clone()))
            .collect();

        if new_urls.is_empty() {
            return;
        }

        let cache_dir = self.cache_dir.clone();

        std::thread::spawn(move || {
            let client = http_client();
            for url in new_urls {
                let cache_path = {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    url.hash(&mut hasher);
                    cache_dir.join(format!("{:016x}", hasher.finish()))
                };

                // Try disk cache first
                if cache_path.exists() {
                    if let Ok(bytes) = std::fs::read(&cache_path) {
                        if let Some(img) = decode_image(&bytes) {
                            let _ = tx.send(DataEvent::ImageLoaded {
                                url: url.clone(),
                                image_data: img,
                            });
                            continue;
                        }
                    }
                }

                // Download with proper User-Agent
                let bytes = match client.get(&url).send() {
                    Ok(resp) if resp.status().is_success() => match resp.bytes() {
                        Ok(b) => b,
                        Err(_) => continue,
                    },
                    _ => continue,
                };

                // Save to disk cache
                let _ = std::fs::write(&cache_path, &bytes);

                // Decode (raster or SVG)
                if let Some(img) = decode_image(&bytes) {
                    let _ = tx.send(DataEvent::ImageLoaded {
                        url: url.clone(),
                        image_data: img,
                    });
                }
            }
        });
    }
}
