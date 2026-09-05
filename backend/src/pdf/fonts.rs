use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::{FileId, Source};
use typst_as_lib::file_resolver::FileResolver;
use typst_as_lib::TypstEngine;

use crate::fsutil::lock;

static FONT_CJK: &[u8] = include_bytes!("../../assets/fonts/NotoSansSC.ttf");
static FONT_EMOJI: &[u8] = include_bytes!("../../assets/fonts/NotoEmoji.ttf");

fn font_files() -> Vec<&'static [u8]> {
    let mut files: Vec<&'static [u8]> = typst_assets::fonts().collect();
    files.push(FONT_CJK);
    files.push(FONT_EMOJI);
    files
}

/// Registry of generated sources: each compile registers its generated main
/// file under a unique virtual path before compiling, and removes it again
/// afterwards so the texts do not accumulate for the lifetime of the process.
#[derive(Default)]
pub struct SourceRegistry {
    sources: Mutex<HashMap<String, String>>,
}

impl SourceRegistry {
    pub fn set(&self, path: &str, text: String) {
        lock(&self.sources).insert(path.to_string(), text);
    }

    pub fn remove(&self, path: &str) {
        lock(&self.sources).remove(path);
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        lock(&self.sources).len()
    }
}

pub struct RegistryFileResolver(pub Arc<SourceRegistry>);

impl FileResolver for RegistryFileResolver {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        self.0.resolve_binary(id)
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        self.0.resolve_source(id)
    }
}

impl FileResolver for SourceRegistry {
    fn resolve_binary(&self, id: FileId) -> FileResult<Cow<'_, Bytes>> {
        Err(FileError::NotFound(id.vpath().get_without_slash().into()))
    }

    fn resolve_source(&self, id: FileId) -> FileResult<Cow<'_, Source>> {
        let sources = lock(&self.sources);
        let text = sources
            .get(id.vpath().get_without_slash())
            .ok_or_else(|| FileError::NotFound(id.vpath().get_without_slash().into()))?;
        Ok(Cow::Owned(Source::new(id, text.clone())))
    }
}

/// Built once per process; every export compiles through it so comemo
/// memoization and font parsing are shared.
pub fn build_engine(registry: std::sync::Arc<SourceRegistry>) -> TypstEngine {
    static MITEX_PRELUDE: &str = include_str!("../../assets/mitex/prelude.typ");
    static MITEX_STANDARD: &str = include_str!("../../assets/mitex/standard.typ");

    TypstEngine::builder()
        .fonts(font_files())
        .with_static_source_file_resolver([
            ("mitex/prelude.typ", MITEX_PRELUDE),
            ("mitex/standard.typ", MITEX_STANDARD),
        ])
        .add_file_resolver(RegistryFileResolver(registry))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_set_and_remove_keep_it_empty() {
        let registry = SourceRegistry::default();
        assert_eq!(registry.len(), 0);
        registry.set("exports/gen-1/main.typ", "source".to_string());
        assert_eq!(registry.len(), 1);
        registry.remove("exports/gen-1/main.typ");
        assert_eq!(registry.len(), 0, "removed entries free the source text");
        registry.remove("never-set");
        assert_eq!(registry.len(), 0, "removing an unknown entry is a no-op");
    }
}
