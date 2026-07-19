#[derive(Clone, Debug)]
pub struct Verse {
    pub translation: String,
    pub reference: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct SongStanza {
    pub name: String,
    pub lyrics: String,
    pub bg_type: String, // "transparent", "lower_transparent", "image"
    pub bg_path: Option<String>,
    pub font_size: f64,
    pub scale: f64,
    pub align: String, // "left", "right", "center"
    pub shadow: bool,
    pub lower_bar_height: f64, // fraction 0.0–1.0 of canvas height (only used for lower_transparent)
}

#[derive(Clone, Debug)]
pub struct Song {
    pub id: Option<i64>,
    pub title: String,
    pub stanzas: Vec<SongStanza>,
}

pub struct AppState {
    pub verses: Vec<Verse>,
    pub selected_translation: &'static str,
    pub selected_verse_index: Option<usize>,

    pub songs: Vec<Song>,
    pub selected_song_index: Option<usize>,
    pub selected_stanza_index: Option<usize>,

    // 0 = Scripture, 1 = Song
    pub current_selection_type: u8,

    // Live display state
    pub live_title: String,
    pub live_slides: Vec<(String, String)>, // (header, body)
    pub live_active_index: Option<usize>,

    pub search_parsed_verse: Option<i32>,
    pub search_by_keyword: bool,

    // App state flags
    pub selected_theme: &'static str, // "classic-red", "royal-blue", "forest-green", "dark-slate", "custom"
    pub blackout: bool,
    pub clearout: bool,
    pub logo_mode: bool,
    pub go_live_active: bool,
    pub logo_image_path: Option<String>,
    pub live_song_stanzas: Option<Vec<SongStanza>>,
    pub preview_song_stanzas: Option<Vec<SongStanza>>,

    // Custom media themes
    pub custom_themes: Vec<(String, String)>, // (name, path)
    pub custom_background_path: Option<String>,

    // Preview slide state
    pub preview_header: String,
    pub preview_body: String,

    // Live monitor state (for transitions)
    pub live_current_header: String,
    pub live_current_body: String,
    pub live_prev_header: String,
    pub live_prev_body: String,
    pub live_trans_start: Option<std::time::Instant>,
    // Default song backgrounds
    pub default_song_bg_type: String,
    pub default_song_bg_val: Option<String>,
}
