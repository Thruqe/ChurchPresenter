use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Button, Entry, FlowBox, Image, Label, ListBox, ListBoxRow,
    Orientation, PolicyType, Popover, ScrolledWindow, Separator, Stack,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::db::{
    autocomplete_book_name, get_songs, parse_reference, query_verses, query_verses_by_mode,
};
use crate::models::{AppState, Verse};

fn draw_background(cr: &gtk::cairo::Context, width: f64, height: f64, theme: &str, blackout: bool) {
    if blackout {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.paint();
    } else {
        let path = std::path::Path::new(theme);
        if path.exists() && path.is_file() {
            if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_file(theme) {
                if let Some(scaled) = pixbuf.scale_simple(
                    width as i32,
                    height as i32,
                    gtk::gdk_pixbuf::InterpType::Bilinear,
                ) {
                    cr.set_source_pixbuf(&scaled, 0.0, 0.0);
                    let _ = cr.paint();
                } else {
                    cr.set_source_rgb(0.0, 0.0, 0.0);
                    let _ = cr.paint();
                }
            } else {
                cr.set_source_rgb(0.0, 0.0, 0.0);
                let _ = cr.paint();
            }
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

fn draw_single_slide_text(
    cr: &gtk::cairo::Context,
    width: f64,
    height: f64,
    header: &str,
    body: &str,
    logo_mode: bool,
    clearout: bool,
    blackout: bool,
    alpha: f64,
) {
    use gtk::cairo::{FontSlant, FontWeight};
    cr.select_font_face("Tahoma", FontSlant::Normal, FontWeight::Bold);

    if logo_mode && !blackout {
        cr.set_font_size(height * 0.074); // Scale font size based on height
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
    } else if !blackout && !clearout {
        let body_font_size = height * 0.06;
        let header_font_size = height * 0.045;

        println!(
            "DEBUG: UI canvas draw_single_slide_text — height={:.1}, body_font_size={:.1}",
            height, body_font_size
        );

        // Wrap body text
        let max_width = width - width * 0.15; // 15% margin
        let mut wrapped_lines = Vec::new();

        cr.set_font_size(body_font_size);
        cr.set_source_rgba(1.0, 1.0, 1.0, alpha);

        for line in body.lines() {
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
        let mut current_y = start_y + body_font_size * 0.8; // adjust for baseline
        for line in &wrapped_lines {
            if let Ok(ext) = cr.text_extents(line) {
                cr.move_to((width - ext.width()) / 2.0, current_y);
                let _ = cr.show_text(line);
            }
            current_y += line_spacing;
        }

        // Draw header (verse reference) below the body, aligned right
        cr.set_font_size(header_font_size);
        cr.set_source_rgba(0.85, 0.85, 0.85, alpha); // slightly gray for contrast
        if let Ok(ext) = cr.text_extents(header) {
            // Aligned right with same margin as body (width * 0.075 from right margin)
            let header_x = width - ext.width() - width * 0.075;
            let header_y = current_y + height * 0.02; // spacing below body
            cr.move_to(header_x, header_y);
            let _ = cr.show_text(header);
        }
    } else if clearout && !blackout {
        // Clearout (Header/reference only, centered in the middle since body is cleared)
        let header_font_size = height * 0.050;
        cr.set_font_size(header_font_size);
        cr.set_source_rgba(0.85, 0.85, 0.85, alpha);
        if let Ok(ext) = cr.text_extents(header) {
            cr.move_to((width - ext.width()) / 2.0, (height + ext.height()) / 2.0);
            let _ = cr.show_text(header);
        }
    }
}

fn draw_slide_cairo(
    cr: &gtk::cairo::Context,
    width: f64,
    height: f64,
    prev_header: &str,
    prev_body: &str,
    header: &str,
    body: &str,
    trans_start: Option<std::time::Instant>,
    theme: &str,
    blackout: bool,
    logo_mode: bool,
    clearout: bool,
) {
    // 1. Draw target background instantly
    draw_background(cr, width, height, theme, blackout);

    // 2. Draw text with transition if active
    if let Some(start) = trans_start {
        let elapsed = start.elapsed().as_millis() as f64;
        let duration = 800.0;
        if elapsed < duration {
            let progress = elapsed / duration;
            // Draw previous slide text fading out
            draw_single_slide_text(
                cr,
                width,
                height,
                prev_header,
                prev_body,
                logo_mode,
                clearout,
                blackout,
                1.0 - progress,
            );
            // Draw new slide text fading in
            draw_single_slide_text(
                cr, width, height, header, body, logo_mode, clearout, blackout, progress,
            );
            return;
        }
    }

    // Default: draw only target text at 100% opacity
    draw_single_slide_text(
        cr, width, height, header, body, logo_mode, clearout, blackout, 1.0,
    );
}

fn start_live_transition(drawing_area: &gtk::DrawingArea, state: &Rc<RefCell<AppState>>) {
    let drawing_area = drawing_area.clone();
    let state = state.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let elapsed = {
            let s = state.borrow();
            s.live_trans_start
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(999)
        };
        drawing_area.queue_draw();
        if elapsed >= 800 {
            let mut s = state.borrow_mut();
            s.live_trans_start = None;
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
}

fn get_media_dimensions(path: &str) -> Option<(i32, i32)> {
    let output = std::process::Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=s=x:p=0",
            path
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let parts: Vec<&str> = trimmed.split('x').collect();
    if parts.len() >= 2 {
        let width = parts[0].parse::<i32>().ok()?;
        let height = parts[1].parse::<i32>().ok()?;
        return Some((width, height));
    }
    None
}

fn add_theme_card(
    themes_flow: &gtk::FlowBox,
    filename: &str,
    abs_path: &str,
    state: &std::rc::Rc<std::cell::RefCell<crate::models::AppState>>,
    update_theme: &std::rc::Rc<dyn Fn()>,
) {
    use gtk::prelude::*;
    use gtk::{Box, Button, Label, Orientation, Popover};

    let theme_card = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(120)
        .spacing(6)
        .build();
    theme_card.add_css_class("media-card");

    let is_video = abs_path.to_lowercase().ends_with(".mp4")
        || abs_path.to_lowercase().ends_with(".mkv")
        || abs_path.to_lowercase().ends_with(".avi");

    let theme_thumb = Box::builder().height_request(80).build();
    theme_thumb.add_css_class("media-thumbnail-placeholder");

    if is_video {
        let icon = gtk::Image::from_icon_name("video-x-generic");
        icon.set_pixel_size(48);
        theme_thumb.append(&icon);
        theme_thumb.add_css_class("theme-dark-slate");
    } else {
        let provider = gtk::CssProvider::new();
        let formatted_path = abs_path.replace("\\", "/").replace(" ", "%20");
        let css_data = format!(
            "* {{ background-image: url('file:///{}'); background-size: cover; background-repeat: no-repeat; background-position: center; }}",
            formatted_path
        );
        provider.load_from_data(&css_data);
        theme_thumb
            .style_context()
            .add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    let theme_lbl = Label::builder().label(filename).build();
    theme_lbl.add_css_class("media-card-title");

    theme_card.append(&theme_thumb);
    theme_card.append(&theme_lbl);

    let click_gesture = gtk::GestureClick::new();
    let state_clone = state.clone();
    let path_str_clone = abs_path.to_string();
    let update_theme_clone = update_theme.clone();

    click_gesture.connect_pressed(move |_, _, _, _| {
        println!("DEBUG: Custom theme card selected: {}", path_str_clone);
        let mut s = state_clone.borrow_mut();
        s.selected_theme = "custom";
        s.custom_background_path = Some(path_str_clone.clone());
        drop(s);
        update_theme_clone();
    });
    theme_card.add_controller(click_gesture);

    let popover = Popover::builder().build();
    let popover_box = Box::builder().orientation(Orientation::Vertical).build();
    let delete_btn = Button::builder().label("Delete").has_frame(false).build();
    delete_btn.add_css_class("menu-item-button");
    popover_box.append(&delete_btn);
    popover.set_child(Some(&popover_box));
    popover.set_parent(&theme_card);

    let popover_clone = popover.clone();
    let right_click_gesture = gtk::GestureClick::builder().button(3).build();
    right_click_gesture.connect_pressed(move |_, _, x, y| {
        println!("DEBUG: Right-click triggered on custom theme card.");
        popover_clone.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover_clone.popup();
    });
    theme_card.add_controller(right_click_gesture);

    let theme_card_clone = theme_card.clone();
    let themes_flow_delete = themes_flow.clone();
    let popover_delete = popover.clone();
    let path_str_delete = abs_path.to_string();
    let state_delete_clone = state.clone();

    delete_btn.connect_clicked(move |_| {
        println!("DEBUG: Delete clicked for custom theme card.");
        popover_delete.popdown();
        crate::db::delete_theme(&path_str_delete);
        let mut s = state_delete_clone.borrow_mut();
        s.custom_themes.retain(|(_, p)| p != &path_str_delete);
        drop(s);
        themes_flow_delete.remove(&theme_card_clone);
    });

    themes_flow.insert(&theme_card, -1);
}

fn add_media_card(
    media_flow: &gtk::FlowBox,
    themes_flow: &gtk::FlowBox,
    filename: &str,
    abs_path: &str,
    state: &std::rc::Rc<std::cell::RefCell<crate::models::AppState>>,
    update_theme: &std::rc::Rc<dyn Fn()>,
) {
    use gtk::prelude::*;
    use gtk::{Box, Button, Label, Orientation, Popover};

    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(120)
        .spacing(6)
        .build();
    card.add_css_class("media-card");

    let is_video = abs_path.to_lowercase().ends_with(".mp4")
        || abs_path.to_lowercase().ends_with(".mkv")
        || abs_path.to_lowercase().ends_with(".avi");

    let thumb_container = Box::builder().height_request(80).build();
    thumb_container.add_css_class("media-thumbnail-placeholder");

    if is_video {
        println!("DEBUG: Importing a video file. Setting generic video icon.");
        let icon = gtk::Image::from_icon_name("video-x-generic");
        icon.set_pixel_size(48);
        icon.set_valign(gtk::Align::Center);
        icon.set_halign(gtk::Align::Center);
        thumb_container.append(&icon);
        thumb_container.add_css_class("theme-dark-slate");
    } else {
        println!("DEBUG: Importing an image file. Loading via CSS absolute path provider.");
        let provider = gtk::CssProvider::new();
        let formatted_path = abs_path.replace("\\", "/").replace(" ", "%20");
        let css_data = format!(
            "* {{ background-image: url('file:///{}'); background-size: cover; background-repeat: no-repeat; background-position: center; }}",
            formatted_path
        );
        println!("DEBUG: Loading CSS for thumbnail: {}", css_data);
        provider.load_from_data(&css_data);
        thumb_container
            .style_context()
            .add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    let lbl = Label::builder().label(filename).build();
    lbl.add_css_class("media-card-title");

    card.append(&thumb_container);
    card.append(&lbl);

    let popover = Popover::builder().build();
    let popover_box = Box::builder().orientation(Orientation::Vertical).build();

    let copy_theme_btn = Button::builder()
        .label("Copy to Themes")
        .has_frame(false)
        .build();
    copy_theme_btn.add_css_class("menu-item-button");

    let delete_btn = Button::builder().label("Delete").has_frame(false).build();
    delete_btn.add_css_class("menu-item-button");

    popover_box.append(&copy_theme_btn);
    popover_box.append(&delete_btn);
    popover.set_child(Some(&popover_box));
    popover.set_parent(&card);

    let popover_clone = popover.clone();
    let gesture = gtk::GestureClick::builder().button(3).build();
    gesture.connect_pressed(move |_, _, x, y| {
        println!("DEBUG: Right-click triggered on media card popover.");
        popover_clone.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover_clone.popup();
    });
    card.add_controller(gesture);

    let card_clone = card.clone();
    let media_flow_delete = media_flow.clone();
    let popover_delete = popover.clone();
    let abs_path_delete = abs_path.to_string();
    delete_btn.connect_clicked(move |_| {
        println!("DEBUG: Delete clicked for media card.");
        popover_delete.popdown();
        // Delete from database
        crate::db::delete_media(&abs_path_delete);
        media_flow_delete.remove(&card_clone);
    });

    let path_str_clone = abs_path.to_string();
    let filename_clone = filename.to_string();
    let themes_flow_clone = themes_flow.clone();
    let state_clone = state.clone();
    let update_theme_clone = update_theme.clone();

    copy_theme_btn.connect_clicked(move |_| {
        println!("DEBUG: Copy to Themes clicked for media card.");
        popover.popdown();
        let mut s = state_clone.borrow_mut();
        if !s.custom_themes.iter().any(|(_, p)| p == &path_str_clone) {
            s.custom_themes.push((filename_clone.clone(), path_str_clone.clone()));
            crate::db::add_theme(&filename_clone, &path_str_clone);
            add_theme_card(
                &themes_flow_clone,
                &filename_clone,
                &path_str_clone,
                &state_clone,
                &update_theme_clone,
            );
        }
    });

    media_flow.insert(&card, -1);
}

pub fn build_ui(app: &Application) {
    crate::db::init_media_table();
    crate::db::init_themes_table();
    let persisted_themes = crate::db::get_all_themes();

    // 1. Initialize Stylesheet
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // 2. Setup Shared Application State
    let state = Rc::new(RefCell::new(AppState {
        verses: query_verses("", "KJV"),
        selected_translation: "KJV",
        selected_verse_index: Some(0),

        songs: get_songs(),
        selected_song_index: None,
        selected_stanza_index: None,

        current_selection_type: 0,

        live_title: "Live - Standby".to_string(),
        live_slides: vec![],
        live_active_index: None,

        search_parsed_verse: Some(1),
        search_by_keyword: false,

        selected_theme: "classic-red",
        blackout: false,
        clearout: false,
        logo_mode: false,
        custom_themes: persisted_themes.clone(),
        custom_background_path: None,
        preview_header: "Genesis 1:1 (KJV)".to_string(),
        preview_body: "In the beginning God created the heaven and the earth.".to_string(),
        live_current_header: "EasyWorship".to_string(),
        live_current_body: "[Standby - Projection Off]".to_string(),
        live_prev_header: String::new(),
        live_prev_body: String::new(),
        live_trans_start: None,
    }));

    // 3. Assemble Main Layout Box
    let main_box = Box::builder().orientation(Orientation::Vertical).build();

    // --- MENU BAR ---
    let menu_bar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();
    menu_bar.add_css_class("menubar-container");

    let menus = vec![
        ("File", vec!["Restart", "Quit"]),
        ("View", vec!["Zoom In", "Zoom Out", "Fullscreen"]),
        ("Help", vec!["About"]),
    ];
    for (menu_name, items) in menus {
        let popover = Popover::builder().build();
        let popover_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        for item_name in items {
            let item_btn = Button::builder().label(item_name).can_focus(false).build();
            item_btn.add_css_class("menu-item-button");
            item_btn.set_has_frame(false);

            let popover_clone = popover.clone();
            let app_clone = app.clone();
            let item_name_str = item_name.to_string();

            item_btn.connect_clicked(move |_| {
                println!("DEBUG: connect_clicked triggered at line 299");
                popover_clone.popdown(); // Hide the popover menu

                match item_name_str.as_str() {
                    "Quit" => app_clone.quit(),
                    "Restart" => {
                        if let Ok(current_exe) = std::env::current_exe() {
                            let _ = std::process::Command::new(current_exe).spawn();
                            app_clone.quit();
                        }
                    },
                    "Fullscreen" => {
                        if let Some(window) = app_clone.active_window() {
                            if window.is_fullscreen() {
                                window.unfullscreen();
                            } else {
                                window.fullscreen();
                            }
                        }
                    },

                    "Undo" => {
                        if let Some(w) = app_clone.active_window().and_then(|win| gtk::prelude::RootExt::focus(&win)) {
                            let _ = w.activate_action("text.undo", None);
                        }
                    },
                    "Redo" => {
                        if let Some(w) = app_clone.active_window().and_then(|win| gtk::prelude::RootExt::focus(&win)) {
                            let _ = w.activate_action("text.redo", None);
                        }
                    },
                    "Cut" => {
                        if let Some(w) = app_clone.active_window().and_then(|win| gtk::prelude::RootExt::focus(&win)) {
                            let _ = w.activate_action("clipboard.cut", None);
                        }
                    },
                    "Copy" => {
                        if let Some(w) = app_clone.active_window().and_then(|win| gtk::prelude::RootExt::focus(&win)) {
                            let _ = w.activate_action("clipboard.copy", None);
                        }
                    },
                    "Paste" => {
                        if let Some(w) = app_clone.active_window().and_then(|win| gtk::prelude::RootExt::focus(&win)) {
                            let _ = w.activate_action("clipboard.paste", None);
                        }
                    },
                    "About" => {
                        if let Some(window) = app_clone.active_window() {
                            let about_win = gtk::Window::builder()
                                .title("About EasyWorship")
                                .modal(true)
                                .transient_for(&window)
                                .default_width(460)
                                .default_height(340)
                                .resizable(false)
                                .build();

                            let vbox = Box::builder()
                                .orientation(Orientation::Vertical)
                                .spacing(10)
                                .margin_top(15)
                                .margin_bottom(15)
                                .margin_start(20)
                                .margin_end(20)
                                .build();

                            let title_lbl = Label::builder()
                                .label("EasyWorship - GTK4 Edition")
                                .build();
                            title_lbl.add_css_class("about-title");

                            let ver_lbl = Label::builder()
                                .label("Version 1.0.0")
                                .build();
                            ver_lbl.add_css_class("about-version");

                            let desc_lbl = Label::builder()
                                .label("A fast, modernized presentation software built with Rust and GTK4.")
                                .wrap(true)
                                .justify(gtk::Justification::Center)
                                .build();
                            desc_lbl.add_css_class("about-description");

                            let details_lbl = Label::builder()
                                .label("Key Features:\n• Quick Scripture SQL Lookup (KJV, HCSB, RVA)\n• Dynamic Word-Wrapping with Auto-Fit Scaling\n• Presentation Theme Control & Custom Branding\n• OS-native Windows 11 Title Bar\n\nBuilt with Rust, rusqlite, and GObject GTK4 Bindings.")
                                .wrap(true)
                                .xalign(0.0)
                                .build();
                            details_lbl.add_css_class("about-details");

                            let close_btn = Button::builder()
                                .label("Close")
                                .halign(gtk::Align::Center)
                                .build();
                            close_btn.add_css_class("about-close-btn");

                            let win_clone = about_win.clone();
                            close_btn.connect_clicked(move |_| {
                                win_clone.close();
                            });

                            vbox.append(&title_lbl);
                            vbox.append(&ver_lbl);
                            vbox.append(&desc_lbl);
                            vbox.append(&details_lbl);
                            vbox.append(&close_btn);

                            about_win.set_child(Some(&vbox));
                            about_win.present();
                        }
                    },
                    _ => {
                        if let Some(window) = app_clone.active_window() {
                            let dialog = gtk::MessageDialog::builder()
                                .message_type(gtk::MessageType::Info)
                                .buttons(gtk::ButtonsType::Ok)
                                .text(&format!("'{}' Functionality Coming Soon", item_name_str))
                                .secondary_text("This feature is not yet fully implemented in this version.")
                                .modal(true)
                                .build();
                            dialog.set_transient_for(Some(&window));
                            dialog.connect_response(|d, _| d.close());
                            dialog.present();
                        }
                    }
                }
            });

            popover_box.append(&item_btn);
        }
        popover.set_child(Some(&popover_box));

        let btn = Button::builder()
            .label(menu_name)
            .has_frame(false)
            .can_focus(false)
            .build();
        popover.set_parent(&btn);

        let popover_clone = popover.clone();
        btn.connect_clicked(move |_| {
            println!("DEBUG: connect_clicked triggered at line 354");
            popover_clone.popup();
        });
        btn.add_css_class("menubar-button");
        menu_bar.append(&btn);
    }
    main_box.append(&menu_bar);

    // --- TOOLBAR ---
    let toolbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    toolbar.add_css_class("toolbar-container");

    // Left toolbar actions
    let create_toolbar_btn = |icon_name: &str, label: &str| -> Button {
        let btn_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        let icon_img = Image::builder().icon_name(icon_name).pixel_size(20).build();

        let txt_lbl = Label::builder().label(label).build();
        txt_lbl.add_css_class("toolbar-button-label");

        btn_box.append(&icon_img);
        btn_box.append(&txt_lbl);

        let btn = Button::builder().child(&btn_box).build();
        btn.add_css_class("toolbar-button");
        btn
    };

    let new_btn = create_toolbar_btn("document-new-symbolic", "New");
    let open_btn = create_toolbar_btn("document-open-symbolic", "Open");
    let save_btn = create_toolbar_btn("document-save-symbolic", "Save");

    toolbar.append(&new_btn);
    toolbar.append(&open_btn);
    toolbar.append(&save_btn);

    // Add separator spacer
    let sep1 = Separator::builder()
        .orientation(Orientation::Vertical)
        .build();
    sep1.add_css_class("toolbar-spacer");
    toolbar.append(&sep1);

    // Center/Right actions
    let go_live_btn = create_toolbar_btn("media-playback-start-symbolic", "Go Live");
    let logo_btn = create_toolbar_btn("image-x-generic-symbolic", "Logo");
    let black_btn = create_toolbar_btn("video-display-symbolic", "Black");
    let clear_btn = create_toolbar_btn("edit-clear-symbolic", "Clear");
    let monitor_btn = create_toolbar_btn("computer-symbolic", "Monitor");

    toolbar.append(&go_live_btn);
    toolbar.append(&logo_btn);
    toolbar.append(&black_btn);
    toolbar.append(&clear_btn);
    toolbar.append(&monitor_btn);

    main_box.append(&toolbar);

    // --- MAIN FIXED CONTAINERS ---
    let main_box_inner = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    main_box.append(&main_box_inner);

    let top_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .spacing(6)
        .vexpand(true)
        .build();
    main_box_inner.append(&top_box);

    // --- PREVIEW PANEL (TOP LEFT) ---
    let preview_box = Box::builder().orientation(Orientation::Vertical).build();

    let preview_header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    preview_header.add_css_class("panel-header");
    preview_header.set_size_request(-1, 44);
    let preview_title_label = Label::builder()
        .label("Preview - Philippians 4:8 (KJV)")
        .xalign(0.0)
        .build();
    preview_title_label.add_css_class("panel-title");
    preview_header.append(&preview_title_label);
    // View-mode toggle for Preview (Visual / Text)
    let preview_view_toggle = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(gtk::Align::End)
        .hexpand(true)
        .build();
    preview_view_toggle.add_css_class("view-toggle-box");

    let preview_toggle_visual = Button::builder().label("Visual").build();
    preview_toggle_visual.add_css_class("view-toggle-btn");
    preview_toggle_visual.add_css_class("view-toggle-btn-active");
    let preview_toggle_text = Button::builder().label("Text").build();
    preview_toggle_text.add_css_class("view-toggle-btn");
    preview_view_toggle.append(&preview_toggle_visual);
    preview_view_toggle.append(&preview_toggle_text);
    preview_header.append(&preview_view_toggle);
    preview_box.append(&preview_header);

    // Preview: Stack switching between visual card and text output
    let preview_stack = Stack::builder().vexpand(true).hexpand(true).build();

    let preview_slide_container = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(gtk::Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .build();
    preview_slide_container.add_css_class("preview-slide-container");
    preview_slide_container.set_size_request(-1, 260); // fixed minimum height regardless of content

    let preview_drawing_area = gtk::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    preview_drawing_area.add_css_class("preview-slide-card");

    let state_draw_preview = state.clone();
    preview_drawing_area.set_draw_func(move |_area, cr, width, height| {
        let s = state_draw_preview.borrow();
        let theme = s.selected_theme;
        let theme_str = if theme == "custom" {
            s.custom_background_path.as_deref().unwrap_or("")
        } else {
            theme
        };
        draw_slide_cairo(
            cr,
            width as f64,
            height as f64,
            "",
            "",
            &s.preview_header,
            &s.preview_body,
            None,
            theme_str,
            false,
            false,
            false,
        );
    });

    let preview_aspect_frame = gtk::AspectFrame::builder()
        .ratio(16.0 / 9.0)
        .obey_child(false)
        .xalign(0.5)
        .yalign(0.5)
        .hexpand(true)
        .vexpand(true)
        .child(&preview_drawing_area)
        .build();

    preview_slide_container.append(&preview_aspect_frame);

    // --- Text output view ---
    let preview_text_container = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    preview_text_container.add_css_class("text-output-container");

    let preview_text_ref_label = Label::builder()
        .label("Genesis 1:1 (KJV)")
        .xalign(0.0)
        .wrap(false)
        .build();
    preview_text_ref_label.add_css_class("text-output-reference");

    let preview_text_body_label = Label::builder()
        .label("")
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .width_chars(1)
        .build();
    preview_text_body_label.set_size_request(10, 10);
    preview_text_body_label.add_css_class("text-output-body");

    preview_text_container.append(&preview_text_ref_label);
    preview_text_container.append(&preview_text_body_label);

    let preview_text_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(260)
        .child(&preview_text_container)
        .vexpand(true)
        .build();

    preview_stack.add_named(&preview_slide_container, Some("visual"));
    preview_stack.add_named(&preview_text_scrolled, Some("text"));
    preview_box.append(&preview_stack);
    top_box.append(&preview_box);

    // --- LIVE PANEL (TOP RIGHT) ---
    let live_box = Box::builder().orientation(Orientation::Vertical).build();

    let live_header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    live_header.add_css_class("panel-header");
    live_header.set_size_request(-1, 44);
    let live_title_label = Label::builder()
        .label("Live - ministration")
        .xalign(0.0)
        .build();
    live_title_label.add_css_class("panel-title");
    live_header.append(&live_title_label);
    // View-mode toggle for Live (Visual / Text)
    let live_view_toggle = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(gtk::Align::End)
        .hexpand(true)
        .build();
    live_view_toggle.add_css_class("view-toggle-box");
    let live_toggle_visual = Button::builder().label("Visual").build();
    live_toggle_visual.add_css_class("view-toggle-btn");
    live_toggle_visual.add_css_class("view-toggle-btn-active");
    let live_toggle_text = Button::builder().label("Text").build();
    live_toggle_text.add_css_class("view-toggle-btn");
    live_view_toggle.append(&live_toggle_visual);
    live_view_toggle.append(&live_toggle_text);
    live_header.append(&live_view_toggle);
    live_box.append(&live_header);

    // Fixed right panel: Top is Live list queue, Bottom is small live monitor
    let live_inner_container = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    live_box.append(&live_inner_container);

    // Live slides ListBox
    // Live slides ListBox — only shown in Text mode (Visual output is the red card only)
    let live_slides_list = ListBox::builder().build();
    live_slides_list.add_css_class("live-slides-list");

    let live_scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&live_slides_list)
        .height_request(180)
        .build();
    live_scrolled_window.set_visible(false); // hidden by default (Visual is the default mode)
    live_inner_container.append(&live_scrolled_window);

    let live_monitor_container = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    live_monitor_container.add_css_class("preview-slide-container");
    live_monitor_container.set_size_request(-1, 260);

    let live_drawing_area = gtk::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    live_drawing_area.add_css_class("preview-slide-card");

    let state_draw_live = state.clone();
    live_drawing_area.set_draw_func(move |_area, cr, width, height| {
        let s = state_draw_live.borrow();
        let theme = s.selected_theme;
        let theme_str = if theme == "custom" {
            s.custom_background_path.as_deref().unwrap_or("")
        } else {
            theme
        };

        let mut active_header = String::new();
        let mut active_body = String::new();
        if let Some(active_idx) = s.live_active_index {
            if let Some((header, body)) = s.live_slides.get(active_idx) {
                active_header = header.clone();
                active_body = body.clone();
            }
        } else if s.live_slides.is_empty() {
            active_header = "EasyWorship".to_string();
            active_body = "[Standby - Projection Off]".to_string();
        }

        draw_slide_cairo(
            cr,
            width as f64,
            height as f64,
            &s.live_prev_header,
            &s.live_prev_body,
            &active_header,
            &active_body,
            s.live_trans_start,
            theme_str,
            s.blackout,
            s.logo_mode,
            s.clearout,
        );
    });

    let live_aspect_frame = gtk::AspectFrame::builder()
        .ratio(16.0 / 9.0)
        .obey_child(false)
        .xalign(0.5)
        .yalign(0.5)
        .hexpand(true)
        .vexpand(true)
        .child(&live_drawing_area)
        .build();

    live_monitor_container.append(&live_aspect_frame);

    // Text-mode view for Live
    let live_text_container = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    live_text_container.add_css_class("text-output-container");

    let live_text_ref_label = Label::builder()
        .label("LIVE OUTPUT MONITOR")
        .xalign(0.0)
        .build();
    live_text_ref_label.add_css_class("text-output-reference");

    let live_text_body_label = Label::builder()
        .label("[Standby - Projection Off]")
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .width_chars(1)
        .build();
    live_text_body_label.set_size_request(10, 10);
    live_text_body_label.add_css_class("text-output-body");

    live_text_container.append(&live_text_ref_label);
    live_text_container.append(&live_text_body_label);

    let live_text_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(260)
        .child(&live_text_container)
        .vexpand(true)
        .build();

    // Stack to switch between visual and text for the live monitor
    let live_monitor_stack = Stack::builder().hexpand(true).vexpand(true).build();
    live_monitor_stack.add_named(&live_monitor_container, Some("visual"));
    live_monitor_stack.add_named(&live_text_scrolled, Some("text"));
    live_inner_container.append(&live_monitor_stack);

    // --- Wire up View Mode Toggles ---
    {
        let stack = preview_stack.clone();
        let btn_vis = preview_toggle_visual.clone();
        let btn_txt = preview_toggle_text.clone();
        preview_toggle_visual.connect_clicked(move |_| {
            println!("DEBUG: connect_clicked triggered at line 696");
            stack.set_visible_child_name("visual");
            btn_vis.add_css_class("view-toggle-btn-active");
            btn_txt.remove_css_class("view-toggle-btn-active");
        });

        let stack = preview_stack.clone();
        let btn_vis = preview_toggle_visual.clone();
        let btn_txt = preview_toggle_text.clone();
        preview_toggle_text.connect_clicked(move |_| {
            println!("DEBUG: connect_clicked triggered at line 706");
            stack.set_visible_child_name("text");
            btn_txt.add_css_class("view-toggle-btn-active");
            btn_vis.remove_css_class("view-toggle-btn-active");
        });

        let stack = live_monitor_stack.clone();
        let btn_vis = live_toggle_visual.clone();
        let btn_txt = live_toggle_text.clone();
        let queue_list = live_scrolled_window.clone();
        live_toggle_visual.connect_clicked(move |_| {
            println!("DEBUG: live_toggle_visual clicked!");
            println!("DEBUG: connect_clicked triggered at line 717");
            stack.set_visible_child_name("visual");
            btn_vis.add_css_class("view-toggle-btn-active");
            btn_txt.remove_css_class("view-toggle-btn-active");
            queue_list.set_visible(false);
        });

        let stack = live_monitor_stack.clone();
        let btn_vis = live_toggle_visual.clone();
        let btn_txt = live_toggle_text.clone();
        let queue_list = live_scrolled_window.clone();
        live_toggle_text.connect_clicked(move |_| {
            println!("DEBUG: live_toggle_text clicked!");
            println!("DEBUG: connect_clicked triggered at line 730");
            stack.set_visible_child_name("text");
            btn_txt.add_css_class("view-toggle-btn-active");
            btn_vis.remove_css_class("view-toggle-btn-active");
            queue_list.set_visible(true);
        });
    }

    top_box.append(&live_box);

    let bottom_box = Box::builder().orientation(Orientation::Vertical).build();
    bottom_box.set_size_request(-1, 390);
    main_box_inner.append(&bottom_box);

    // Start NDI Broadcast Output
    let ndi_out = crate::ndi_out::NdiOutput::new();

    // Setup active state updater for slide previews
    let update_slide_theme_classes: Rc<dyn Fn()> = Rc::new({
        let preview_drawing_area = preview_drawing_area.clone();
        let live_drawing_area = live_drawing_area.clone();
        let state = state.clone();
        let ndi_out = ndi_out.clone();

        move || {
            let s = state.borrow();
            let theme = s.selected_theme;

            preview_drawing_area.queue_draw();
            live_drawing_area.queue_draw();

            // Update NDI output
            let mut active_header = String::new();
            let mut active_body = String::new();
            if let Some(active_idx) = s.live_active_index {
                if let Some((header, body)) = s.live_slides.get(active_idx) {
                    active_header = header.clone();
                    active_body = body.clone();
                }
            } else if s.live_slides.is_empty() {
                active_header = "EasyWorship".to_string();
                active_body = "[Standby - Projection Off]".to_string();
            }

            ndi_out.update_slide(
                active_header,
                active_body,
                if theme == "custom" {
                    s.custom_background_path.clone().unwrap_or_default()
                } else {
                    theme.to_string()
                },
                s.blackout,
                s.logo_mode,
                s.clearout,
            );
        }
    });

    // Tab Bar container
    let tab_bar = Box::builder().orientation(Orientation::Horizontal).build();
    tab_bar.add_css_class("tabbar-container");
    tab_bar.set_size_request(-1, 44);

    let tab_btn_songs = Button::builder().label("Songs").build();
    tab_btn_songs.add_css_class("tab-button");

    let tab_btn_scriptures = Button::builder().label("Scriptures").build();
    tab_btn_scriptures.add_css_class("tab-button");
    tab_btn_scriptures.add_css_class("tab-button-active"); // Active by default

    let tab_btn_media = Button::builder().label("Media").build();
    tab_btn_media.add_css_class("tab-button");

    let tab_btn_presentations = Button::builder().label("Presentations").build();
    tab_btn_presentations.add_css_class("tab-button");

    let tab_btn_themes = Button::builder().label("Themes").build();
    tab_btn_themes.add_css_class("tab-button");

    tab_bar.append(&tab_btn_songs);
    tab_bar.append(&tab_btn_scriptures);
    tab_bar.append(&tab_btn_media);
    tab_bar.append(&tab_btn_presentations);
    tab_bar.append(&tab_btn_themes);

    // Spacer
    let tab_spacer = Box::builder().hexpand(true).build();
    tab_bar.append(&tab_spacer);

    // Tab actions on the right side
    let tab_actions = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();
    tab_actions.add_css_class("tabbar-actions");
    let add_item_btn = Button::builder().label("+").build();
    add_item_btn.add_css_class("tab-action-button");
    let play_item_btn = Button::builder().label(">").build();
    play_item_btn.add_css_class("tab-action-button");

    tab_actions.append(&add_item_btn);
    tab_actions.append(&play_item_btn);
    tab_bar.append(&tab_actions);

    bottom_box.append(&tab_bar);

    // Stack container for tab panels
    let resource_stack = Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    bottom_box.append(&resource_stack);

    // --- TAB PAGE 1: SONGS ---
    let songs_view = Box::builder().orientation(Orientation::Horizontal).build();

    // Songs Sidebar List
    let songs_sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(220)
        .build();
    songs_sidebar.add_css_class("sidebar-container");

    let songs_sidebar_lbl = Label::builder().label("SONGS DATABASE").xalign(0.0).build();
    songs_sidebar_lbl.add_css_class("sidebar-section-header");
    songs_sidebar.append(&songs_sidebar_lbl);

    let songs_list_box = ListBox::builder().build();
    let songs_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&songs_list_box)
        .vexpand(true)
        .build();
    songs_sidebar.append(&songs_scrolled);
    songs_view.append(&songs_sidebar);

    // Songs Main table (Stanzas)
    let songs_main = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();

    let songs_table_header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    songs_table_header.add_css_class("table-header");
    let s_col1 = Label::builder()
        .label("Stanza")
        .xalign(0.0)
        .width_request(80)
        .build();
    s_col1.add_css_class("table-header-col");
    let s_col2 = Label::builder()
        .label("Lyrics / Text Content")
        .xalign(0.0)
        .hexpand(true)
        .build();
    s_col2.add_css_class("table-header-col");
    songs_table_header.append(&s_col1);
    songs_table_header.append(&s_col2);
    songs_main.append(&songs_table_header);

    let song_stanzas_list_box = ListBox::builder().build();
    let song_stanzas_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&song_stanzas_list_box)
        .vexpand(true)
        .build();
    songs_main.append(&song_stanzas_scrolled);
    songs_view.append(&songs_main);

    resource_stack.add_named(&songs_view, Some("songs"));

    // Populate Songs Sidebar
    for song in state.borrow().songs.iter() {
        let row_lbl = Label::builder()
            .label(&format!("{}", song.title))
            .xalign(0.0)
            .build();
        row_lbl.add_css_class("sidebar-row");
        songs_list_box.append(&row_lbl);
    }

    // --- TAB PAGE 2: SCRIPTURES ---
    let scriptures_view = Box::builder().orientation(Orientation::Horizontal).build();

    // Scriptures Sidebar
    let scriptures_sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(220)
        .build();
    scriptures_sidebar.add_css_class("sidebar-container");

    let script_search = Entry::builder()
        .text("Genesis 1:1")
        .primary_icon_name("view-list-symbolic")
        .primary_icon_tooltip_text("Searching by Reference (Click to toggle keyword search)")
        .placeholder_text("Search...")
        .build();
    script_search.add_css_class("sidebar-search");

    let state_clone = state.clone();
    script_search.connect_icon_press(move |entry, icon_pos| {
        if icon_pos == gtk::EntryIconPosition::Primary {
            let mut s = state_clone.borrow_mut();
            s.search_by_keyword = !s.search_by_keyword;
            let is_keyword = s.search_by_keyword;
            drop(s);

            if is_keyword {
                entry.set_primary_icon_name(Some("edit-find-symbolic"));
                entry.set_primary_icon_tooltip_text(Some(
                    "Searching by Keyword (Click to toggle reference search)",
                ));
            } else {
                entry.set_primary_icon_name(Some("view-list-symbolic"));
                entry.set_primary_icon_tooltip_text(Some(
                    "Searching by Reference (Click to toggle keyword search)",
                ));
            }

            entry.activate();
        }
    });

    scriptures_sidebar.append(&script_search);

    let script_sidebar_lbl = Label::builder().label("TRANSLATIONS").xalign(0.0).build();
    script_sidebar_lbl.add_css_class("sidebar-section-header");
    scriptures_sidebar.append(&script_sidebar_lbl);
    let translations_list_box = ListBox::builder().build();

    let t_versions = vec!["HCSB", "KJV", "RVA"];
    for version in &t_versions {
        let row = Box::builder().orientation(Orientation::Horizontal).build();
        row.add_css_class("sidebar-row");
        if *version == "KJV" {
            row.add_css_class("sidebar-row-selected");
        }
        let lbl = Label::builder()
            .label(&format!("{}", version))
            .xalign(0.0)
            .build();
        row.append(&lbl);
        translations_list_box.append(&row);
    }

    let t_more = Label::builder()
        .label("More Available...")
        .xalign(0.0)
        .build();
    t_more.add_css_class("sidebar-row");
    translations_list_box.append(&t_more);

    let script_sidebar_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&translations_list_box)
        .height_request(140)
        .vexpand(false)
        .build();
    scriptures_sidebar.append(&script_sidebar_scrolled);

    let sep = Separator::new(Orientation::Horizontal);
    scriptures_sidebar.append(&sep);

    let books_section_lbl = Label::builder().label("BOOKS").xalign(0.0).build();
    books_section_lbl.add_css_class("sidebar-section-header");
    scriptures_sidebar.append(&books_section_lbl);

    let books_list_box = ListBox::builder().build();
    let books = crate::db::get_all_books();
    for book in &books {
        let row_lbl = Label::builder().label(book).xalign(0.0).build();
        row_lbl.add_css_class("sidebar-row");
        books_list_box.append(&row_lbl);
    }

    {
        let script_search = script_search.clone();
        let books = books.clone();
        books_list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let idx = row.index() as usize;
                if idx < books.len() {
                    let book_name = &books[idx];
                    script_search.set_text(&format!("{} 1:1", book_name));
                    script_search.activate();
                }
            }
        });
    }

    let books_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&books_list_box)
        .vexpand(true)
        .build();
    scriptures_sidebar.append(&books_scrolled);
    scriptures_view.append(&scriptures_sidebar);

    // Scriptures Main Table (Verses List)
    let scriptures_main = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();

    let scriptures_table_header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .build();
    scriptures_table_header.add_css_class("table-header");
    scriptures_table_header.set_size_request(-1, 36);

    let v_col1 = Label::builder()
        .label("Translation")
        .xalign(0.0)
        .width_request(100)
        .build();
    v_col1.add_css_class("table-header-col");
    v_col1.add_css_class("table-cell-border");
    let v_col2 = Label::builder()
        .label("Reference")
        .xalign(0.0)
        .width_request(150)
        .build();
    v_col2.add_css_class("table-header-col");
    v_col2.add_css_class("table-cell-border");
    let v_col3 = Label::builder()
        .label("Scripture")
        .xalign(0.0)
        .hexpand(true)
        .build();
    v_col3.add_css_class("table-header-col");

    scriptures_table_header.append(&v_col1);
    scriptures_table_header.append(&v_col2);
    scriptures_table_header.append(&v_col3);
    scriptures_main.append(&scriptures_table_header);

    let scriptures_list_box = ListBox::builder().activate_on_single_click(false).build();
    let scriptures_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&scriptures_list_box)
        .vexpand(true)
        .build();
    scriptures_main.append(&scriptures_scrolled);
    scriptures_view.append(&scriptures_main);

    resource_stack.add_named(&scriptures_view, Some("scriptures"));

    // --- TAB PAGE 5: THEMES (instantiated early for scope sharing) ---
    let themes_flow = FlowBox::builder()
        .max_children_per_line(5)
        .min_children_per_line(2)
        .selection_mode(gtk::SelectionMode::None)
        .build();
    themes_flow.add_css_class("media-grid");

    // --- TAB PAGE 3: MEDIA ---
    let media_main = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    let media_toolbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(10)
        .margin_end(10)
        .build();

    let import_media_btn = Button::builder().label("Import Media").build();
    import_media_btn.add_css_class("tab-action-button");
    media_toolbar.append(&import_media_btn);
    media_main.append(&media_toolbar);

    let media_flow = FlowBox::builder()
        .max_children_per_line(5)
        .min_children_per_line(2)
        .selection_mode(gtk::SelectionMode::None)
        .build();
    media_flow.add_css_class("media-grid");

    let media_scrolled = ScrolledWindow::builder()
        .child(&media_flow)
        .vexpand(true)
        .build();
    media_main.append(&media_scrolled);

    let create_media_card = |title: &str, color_class: &str| -> Box {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(120)
            .spacing(6)
            .build();
        card.add_css_class("media-card");

        let thumb = Box::builder().height_request(80).build();
        thumb.add_css_class("media-thumbnail-placeholder");
        thumb.add_css_class(color_class);

        let lbl = Label::builder().label(title).build();
        lbl.add_css_class("media-card-title");

        card.append(&thumb);
        card.append(&lbl);
        card
    };

    media_flow.insert(&create_media_card("Abstract Blue", "theme-royal-blue"), -1);
    media_flow.insert(
        &create_media_card("Classic Crimson", "theme-classic-red"),
        -1,
    );
    media_flow.insert(&create_media_card("Deep Emerald", "theme-forest-green"), -1);
    media_flow.insert(&create_media_card("Slate Motion", "theme-dark-slate"), -1);

    // Load persisted media cards from SQLite
    let persisted_media = crate::db::get_all_media();
    for (name, path) in &persisted_media {
        add_media_card(
            &media_flow,
            &themes_flow,
            name,
            path,
            &state,
            &update_slide_theme_classes,
        );
    }

    resource_stack.add_named(&media_main, Some("media"));

    // Set up file chooser and card adding logic
    let media_flow_clone = media_flow.clone();
    let themes_flow_clone = themes_flow.clone();
    let state_clone = state.clone();
    let update_theme_clone = update_slide_theme_classes.clone();
    let scriptures_list_box_clone = scriptures_list_box.clone();

    import_media_btn.connect_clicked(move |_| {
        if let Some(window) = scriptures_list_box_clone
            .root()
            .and_then(|r| r.downcast::<gtk::Window>().ok())
        {
            let dialog = gtk::FileChooserNative::new(
                Some("Import Media File"),
                Some(&window),
                gtk::FileChooserAction::Open,
                Some("Import"),
                Some("Cancel"),
            );

            let filter = gtk::FileFilter::new();
            filter.add_pattern("*.png");
            filter.add_pattern("*.jpg");
            filter.add_pattern("*.jpeg");
            filter.add_pattern("*.mp4");
            filter.set_name(Some("Media Files (PNG, JPG, MP4)"));
            dialog.add_filter(&filter);

            let media_flow = media_flow_clone.clone();
            let themes_flow = themes_flow_clone.clone();
            let state = state_clone.clone();
            let update_theme = update_theme_clone.clone();

            dialog.connect_response(move |d, res| {
                println!(
                    "DEBUG: connect_response triggered. Response type: {:?}",
                    res
                );
                if res == gtk::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            let filename = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Imported Media".to_string());

                            println!("DEBUG: Chosen file path: {}", path_str);

                            // Copy/Resize/Compress file to workspace saves/imported_media folder
                            let saves_dir = "/home/thruqe/Documents/Church-Presenter/saves";
                            let media_dir = format!("{}/imported_media", saves_dir);
                            std::fs::create_dir_all(&media_dir).ok();
                            let dest_path = format!("{}/{}", media_dir, filename);

                            // Check if the destination path already exists
                            let dest_path_buf = std::path::Path::new(&dest_path);
                            let already_exists = dest_path_buf.exists();

                            if !already_exists {
                                // Check dimensions using ffprobe
                                let needs_resize = if let Some((w, h)) = get_media_dimensions(&path_str) {
                                    w > 1920 || h > 1080
                                } else {
                                    false
                                };

                                if needs_resize {
                                    let is_video = path_str.to_lowercase().ends_with(".mp4")
                                        || path_str.to_lowercase().ends_with(".mkv")
                                        || path_str.to_lowercase().ends_with(".avi");

                                    let mut cmd = std::process::Command::new("ffmpeg");
                                    cmd.arg("-y").arg("-i").arg(&path_str);
                                    cmd.arg("-vf").arg("scale='min(1920,iw)':'min(1080,ih)':force_original_aspect_ratio=decrease");
                                    if is_video {
                                        cmd.arg("-c:v").arg("libx264").arg("-crf").arg("20");
                                    } else {
                                        let is_png = path_str.to_lowercase().ends_with(".png");
                                        if is_png {
                                            cmd.arg("-compression_level").arg("9");
                                        } else {
                                            cmd.arg("-q:v").arg("8");
                                        }
                                    }
                                    cmd.arg(&dest_path);

                                    println!("DEBUG: Running ffmpeg to resize/compress: {:?}", cmd);
                                    match cmd.output() {
                                        Ok(output) => {
                                            if !output.status.success() {
                                                let err_msg = String::from_utf8_lossy(&output.stderr);
                                                println!("DEBUG: Ffmpeg command failed: {}. Falling back to copying.", err_msg);
                                                let _ = std::fs::copy(&path_str, &dest_path);
                                            } else {
                                                println!("DEBUG: Ffmpeg successfully resized and saved media to {}", dest_path);
                                            }
                                        }
                                        Err(e) => {
                                            println!("DEBUG: Failed to execute ffmpeg: {:?}. Falling back to copying.", e);
                                            let _ = std::fs::copy(&path_str, &dest_path);
                                        }
                                    }
                                } else {
                                    println!("DEBUG: Media is <= 1920x1080, copying directly.");
                                    if let Err(e) = std::fs::copy(&path_str, &dest_path) {
                                        println!("DEBUG: Copy error: {:?}", e);
                                    }
                                }
                            } else {
                                println!("DEBUG: Media already exists in saves/imported_media, skipping copy/compression.");
                            }

                            // Build clean absolute path of copied file
                            let abs_path = if let Ok(cd) = std::env::current_dir() {
                                let joined = cd.join(&dest_path);
                                let clean = joined.to_string_lossy().to_string();
                                if clean.starts_with("\\\\?\\") {
                                    clean[4..].to_string()
                                } else {
                                    clean
                                }
                            } else {
                                dest_path.clone()
                            };
                            println!("DEBUG: Resolved absolute destination path: {}", abs_path);

                            // Save to SQLite
                            crate::db::add_media(&filename, &abs_path);

                            // Add to UI
                            add_media_card(
                                &media_flow,
                                &themes_flow,
                                &filename,
                                &abs_path,
                                &state,
                                &update_theme,
                            );
                        }
                    }
                }
                d.destroy();
            });

            dialog.show();
        }
    });

    // --- TAB PAGE 4: PRESENTATIONS ---
    let pres_list = ListBox::builder().build();
    let p_lbl1 = Label::builder()
        .label("Sunday Worship Service 2026-07-12")
        .xalign(0.0)
        .build();
    p_lbl1.add_css_class("sidebar-row");
    let p_lbl2 = Label::builder()
        .label("Youth Fellowship Slides")
        .xalign(0.0)
        .build();
    p_lbl2.add_css_class("sidebar-row");

    pres_list.append(&p_lbl1);
    pres_list.append(&p_lbl2);

    let pres_scrolled = ScrolledWindow::builder().child(&pres_list).build();
    resource_stack.add_named(&pres_scrolled, Some("presentations"));

    // --- TAB PAGE 5: THEMES ---
    let themes_scrolled = ScrolledWindow::builder().child(&themes_flow).build();

    let create_theme_card = |title: &str, class: &str| -> Box {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(120)
            .spacing(6)
            .build();
        card.add_css_class("media-card");

        let preview = Box::builder().height_request(80).build();
        preview.add_css_class("media-thumbnail-placeholder");
        preview.add_css_class(class);

        let lbl = Label::builder().label(title).build();
        lbl.add_css_class("media-card-title");

        card.append(&preview);
        card.append(&lbl);
        card
    };

    let theme_card_red = create_theme_card("Classic Red", "theme-classic-red");
    let theme_card_blue = create_theme_card("Royal Blue", "theme-royal-blue");
    let theme_card_green = create_theme_card("Forest Green", "theme-forest-green");
    let theme_card_slate = create_theme_card("Dark Slate", "theme-dark-slate");
    let theme_card_black = create_theme_card("Black", "theme-black");

    themes_flow.insert(&theme_card_red, -1);
    themes_flow.insert(&theme_card_blue, -1);
    themes_flow.insert(&theme_card_green, -1);
    themes_flow.insert(&theme_card_slate, -1);
    themes_flow.insert(&theme_card_black, -1);

    for (name, path) in &persisted_themes {
        add_theme_card(
            &themes_flow,
            name,
            path,
            &state,
            &update_slide_theme_classes,
        );
    }

    resource_stack.add_named(&themes_scrolled, Some("themes"));

    // Default stack page
    resource_stack.set_visible_child_name("scriptures");

    // --- STATUS BAR ---
    let status_bar = Box::builder().orientation(Orientation::Horizontal).build();
    status_bar.add_css_class("statusbar");
    let status_lbl = Label::builder()
        .label("KJV  |  Genesis 1:1 (Selected)  |  31 references available")
        .hexpand(true)
        .halign(gtk::Align::End)
        .build();
    status_bar.append(&status_lbl);
    main_box.append(&status_bar);

    // ==========================================
    // INTERACTIVE FUNCTIONS & EVENT HANDLERS
    // ==========================================

    // Shared context menu popover for scripture copying to prevent duplicate parenting assertions
    let context_popover = Popover::builder().has_arrow(true).build();
    let context_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    let copy_btn = Button::builder()
        .label("Copy Verse")
        .focusable(false)
        .build();
    copy_btn.add_css_class("menu-item-button");
    copy_btn.set_has_frame(false);
    context_box.append(&copy_btn);
    context_popover.set_child(Some(&context_box));

    let text_to_copy = Rc::new(RefCell::new(String::new()));
    {
        let text_to_copy = text_to_copy.clone();
        let copy_btn_clone = copy_btn.clone();
        let context_popover_clone = context_popover.clone();
        let copy_btn_action = copy_btn.clone();
        copy_btn_clone.connect_clicked(move |_| {
            let text = text_to_copy.borrow().clone();
            if let Some(display) = gtk::gdk::Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_text(&text);
            }
            copy_btn_action.set_label("Copied!");
            copy_btn_action.add_css_class("copied-active");

            let popover_c = context_popover_clone.clone();
            let btn_c = copy_btn_action.clone();
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(800), move || {
                popover_c.popdown();
                btn_c.set_label("Copy Verse");
                btn_c.remove_css_class("copied-active");
            });
        });
    }

    // Helper to reload/filter the verses list in the Scriptures table
    let populate_verses_table = {
        let state = state.clone();
        let scriptures_list_box = scriptures_list_box.clone();
        let context_popover = context_popover.clone();
        let text_to_copy = text_to_copy.clone();

        move || {
            println!("DEBUG: Closure executing...");
            println!("DEBUG: move || closure triggered at line 1104");
            // Unparent the shared context popover first to prevent critical GTK assertion failures
            if context_popover.parent().is_some() {
                context_popover.unparent();
            }

            // Clear current listbox
            while let Some(child) = scriptures_list_box.first_child() {
                scriptures_list_box.remove(&child);
            }

            // Extract values and close the borrow immediately to prevent RefCell borrow panics during connect_row_selected
            let (verses, selected_translation, search_parsed_verse, selected_verse_index) = {
                let s = state.borrow();
                (
                    s.verses.clone(),
                    s.selected_translation,
                    s.search_parsed_verse,
                    s.selected_verse_index,
                )
            };

            let mut target_row: Option<ListBoxRow> = None;

            for (idx, verse) in verses.iter().enumerate() {
                if verse.translation == selected_translation {
                    let row_box = Box::builder()
                        .orientation(Orientation::Horizontal)
                        .spacing(0)
                        .build();
                    row_box.add_css_class("table-row");

                    let trans_lbl = Label::builder()
                        .label(&verse.translation)
                        .xalign(0.0)
                        .width_request(100)
                        .build();
                    trans_lbl.add_css_class("table-cell-text");
                    trans_lbl.add_css_class("table-cell-border");

                    let ref_lbl = Label::builder()
                        .label(&verse.reference)
                        .xalign(0.0)
                        .width_request(150)
                        .build();
                    ref_lbl.add_css_class("table-cell-text");
                    ref_lbl.add_css_class("table-cell-border");

                    let body_lbl = Label::builder()
                        .label(&verse.text)
                        .xalign(0.0)
                        .hexpand(true)
                        .wrap(true)
                        .wrap_mode(gtk::pango::WrapMode::WordChar)
                        .build();
                    body_lbl.add_css_class("table-cell-text");
                    body_lbl.set_max_width_chars(1);

                    row_box.append(&trans_lbl);
                    row_box.append(&ref_lbl);
                    row_box.append(&body_lbl);

                    let row = ListBoxRow::builder().child(&row_box).build();
                    row.set_size_request(-1, 48);

                    let gesture = gtk::GestureClick::builder().button(3).build();
                    let context_popover_clone = context_popover.clone();
                    let text_to_copy_clone = text_to_copy.clone();
                    let verse_text = format!("{} - {}", verse.reference, verse.text);
                    let row_clone = row.clone();
                    gesture.connect_pressed(move |_, _, x, y| {
                        *text_to_copy_clone.borrow_mut() = verse_text.clone();

                        if context_popover_clone.parent().is_some() {
                            context_popover_clone.unparent();
                        }
                        context_popover_clone.set_parent(&row_clone);

                        context_popover_clone.set_pointing_to(Some(&gtk::gdk::Rectangle::new(
                            x as i32, y as i32, 1, 1,
                        )));
                        context_popover_clone.popup();
                    });
                    row.add_controller(gesture);

                    scriptures_list_box.append(&row);

                    // Check if we should select this row
                    if let Some(target_v) = search_parsed_verse {
                        if let Some(colon_idx) = verse.reference.rfind(':') {
                            if let Ok(v_num) = verse.reference[colon_idx + 1..].parse::<i32>() {
                                if v_num == target_v {
                                    target_row = Some(row.clone());
                                }
                            }
                        }
                    } else if Some(idx) == selected_verse_index {
                        target_row = Some(row.clone());
                    }
                }
            }

            if let Some(row) = target_row {
                scriptures_list_box.select_row(Some(&row));
                let row_clone = row.clone();
                gtk::glib::idle_add_local_once(move || {
                    row_clone.grab_focus();
                });
            } else {
                if let Some(row) = scriptures_list_box.row_at_index(0) {
                    scriptures_list_box.select_row(Some(&row));
                    let row_clone = row.clone();
                    gtk::glib::idle_add_local_once(move || {
                        row_clone.grab_focus();
                    });
                }
            }
        }
    };

    // Load initial verses
    populate_verses_table();

    // Callback when selecting a scripture verse in the table

    let preview_title_label_clone = preview_title_label.clone();
    let preview_drawing_area_clone = preview_drawing_area.clone();
    let preview_text_ref_label_clone = preview_text_ref_label.clone();
    let preview_text_body_label_clone = preview_text_body_label.clone();
    let status_lbl_clone = status_lbl.clone();
    let state_clone = state.clone();

    scriptures_list_box.connect_row_selected(move |_, row| {
        println!("DEBUG: connect_row_selected triggered at line 1243");
        if let Some(row) = row {
            let row_idx = row.index() as usize;
            let mut s = state_clone.borrow_mut();

            // Map the filtered row index back to the original index
            let verse_data = {
                let filtered_verses: Vec<(usize, Verse)> = s
                    .verses
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| v.translation == s.selected_translation)
                    .map(|(idx, v)| (idx, v.clone()))
                    .collect();
                filtered_verses.get(row_idx).cloned()
            };

            if let Some((orig_idx, verse)) = verse_data {
                s.selected_verse_index = Some(orig_idx);
                s.current_selection_type = 0; // Scripture

                println!(
                    "DEBUG: Selected verse: {} ({})",
                    verse.reference, verse.translation
                );
                println!("DEBUG: Verse text length: {} chars", verse.text.len());
                println!("DEBUG: Verse text: {}", verse.text);

                let ref_str = format!("{} ({})", verse.reference, verse.translation);
                preview_title_label_clone.set_text(&format!("Preview - {}", ref_str));

                s.preview_header = ref_str;
                s.preview_body = verse.text.clone();

                let verses_len = s.verses.len();
                drop(s);
                preview_drawing_area_clone.queue_draw();

                preview_text_ref_label_clone
                    .set_text(&format!("{} ({})", verse.reference, verse.translation));
                preview_text_body_label_clone.set_text(&verse.text);
                status_lbl_clone.set_text(&format!(
                    "{}  |  {} (Selected)  |  {} references available",
                    verse.translation, verse.reference, verses_len
                ));
            }
        }
    });

    // Callback when double-clicking a scripture verse -> Go Live
    let live_slides_list_clone = live_slides_list.clone();
    let live_title_label_clone = live_title_label.clone();
    let live_text_ref_label_clone = live_text_ref_label.clone();
    let live_text_body_label_clone = live_text_body_label.clone();
    let state_clone = state.clone();

    // Helper to push state's active live items to the Live layout
    let update_live_layout = {
        let live_slides_list = live_slides_list_clone.clone();
        let live_title_label = live_title_label_clone.clone();
        let live_drawing_area = live_drawing_area.clone();
        let live_text_ref_label = live_text_ref_label_clone.clone();
        let live_text_body_label = live_text_body_label_clone.clone();
        let state = state_clone.clone();
        let ndi_out = ndi_out.clone();

        move || {
            println!("DEBUG: Closure executing...");
            println!("DEBUG: move || closure triggered at line 1308");
            // Clear current listbox
            while let Some(child) = live_slides_list.first_child() {
                live_slides_list.remove(&child);
            }

            let (live_slides, live_active_index, live_title, blackout_val, logo_val, clearout_val) = {
                let s = state.borrow();
                (
                    s.live_slides.clone(),
                    s.live_active_index,
                    s.live_title.clone(),
                    s.blackout,
                    s.logo_mode,
                    s.clearout,
                )
            };

            // Update title
            live_title_label.set_text(&live_title);

            // Populate rows
            for (i, (header, body)) in live_slides.iter().enumerate() {
                let row_box = Box::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(8)
                    .build();
                row_box.add_css_class("live-slide-row");

                let num_lbl = Label::builder().label(&format!("{}", i + 1)).build();
                num_lbl.add_css_class("live-slide-number");

                let text_lbl = Label::builder()
                    .label(&body.chars().take(80).collect::<String>()) // truncated summary
                    .xalign(0.0)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .build();
                text_lbl.add_css_class("live-slide-text");

                row_box.append(&num_lbl);
                row_box.append(&text_lbl);

                let row = ListBoxRow::builder().child(&row_box).build();

                if Some(i) == live_active_index {
                    row.add_css_class("live-slide-active");

                    // Update monitor screen contents
                    if blackout_val {
                        live_text_ref_label.set_text("");
                        live_text_body_label.set_text("");
                    } else if logo_val {
                        live_text_ref_label.set_text("EasyWorship");
                        live_text_body_label.set_text("✝\nStandby Screen");
                    } else if clearout_val {
                        live_text_ref_label.set_text(header);
                        live_text_body_label.set_text("");
                    } else {
                        live_text_ref_label.set_text(header);
                        live_text_body_label.set_text(body);
                    }
                }

                live_slides_list.append(&row);
            }

            if live_slides.is_empty() {
                live_text_ref_label.set_text("LIVE OUTPUT MONITOR");
                live_text_body_label.set_text("[Standby - Projection Off]");
            }

            // Update NDI output
            let mut active_header = String::new();
            let mut active_body = String::new();
            if let Some(active_idx) = live_active_index {
                if let Some((header, body)) = live_slides.get(active_idx) {
                    active_header = header.clone();
                    active_body = body.clone();
                }
            } else if live_slides.is_empty() {
                active_header = "EasyWorship".to_string();
                active_body = "[Standby - Projection Off]".to_string();
            }

            let mut s = state.borrow_mut();
            let theme_str = if s.selected_theme == "custom" {
                s.custom_background_path.clone().unwrap_or_default()
            } else {
                s.selected_theme.to_string()
            };

            let live_changed =
                s.live_current_header != active_header || s.live_current_body != active_body;
            if live_changed {
                s.live_prev_header = s.live_current_header.clone();
                s.live_prev_body = s.live_current_body.clone();
                s.live_current_header = active_header.clone();
                s.live_current_body = active_body.clone();
                s.live_trans_start = Some(std::time::Instant::now());

                drop(s);
                start_live_transition(&live_drawing_area, &state);
            } else {
                drop(s);
                live_drawing_area.queue_draw();
            }

            ndi_out.update_slide(
                active_header,
                active_body,
                theme_str,
                blackout_val,
                logo_val,
                clearout_val,
            );
        }
    };

    let update_live_layout_clone = update_live_layout.clone();
    let state_clone = state.clone();

    scriptures_list_box.connect_row_activated(move |_, row| {
        let mut s = state_clone.borrow_mut();
        let row_idx = row.index() as usize;

        if row_idx < s.verses.len() {
            // Only push the single activated verse to live — not the whole chapter
            let verse = s
                .verses
                .iter()
                .filter(|v| v.translation == s.selected_translation)
                .nth(row_idx);
            if let Some(v) = verse {
                let slide = (
                    format!("{} ({})", v.reference, v.translation),
                    v.text.clone(),
                );
                s.live_slides = vec![slide.clone()];
                s.live_title = format!("Live - {}", slide.0);
                s.live_active_index = Some(0);

                // Reset screen flags
                s.blackout = false;
                s.clearout = false;
                s.logo_mode = false;
            }
        }

        drop(s);
        update_live_layout_clone();
    });

    // ── Book-name autocomplete + free-text chapter:verse search ────────────────
    {
        let run_search = {
            let state = state.clone();
            let populate = populate_verses_table.clone();
            let entry = script_search.clone();
            move || {
                println!("DEBUG: Closure executing...");
                println!("DEBUG: move || closure triggered at line 1422");
                let query = entry.text().to_string();
                let active_trans = state.borrow().selected_translation;
                let by_keyword = state.borrow().search_by_keyword;
                let new_verses = query_verses_by_mode(&query, active_trans, by_keyword);
                let (_book, _chap, verse) = parse_reference(&query);
                let mut s = state.borrow_mut();
                s.verses = new_verses;
                s.selected_verse_index = None;
                s.search_parsed_verse = verse;
                drop(s);
                populate();
            }
        };
        let run_search = Rc::new(run_search);

        // Handles Right-arrow (accept autocomplete) and Enter (run search)
        // Handles Right-arrow (accept autocomplete suggestion)
        let key_ctrl = gtk::EventControllerKey::new();
        key_ctrl.connect_key_pressed({
            let entry = script_search.clone();
            let run_search = run_search.clone();
            let state = state.clone();
            move |_ctrl, key, _code, _mods| {
                use gtk::gdk::Key;

                if state.borrow().search_by_keyword {
                    return gtk::glib::Propagation::Proceed;
                }

                if key == Key::Right {
                    if let Some((start, end)) = entry.selection_bounds() {
                        if end == entry.text().len() as i32 && start < end {
                            entry.select_region(end, end);
                            entry.set_position(end);

                            // Book just got accepted — auto-populate chapter 1
                            // if the entry doesn't already have a chapter/verse.
                            let text = entry.text().to_string();
                            if !text.contains(':') {
                                run_search();
                            }
                            return gtk::glib::Propagation::Stop;
                        }
                    }
                }

                gtk::glib::Propagation::Proceed
            }
        });
        script_search.add_controller(key_ctrl);

        // Auto-populate chapter 1 when the entry loses focus with just a book name typed
        let focus_ctrl = gtk::EventControllerFocus::new();
        focus_ctrl.connect_leave({
            let entry = script_search.clone();
            let run_search = run_search.clone();
            let state = state.clone();
            move |_ctrl| {
                if state.borrow().search_by_keyword {
                    return;
                }
                let text = entry.text().to_string();
                if !text.is_empty() && !text.contains(':') {
                    run_search();
                }
            }
        });
        script_search.add_controller(focus_ctrl);

        // Enter key: GtkEntry fires its own "activate" signal on Return —
        // EventControllerKey never sees Enter because Entry consumes it internally first.
        script_search.connect_activate({
            let run_search = run_search.clone();
            move |_entry| {
                run_search();
            }
        });

        // Autocomplete on key release only — avoids re-entering connect_changed
        let autocomplete_ctrl = gtk::EventControllerKey::new();
        autocomplete_ctrl.connect_key_released({
            let entry = script_search.clone();
            let state = state.clone();
            move |_ctrl, key, _code, _mods| {
                use gtk::gdk::Key;

                if state.borrow().search_by_keyword {
                    return;
                }

                if matches!(
                    key,
                    Key::Right
                        | Key::Left
                        | Key::Return
                        | Key::KP_Enter
                        | Key::BackSpace
                        | Key::Tab
                ) {
                    return;
                }

                let text = entry.text().to_string();
                if text.is_empty() || text.contains(':') {
                    return;
                }
                if entry.selection_bounds().is_some() {
                    return;
                }

                if let Some(completed) = autocomplete_book_name(&text) {
                    if completed.len() > text.len()
                        && completed.to_lowercase().starts_with(&text.to_lowercase())
                    {
                        let cursor_pos = text.len() as i32;
                        entry.set_text(&completed);
                        entry.select_region(cursor_pos, completed.len() as i32);
                    }
                }
            }
        });
        script_search.add_controller(autocomplete_ctrl);

        // Initial query on startup
        run_search();
    }

    // Callback when selecting a translation in the sidebar
    let state_clone = state.clone();
    let populate_verses_table_clone = populate_verses_table.clone();
    let script_search_clone = script_search.clone();
    translations_list_box.connect_row_selected(move |listbox, row| {
        println!("DEBUG: connect_row_selected triggered at line 1541");
        if let Some(row) = row {
            let row_idx_i32 = row.index();
            let row_idx = row_idx_i32 as usize;
            let mut s = state_clone.borrow_mut();
            if row_idx == 0 {
                s.selected_translation = "HCSB";
            } else if row_idx == 1 {
                s.selected_translation = "KJV";
            } else if row_idx == 2 {
                s.selected_translation = "RVA";
            } else {
                return;
            }

            // Update row highlights visually
            for i in 0..3 {
                if let Some(r) = listbox.row_at_index(i) {
                    r.remove_css_class("sidebar-row-selected");
                    if i == row_idx_i32 {
                        r.add_css_class("sidebar-row-selected");
                    }
                }
            }

            // Query database dynamically with the new translation tag
            let query_text = script_search_clone.text().to_string();
            let new_verses =
                query_verses_by_mode(&query_text, s.selected_translation, s.search_by_keyword);
            s.verses = new_verses;

            drop(s);
            populate_verses_table_clone();
        }
    });

    // Callback when selecting a song in the Songs sidebar list
    let state_clone = state.clone();
    let song_stanzas_list_box_clone = song_stanzas_list_box.clone();

    let populate_song_stanzas = {
        let state = state_clone.clone();
        let song_stanzas_list_box = song_stanzas_list_box_clone.clone();
        let preview_title_label = preview_title_label.clone();
        let preview_drawing_area = preview_drawing_area.clone();

        move || {
            println!("DEBUG: Closure executing...");
            println!("DEBUG: move || closure triggered at line 1589");
            while let Some(child) = song_stanzas_list_box.first_child() {
                song_stanzas_list_box.remove(&child);
            }

            let song_data = {
                let s = state.borrow();
                s.selected_song_index.map(|idx| s.songs[idx].clone())
            };

            if let Some(song) = song_data {
                for (stanza_idx, stanza_text) in song.stanzas.iter().enumerate() {
                    let row_box = Box::builder()
                        .orientation(Orientation::Horizontal)
                        .spacing(10)
                        .build();
                    row_box.add_css_class("table-row");

                    let label_idx = Label::builder()
                        .label(&format!("Stanza {}", stanza_idx + 1))
                        .xalign(0.0)
                        .width_request(80)
                        .build();
                    label_idx.add_css_class("table-cell-text");

                    let label_text = Label::builder()
                        .label(&stanza_text.replace("\n", " / "))
                        .xalign(0.0)
                        .hexpand(true)
                        .wrap(true)
                        .build();
                    label_text.add_css_class("table-cell-text");

                    row_box.append(&label_idx);
                    row_box.append(&label_text);

                    song_stanzas_list_box.append(&row_box);
                }

                let mut s = state.borrow_mut();
                // Default preview to first stanza
                s.selected_stanza_index = Some(0);
                s.current_selection_type = 1; // Song

                preview_title_label.set_text(&format!("Preview - {} (Stanza 1)", song.title));
                s.preview_header = format!("{} - Stanza 1", song.title);
                s.preview_body = song.stanzas[0].to_string();
                preview_drawing_area.queue_draw();
            }
        }
    };

    let populate_song_stanzas_clone = populate_song_stanzas.clone();
    let state_clone = state.clone();
    songs_list_box.connect_row_selected(move |_, row| {
        println!("DEBUG: connect_row_selected triggered at line 1643");
        if let Some(row) = row {
            let row_idx = row.index() as usize;
            let mut s = state_clone.borrow_mut();
            s.selected_song_index = Some(row_idx);
            drop(s);
            populate_song_stanzas_clone();
        }
    });

    // Callback when selecting a stanza in the Song Stanzas table
    let state_clone = state.clone();
    let preview_title_label_clone = preview_title_label.clone();
    let preview_drawing_area_clone = preview_drawing_area.clone();

    song_stanzas_list_box.connect_row_selected(move |_, row| {
        println!("DEBUG: connect_row_selected triggered at line 1660");
        if let Some(row) = row {
            let row_idx = row.index() as usize;
            let mut s = state_clone.borrow_mut();
            s.selected_stanza_index = Some(row_idx);
            s.current_selection_type = 1; // Song

            if let Some(song_idx) = s.selected_song_index {
                let song = &s.songs[song_idx];
                if let Some(stanza_text) = song.stanzas.get(row_idx) {
                    let title = song.title;
                    let text = stanza_text.to_string();
                    preview_title_label_clone.set_text(&format!(
                        "Preview - {} (Stanza {})",
                        title,
                        row_idx + 1
                    ));
                    s.preview_header = format!("{} - Stanza {}", title, row_idx + 1);
                    s.preview_body = text;

                    drop(s);
                    preview_drawing_area_clone.queue_draw();
                }
            }
        }
    });

    // Double-click song stanza -> Go Live
    let update_live_layout_clone2 = update_live_layout.clone();
    let state_clone = state.clone();
    song_stanzas_list_box.connect_row_activated(move |_, row| {
        let mut s = state_clone.borrow_mut();
        let row_idx = row.index() as usize;

        if let Some(song_idx) = s.selected_song_index {
            let song = s.songs[song_idx].clone();
            s.live_slides = song
                .stanzas
                .iter()
                .enumerate()
                .map(|(idx, text)| {
                    (
                        format!("{} - Stanza {}", song.title, idx + 1),
                        text.to_string(),
                    )
                })
                .collect();
            s.live_title = format!("Live - {}", song.title);
            s.live_active_index = Some(row_idx);

            // Reset screen flags
            s.blackout = false;
            s.clearout = false;
            s.logo_mode = false;
        }

        drop(s);
        update_live_layout_clone2();
    });

    // Callback when selecting an item in the Live slides list
    let state_clone = state.clone();
    let update_live_layout_clone3 = update_live_layout.clone();
    live_slides_list.connect_row_selected(move |_, row| {
        println!("DEBUG: connect_row_selected triggered at line 1723");
        if let Some(row) = row {
            let row_idx = row.index() as usize;
            let mut s = state_clone.borrow_mut();
            s.live_active_index = Some(row_idx);
            drop(s);
            update_live_layout_clone3();
        }
    });

    // --- TOOLBAR BUTTON HANDLERS ---
    // GO LIVE
    let state_clone = state.clone();
    let update_live_layout_clone4 = update_live_layout.clone();
    go_live_btn.connect_clicked(move |_| {
        println!("DEBUG: go_live_btn clicked!");
        println!("DEBUG: connect_clicked triggered at line 1738");
        let mut s = state_clone.borrow_mut();
        if s.current_selection_type == 0 {
            // Go live with only the single selected verse
            if let Some(sel_idx) = s.selected_verse_index {
                if sel_idx < s.verses.len() {
                    let v = &s.verses[sel_idx];
                    let slide = (
                        format!("{} ({})", v.reference, v.translation),
                        v.text.clone(),
                    );
                    s.live_slides = vec![slide.clone()];
                    s.live_title = format!("Live - {}", slide.0);
                    s.live_active_index = Some(0);
                }
            } else {
                // No explicit selection: use the first verse in the chapter
                let first = s
                    .verses
                    .iter()
                    .filter(|v| v.translation == s.selected_translation)
                    .next()
                    .map(|v| {
                        (
                            format!("{} ({})", v.reference, v.translation),
                            v.text.clone(),
                        )
                    });
                if let Some(slide) = first {
                    s.live_slides = vec![slide.clone()];
                    s.live_title = format!("Live - {}", slide.0);
                    s.live_active_index = Some(0);
                }
            }
        } else {
            // Live-ify current song stanzas
            if let Some(song_idx) = s.selected_song_index {
                let song = s.songs[song_idx].clone();
                let stanza_idx = s.selected_stanza_index.unwrap_or(0);

                s.live_slides = song
                    .stanzas
                    .iter()
                    .enumerate()
                    .map(|(idx, text)| {
                        (
                            format!("{} - Stanza {}", song.title, idx + 1),
                            text.to_string(),
                        )
                    })
                    .collect();
                s.live_title = format!("Live - {}", song.title);
                s.live_active_index = Some(stanza_idx);
            }
        }
        // Reset screen flags
        s.blackout = false;
        s.clearout = false;
        s.logo_mode = false;

        drop(s);
        update_live_layout_clone4();
    });

    // BLACK — toggles blackout and shows active visual state on button
    let state_clone = state.clone();
    let update_live_layout_clone5 = update_live_layout.clone();
    let update_slide_theme_classes_clone = update_slide_theme_classes.clone();
    black_btn.connect_clicked(move |btn| {
        println!("DEBUG: black_btn clicked!");
        let mut s = state_clone.borrow_mut();
        s.blackout = !s.blackout;
        let is_on = s.blackout;
        drop(s);
        if is_on {
            btn.add_css_class("toolbar-button-active");
        } else {
            btn.remove_css_class("toolbar-button-active");
        }
        update_live_layout_clone5();
        update_slide_theme_classes_clone();
    });

    // CLEAR — clears live text and shows active visual state on button
    let state_clone = state.clone();
    let update_live_layout_clone6 = update_live_layout.clone();
    clear_btn.connect_clicked(move |btn| {
        println!("DEBUG: clear_btn clicked!");
        let mut s = state_clone.borrow_mut();
        s.clearout = !s.clearout;
        let is_on = s.clearout;
        drop(s);
        if is_on {
            btn.add_css_class("toolbar-button-active");
        } else {
            btn.remove_css_class("toolbar-button-active");
        }
        update_live_layout_clone6();
    });

    // LOGO
    let state_clone = state.clone();
    let update_live_layout_clone7 = update_live_layout.clone();
    logo_btn.connect_clicked(move |_| {
        println!("DEBUG: logo_btn clicked!");
        println!("DEBUG: connect_clicked triggered at line 1840");
        let mut s = state_clone.borrow_mut();
        s.logo_mode = !s.logo_mode;
        drop(s);
        update_live_layout_clone7();
    });

    // THEMES CARD SELECTIONS
    let update_slide_theme_classes_clone2 = update_slide_theme_classes.clone();

    let connect_theme_click = |card: &Box,
                               theme_name: &'static str,
                               state: Rc<RefCell<AppState>>,
                               update_theme: Rc<dyn Fn()>| {
        let gesture = gtk::GestureClick::new();
        gesture.connect_pressed(move |_, _, _, _| {
            let mut s = state.borrow_mut();
            s.selected_theme = theme_name;
            drop(s);
            update_theme();
        });
        card.add_controller(gesture);
    };

    connect_theme_click(
        &theme_card_red,
        "classic-red",
        state.clone(),
        update_slide_theme_classes_clone2.clone(),
    );
    connect_theme_click(
        &theme_card_blue,
        "royal-blue",
        state.clone(),
        update_slide_theme_classes_clone2.clone(),
    );
    connect_theme_click(
        &theme_card_green,
        "forest-green",
        state.clone(),
        update_slide_theme_classes_clone2.clone(),
    );
    connect_theme_click(
        &theme_card_slate,
        "dark-slate",
        state.clone(),
        update_slide_theme_classes_clone2.clone(),
    );
    connect_theme_click(
        &theme_card_black,
        "black",
        state.clone(),
        update_slide_theme_classes_clone2,
    );

    // TAB BUTTON NAVIGATION
    let tab_buttons = vec![
        tab_btn_songs.clone(),
        tab_btn_scriptures.clone(),
        tab_btn_media.clone(),
        tab_btn_presentations.clone(),
        tab_btn_themes.clone(),
    ];

    let make_tab_click_handler =
        |btn: &Button, page_name: &'static str, stack: &Stack, all_btns: Vec<Button>| {
            let stack = stack.clone();
            btn.connect_clicked(move |clicked_btn| {
                for other_btn in &all_btns {
                    other_btn.remove_css_class("tab-button-active");
                }
                clicked_btn.add_css_class("tab-button-active");
                stack.set_visible_child_name(page_name);
            });
        };

    make_tab_click_handler(
        &tab_btn_songs,
        "songs",
        &resource_stack,
        tab_buttons.clone(),
    );
    make_tab_click_handler(
        &tab_btn_scriptures,
        "scriptures",
        &resource_stack,
        tab_buttons.clone(),
    );
    make_tab_click_handler(
        &tab_btn_media,
        "media",
        &resource_stack,
        tab_buttons.clone(),
    );
    make_tab_click_handler(
        &tab_btn_presentations,
        "presentations",
        &resource_stack,
        tab_buttons.clone(),
    );
    make_tab_click_handler(&tab_btn_themes, "themes", &resource_stack, tab_buttons);

    // MOCK DIALOGS FOR REMAINING BUTTONS
    new_btn.connect_clicked(move |_| {
        println!("DEBUG: connect_clicked triggered at line 1938");
        println!("Toolbar Action: New Presentation/Scripture triggered");
    });
    open_btn.connect_clicked(move |_| {
        println!("DEBUG: connect_clicked triggered at line 1942");
        println!("Toolbar Action: Open File triggered");
    });
    save_btn.connect_clicked(move |_| {
        println!("DEBUG: connect_clicked triggered at line 1946");
        println!("Toolbar Action: Save current set triggered");
    });
    // --- APPLICATION WINDOW CREATION ---

    let window = ApplicationWindow::builder()
        .application(app)
        .title("EasyWorship - GTK4 Edition")
        .default_width(1280)
        .default_height(1000)
        .build();
    window.set_resizable(true);

    // Prevent content (long verses, wide rows) from forcing the window to grow
    // past a reasonable size — GTK4 windows have no max-size API, so instead we
    // ensure the *content itself* never requests more than the window's width by
    // giving the scrollable containers a hard width constraint via CSS/max-width
    // behavior on their children, and by wrapping instead of growing.
    main_box.set_hexpand(true);
    main_box.set_vexpand(true);
    window.set_child(Some(&main_box));

    // KEYBOARD EVENT CONTROLLER FOR ARROW KEY NAVIGATION
    let key_controller = gtk::EventControllerKey::new();
    let scriptures_list_box_clone = scriptures_list_box.clone();

    key_controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
        if key == gtk::gdk::Key::Right || key == gtk::gdk::Key::Left {
            if let Some(selected_row) = scriptures_list_box_clone.selected_row() {
                let current_idx = selected_row.index();
                let next_idx = if key == gtk::gdk::Key::Right {
                    current_idx + 1
                } else {
                    current_idx - 1
                };

                if next_idx >= 0 {
                    if let Some(row) = scriptures_list_box_clone.row_at_index(next_idx) {
                        scriptures_list_box_clone.select_row(Some(&row));
                        return gtk::glib::Propagation::Stop;
                    }
                }
            }
        }
        gtk::glib::Propagation::Proceed
    });
    window.add_controller(key_controller);

    window.present();
}
