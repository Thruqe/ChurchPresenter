#[derive(Clone, Debug)]
pub struct Verse {
    pub translation: String,
    pub reference: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Song {
    pub title: &'static str,
    pub stanzas: Vec<&'static str>,
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
}
