mod api;
mod app;
mod demo;
mod fsutil;
mod images;
mod model;
mod parsing;
mod pdf;
mod store;

use app::App;
use fsutil::log_line;

#[tokio::main]
async fn main() {
    let app = App::from_env();
    print_startup(&app);
    let router = api::router(app.clone());
    let addr = format!("{}:{}", app.host, app.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("port bind failed (set DRILL_PORT / DRILL_HOST to change)");
    log_line(&format!("serving on http://{addr}"));
    axum::serve(listener, router).await.unwrap();
}

fn print_startup(app: &App) {
    let sources = app.sources.list();
    let count: usize = sources
        .iter()
        .map(|s| {
            let excluded: std::collections::HashSet<String> = s.excluded.iter().cloned().collect();
            parsing::load_questions_from(&s.path, &s.id, s.recursive, &excluded).len()
        })
        .sum();
    log_line(&format!(
        "parsed {count} questions from {} source(s)",
        sources.len()
    ));
    log_line("pdf engine: embedded Typst (fonts bundled, no external runtime)");
}
