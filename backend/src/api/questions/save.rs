use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use super::rel_under_source;
use super::save_fs::{move_file, split_rel};
use crate::api::server_error;
use crate::app::App;
use crate::fsutil::{atomic_write, is_safe_rel_path};
use crate::images::image_ext;
use crate::model::Question;
use crate::parsing::{self, Classified};

#[derive(Deserialize)]
pub struct SaveImage {
    pub temp: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct RenameImage {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
pub struct SaveQuestion {
    pub source: String,
    pub dir: String,
    pub filename: String,
    pub markdown: String,
    #[serde(default)]
    pub images: Vec<SaveImage>,
    #[serde(default)]
    pub renames: Vec<RenameImage>,
    #[serde(default)]
    pub original: Option<String>,
}

static SAVE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct SavePlan {
    src_path: PathBuf,
    question: Question,
    target_dir: PathBuf,
    target: PathBuf,
    old_path: Option<PathBuf>,
}

pub async fn save(State(app): State<App>, Json(body): Json<SaveQuestion>) -> Response {
    let _guard = crate::fsutil::lock(&SAVE_LOCK);
    let plan = match plan_save(&app, &body) {
        Ok(plan) => plan,
        Err(resp) => return *resp,
    };
    match apply(&app, &body, &plan) {
        Ok(()) => {
            let status = if plan.old_path.is_some() {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            (status, Json(plan.question)).into_response()
        }
        Err(resp) => *resp,
    }
}

fn plan_save(app: &App, body: &SaveQuestion) -> Result<SavePlan, Box<Response>> {
    let src = app
        .sources
        .get(&body.source)
        .ok_or_else(not_found_response)?;
    let filename = normalize_filename(&body.filename)
        .ok_or_else(|| bad_request_response("invalid file name"))?;
    if !is_safe_rel_path(&body.dir) {
        return Err(bad_request_response("invalid target folder"));
    }
    let question = classified_question(&filename, &body.markdown)?;
    if question.id != filename {
        return Err(bad_request_response(&format!(
            "元信息「定位」({}) 与文件名 ({filename}) 不一致",
            question.id
        )));
    }
    let target_dir = src.path.join(&body.dir);
    let target = target_dir.join(format!("{filename}.md"));
    let old_path = original_path(&src.path, body)?;
    check_target_free(&src.path, &target, &old_path, &filename)?;
    check_images(app, &target_dir, body)?;
    check_renames(&src.path, body)?;
    Ok(SavePlan {
        src_path: src.path,
        question,
        target_dir,
        target,
        old_path,
    })
}

fn not_found_response() -> Box<Response> {
    Box::new((StatusCode::NOT_FOUND, "unknown source id").into_response())
}

fn bad_request_response(msg: &str) -> Box<Response> {
    Box::new((StatusCode::BAD_REQUEST, msg.to_string()).into_response())
}

fn conflict_response(msg: &str) -> Box<Response> {
    Box::new((StatusCode::CONFLICT, msg.to_string()).into_response())
}

fn classified_question(filename: &str, markdown: &str) -> Result<Question, Box<Response>> {
    match parsing::classify(&format!("{filename}.md"), markdown) {
        Classified::Question(q) => Ok(*q),
        Classified::Rejected(reason) => {
            Err(bad_request_response(&format!("内容不符合题目格式：{reason}")))
        }
    }
}

fn original_path(src_path: &std::path::Path, body: &SaveQuestion) -> Result<Option<PathBuf>, Box<Response>> {
    let Some(original) = &body.original else {
        return Ok(None);
    };
    let (old_dir, old_file) = split_rel(original);
    if !is_safe_rel_path(&old_dir)
        || !crate::fsutil::is_safe_component(&old_file)
        || !old_file.ends_with(".md")
    {
        return Err(bad_request_response("invalid original path"));
    }
    let p = src_path.join(&old_dir).join(old_file);
    if !p.is_file() {
        return Err(not_found_response());
    }
    Ok(Some(p))
}

#[allow(clippy::too_many_arguments)]
fn check_target_free(
    src_path: &std::path::Path,
    target: &std::path::Path,
    old_path: &Option<PathBuf>,
    filename: &str,
) -> Result<(), Box<Response>> {
    if target.exists() && old_path.as_deref() != Some(target) {
        return Err(conflict_response(&format!("{filename}.md 已存在于该文件夹")));
    }
    for existing in crate::fsutil::find_all_recursive(src_path, &format!("{filename}.md")) {
        if existing != target && old_path.as_deref() != Some(existing.as_path()) {
            let dir = rel_under_source(src_path, &existing);
            return Err(conflict_response(&format!(
                "题目 {filename}.md 已存在于该题源的其他位置（{}）",
                if dir.is_empty() { "根目录".to_string() } else { format!("{dir}/") }
            )));
        }
    }
    Ok(())
}

fn check_images(app: &App, target_dir: &std::path::Path, body: &SaveQuestion) -> Result<(), Box<Response>> {
    let mut seen: HashSet<&str> = HashSet::new();
    for img in &body.images {
        if !crate::fsutil::is_safe_component(&img.name) || image_ext(&img.name).is_none() {
            return Err(bad_request_response(&format!("非法图片名：{}", img.name)));
        }
        if img.temp == img.name {
            return Err(bad_request_response("图片名不能是临时上传 id"));
        }
        if !seen.insert(img.name.as_str()) {
            return Err(bad_request_response(&format!("图片名重复：{}", img.name)));
        }
        if target_dir.join(&img.name).exists() {
            return Err(conflict_response(&format!("图片 {} 已存在于该文件夹", img.name)));
        }
        if !crate::fsutil::is_safe_component(&img.temp) {
            return Err(bad_request_response(&format!("非法上传 id：{}", img.temp)));
        }
        if !app.tmp_dir.join(&img.temp).is_file() {
            return Err(bad_request_response(&format!("上传已失效：{}", img.temp)));
        }
    }
    Ok(())
}

fn check_renames(src_path: &std::path::Path, body: &SaveQuestion) -> Result<(), Box<Response>> {
    for r in &body.renames {
        if r.from == r.to {
            continue;
        }
        for name in [&r.from, &r.to] {
            if !crate::fsutil::is_safe_component(name) || image_ext(name).is_none() {
                return Err(bad_request_response(&format!("非法图片名：{name}")));
            }
        }
        if crate::fsutil::find_recursive(src_path, &r.from).is_none() {
            return Err(bad_request_response(&format!(
                "图片 {} 不在该题源中，无法重命名",
                r.from
            )));
        }
        if crate::fsutil::find_recursive(src_path, &r.to).is_some() {
            return Err(conflict_response(&format!("图片名 {} 已被使用", r.to)));
        }
        if body.images.iter().any(|i| i.name == r.to) {
            return Err(bad_request_response(&format!("图片名重复：{}", r.to)));
        }
    }
    Ok(())
}

fn apply(app: &App, body: &SaveQuestion, plan: &SavePlan) -> Result<(), Box<Response>> {
    let mut undo: Vec<Box<dyn FnOnce()>> = Vec::new();

    let result = (|| -> Result<(), Box<Response>> {
        apply_renames(&plan.src_path, body, &mut undo)?;
        apply_uploads(app, &plan.target_dir, body, &mut undo)?;
        write_and_replace(plan, body, &mut undo)?;
        Ok(())
    })();
    if let Err(resp) = result {
        for step in undo.into_iter().rev() {
            step();
        }
        return Err(resp);
    }
    Ok(())
}

fn server_error_boxed(msg: String) -> Box<Response> {
    Box::new(server_error(&msg))
}

fn apply_renames(
    src_path: &std::path::Path,
    body: &SaveQuestion,
    undo: &mut Vec<Box<dyn FnOnce()>>,
) -> Result<(), Box<Response>> {
    for r in &body.renames {
        if r.from == r.to {
            continue;
        }
        let from_path = crate::fsutil::find_recursive(src_path, &r.from)
            .ok_or_else(|| server_error_boxed(format!("rename source vanished: {}", r.from)))?;
        let to_path = from_path.with_file_name(&r.to);
        fs::rename(&from_path, &to_path).map_err(|e| {
            server_error_boxed(format!("重命名 {} 失败：{e}", r.from))
        })?;
        let (back, done) = (from_path.clone(), to_path.clone());
        undo.push(Box::new(move || {
            let _ = fs::rename(&done, &back);
        }));
    }
    Ok(())
}

fn apply_uploads(
    app: &App,
    target_dir: &std::path::Path,
    body: &SaveQuestion,
    undo: &mut Vec<Box<dyn FnOnce()>>,
) -> Result<(), Box<Response>> {
    for img in &body.images {
        let tmp = app.tmp_dir.join(&img.temp);
        let dest = target_dir.join(&img.name);
        move_file(&tmp, &dest)
            .map_err(|e| server_error_boxed(format!("保存图片 {} 失败：{e}", img.name)))?;
        let (back, done) = (tmp.clone(), dest.clone());
        undo.push(Box::new(move || {
            let _ = move_file(&done, &back);
        }));
    }
    Ok(())
}

fn write_and_replace(
    plan: &SavePlan,
    body: &SaveQuestion,
    undo: &mut Vec<Box<dyn FnOnce()>>,
) -> Result<(), Box<Response>> {
    atomic_write(&plan.target, body.markdown.as_bytes())
        .map_err(|e| server_error_boxed(format!("save failed: {e}")))?;

    if plan.old_path.as_deref() != Some(plan.target.as_path()) {
        let done = plan.target.clone();
        undo.push(Box::new(move || {
            let _ = fs::remove_file(&done);
        }));
    }
    if let Some(old) = &plan.old_path {
        if old != &plan.target {

            fs::remove_file(old)
                .map_err(|e| server_error_boxed(format!("删除原文件失败：{e}")))?;
        }
    }
    Ok(())
}

fn normalize_filename(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let name = trimmed.strip_suffix(".md").unwrap_or(trimmed).trim();
    crate::fsutil::is_safe_component(name).then(|| name.to_string())
}
