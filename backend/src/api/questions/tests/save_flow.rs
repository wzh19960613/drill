use super::*;

#[tokio::test]
async fn update_moves_and_renames_the_file() {
    let app = test_app("move");
    let bank = app.sources.get("s1").unwrap().path.clone();

    let resp = call(
        &app,
        "POST",
        "/api/questions/save",
        Some(save_body("P9-9", "s1")),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = json!({
        "source": "s1", "dir": "", "filename": "P9-10",
        "markdown": "> @ P9-10\n\n题干二______。\n\n## 答案\n\n43。\n",
        "original": "P9-9.md"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", text(resp).await);
    assert!(!bank.join("P9-9.md").exists());
    assert!(bank.join("P9-10.md").is_file());

    move_into_subfolder_and_overwrite_in_place(&app, &bank).await;
    saving_onto_a_foreign_file_conflicts(&app).await;
}

async fn save_expect(app: &App, body: serde_json::Value, status: StatusCode) {
    let resp = call(app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), status, "{}", text(resp).await);
}

async fn move_into_subfolder_and_overwrite_in_place(app: &App, bank: &std::path::Path) {
    let body = json!({
        "source": "s1", "dir": "子卷", "filename": "P9-10",
        "markdown": "> @ P9-10\n\n题干二______。\n\n## 答案\n\n43。\n",
        "original": "P9-10.md"
    });
    save_expect(app, body, StatusCode::OK).await;
    assert!(!bank.join("P9-10.md").exists());
    assert!(bank.join("子卷/P9-10.md").is_file());

    let resp = call(app, "GET", "/api/questions/P9-10/raw?source=s1", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    assert_eq!(v["dir"], "子卷");

    let body = json!({
        "source": "s1", "dir": "子卷", "filename": "P9-10",
        "markdown": "> @ P9-10\n\n题干三______。\n\n## 答案\n\n44。\n",
        "original": "子卷/P9-10.md"
    });
    save_expect(app, body, StatusCode::OK).await;
}

async fn saving_onto_a_foreign_file_conflicts(app: &App) {
    let body = json!({
        "source": "s1", "dir": "", "filename": "P1-1",
        "markdown": "> @ P1-1\n\n题干______。\n\n## 答案\n\n42。\n"
    });
    save_expect(app, body, StatusCode::CONFLICT).await;
}

#[tokio::test]
async fn save_moves_uploads_and_applies_renames() {
    let app = test_app("images");
    let bank = app.sources.get("s1").unwrap().path.clone();

    std::fs::write(bank.join("旧图.svg"), "<svg/>").unwrap();

    let temp_id = stage_upload(&app, "shot.png").await;

    let md =
        "> @ P9-9\n\n题干 ![[P9-9-01.png]] ![[新图.svg]]______。\n\n## 答案\n\n42。\n".to_string();
    let body = json!({
        "source": "s1", "dir": "子卷", "filename": "P9-9",
        "markdown": md,
        "images": [ { "temp": temp_id, "name": "P9-9-01.png" } ],
        "renames": [ { "from": "旧图.svg", "to": "新图.svg" } ]
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::CREATED, "{}", text(resp).await);

    assert!(bank.join("子卷/P9-9-01.png").is_file(), "upload moved in");
    assert!(!app.tmp_dir.join(&temp_id).exists(), "temp consumed");
    assert!(bank.join("新图.svg").is_file(), "rename applied");
    assert!(!bank.join("旧图.svg").exists(), "old name gone");

    let temp2 = stage_upload(&app, "shot2.png").await;
    let body = json!({
        "source": "s1", "dir": "子卷", "filename": "P9-8",
        "markdown": "> @ P9-8\n\n题干______。\n\n## 答案\n\n42。\n",
        "images": [ { "temp": temp2, "name": "P9-9-01.png" } ]
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert!(app.tmp_dir.join(&temp2).exists(), "temp untouched on conflict");
    assert!(!bank.join("子卷/P9-8.md").exists(), "md not written on conflict");
}

#[tokio::test]
async fn save_rejects_traversal_temps_and_non_md_originals() {
    let app = test_app("save-unsafe");
    let bank = app.sources.get("s1").unwrap().path.clone();

    std::fs::write(bank.join("共享图.svg"), "<svg/>").unwrap();

    let body = json!({
        "source": "s1", "dir": "", "filename": "X9",
        "markdown": "> @ X9\n\n题干______。\n\n## 答案\n\n42。\n",
        "images": [ { "temp": "../../bank/共享图.svg", "name": "X9-01.svg" } ]
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{}", text(resp).await);
    assert!(bank.join("共享图.svg").is_file(), "nothing was moved");
    assert!(!bank.join("X9-01.svg").exists(), "no image written");

    let body = json!({
        "source": "s1", "dir": "", "filename": "X9",
        "markdown": "> @ X9\n\n题干______。\n\n## 答案\n\n42。\n",
        "original": "共享图.svg"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{}", text(resp).await);
    assert!(bank.join("共享图.svg").is_file(), "foreign file not deleted");
}

#[tokio::test]
async fn save_rejects_duplicate_id_in_another_folder() {
    let app = test_app("save-dup");
    let bank = app.sources.get("s1").unwrap().path.clone();
    std::fs::create_dir_all(bank.join("子卷")).unwrap();

    let body = json!({
        "source": "s1", "dir": "子卷", "filename": "P1-1",
        "markdown": "> @ P1-1\n\n题干二______。\n\n## 答案\n\n43。\n"
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT, "{}", text(resp).await);
    assert!(!bank.join("子卷/P1-1.md").exists());
}

#[tokio::test]
async fn save_failure_rolls_back_renames_and_uploads() {
    let app = test_app("save-rollback");
    let bank = app.sources.get("s1").unwrap().path.clone();

    std::fs::write(bank.join("blocker"), b"x").unwrap();
    std::fs::write(bank.join("旧图.svg"), "<svg/>").unwrap();

    let temp_id = stage_upload(&app, "shot.png").await;

    let body = json!({
        "source": "s1", "dir": "blocker", "filename": "X1",
        "markdown": "> @ X1\n\n题干______。\n\n## 答案\n\n42。\n",
        "images": [ { "temp": temp_id, "name": "X1-01.png" } ],
        "renames": [ { "from": "旧图.svg", "to": "新图.svg" } ]
    });
    let resp = call(&app, "POST", "/api/questions/save", Some(body)).await;
    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "{}",
        text(resp).await
    );

    assert!(bank.join("旧图.svg").is_file(), "rename rolled back");
    assert!(!bank.join("新图.svg").exists());
    assert!(app.tmp_dir.join(&temp_id).is_file(), "upload back in tmp");
    assert!(!bank.join("blocker/X1.md").exists(), "no md written");
}

