use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex as AsyncMutex;

use crate::api::export_cache::ExportCache;
use crate::store::{BookStore, MasteryStore, PausedStore, RecordStore, SourceStore};

#[derive(Clone)]
pub struct App {
    pub root: PathBuf,
    pub dist: PathBuf,
    pub data_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub records: Arc<RecordStore>,
    pub books: Arc<BookStore>,
    pub sources: Arc<SourceStore>,
    pub mastery: Arc<MasteryStore>,
    pub paused: Arc<PausedStore>,
    pub export_cache: Arc<ExportCache>,
    pub pdf_lock: Arc<AsyncMutex<()>>,
}

impl App {
    pub fn from_env() -> App {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // an explicitly configured bank root is registered as the default
        // source on first run; the from-source default seeds nothing
        let explicit_root = std::env::var("DRILL_ROOT").ok().map(PathBuf::from);
        let root = explicit_root
            .as_deref()
            .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
            .unwrap_or_else(|| manifest.join("../.."));
        let data_dir = env_path("DRILL_DATA", || manifest.join("data"));
        let dist = env_path("DRILL_DIST", || manifest.join("../frontend/dist"));
        let port = std::env::var("DRILL_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8787);
        let host = std::env::var("DRILL_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        // must be computed before SourceStore::load creates sources.json
        let first_run = !data_dir.join("sources.json").exists();
        let sources = Arc::new(SourceStore::load(
            data_dir.join("sources.json"),
            explicit_root.as_deref(),
        ));
        let books = Arc::new(BookStore::load(data_dir.join("books.json")));
        if first_run {
            crate::demo::seed_if_empty(&sources, &books, &data_dir.join("example"));
        }
        App {
            records: Arc::new(RecordStore::load(data_dir.join("records.json"))),
            mastery: Arc::new(MasteryStore::load(data_dir.join("mastery.json"))),
            paused: Arc::new(PausedStore::load(data_dir.join("paused.json"))),
            export_cache: Arc::new(ExportCache::new(data_dir.join("exports"))),
            pdf_lock: Arc::new(AsyncMutex::new(())),
            sources,
            books,
            root,
            dist,
            data_dir,
            host,
            port,
        }
    }
}

fn env_path(key: &str, default: impl FnOnce() -> PathBuf) -> PathBuf {
    let p = std::env::var(key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| default());
    p.canonicalize().unwrap_or(p)
}
