//! 图像处理模块

use hyper::{Response, StatusCode, header};
use http_body_util::Full;
use std::io::Cursor;
use image::{ImageFormat, ImageOutputFormat};

pub struct ImageProcessor;

#[derive(Debug)]
pub struct ImageOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: Option<u8>,
    pub format: Option<ImageFormat>,
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self
    }

    /// 调整图像大小
    pub fn resize_image(
        &self,
        image_data: &[u8],
        options: &ImageOptions,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(image_data)?;
        
        let (target_width, target_height) = match (options.width, options.height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => {
                let ratio = img.height() as f32 / img.width() as f32;
                (w, (w as f32 * ratio) as u32)
            },
            (None, Some(h)) => {
                let ratio = img.width() as f32 / img.height() as f32;
                ((h as f32 * ratio) as u32, h)
            },
            (None, None) => (img.width(), img.height()),
        };
        
        let resized_img = img.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);
        
        let mut output_buffer = Vec::new();
        let output_format = options.format.unwrap_or(ImageFormat::Jpeg);
        
        match output_format {
            ImageFormat::Jpeg => {
                let quality = options.quality.unwrap_or(85);
                resized_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::Jpeg(quality))?;
            },
            ImageFormat::Png => {
                resized_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::Png)?;
            },
            ImageFormat::Gif => {
                resized_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::Gif)?;
            },
            ImageFormat::WebP => {
                #[cfg(feature = "webp")]
                {
                    let quality = options.quality.unwrap_or(85);
                    resized_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::WebP)?;
                }
                #[cfg(not(feature = "webp"))]
                {
                    // 如果不支持 WebP，回退到 JPEG
                    let quality = options.quality.unwrap_or(85);
                    resized_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::Jpeg(quality))?;
                }
            },
            _ => {
                // 默认使用 JPEG
                let quality = options.quality.unwrap_or(85);
                resized_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::Jpeg(quality))?;
            }
        }
        
        Ok(output_buffer)
    }

    /// 裁剪图像
    pub fn crop_image(
        &self,
        image_data: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(image_data)?;
        let cropped_img = img.crop(x, y, width, height);
        
        let mut output_buffer = Vec::new();
        cropped_img.write_to(&mut Cursor::new(&mut output_buffer), ImageOutputFormat::Jpeg(85))?;
        
        Ok(output_buffer)
    }

    /// 创建图像处理响应
    pub fn create_image_response(
        &self,
        processed_image: Vec<u8>,
        format: ImageFormat,
    ) -> Response<Full<bytes::Bytes>> {
        let content_type = match format {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::Gif => "image/gif",
            ImageFormat::WebP => "image/webp",
            _ => "image/jpeg",
        };
        
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, processed_image.len())
            .body(Full::new(bytes::Bytes::from(processed_image)))
            .unwrap()
    }

    /// 检查是否为支持的图像格式
    pub fn is_supported_image_format(&self, content_type: &str) -> bool {
        matches!(content_type, "image/jpeg" | "image/png" | "image/gif" | "image/webp")
    }

    /// 根据内容类型获取图像格式
    pub fn get_image_format_from_content_type(&self, content_type: &str) -> Option<ImageFormat> {
        match content_type {
            "image/jpeg" => Some(ImageFormat::Jpeg),
            "image/png" => Some(ImageFormat::Png),
            "image/gif" => Some(ImageFormat::Gif),
            "image/webp" => Some(ImageFormat::WebP),
            _ => None,
        }
    }
}