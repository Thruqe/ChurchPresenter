use rusqlite::{Connection, params};
use crate::models::{Verse, Song};

pub fn get_songs() -> Vec<Song> {
    vec![]
}

pub fn parse_reference(query: &str) -> (String, Option<i32>, Option<i32>) {
    let mut q = query.trim();
    if q.starts_with('=') {
        q = q[1..].trim();
    }

    let (left, verse) = if let Some(colon_idx) = q.rfind(':') {
        let (l, r) = q.split_at(colon_idx);
        let v_num = r[1..].trim().parse::<i32>().ok();
        (l.trim(), v_num)
    } else {
        (q, None)
    };

    if let Some(space_idx) = left.rfind(' ') {
        let (book, chap) = left.split_at(space_idx);
        if let Some(chap_num) = chap.trim().parse::<i32>().ok() {
            return (book.trim().to_string(), Some(chap_num), verse);
        }
    }

    (left.to_string(), None, verse)
}

pub fn query_verses_by_mode(
    search_query: &str,
    translation: &str,
    search_by_keyword: bool,
) -> Vec<Verse> {
    let conn_res = Connection::open("KJV.sqlite");
    if conn_res.is_err() {
        return vec![];
    }
    let conn = conn_res.unwrap();

    let trimmed = search_query.trim();
    if trimmed.is_empty() {
        return load_default_genesis_1(&conn, translation);
    }

    if search_by_keyword {
        let keyword_sql = "SELECT v.chapter, v.verse, v.text, b.name FROM verse v JOIN book b ON v.book_id = b.id WHERE v.text LIKE ? ORDER BY b.id, v.chapter, v.verse LIMIT 100";
        let mut stmt = if let Ok(s) = conn.prepare(keyword_sql) {
            s
        } else {
            return vec![];
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
        if let Ok(rows) = rows_res {
            return rows.filter_map(|r| r.ok()).collect();
        }
        return vec![];
    }

    let (book_name, chapter, _verse) = parse_reference(trimmed);

    let book_id_query = "SELECT id, name FROM book WHERE name LIKE ? ORDER BY id LIMIT 1";
    let book_res = conn.query_row(book_id_query, params![format!("{}%", book_name)], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    });

    if let Ok((book_id, real_book_name)) = book_res {
        let chap_num = chapter.unwrap_or(1);

        let mut stmt = if let Ok(s) = conn.prepare("SELECT chapter, verse, text FROM verse WHERE book_id = ? AND chapter = ? ORDER BY verse") { s } else { return vec![]; };
        if let Ok(rows) = stmt.query_map(params![book_id, chap_num], |row| {
            Ok(Verse {
                translation: translation.to_string(),
                reference: format!(
                    "{} {}:{}",
                    real_book_name,
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?
                ),
                text: row.get::<_, String>(2)?.replace("[", "").replace("]", ""),
            })
        }) {
            return rows.filter_map(|r| r.ok()).collect();
        }
    } else {
        let keyword_sql = "SELECT v.chapter, v.verse, v.text, b.name FROM verse v JOIN book b ON v.book_id = b.id WHERE v.text LIKE ? ORDER BY b.id, v.chapter, v.verse LIMIT 100";
        let mut stmt = if let Ok(s) = conn.prepare(keyword_sql) {
            s
        } else {
            return vec![];
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
        if let Ok(rows) = rows_res {
            return rows.filter_map(|r| r.ok()).collect();
        }
    }

    vec![]
}

pub fn query_verses(search_query: &str, translation: &str) -> Vec<Verse> {
    query_verses_by_mode(search_query, translation, false)
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
            Ok(Verse {
                translation: translation.to_string(),
                reference: format!(
                    "{} {}:{}",
                    real_book_name,
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?
                ),
                text: row.get::<_, String>(2)?.replace("[", "").replace("]", ""),
            })
        });
        if let Ok(rows) = rows_res {
            return rows.filter_map(|r| r.ok()).collect();
        }
    }
    vec![]
}

pub fn autocomplete_book_name(text: &str) -> Option<String> {
    let conn_res = Connection::open("KJV.sqlite");
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
    let conn_res = Connection::open("KJV.sqlite");
    if conn_res.is_err() {
        return vec![];
    }
    let conn = conn_res.unwrap();
    let mut stmt = if let Ok(s) = conn.prepare("SELECT name FROM book ORDER BY id") { s } else { return vec![]; };
    let rows_res = stmt.query_map([], |row| row.get::<_, String>(0));
    if let Ok(rows) = rows_res {
        return rows.filter_map(|r| r.ok()).collect();
    }
    vec![]
}

pub fn get_data_db_path() -> String {
    let path = "/home/thruqe/Documents/Church-Presenter/saves/data.sqlite";
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    path.to_string()
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
