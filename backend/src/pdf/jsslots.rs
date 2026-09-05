use boa_engine::{Context, JsString, JsValue, Source};

use super::payload::ExportPayload;
use crate::fsutil::log_warn;

pub struct PageTexts {
    pub header: (String, String, String),
    pub footer: (String, String, String),
}

pub fn page_texts(payload: &ExportPayload, page: usize, total: usize) -> PageTexts {
    let ctx = context_json(payload, page, total);
    PageTexts {
        header: (
            slot_text(
                &payload.header.binding,
                &ctx,
                default_header_binding(payload),
            ),
            slot_text(&payload.header.center, &ctx, String::new()),
            slot_text(&payload.header.flip, &ctx, default_header_flip(payload)),
        ),
        footer: (
            slot_text(&payload.footer.binding, &ctx, String::new()),
            slot_text(&payload.footer.center, &ctx, String::new()),
            slot_text(&payload.footer.flip, &ctx, page.to_string()),
        ),
    }
}

pub fn question_meta(
    payload: &ExportPayload,
    seq: usize,
    chapter: &str,
    locate: &str,
    subject: &str,
) -> String {
    let source = if payload.is_workbook() {
        &payload.meta.workbook
    } else {
        &payload.meta.answers
    };
    let q = serde_json::json!({
        "index": seq,
        "total": payload.items.len(),
        "chapter": chapter,
        "locate": locate,
        "subject": subject,
        "doc": payload.doc,
        "answers": !payload.is_workbook(),
    });
    let default = if locate.is_empty() {
        chapter.to_string()
    } else {
        format!("{chapter} @ {locate}")
    };
    slot_text(source, &q, default)
}

fn context_json(payload: &ExportPayload, page: usize, total: usize) -> serde_json::Value {
    let subject = payload
        .items
        .first()
        .map(|i| i.question.subject.as_str())
        .unwrap_or("");
    serde_json::json!({
        "page": page,
        "totalPages": total,
        "totalQuestions": payload.items.len(),
        "title": payload.title,
        "date": payload.date,
        "subject": subject,
        "doc": payload.doc,
        "answers": !payload.is_workbook(),
        "paper": payload.paper,
        "binding": payload.binding_long,
        "firstPageLeft": payload.first_page_left,
    })
}

fn default_header_binding(payload: &ExportPayload) -> String {
    format!("{} · 共 {} 题", payload.date, payload.items.len())
}

fn default_header_flip(payload: &ExportPayload) -> String {
    if payload.is_workbook() {
        payload.title.clone()
    } else {
        format!("{} - 答案", payload.title)
    }
}

fn slot_text(source: &str, ctx: &serde_json::Value, default: String) -> String {
    if source.trim().is_empty() {
        return default;
    }
    let run = || -> Result<String, String> {
        let mut js = Context::default();
        let func = js
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(|e| e.to_string())?;
        let mut arg = JsValue::from_json(ctx, &mut js).map_err(|e| e.to_string())?;
        upgrade_date_fields(&mut arg, &mut js);
        let callable = func
            .as_callable()
            .ok_or_else(|| "slot is not a function".to_string())?;
        let out = callable
            .call(&JsValue::undefined(), &[arg], &mut js)
            .map_err(|e| e.to_string())?;
        out.as_string()
            .map(|s| s.to_std_string_escaped())
            .ok_or_else(|| "slot did not return a string".to_string())
    };
    match run() {
        Ok(text) => text,
        Err(e) => {
            log_warn(&format!(
                "[warn] header/footer slot failed, using default: {e}"
            ));
            default
        }
    }
}

fn upgrade_date_fields(arg: &mut JsValue, js: &mut Context) {
    let Some(obj) = arg.as_object().map(|o| o.to_owned()) else {
        return;
    };
    let Ok(date_val0) = obj.get(JsString::from("date"), js) else {
        return;
    };
    let Some(iso_val) = date_val0.as_string() else {
        return;
    };
    let iso = iso_val.to_std_string_escaped();
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() < 3 {
        // non-ISO date (e.g. "2026/9/4"): leave the raw string untouched
        return;
    }
    let (y, m, d) = (parts[0], parts[1], parts[2]);
    fn no_lead(s: &str) -> &str {
        s.trim_start_matches('0')
    }
    let date_strs = serde_json::json!({
        "year": y,
        "month": m,
        "day": d,
        "ymd": format!("{y}-{m}-{d}"),
        "zh": format!("{}年{}月{}日", no_lead(y), no_lead(m), no_lead(d)),
        "compact": format!("{y}{m}{d}"),
    });
    if let Ok(date_val) = js.eval(Source::from_bytes(
        format!("new Date(\"{iso}\")").as_bytes(),
    )) {
        let _ = obj.set(JsString::from("date"), date_val, false, js);
    }
    if let Ok(strs) = JsValue::from_json(&date_strs, js) {
        let _ = obj.set(JsString::from("dateStrs"), strs, false, js);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload_with_meta(workbook: &str, answers: &str) -> ExportPayload {
        serde_json::from_str(&format!(
            r#"{{
                "doc": "workbook", "paper": "A4", "perPage": 2, "title": "t", "date": "2026-09-04",
                "meta": {{"workbook": "{workbook}", "answers": "{answers}"}},
                "items": [{{"id": "P1", "seq": 1, "source": "s", "subject": "数学", "origin": "o",
                    "locate": "P1-1", "chapter": "第1章", "qtype": "选择题", "stem": [],
                    "options": [], "correct_ids": [], "answer_line": "", "solution": [], "notes": []}}]
            }}"#
        ))
        .unwrap()
    }

    #[test]
    fn question_meta_runs_js_and_falls_back() {
        let p = payload_with_meta(r"(q) => 'No.' + q.index + ' of ' + q.total", "");
        assert_eq!(question_meta(&p, 1, "第1章", "P1-1", "数学"), "No.1 of 1");

        let p2 = payload_with_meta("", "");
        assert_eq!(
            question_meta(&p2, 1, "第1章", "P1-1", "数学"),
            "第1章 @ P1-1"
        );

        let p3 = payload_with_meta("(q) => null.x", "");
        assert_eq!(
            question_meta(&p3, 1, "第1章", "P1-1", "数学"),
            "第1章 @ P1-1"
        );
    }

    #[test]
    fn header_ctx_has_date_object_and_parts() {
        let json = r#"{
            "doc": "workbook", "paper": "A4", "perPage": 2, "title": "t", "date": "2026-09-04",
            "header": { "center": "(c) => c.date.getFullYear() + ' ' + c.dateStrs.zh" },
            "items": [{"id": "P1", "seq": 1, "source": "s", "subject": "数学", "origin": "o",
                "locate": "P1-1", "chapter": "第1章", "qtype": "选择题", "stem": [],
                "options": [], "correct_ids": [], "answer_line": "", "solution": [], "notes": []}]
        }"#;
        let payload: ExportPayload = serde_json::from_str(json).unwrap();
        let texts = page_texts(&payload, 1, 1);
        assert_eq!(texts.header.1, "2026 2026年9月4日");
    }

    #[test]
    fn non_iso_date_keeps_the_raw_string() {
        // "2026/9/4" is not ISO: no Date object, no dateStrs are injected
        let json = r#"{
            "doc": "workbook", "paper": "A4", "perPage": 2, "title": "t", "date": "2026/9/4",
            "header": { "center": "(c) => typeof c.date + '/' + (c.dateStrs ? 'has' : 'none')" },
            "items": [{"id": "P1", "seq": 1, "source": "s", "subject": "数学", "origin": "o",
                "locate": "P1-1", "chapter": "第1章", "qtype": "选择题", "stem": [],
                "options": [], "correct_ids": [], "answer_line": "", "solution": [], "notes": []}]
        }"#;
        let payload: ExportPayload = serde_json::from_str(json).unwrap();
        let texts = page_texts(&payload, 1, 1);
        assert_eq!(texts.header.1, "string/none");

        // sanity: the same slot with an ISO date sees a Date object
        let json = json.replace("2026/9/4", "2026-09-04");
        let payload: ExportPayload = serde_json::from_str(&json).unwrap();
        let texts = page_texts(&payload, 1, 1);
        assert_eq!(texts.header.1, "object/has");
    }
}
