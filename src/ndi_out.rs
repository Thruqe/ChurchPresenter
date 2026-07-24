use crate::{eprintln, println};
use gtk::cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
#[cfg(not(target_os = "macos"))]
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
    pub bg_type: String,
    pub bg_path: Option<String>,
    pub font_size: f64,
    pub scale: f64,
    pub align: String,
    pub shadow: bool,
    pub default_song_bg_type: String,
    pub default_song_bg_val: Option<String>,
    pub lower_bar_height: f64, // fraction 0.0–1.0
}

#[derive(Clone)]
pub struct NdiOutput {
    current_slide: Arc<Mutex<Option<NdiSlideData>>>,
}

fn load_image_surface(_path_str: &str) -> Option<ImageSurface> {
    None
}

fn draw_surface_scaled(cr: &Context, surf: &ImageSurface, width: f64, height: f64) {
    let img_w = surf.width() as f64;
    let img_h = surf.height() as f64;
    if img_w > 0.0 && img_h > 0.0 {
        if let Ok(_) = cr.save() {
            cr.scale(width / img_w, height / img_h);
            let _ = cr.set_source_surface(surf, 0.0, 0.0);
            let _ = cr.paint();
            let _ = cr.restore();
        }
    }
}

fn draw_background(
    cr: &Context,
    width: f64,
    height: f64,
    theme: &str,
    blackout: bool,
    cached_background_surface: &Option<ImageSurface>,
    bg_type: &str,
    bg_path: Option<&str>,
    default_song_bg_type: &str,
    default_song_bg_val: Option<&str>,
    lower_bar_height: f64,
) {
    if blackout {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.paint();
    } else {
        let is_song = !bg_type.is_empty();
        let (actual_bg_type, actual_bg_path) = if is_song && bg_type == "transparent" {
            (default_song_bg_type, default_song_bg_val)
        } else {
            (bg_type, bg_path)
        };

        if !is_song {
            // Scripture or standby slide -> Draw standard theme background
            if let Some(surf) = cached_background_surface {
                draw_surface_scaled(cr, surf, width, height);
            } else {
                match theme {
                    "classic-red" | "theme-classic-red" => cr.set_source_rgb(0.5, 0.0, 0.0),
                    "royal-blue" | "theme-royal-blue" => cr.set_source_rgb(0.0, 0.1, 0.4),
                    "forest-green" | "theme-forest-green" => cr.set_source_rgb(0.0, 0.3, 0.1),
                    "dark-slate" | "theme-dark-slate" => cr.set_source_rgb(0.1, 0.12, 0.15),
                    "black" | "theme-black" => cr.set_source_rgb(0.0, 0.0, 0.0),
                    _ => {
                        if let Some(surf) = load_image_surface(theme) {
                            draw_surface_scaled(cr, &surf, width, height);
                            return;
                        }
                        cr.set_source_rgb(0.0, 0.0, 0.0);
                    }
                }
                let _ = cr.paint();
            }
        } else {
            // Song stanza card -> Render background according to resolved types
            if actual_bg_type == "image" {
                if let Some(surf) = cached_background_surface {
                    draw_surface_scaled(cr, surf, width, height);
                } else {
                    cr.set_source_rgb(0.0, 0.0, 0.0);
                    let _ = cr.paint();
                }
            } else if actual_bg_type == "color" || actual_bg_type == "theme" {
                let color_theme = actual_bg_path.unwrap_or("dark-slate");
                match color_theme {
                    "classic-red" | "theme-classic-red" => cr.set_source_rgb(0.5, 0.0, 0.0),
                    "royal-blue" | "theme-royal-blue" => cr.set_source_rgb(0.0, 0.1, 0.4),
                    "forest-green" | "theme-forest-green" => cr.set_source_rgb(0.0, 0.3, 0.1),
                    "dark-slate" | "theme-dark-slate" => cr.set_source_rgb(0.1, 0.12, 0.15),
                    "black" | "theme-black" => cr.set_source_rgb(0.0, 0.0, 0.0),
                    _ => {
                        if let Some(surf) = cached_background_surface {
                            draw_surface_scaled(cr, surf, width, height);
                        } else {
                            cr.set_source_rgb(0.1, 0.12, 0.15);
                        }
                    }
                }
                let _ = cr.paint();
            } else if actual_bg_type == "lower_transparent" {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                let _ = cr.paint();

                cr.set_source_rgba(0.0, 0.0, 0.0, 0.6);
                let rect_height = height * lower_bar_height;
                let rect_y = height - rect_height;
                cr.rectangle(0.0, rect_y, width, rect_height);
                let _ = cr.fill();
            } else {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                let _ = cr.paint();
            }
        }
    }
}

fn draw_song_text_cairo(
    cr: &Context,
    width: f64,
    height: f64,
    lyrics: &str,
    font_size: f64,
    scale: f64,
    align: &str,
    shadow: bool,
    bg_type: &str,
    alpha: f64,
    lower_bar_height: f64,
) {
    let actual_font_size = font_size * scale;
    cr.set_font_size(actual_font_size);

    let (text_min_y, text_max_y, margin_x) = if bg_type == "lower_transparent" {
        (
            height - height * lower_bar_height + 10.0,
            height - 10.0,
            width * 0.05,
        )
    } else {
        (height * 0.1, height * 0.9, width * 0.075)
    };

    let max_text_width = width - margin_x * 2.0;

    let mut wrapped_lines = Vec::new();
    for line in lyrics.lines() {
        let mut current_line = String::new();
        for word in line.split_whitespace() {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };
            if let Ok(ext) = cr.text_extents(&test_line) {
                if ext.width() > max_text_width {
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

    let line_spacing = actual_font_size * 1.35;
    let total_height = if wrapped_lines.is_empty() {
        0.0
    } else {
        (wrapped_lines.len() - 1) as f64 * line_spacing + actual_font_size
    };

    let start_y = text_min_y + ((text_max_y - text_min_y) - total_height) / 2.0;
    let mut current_y = start_y + actual_font_size * 0.8;

    for line in &wrapped_lines {
        if let Ok(ext) = cr.text_extents(line) {
            let x = match align {
                "left" => margin_x,
                "right" => width - ext.width() - margin_x,
                _ => (width - ext.width()) / 2.0,
            };

            if shadow {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.7 * alpha);
                cr.move_to(x + 2.0, current_y + 2.0);
                let _ = cr.show_text(line);
            }

            cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
            cr.move_to(x, current_y);
            let _ = cr.show_text(line);
        }
        current_y += line_spacing;
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
            let logo_lbl = "ChurchPresenter - Standby";
            if let Ok(ext) = cr.text_extents(logo_lbl) {
                cr.move_to(
                    (width - ext.width()) / 2.0,
                    (height - ext.height()) / 2.0 + height * 0.055,
                );
                let _ = cr.show_text(logo_lbl);
            }
        }
    } else if !slide.blackout && !slide.clearout {
        if !slide.bg_type.is_empty() {
            // Draw lower transparent rect using the stanza's stored bar height
            if slide.bg_type == "lower_transparent" {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.6 * alpha);
                let rect_height = height * slide.lower_bar_height;
                let rect_y = height - rect_height;
                cr.rectangle(0.0, rect_y, width, rect_height);
                let _ = cr.fill();
            }

            draw_song_text_cairo(
                cr,
                width,
                height,
                &slide.body,
                slide.font_size,
                slide.scale,
                &slide.align,
                slide.shadow,
                &slide.bg_type,
                alpha,
                slide.lower_bar_height,
            );
        } else {
            let body_font_size = height * 0.06;
            let header_font_size = height * 0.045;

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

            let body_start_y = (height - total_body_height) / 2.0;

            // 1. Draw Wrapped Body Lines
            cr.set_font_size(body_font_size);
            cr.set_source_rgba(1.0, 1.0, 1.0, alpha);

            let mut current_y = body_start_y + body_font_size * 0.8;
            for line in &wrapped_lines {
                if let Ok(ext) = cr.text_extents(line) {
                    cr.move_to((width - ext.width()) / 2.0, current_y);
                    let _ = cr.show_text(line);
                }
                current_y += line_spacing;
            }

            // 2. Draw Header (reference like "Genesis 1:1") — below body, right-aligned
            if !slide.header.is_empty() {
                cr.set_font_size(header_font_size);
                cr.set_source_rgba(0.85, 0.85, 0.85, alpha);
                if let Ok(ext) = cr.text_extents(&slide.header) {
                    let header_x = width - ext.width() - width * 0.075;
                    let header_y = current_y + height * 0.02;
                    cr.move_to(header_x, header_y);
                    let _ = cr.show_text(&slide.header);
                }
            }
        }
    }
}

impl NdiOutput {
    pub fn new() -> Self {
        let current_slide = Arc::new(Mutex::new(None::<NdiSlideData>));
        let thread_slide = current_slide.clone();

        #[cfg(target_os = "macos")]
        {
            println!("NDI Broadcast is not supported on macOS.");
        }

        #[cfg(not(target_os = "macos"))]
        thread::spawn(move || {
            let res = std::panic::catch_unwind(move || {
                // NDI initialization
                if let Err(e) = ndi::initialize() {
                    eprintln!("Failed to initialize NDI: {:?}", e);
                    return;
                }

                // Create Sender
                let sender = match ndi::SendBuilder::new()
                    .ndi_name("ChurchPresenter Live Output".to_string())
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
                let mut cached_background_surface: Option<ImageSurface> = None;

                println!("NDI Broadcast active: 'ChurchPresenter Live Output'");

                let mut last_slide: Option<NdiSlideData> = None;
                let mut trans_prev_slide: Option<NdiSlideData> = None;
                let mut trans_start: Option<std::time::Instant> = None;
                let mut last_sent_time = std::time::Instant::now();

                loop {
                    // Sleep for 33ms to target ~30fps
                    thread::sleep(Duration::from_millis(33));

                    // Get current slide data
                    let slide_opt = {
                        let lock = thread_slide.lock().unwrap();
                        lock.clone()
                    };

                    if let Some(slide) = slide_opt {
                        if !slide.go_live {
                            continue;
                        }

                        let slide_changed = match &last_slide {
                            None => true,
                            Some(last) => {
                                last.header != slide.header
                                    || last.body != slide.body
                                    || last.theme != slide.theme
                                    || last.blackout != slide.blackout
                                    || last.logo_mode != slide.logo_mode
                                    || last.clearout != slide.clearout
                                    || last.logo_image_path != slide.logo_image_path
                                    || last.bg_type != slide.bg_type
                                    || last.bg_path != slide.bg_path
                                    || last.font_size != slide.font_size
                                    || last.scale != slide.scale
                                    || last.align != slide.align
                                    || last.shadow != slide.shadow
                                    || last.default_song_bg_type != slide.default_song_bg_type
                                    || last.default_song_bg_val != slide.default_song_bg_val
                            }
                        };

                        if slide_changed {
                            trans_prev_slide = last_slide.clone();
                            trans_start = Some(std::time::Instant::now());
                            last_slide = Some(slide.clone());
                        }

                        // Check transition status
                        let mut is_animating = false;
                        let mut progress = 1.0f64;
                        if let Some(start) = trans_start {
                            let elapsed = start.elapsed().as_millis() as f64;
                            let duration = 300.0f64; // 300ms transition
                            if elapsed < duration {
                                is_animating = true;
                                progress = elapsed / duration;
                            } else {
                                trans_start = None;
                                trans_prev_slide = None;
                            }
                        }

                        let time_for_keep_alive =
                            last_sent_time.elapsed() >= Duration::from_millis(1000);

                        if slide_changed || is_animating || time_for_keep_alive {
                            // Apply default song background if stanza background is transparent
                            let (res_bg_type, res_bg_path) = if slide.bg_type == "transparent" {
                                (
                                    slide.default_song_bg_type.as_str(),
                                    slide.default_song_bg_val.as_deref(),
                                )
                            } else {
                                (slide.bg_type.as_str(), slide.bg_path.as_deref())
                            };

                            let active_bg_path = if slide.logo_mode {
                                slide.logo_image_path.as_deref().unwrap_or("")
                            } else if res_bg_type == "image" {
                                res_bg_path.unwrap_or("")
                            } else if (res_bg_type == "color" || res_bg_type == "theme")
                                && res_bg_path
                                    .map(|p| {
                                        std::path::Path::new(p).exists()
                                            && std::path::Path::new(p).is_file()
                                    })
                                    .unwrap_or(false)
                            {
                                res_bg_path.unwrap_or("")
                            } else if res_bg_type.is_empty() && !slide.theme.is_empty() {
                                &slide.theme
                            } else {
                                ""
                            };

                            let path = std::path::Path::new(active_bg_path);
                            if path.exists() && path.is_file() {
                                if active_bg_path != cached_background_path
                                    || cached_background_surface.is_none()
                                {
                                    if let Some(surf) = load_image_surface(active_bg_path) {
                                        cached_background_surface = Some(surf);
                                        cached_background_path = active_bg_path.to_string();
                                    }
                                }
                            } else {
                                cached_background_surface = None;
                                cached_background_path = String::new();
                            }

                            // Create cairo ImageSurface to render slide to
                            let surface_res =
                                ImageSurface::create(Format::ARgb32, width as i32, height as i32);
                            if let Ok(mut surface) = surface_res {
                                if let Ok(cr) = Context::new(&surface) {
                                    // 1. Draw target background instantly
                                    draw_background(
                                        &cr,
                                        width as f64,
                                        height as f64,
                                        if slide.logo_mode { "" } else { &slide.theme },
                                        slide.blackout,
                                        &cached_background_surface,
                                        &slide.bg_type,
                                        slide.bg_path.as_deref(),
                                        &slide.default_song_bg_type,
                                        slide.default_song_bg_val.as_deref(),
                                        slide.lower_bar_height,
                                    );

                                    // 2. Draw text
                                    if is_animating {
                                        if let Some(ref prev) = trans_prev_slide {
                                            draw_single_slide_text(
                                                &cr,
                                                width as f64,
                                                height as f64,
                                                prev,
                                                1.0 - progress,
                                            );
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
                                        draw_single_slide_text(
                                            &cr,
                                            width as f64,
                                            height as f64,
                                            &slide,
                                            1.0,
                                        );
                                    }

                                    // Drop cairo Context to release surface borrow before accessing raw data!
                                    drop(cr);

                                    // Flush and copy data
                                    surface.flush();
                                    if let Ok(data) = surface.data() {
                                        pixel_buffer.copy_from_slice(&*data);
                                        last_sent_time = std::time::Instant::now();
                                    }

                                    let video_data = VideoData::from_buffer(
                                        width as i32,
                                        height as i32,
                                        FourCCVideoType::BGRA,
                                        60,
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
                    }
                }
            });
            if let Err(e) = res {
                eprintln!("NDI background thread panicked/unwound safely: {:?}", e);
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
        bg_type: String,
        bg_path: Option<String>,
        font_size: f64,
        scale: f64,
        align: String,
        shadow: bool,
        default_song_bg_type: String,
        default_song_bg_val: Option<String>,
        lower_bar_height: f64,
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
            bg_type,
            bg_path,
            font_size,
            scale,
            align,
            shadow,
            default_song_bg_type,
            default_song_bg_val,
            lower_bar_height,
        });
    }
}
