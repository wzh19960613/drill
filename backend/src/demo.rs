use std::path::Path;

use crate::model::{now_ms, BookDef, BookItemDef};
use crate::parsing;
use crate::store::{BookStore, SourceStore};

const DEMO_ID: &str = "demo";

/// Example bank embedded at compile time and materialized into the data
/// directory on first run: user edits land on the private copy instead of
/// the repository, and a distributed binary ships the demo without the
/// source tree.
const EXAMPLE_FILES: &[(&str, &str)] = &[
    ("E1.md", include_str!("../example/E1.md")),
    ("E2.md", include_str!("../example/E2.md")),
    ("E3.md", include_str!("../example/E3.md")),
    ("E4.md", include_str!("../example/E4.md")),
    ("E5.md", include_str!("../example/E5.md")),
    ("E6.md", include_str!("../example/E6.md")),
    ("E7.md", include_str!("../example/E7.md")),
    ("E8.md", include_str!("../example/E8.md")),
    ("E9.md", include_str!("../example/E9.md")),
    (
        "示例-三次曲线.svg",
        include_str!("../example/示例-三次曲线.svg"),
    ),
];

/// Seed the example bank and a demo book on first run. The bank is copied
/// from the embedded files into `example_dir`, a fresh directory inside the
/// data dir. Idempotent: skips when the demo book already exists or the
/// example directory is already registered as a source (the default source
/// seeded by SourceStore does not block the demo), and never overwrites
/// files already on disk, so a re-seed never clobbers user edits.
pub fn seed_if_empty(sources: &SourceStore, books: &BookStore, example_dir: &Path) {
    if books.list().iter().any(|b| b.id == DEMO_ID) {
        return;
    }
    // the already-registered guard must run before materialization: a
    // directory that is already a source must never be rewritten
    if let Ok(path) = example_dir.canonicalize() {
        if sources.list().iter().any(|s| s.path == path) {
            return;
        }
    }
    if materialize(example_dir).is_err() {
        return;
    }
    // add() rejects a path that is already a source, which doubles as the
    // "already seeded" guard
    let Ok(src) = sources.add(&example_dir.to_string_lossy(), example_dir) else {
        return;
    };
    let items: Vec<BookItemDef> = parsing::load_questions_from(&src.path, &src.id)
        .into_iter()
        .map(|q| BookItemDef {
            id: q.id,
            source: q.source,
            option_order: None,
        })
        .collect();
    if items.is_empty() {
        return;
    }
    let created = books.upsert(BookDef {
        id: DEMO_ID.into(),
        name: "示例题本".into(),
        seed: String::new(),
        date: today(),
        created_at: now_ms(),
        items,
    });
    if created.is_ok() {
        books.set_active("", Some(DEMO_ID.into()));
    }
}

/// Write the embedded example files, skipping any that already exist.
fn materialize(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, content) in EXAMPLE_FILES {
        let path = dir.join(name);
        if !path.is_file() {
            std::fs::write(&path, content)?;
        }
    }
    Ok(())
}

fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    iso_from_days((secs / 86400) as i64)
}

fn iso_from_days(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_example_source_and_demo_book() {
        let dir = std::env::temp_dir().join(format!("drill-demo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let example = dir.join("example");

        let sources = SourceStore::load(dir.join("sources.json"), None);
        let books = BookStore::load(dir.join("books.json"));
        seed_if_empty(&sources, &books, &example);

        let list = sources.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].path, example.canonicalize().unwrap());
        assert!(example.join("E1.md").is_file(), "questions materialized");
        assert!(
            example.join("示例-三次曲线.svg").is_file(),
            "asset materialized"
        );
        let book = &books.list()[0];
        assert_eq!((book.id.as_str(), book.name.as_str()), ("demo", "示例题本"));
        assert_eq!(
            book.items.len(),
            EXAMPLE_FILES.len() - 1,
            "one item per md file"
        );
        assert_eq!(book.items[0].id, "E1");
        assert_eq!(
            book.items[0].source, list[0].id,
            "item carries the source id"
        );
        assert_eq!(books.get_active("").as_deref(), Some("demo"));

        seed_if_empty(&sources, &books, &example);
        assert_eq!(books.list().len(), 1, "second call does not re-seed");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn seeds_even_when_the_default_source_already_exists() {
        // production layout: with DRILL_ROOT set, SourceStore::load seeds the
        // root itself as the default source, which used to block the demo
        // book entirely
        let dir = std::env::temp_dir().join(format!("drill-demo-default-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let sources = SourceStore::load(dir.join("sources.json"), Some(&dir));
        assert_eq!(sources.list().len(), 1, "default source seeded by load");
        let books = BookStore::load(dir.join("books.json"));
        seed_if_empty(&sources, &books, &dir.join("example"));

        assert_eq!(
            sources.list().len(),
            2,
            "example bank added next to default"
        );
        assert!(
            books.list().iter().any(|b| b.id == "demo"),
            "demo book must be seeded on first run"
        );
        assert_eq!(books.get_active("").as_deref(), Some("demo"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn demo_book_date_is_iso() {
        let dir = std::env::temp_dir().join(format!("drill-demo-date-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let sources = SourceStore::load(dir.join("sources.json"), None);
        let books = BookStore::load(dir.join("books.json"));
        seed_if_empty(&sources, &books, &dir.join("example"));
        let date = books.list()[0].date.clone();
        assert_eq!(date.len(), 10, "YYYY-MM-DD: {date}");
        assert_eq!(date.as_bytes()[4], b'-');
        assert_eq!(date.as_bytes()[7], b'-');
        assert!(date.bytes().all(|b| b.is_ascii_digit() || b == b'-'));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn materialize_never_overwrites_existing_files() {
        // sources.json lost but the materialized copy survived: re-seeding
        // must fill in gaps without clobbering user edits
        let dir = std::env::temp_dir().join(format!("drill-demo-keep-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let example = dir.join("example");
        std::fs::create_dir_all(&example).unwrap();
        std::fs::write(example.join("E1.md"), "user edits").unwrap();

        let sources = SourceStore::load(dir.join("sources.json"), None);
        let books = BookStore::load(dir.join("books.json"));
        seed_if_empty(&sources, &books, &example);

        assert_eq!(
            std::fs::read_to_string(example.join("E1.md")).unwrap(),
            "user edits",
            "existing file kept as-is"
        );
        assert!(
            example.join("E2.md").is_file(),
            "missing files still filled in"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn seeded_bank_is_not_rewritten_after_restart() {
        let dir = std::env::temp_dir().join(format!("drill-demo-restart-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let example = dir.join("example");
        let sources = SourceStore::load(dir.join("sources.json"), None);
        let books = BookStore::load(dir.join("books.json"));
        seed_if_empty(&sources, &books, &example);
        std::fs::remove_file(example.join("E2.md")).unwrap();

        // simulate a restart: stores reload from disk and the example copy
        // is already a registered source
        let sources = SourceStore::load(dir.join("sources.json"), None);
        let books = BookStore::load(dir.join("books.json"));
        seed_if_empty(&sources, &books, &example);
        assert!(
            !example.join("E2.md").exists(),
            "a registered source must not be re-materialized"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn civil_dates_match_known_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20000), (2024, 10, 4));
        assert_eq!(civil_from_days(20646), (2026, 7, 12));
        assert_eq!(iso_from_days(20646), "2026-07-12");
        assert_eq!(iso_from_days(0), "1970-01-01");
    }
}
