use gtk::cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use ndi::{FourCCVideoType, FrameFormatType, VideoData};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NdiSlideData {
    pub header: String,
    pub body: String,
    pub theme: String,
    pub blackout: bool,
    pub logo_mode: bool,
    pub clearout: bool,
    pub go_live: bool,
    pub logo_image_path: Option<String>,
}

#[derive(Clone)]
pub struct NdiOutput {
    current_slide: Arc<Mutex<Option<NdiSlideData>>>,
}

fn draw_background(
    cr: &Context,
    _width: f64,
    _height: f64,
    theme: &str,
    blackout: bool,
    cached_background_pixbuf: &Option<Pixbuf>,
) {
    if blackout {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.paint();
    } else {
        if let Some(scaled) = cached_background_pixbuf {
            cr.set_source_pixbuf(scaled, 0.0, 0.0);
            let _ = cr.paint();
        } else {
            match theme {
                "classic-red" => cr.set_source_rgb(0.5, 0.0, 0.0),
                "royal-blue" => cr.set_source_rgb(0.0, 0.1, 0.4),
                "forest-green" => cr.set_source_rgb(0.0, 0.3, 0.1),
                "dark-slate" => cr.set_source_rgb(0.1, 0.12, 0.15),
                "black" => cr.set_source_rgb(0.0, 0.0, 0.0),
                _ => cr.set_source_rgb(0.0, 0.0, 0.0),
            }
            let _ = cr.paint();
        }
    }
}

fn draw_single_slide_text(cr: &Context, width: f64, height: f64, slide: &NdiSlideData, alpha: f64) {
    cr.select_font_face("Tahoma", FontSlant::Normal, FontWeight::Bold);

    if slide.logo_mode && !slide.blackout {
        if slide.logo_image_path.is_none() {
            cr.set_font_size(height * 0.074);
            cr.set_source_rgba(1.0, 1.0, 1.0, alpha);

            let logo_cross = "✝";
            if let Ok(ext) = cr.text_extents(logo_cross) {
                cr.move_to(
                    (width - ext.width()) / 2.0,
                    (height - ext.height()) / 2.0 - height * 0.037,
                );
                let _ = cr.show_text(logo_cross);
            }

            cr.set_font_size(height * 0.037);
            let logo_lbl = "EasyWorship - Standby";
            if let Ok(ext) = cr.text_extents(logo_lbl) {
                cr.move_to(
                    (width - ext.width()) / 2.0,
                    (height - ext.height()) / 2.0 + height * 0.055,
                );
                let _ = cr.show_text(logo_lbl);
            }
        }
    } else if !slide.blackout && !slide.clearout {
        let body_font_size = height * 0.06;
        let header_font_size = height * 0.050;

        // Wrap body text
        let max_width = width - width * 0.15;
        let mut wrapped_lines = Vec::new();

        cr.set_font_size(body_font_size);
        cr.set_source_rgba(1.0, 1.0, 1.0, alpha);

        for line in slide.body.lines() {
            let mut current_line = String::new();
            for word in line.split_whitespace() {
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                if let Ok(ext) = cr.text_extents(&test_line) {
                    if ext.width() > max_width {
                        if !current_line.is_empty() {
                            wrapped_lines.push(current_line);
                        }
                        current_line = word.to_string();
                    } else {
                        current_line = test_line;
                    }
                }
            }
            if !current_line.is_empty() {
                wrapped_lines.push(current_line);
            }
        }

        // Calculate vertical metrics
        let line_spacing = height * 0.06;
        let total_body_height = if wrapped_lines.is_empty() {
            0.0
        } else {
            (wrapped_lines.len() - 1) as f64 * line_spacing + body_font_size
        };

        // Center the body block vertically
        let start_y = (height - total_body_height) / 2.0;

        // Draw body lines centered
        let mut current_y = start_y + body_font_size * 0.8;
        for line in &wrapped_lines {
            if let Ok(ext) = cr.text_extents(line) {
                cr.move_to((width - ext.width()) / 2.0, current_y);
                let _ = cr.show_text(line);
            }
            current_y += line_spacing;
        }

        // Draw header aligned right
        cr.set_font_size(header_font_size);
        cr.set_source_rgba(0.85, 0.85, 0.85, alpha);
        if let Ok(ext) = cr.text_extents(&slide.header) {
            let header_x = width - ext.width() - width * 0.075;
            let header_y = current_y + height * 0.02;
            cr.move_to(header_x, header_y);
            let _ = cr.show_text(&slide.header);
        }
    } else if slide.clearout && !slide.blackout {
        // Clearout: show only background, no text drawn
    }
}

impl NdiOutput {
    pub fn new() -> Self {
        let current_slide = Arc::new(Mutex::new(None::<NdiSlideData>));
        let current_slide_clone = Arc::clone(&current_slide);

        thread::spawn(move || {
            // Initialize NDI
            if let Err(e) = ndi::initialize() {
                eprintln!("Failed to initialize NDI: {:?}", e);
                return;
            }

            // Create Sender
            let sender = match ndi::SendBuilder::new()
                .ndi_name("EasyWorship Live Output".to_string())
                .build()
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to build NDI Sender: {:?}", e);
                    return;
                }
            };

            let width = 1920;
            let height = 1080;
            let mut pixel_buffer = vec![0u8; width * height * 4];

            let mut cached_background_path = String::new();
            let mut cached_background_pixbuf: Option<Pixbuf> = None;

            println!("NDI Broadcast active: 'EasyWorship Live Output'");

            let mut last_slide: Option<NdiSlideData> = None;
            let mut trans_prev_slide: Option<NdiSlideData> = None;
            let mut trans_start: Option<std::time::Instant> = None;
            let mut last_sent_time = std::time::Instant::now();

            loop {
                // Sleep for 10ms to check for state updates quickly without burning CPU
                thread::sleep(Duration::from_millis(33));

                // Get current slide data
                let slide_opt = {
                    let lock = current_slide_clone.lock().unwrap();
                    lock.clone()
                };

                if let Some(slide) = slide_opt {
                    let slide_changed = match &last_slide {
                        Some(last) => {
                            last.header != slide.header
                                || last.body != slide.body
                                || last.theme != slide.theme
                                || last.blackout != slide.blackout
                                || last.logo_mode != slide.logo_mode
                                || last.clearout != slide.clearout
                        }
                        None => true,
                    };

                    if slide_changed {
                        println!(
                            "DEBUG: NDI loop - slide changed detected! Header: '{}', Theme: '{}'",
                            slide.header, slide.theme
                        );
                        if let Some(ref prev) = last_slide {
                            trans_prev_slide = Some(prev.clone());
                            trans_start = Some(std::time::Instant::now());
                        }
                        last_slide = Some(slide.clone());
                    }

                    let mut is_animating = false;
                    let mut progress = 1.0;
                    if let Some(start) = trans_start {
                        let elapsed = start.elapsed().as_millis() as f64;
                        if elapsed < 800.0 {
                            is_animating = true;
                            progress = elapsed / 800.0;
                        } else {
                            trans_start = None;
                            trans_prev_slide = None;
                        }
                    }

                    let time_for_keep_alive =
                        last_sent_time.elapsed() >= Duration::from_millis(1000);

                    if slide_changed || is_animating || time_for_keep_alive {
                        // Ensure active background is loaded/scaled if needed
                        let active_bg_path = if slide.logo_mode {
                            slide.logo_image_path.as_deref().unwrap_or("")
                        } else {
                            &slide.theme
                        };

                        let path = std::path::Path::new(active_bg_path);
                        if path.exists() && path.is_file() {
                            if active_bg_path != cached_background_path
                                || cached_background_pixbuf.is_none()
                            {
                                println!(
                                    "DEBUG: NDI background - cache miss: loading and scaling file: {}",
                                    active_bg_path
                                );
                                if let Ok(pixbuf) = Pixbuf::from_file(active_bg_path) {
                                    if let Some(scaled) = pixbuf.scale_simple(
                                        width as i32,
                                        height as i32,
                                        gtk::gdk_pixbuf::InterpType::Bilinear,
                                    ) {
                                        cached_background_pixbuf = Some(scaled);
                                        cached_background_path = active_bg_path.to_string();
                                    }
                                }
                            }
                        } else {
                            cached_background_pixbuf = None;
                            cached_background_path = String::new();
                        }

                        // Create cairo ImageSurface to render slide to
                        let mut surface =
                            ImageSurface::create(Format::ARgb32, width as i32, height as i32)
                                .unwrap();
                        let cr = Context::new(&surface).unwrap();

                        // 1. Draw target background instantly
                        draw_background(
                            &cr,
                            width as f64,
                            height as f64,
                            if slide.logo_mode { "" } else { &slide.theme },
                            slide.blackout,
                            &cached_background_pixbuf,
                        );

                        // 2. Draw text
                        if is_animating {
                            if let Some(ref prev) = trans_prev_slide {
                                // Draw previous slide text fading out
                                draw_single_slide_text(
                                    &cr,
                                    width as f64,
                                    height as f64,
                                    prev,
                                    1.0 - progress,
                                );
                                // Draw new slide text fading in
                                draw_single_slide_text(
                                    &cr,
                                    width as f64,
                                    height as f64,
                                    &slide,
                                    progress,
                                );
                            } else {
                                draw_single_slide_text(
                                    &cr,
                                    width as f64,
                                    height as f64,
                                    &slide,
                                    1.0,
                                );
                            }
                        } else {
                            draw_single_slide_text(&cr, width as f64, height as f64, &slide, 1.0);
                        }

                        // Drop cairo Context to release surface borrow before accessing raw data!
                        drop(cr);

                        // Flush and copy data
                        surface.flush();
                        match surface.data() {
                            Ok(data) => {
                                pixel_buffer.copy_from_slice(&*data);
                            }
                            Err(e) => {
                                println!(
                                    "DEBUG: NDI - failed to access cairo surface data: {:?}",
                                    e
                                );
                            }
                        }

                        last_sent_time = std::time::Instant::now();
                    }

                    // Send NDI frame (uses existing pixel_buffer) only if go_live is true
                    if slide.go_live {
                        let video_data = VideoData::from_buffer(
                            width as i32,
                            height as i32,
                            FourCCVideoType::BGRA,
                            15,
                            1,
                            FrameFormatType::Progressive,
                            0,
                            (width * 4) as i32,
                            None,
                            &mut pixel_buffer,
                        );
                        sender.send_video(&video_data);
                    }
                }
            }
        });

        Self { current_slide }
    }

    pub fn update_slide(
        &self,
        header: String,
        body: String,
        theme: String,
        blackout: bool,
        logo_mode: bool,
        clearout: bool,
        go_live: bool,
        logo_image_path: Option<String>,
    ) {
        let mut lock = self.current_slide.lock().unwrap();
        *lock = Some(NdiSlideData {
            header,
            body,
            theme,
            blackout,
            logo_mode,
            clearout,
            go_live,
            logo_image_path,
        });
    }
}
