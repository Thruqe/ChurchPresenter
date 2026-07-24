use crate::models::{Song, SongStanza, Verse};
use crate::println;
use rusqlite::{params, Connection};

pub fn init_songs_tables() {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL
            )",
            [],
        );
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS song_stanzas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                song_id INTEGER NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                lyrics TEXT NOT NULL,
                bg_type TEXT NOT NULL,
                bg_path TEXT,
                font_size REAL NOT NULL DEFAULT 40.0,
                scale REAL NOT NULL DEFAULT 1.0,
                align TEXT NOT NULL DEFAULT 'center',
                shadow INTEGER NOT NULL DEFAULT 0,
                order_index INTEGER NOT NULL,
                lower_bar_height REAL DEFAULT 0.35
            )",
            [],
        );
        // Migration: add lower_bar_height column if it doesn't exist yet (for existing databases)
        let _ = conn.execute(
            "ALTER TABLE song_stanzas ADD COLUMN lower_bar_height REAL DEFAULT 0.35",
            [],
        );
    }
}

pub fn get_songs() -> Vec<Song> {
    let mut songs = Vec::new();
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        if let Ok(mut stmt) = conn.prepare("SELECT id, title FROM songs ORDER BY id") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            }) {
                for r in rows {
                    if let Ok((song_id, title)) = r {
                        let mut stanzas = Vec::new();
                        if let Ok(mut s_stmt) = conn.prepare(
                            "SELECT name, lyrics, bg_type, bg_path, font_size, scale, align, shadow, lower_bar_height 
                             FROM song_stanzas WHERE song_id = ? ORDER BY order_index"
                        ) {
                            if let Ok(s_rows) = s_stmt.query_map(params![song_id], |s_row| {
                                Ok(SongStanza {
                                    name: s_row.get(0)?,
                                    lyrics: s_row.get(1)?,
                                    bg_type: s_row.get(2)?,
                                    bg_path: s_row.get(3)?,
                                    font_size: s_row.get(4)?,
                                    scale: s_row.get(5)?,
                                    align: s_row.get(6)?,
                                    shadow: s_row.get::<_, i32>(7)? != 0,
                                    lower_bar_height: s_row.get::<_, Option<f64>>(8)?.unwrap_or(0.35),
                                })
                            }) {
                                for sr in s_rows {
                                    if let Ok(stanza) = sr {
                                        stanzas.push(stanza);
                                    }
                                }
                            }
                        }
                        songs.push(Song {
                            id: Some(song_id),
                            title,
                            stanzas,
                        });
                    }
                }
            }
        }
    }
    songs
}

pub fn save_song(song: &Song) -> i64 {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        if let Some(song_id) = song.id {
            let _ = conn.execute(
                "UPDATE songs SET title = ? WHERE id = ?",
                params![song.title, song_id],
            );
            let _ = conn.execute(
                "DELETE FROM song_stanzas WHERE song_id = ?",
                params![song_id],
            );
            for (idx, stanza) in song.stanzas.iter().enumerate() {
                let _ = conn.execute(
                    "INSERT INTO song_stanzas (song_id, name, lyrics, bg_type, bg_path, font_size, scale, align, shadow, order_index, lower_bar_height)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        song_id,
                        stanza.name,
                        stanza.lyrics,
                        stanza.bg_type,
                        stanza.bg_path,
                        stanza.font_size,
                        stanza.scale,
                        stanza.align,
                        if stanza.shadow { 1 } else { 0 },
                        idx as i32,
                        stanza.lower_bar_height
                    ],
                );
            }
            song_id
        } else {
            let _ = conn.execute("INSERT INTO songs (title) VALUES (?)", params![song.title]);
            let song_id = conn.last_insert_rowid();
            for (idx, stanza) in song.stanzas.iter().enumerate() {
                let _ = conn.execute(
                    "INSERT INTO song_stanzas (song_id, name, lyrics, bg_type, bg_path, font_size, scale, align, shadow, order_index, lower_bar_height)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        song_id,
                        stanza.name,
                        stanza.lyrics,
                        stanza.bg_type,
                        stanza.bg_path,
                        stanza.font_size,
                        stanza.scale,
                        stanza.align,
                        if stanza.shadow { 1 } else { 0 },
                        idx as i32,
                        stanza.lower_bar_height
                    ],
                );
            }
            song_id
        }
    } else {
        0
    }
}

pub fn get_max_chapter(conn: &Connection, book_id: i32) -> i32 {
    let sql = "SELECT MAX(chapter) FROM verse WHERE book_id = ?";
    conn.query_row(sql, params![book_id], |row| row.get::<_, Option<i32>>(0))
        .ok()
        .flatten()
        .unwrap_or(1)
}

pub fn get_max_verse(conn: &Connection, book_id: i32, chapter: i32) -> i32 {
    let sql = "SELECT MAX(verse) FROM verse WHERE book_id = ? AND chapter = ?";
    conn.query_row(sql, params![book_id, chapter], |row| row.get::<_, Option<i32>>(0))
        .ok()
        .flatten()
        .unwrap_or(1)
}

pub fn parse_reference(query: &str) -> (String, Option<i32>, Option<i32>) {
    let mut q = query.trim();
    if q.starts_with('=') {
        q = q[1..].trim();
    }
    if q.is_empty() {
        return ("".to_string(), None, None);
    }

    if let Some(colon_idx) = q.rfind(':') {
        let (left, right) = q.split_at(colon_idx);
        let verse = right[1..].trim().parse::<i32>().ok();
        let left_trimmed = left.trim();
        if let Some(space_idx) = left_trimmed.rfind(' ') {
            let (book, chap) = left_trimmed.split_at(space_idx);
            if let Ok(chap_num) = chap.trim().parse::<i32>() {
                return (book.trim().to_string(), Some(chap_num), verse);
            }
        }
        return (left_trimmed.to_string(), None, verse);
    }

    let parts: Vec<&str> = q.split_whitespace().collect();
    if parts.is_empty() {
        return ("".to_string(), None, None);
    }

    let len = parts.len();
    if len >= 3 {
        let last = parts[len - 1].parse::<i32>();
        let second_last = parts[len - 2].parse::<i32>();
        if let (Ok(v_num), Ok(c_num)) = (last, second_last) {
            let book_name = parts[..len - 2].join(" ");
            return (book_name, Some(c_num), Some(v_num));
        }
    }

    if len >= 2 {
        let last = parts[len - 1].parse::<i32>();
        if let Ok(c_num) = last {
            let book_name = parts[..len - 1].join(" ");
            return (book_name, Some(c_num), None);
        }
    }

    (q.to_string(), None, None)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedRef {
    pub book_name: String,
    pub chapter: i32,
    pub verse: Option<i32>,
    pub max_chapter: i32,
    pub max_verse: i32,
}

pub fn query_verses_by_mode_with_ref(
    search_query: &str,
    translation: &str,
    search_by_keyword: bool,
) -> (Vec<Verse>, Option<ParsedRef>) {
    let conn_res = Connection::open(get_kjv_db_path());
    if conn_res.is_err() {
        return (vec![], None);
    }
    let conn = conn_res.unwrap();

    let trimmed = search_query.trim();
    if trimmed.is_empty() {
        let verses = load_default_genesis_1(&conn, translation);
        return (
            verses,
            Some(ParsedRef {
                book_name: "Genesis".to_string(),
                chapter: 1,
                verse: None,
                max_chapter: 50,
                max_verse: 31,
            }),
        );
    }

    if search_by_keyword {
        let keyword_sql = "SELECT v.chapter, v.verse, v.text, b.name FROM verse v JOIN book b ON v.book_id = b.id WHERE v.text LIKE ? ORDER BY b.id, v.chapter, v.verse LIMIT 100";
        let mut stmt = if let Ok(s) = conn.prepare(keyword_sql) {
            s
        } else {
            return (vec![], None);
        };
        let rows_res = stmt.query_map(params![format!("%{}%", trimmed)], |row| {
            let chap = row.get::<_, i32>(0)?;
            let ver = row.get::<_, i32>(1)?;
            let txt = row.get::<_, String>(2)?.replace("[", "").replace("]", "");
            let b_name = row.get::<_, String>(3)?;
            Ok(Verse {
                translation: translation.to_string(),
                reference: format!("{} {}:{}", b_name, chap, ver),
                text: txt,
            })
        });
        let verses = if let Ok(rows) = rows_res {
            rows.filter_map(|r| r.ok()).collect()
        } else {
            vec![]
        };
        return (verses, None);
    }

    let (book_name, raw_chapter, raw_verse) = parse_reference(trimmed);

    let book_id_query = "SELECT id, name FROM book WHERE name LIKE ? ORDER BY id LIMIT 1";
    let book_res = conn.query_row(book_id_query, params![format!("{}%", book_name)], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    });

    if let Ok((book_id, real_book_name)) = book_res {
        let max_chap = get_max_chapter(&conn, book_id);
        let chap_num = match raw_chapter {
            Some(c) => c.clamp(1, max_chap),
            None => 1,
        };

        let max_v = get_max_verse(&conn, book_id, chap_num);
        let clamped_verse = raw_verse.map(|v| v.clamp(1, max_v));

        println!(
            "Query matched book '{}' (id={}): chapter {}/max {}, verse {:?}/max {}",
            real_book_name, book_id, chap_num, max_chap, clamped_verse, max_v
        );

        let mut stmt = if let Ok(s) = conn.prepare(
            "SELECT chapter, verse, text FROM verse WHERE book_id = ? AND chapter = ? ORDER BY verse",
        ) {
            s
        } else {
            return (vec![], None);
        };
        let verses_res = stmt.query_map(params![book_id, chap_num], |row| {
            let raw_txt: String = row.get(2)?;
            Ok(Verse {
                translation: translation.to_string(),
                reference: format!(
                    "{} {}:{}",
                    real_book_name,
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?
                ),
                text: clean_verse_text(&raw_txt),
            })
        });

        let verses = if let Ok(rows) = verses_res {
            rows.filter_map(|r| r.ok()).collect()
        } else {
            vec![]
        };

        let parsed_ref = ParsedRef {
            book_name: real_book_name,
            chapter: chap_num,
            verse: clamped_verse,
            max_chapter: max_chap,
            max_verse: max_v,
        };

        return (verses, Some(parsed_ref));
    } else {
        let keyword_sql = "SELECT v.chapter, v.verse, v.text, b.name FROM verse v JOIN book b ON v.book_id = b.id WHERE v.text LIKE ? ORDER BY b.id, v.chapter, v.verse LIMIT 100";
        let mut stmt = if let Ok(s) = conn.prepare(keyword_sql) {
            s
        } else {
            return (vec![], None);
        };
        let rows_res = stmt.query_map(params![format!("%{}%", trimmed)], |row| {
            let chap = row.get::<_, i32>(0)?;
            let ver = row.get::<_, i32>(1)?;
            let raw_txt: String = row.get(2)?;
            let b_name = row.get::<_, String>(3)?;
            Ok(Verse {
                translation: translation.to_string(),
                reference: format!("{} {}:{}", b_name, chap, ver),
                text: clean_verse_text(&raw_txt),
            })
        });
        let verses = if let Ok(rows) = rows_res {
            rows.filter_map(|r| r.ok()).collect()
        } else {
            vec![]
        };
        return (verses, None);
    }
}

pub fn query_verses_by_mode(
    search_query: &str,
    translation: &str,
    search_by_keyword: bool,
) -> Vec<Verse> {
    let (verses, _) = query_verses_by_mode_with_ref(search_query, translation, search_by_keyword);
    verses
}

pub fn query_verses(search_query: &str, translation: &str) -> Vec<Verse> {
    query_verses_by_mode(search_query, translation, false)
}

pub fn clean_verse_text(raw: &str) -> String {
    let mut s = raw.to_string();
    while let Some(start) = s.find('<') {
        if let Some(end) = s[start..].find('>') {
            s.drain(start..=start + end);
        } else {
            break;
        }
    }
    s.replace('[', "").replace(']', "").trim().to_string()
}

pub fn load_default_genesis_1(conn: &Connection, translation: &str) -> Vec<Verse> {
    let book_res = conn.query_row(
        "SELECT id, name FROM book WHERE name = 'Genesis' LIMIT 1",
        [],
        |row| Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?)),
    );

    if let Ok((book_id, real_book_name)) = book_res {
        let mut stmt = if let Ok(s) = conn.prepare("SELECT chapter, verse, text FROM verse WHERE book_id = ? AND chapter = 1 ORDER BY verse") { s } else { return vec![]; };
        let rows_res = stmt.query_map([book_id], |row| {
            let raw_txt: String = row.get(2)?;
            Ok(Verse {
                translation: translation.to_string(),
                reference: format!(
                    "{} {}:{}",
                    real_book_name,
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?
                ),
                text: clean_verse_text(&raw_txt),
            })
        });
        if let Ok(rows) = rows_res {
            return rows.filter_map(|r| r.ok()).collect();
        }
    }
    vec![]
}

pub fn autocomplete_book_name(text: &str) -> Option<String> {
    let conn_res = Connection::open(get_kjv_db_path());
    if conn_res.is_err() {
        return None;
    }
    let conn = conn_res.unwrap();

    let clean = text.strip_prefix('=').unwrap_or(text).trim();
    if clean.is_empty() {
        return None;
    }

    let query = "SELECT name FROM book WHERE name LIKE ? ORDER BY id LIMIT 1";
    let res = conn.query_row(query, params![format!("{}%", clean)], |row| {
        row.get::<_, String>(0)
    });
    res.ok()
}

pub fn get_all_books() -> Vec<String> {
    let conn_res = Connection::open(get_kjv_db_path());
    if conn_res.is_err() {
        return vec![];
    }
    let conn = conn_res.unwrap();
    let mut stmt = if let Ok(s) = conn.prepare("SELECT name FROM book ORDER BY id") {
        s
    } else {
        return vec![];
    };
    let rows_res = stmt.query_map([], |row| row.get::<_, String>(0));
    if let Ok(rows) = rows_res {
        return rows.filter_map(|r| r.ok()).collect();
    }
    vec![]
}

fn is_dir_writable(dir: &std::path::Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let test_file = dir.join(".write_test");
    if std::fs::write(&test_file, b"test").is_ok() {
        let _ = std::fs::remove_file(test_file);
        true
    } else {
        false
    }
}

fn get_user_data_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            return Some(std::path::PathBuf::from(local_appdata));
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(std::path::PathBuf::from(appdata));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(std::path::PathBuf::from(home).join("Library").join("Application Support"));
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            return Some(std::path::PathBuf::from(data_home));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Some(std::path::PathBuf::from(home).join(".local").join("share"));
        }
    }
    None
}

pub fn get_saves_directory() -> std::path::PathBuf {
    // 1. Try executable's parent saves directory if writable (e.g. portable mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let saves_dir = parent.join("saves");
            if is_dir_writable(&saves_dir) {
                return saves_dir;
            }
        }
    }

    // 2. Try standard User Data Directory (installed mode in Program Files / /usr/bin)
    if let Some(user_data) = get_user_data_dir() {
        let saves_dir = user_data.join("church-presenter").join("saves");
        if is_dir_writable(&saves_dir) {
            return saves_dir;
        }
    }

    // 3. Fallback to local saves directory
    let fallback = std::path::PathBuf::from("saves");
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

pub fn get_data_db_path() -> String {
    let path = get_saves_directory().join("data.sqlite");
    path.to_string_lossy().to_string()
}

pub fn get_kjv_db_path() -> String {
    let saves_dir = get_saves_directory();
    let saves_db = saves_dir.join("KJV.sqlite");

    if saves_db.exists() {
        return saves_db.to_string_lossy().to_string();
    }

    // Check executable directory and bundled asset locations
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let exe_db = parent.join("KJV.sqlite");
            if exe_db.exists() {
                return exe_db.to_string_lossy().to_string();
            }
            let exe_saves_db = parent.join("saves").join("KJV.sqlite");
            if exe_saves_db.exists() {
                return exe_saves_db.to_string_lossy().to_string();
            }
            #[cfg(target_os = "macos")]
            {
                if let Some(contents) = parent.parent() {
                    let res_db = contents.join("Resources").join("KJV.sqlite");
                    if res_db.exists() {
                        return res_db.to_string_lossy().to_string();
                    }
                }
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let sys_db = std::path::PathBuf::from("/usr/share/church-presenter/KJV.sqlite");
        if sys_db.exists() {
            return sys_db.to_string_lossy().to_string();
        }
    }

    // Extract embedded database if not found on disk
    println!("INFO: Extracting bundled KJV.sqlite database...");
    let bytes = include_bytes!("../KJV.sqlite");
    if let Err(e) = std::fs::write(&saves_db, bytes) {
        eprintln!("Error writing KJV.sqlite: {:?}", e);
    }
    saves_db.to_string_lossy().to_string()
}

pub fn init_media_table() {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE
            )",
            [],
        );
    }
}

pub fn get_all_media() -> Vec<(String, String)> {
    let mut media = Vec::new();
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        if let Ok(mut stmt) = conn.prepare("SELECT name, path FROM media ORDER BY id") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                for r in rows {
                    if let Ok(item) = r {
                        media.push(item);
                    }
                }
            }
        }
    }
    media
}

pub fn add_media(name: &str, path: &str) {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO media (name, path) VALUES (?, ?)",
            params![name, path],
        );
    }
}

pub fn delete_media(path: &str) {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute("DELETE FROM media WHERE path = ?", params![path]);
    }
}

pub fn init_themes_table() {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS themes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE
            )",
            [],
        );
    }
}

pub fn get_all_themes() -> Vec<(String, String)> {
    let mut themes = Vec::new();
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        if let Ok(mut stmt) = conn.prepare("SELECT name, path FROM themes ORDER BY id") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                for r in rows {
                    if let Ok(item) = r {
                        themes.push(item);
                    }
                }
            }
        }
    }
    themes
}

pub fn add_theme(name: &str, path: &str) {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO themes (name, path) VALUES (?, ?)",
            params![name, path],
        );
    }
}

pub fn delete_theme(path: &str) {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute("DELETE FROM themes WHERE path = ?", params![path]);
    }
}

pub fn init_config_table() {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        );
    }
}

pub fn set_config_value(key: &str, value: &str) {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)",
            params![key, value],
        );
    }
}

pub fn get_config_value(key: &str) -> Option<String> {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        if let Ok(mut stmt) = conn.prepare("SELECT value FROM config WHERE key = ?") {
            if let Ok(val) = stmt.query_row(params![key], |row| row.get::<_, String>(0)) {
                return Some(val);
            }
        }
    }
    None
}

pub fn delete_song(song_id: i64) {
    if let Ok(conn) = Connection::open(get_data_db_path()) {
        let _ = conn.execute("DELETE FROM songs WHERE id = ?", params![song_id]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reference_patterns() {
        assert_eq!(parse_reference("John"), ("John".to_string(), None, None));
        assert_eq!(parse_reference("John 3"), ("John".to_string(), Some(3), None));
        assert_eq!(parse_reference("John 3:"), ("John".to_string(), Some(3), None));
        assert_eq!(parse_reference("John 3:16"), ("John".to_string(), Some(3), Some(16)));
        assert_eq!(parse_reference("John 3 16"), ("John".to_string(), Some(3), Some(16)));
        assert_eq!(parse_reference("1 John 3 16"), ("1 John".to_string(), Some(3), Some(16)));
        assert_eq!(parse_reference("Song of Solomon 2 4"), ("Song of Solomon".to_string(), Some(2), Some(4)));
    }

    #[test]
    fn test_query_verses_clamping() {
        let (verses_j3, p_ref) = query_verses_by_mode_with_ref("John 3", "KJV", false);
        assert!(!verses_j3.is_empty());
        let p = p_ref.unwrap();
        assert_eq!(p.book_name, "John");
        assert_eq!(p.chapter, 3);
        assert_eq!(p.verse, None);

        // Test max chapter clamping (John has 21 chapters)
        let (verses_j99, p_ref99) = query_verses_by_mode_with_ref("John 99", "KJV", false);
        assert!(!verses_j99.is_empty());
        let p99 = p_ref99.unwrap();
        assert_eq!(p99.book_name, "John");
        assert_eq!(p99.chapter, 21); // clamped from 99 to 21

        // Test max verse clamping (John 3 has 36 verses)
        let (verses_v99, p_ref_v99) = query_verses_by_mode_with_ref("John 3:99", "KJV", false);
        assert!(!verses_v99.is_empty());
        let pv99 = p_ref_v99.unwrap();
        assert_eq!(pv99.book_name, "John");
        assert_eq!(pv99.chapter, 3);
        assert_eq!(pv99.verse, Some(36)); // clamped from 99 to 36
    }

    #[test]
    fn test_psalms_queries() {
        let (v1, p1) = query_verses_by_mode_with_ref("Psalm 23", "KJV", false);
        assert!(!v1.is_empty());
        assert_eq!(p1.unwrap().book_name, "Psalms");

        let (v2, p2) = query_verses_by_mode_with_ref("Psalms 23:1", "KJV", false);
        assert!(!v2.is_empty());
        assert_eq!(p2.unwrap().verse, Some(1));

        let (v3, p3) = query_verses_by_mode_with_ref("Psalm 119:200", "KJV", false);
        assert!(!v3.is_empty());
        let p3_ref = p3.unwrap();
        assert_eq!(p3_ref.chapter, 119);
        assert_eq!(p3_ref.verse, Some(176)); // Psalm 119 has max 176 verses
    }

    #[test]
    fn test_clean_verse_text() {
        let raw = "<A Song of degrees of David.> I was glad when they said unto me...";
        assert_eq!(clean_verse_text(raw), "I was glad when they said unto me...");
    }
}
