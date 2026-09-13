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

    pub max_upload_bytes: usize,
    pub records: Arc<RecordStore>,
    pub books: Arc<BookStore>,
    pub sources: Arc<SourceStore>,
    pub mastery: Arc<MasteryStore>,
    pub paused: Arc<PausedStore>,
    pub export_cache: Arc<ExportCache>,
    pub pdf_lock: Arc<AsyncMutex<()>>,

    pub tmp_dir: PathBuf,
}

impl App {
    pub fn from_env() -> App {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let explicit_root = std::env::var("DRILL_ROOT").ok().map(PathBuf::from);
        let root = explicit_root
            .as_deref()
            .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
            .unwrap_or_else(|| manifest.join("../.."));
        let data_dir = env_path("DRILL_DATA", || manifest.join("data"));
        let dist = env_path("DRILL_DIST", || manifest.join("../frontend/dist"));
        let (sources, books) = seeded_stores(&data_dir, explicit_root.as_deref());
        let tmp_dir = stage_uploads(&data_dir);
        App {
            records: Arc::new(RecordStore::load(data_dir.join("records.json"))),
            mastery: Arc::new(MasteryStore::load(data_dir.join("mastery.json"))),
            paused: Arc::new(PausedStore::load(data_dir.join("paused.json"))),
            export_cache: Arc::new(ExportCache::new(data_dir.join("exports"))),
            pdf_lock: Arc::new(AsyncMutex::new(())),
            tmp_dir,
            sources,
            books,
            root,
            dist,
            data_dir,
            host: std::env::var("DRILL_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env_num("DRILL_PORT").unwrap_or(8787),
            max_upload_bytes: max_upload_bytes(),
        }
    }
}

fn seeded_stores(
    data_dir: &std::path::Path,
    explicit_root: Option<&std::path::Path>,
) -> (Arc<SourceStore>, Arc<BookStore>) {
    let first_run = !data_dir.join("sources.json").exists();
    let sources = Arc::new(SourceStore::load(data_dir.join("sources.json"), explicit_root));
    let books = Arc::new(BookStore::load(data_dir.join("books.json")));
    if first_run {
        crate::demo::seed_if_empty(&sources, &books, &data_dir.join("example"));
    }
    (sources, books)
}

fn stage_uploads(data_dir: &std::path::Path) -> PathBuf {
    let tmp_dir = data_dir.join("tmp-uploads");
    let _ = std::fs::create_dir_all(&tmp_dir);
    crate::api::uploads::sweep(&tmp_dir);
    tmp_dir
}

fn env_num<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

fn max_upload_bytes() -> usize {
    env_num::<usize>("DRILL_MAX_UPLOAD_MB")
        .map(|mb| mb.clamp(1, 200))
        .unwrap_or(crate::api::uploads::MAX_UPLOAD_BYTES / (1024 * 1024))
        * 1024
        * 1024
}

fn env_path(key: &str, default: impl FnOnce() -> PathBuf) -> PathBuf {
    let p = std::env::var(key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| default());
    p.canonicalize().unwrap_or(p)
}
