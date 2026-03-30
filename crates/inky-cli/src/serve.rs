use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use inky_core::{Config, Inky};

use crate::build;

/// A rendered template held in memory.
struct RenderedTemplate {
    html: String,
}

/// Global version counter incremented on each rebuild.
static VERSION: AtomicU64 = AtomicU64::new(1);

pub fn cmd_serve(
    input: PathBuf,
    build_ctx: crate::build::BuildContext,
    data_path: Option<PathBuf>,
    port: u16,
) {
    if !input.is_dir() {
        eprintln!(
            "{} Input path '{}' is not a directory",
            "error:".red().bold(),
            input.display()
        );
        std::process::exit(1);
    }

    let input = std::fs::canonicalize(&input).unwrap_or(input);

    let config = Config {
        column_count: build_ctx.columns,
        output_mode: build_ctx.output_mode,
        ..Config::default()
    };

    // Load merge data
    let merge_data = crate::util::load_json_data(data_path.as_deref());

    // Build all templates into memory
    let templates: Arc<RwLock<HashMap<String, RenderedTemplate>>> =
        Arc::new(RwLock::new(HashMap::new()));

    build_all_templates(&input, &config, &build_ctx, merge_data.as_ref(), &templates);

    let addr = format!("0.0.0.0:{}", port);
    let server = tiny_http::Server::http(&addr).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to start server on {}: {}",
            "error:".red().bold(),
            addr,
            e
        );
        std::process::exit(1);
    });

    eprintln!("\n  {} http://localhost:{}", "serving".green().bold(), port);
    eprintln!("  {} {}", "watching".cyan().bold(), input.display());
    eprintln!("  press {} to stop\n", "Ctrl+C".bold());

    // Spawn file watcher thread
    let watcher_templates = Arc::clone(&templates);
    let watcher_input = input.clone();
    let watcher_data_path = data_path.clone();
    let watcher_ctx = build_ctx.clone();

    std::thread::spawn(move || {
        run_file_watcher(
            watcher_input,
            config,
            watcher_ctx,
            watcher_data_path,
            watcher_templates,
        );
    });

    // Handle HTTP requests
    let server = Arc::new(server);
    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if url == "/_poll" {
            let version = VERSION.load(Ordering::Relaxed);
            let response = tiny_http::Response::from_string(version.to_string())
                .with_header(
                    "Content-Type: text/plain"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_header(
                    "Cache-Control: no-cache"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                );
            let _ = request.respond(response);
        } else if url == "/" {
            let index_html = build_index_page(&templates, port);
            let response = tiny_http::Response::from_string(index_html).with_header(
                "Content-Type: text/html; charset=utf-8"
                    .parse::<tiny_http::Header>()
                    .unwrap(),
            );
            let _ = request.respond(response);
        } else {
            // Strip leading slash to get the template name
            let name = url.trim_start_matches('/');
            let state = templates.read().unwrap();
            if let Some(tmpl) = state.get(name) {
                let html = inject_reload_script(&tmpl.html);
                let response = tiny_http::Response::from_string(html).with_header(
                    "Content-Type: text/html; charset=utf-8"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                );
                let _ = request.respond(response);
            } else {
                let response = tiny_http::Response::from_string("404 Not Found")
                    .with_status_code(404)
                    .with_header(
                        "Content-Type: text/plain"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    );
                let _ = request.respond(response);
            }
        }
    }
}

fn build_all_templates(
    input: &Path,
    config: &Config,
    build_ctx: &crate::build::BuildContext,
    merge_data: Option<&serde_json::Value>,
    templates: &Arc<RwLock<HashMap<String, RenderedTemplate>>>,
) {
    let inky = Inky::with_config(config.clone());
    let files = crate::util::find_files(input, crate::util::TEMPLATE_EXTENSIONS);

    let mut state = templates.write().unwrap();
    state.clear();

    for file in &files {
        let name = template_name(file, input);
        match std::fs::read_to_string(file) {
            Ok(html) => {
                let result =
                    build::process_template(&inky, &html, build_ctx, file.parent(), merge_data);
                eprintln!("  {} {}", "built".green().bold(), name);
                state.insert(name, RenderedTemplate { html: result });
            }
            Err(e) => {
                eprintln!(
                    "  {} Failed to read {}: {}",
                    "warning:".yellow().bold(),
                    file.display(),
                    e
                );
            }
        }
    }
}

/// Derive a template name from a file path relative to the input directory.
/// e.g. /path/to/input/welcome.inky -> "welcome.html"
fn template_name(file: &Path, input_dir: &Path) -> String {
    let relative = file.strip_prefix(input_dir).unwrap_or(file);
    let mut name = relative.to_string_lossy().to_string();
    if name.ends_with(".inky") {
        name = name[..name.len() - 5].to_string() + ".html";
    }
    name
}

fn run_file_watcher(
    input: PathBuf,
    config: Config,
    build_ctx: crate::build::BuildContext,
    data_path: Option<PathBuf>,
    templates: Arc<RwLock<HashMap<String, RenderedTemplate>>>,
) {
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(300), tx).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to create file watcher: {}",
            "error:".red().bold(),
            e
        );
        std::process::exit(1);
    });

    debouncer
        .watcher()
        .watch(&input, notify::RecursiveMode::Recursive)
        .unwrap_or_else(|e| {
            eprintln!(
                "{} Failed to watch directory '{}': {}",
                "error:".red().bold(),
                input.display(),
                e
            );
            std::process::exit(1);
        });

    // Watch data file directory
    if let Some(ref data_file) = data_path {
        if let Some(parent) = data_file.parent() {
            let canonical = std::fs::canonicalize(parent).unwrap_or(parent.to_path_buf());
            debouncer
                .watcher()
                .watch(&canonical, notify::RecursiveMode::NonRecursive)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "  {} Failed to watch data file directory '{}': {}",
                        "warning:".yellow().bold(),
                        canonical.display(),
                        e
                    );
                });
        }
    }

    // Watch include directories
    let include_dirs = crate::util::find_include_dirs(&input);
    for dir in &include_dirs {
        if dir != &input {
            eprintln!("  {} {}", "watching".cyan().bold(), dir.display());
            debouncer
                .watcher()
                .watch(dir, notify::RecursiveMode::Recursive)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "  {} Failed to watch include directory '{}': {}",
                        "warning:".yellow().bold(),
                        dir.display(),
                        e
                    );
                });
        }
    }

    let mut merge_data = crate::util::load_json_data(data_path.as_deref());

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let mut needs_rebuild = false;
                let mut data_changed = false;

                for event in &events {
                    let path = &event.path;

                    // Check if data file changed
                    if let Some(ref data_file) = data_path {
                        let canonical_data =
                            std::fs::canonicalize(data_file).unwrap_or(data_file.clone());
                        let canonical_event = std::fs::canonicalize(path).unwrap_or(path.clone());
                        if canonical_event == canonical_data {
                            data_changed = true;
                            continue;
                        }
                    }

                    if !crate::util::is_watchable_file(path) {
                        continue;
                    }

                    if let DebouncedEventKind::Any = event.kind {
                        needs_rebuild = true;
                    }
                }

                if data_changed {
                    eprintln!("  data file changed, reloading...");
                    merge_data = crate::util::load_json_data(data_path.as_deref());
                    needs_rebuild = true;
                }

                if needs_rebuild {
                    eprintln!("  rebuilding templates...");
                    build_all_templates(
                        &input,
                        &config,
                        &build_ctx,
                        merge_data.as_ref(),
                        &templates,
                    );
                    VERSION.fetch_add(1, Ordering::Relaxed);
                    eprintln!("  {} templates updated", "done".green().bold());
                }
            }
            Ok(Err(error)) => {
                eprintln!("  {} watch error: {}", "error:".red().bold(), error);
            }
            Err(e) => {
                eprintln!("{} Watch channel closed: {}", "error:".red().bold(), e);
                return;
            }
        }
    }
}

fn build_index_page(
    templates: &Arc<RwLock<HashMap<String, RenderedTemplate>>>,
    port: u16,
) -> String {
    let state = templates.read().unwrap();
    let mut names: Vec<&String> = state.keys().collect();
    names.sort();

    let mut links = String::new();
    for name in &names {
        links.push_str(&format!(
            "        <li><a href=\"/{}\">{}</a></li>\n",
            name, name
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Inky Dev Server</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; max-width: 600px; margin: 40px auto; padding: 0 20px; color: #333; }}
        h1 {{ font-size: 1.5em; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 8px 0; border-bottom: 1px solid #eee; }}
        a {{ color: #0066cc; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        .info {{ color: #888; font-size: 0.9em; margin-top: 20px; }}
    </style>
</head>
<body>
    <h1>Inky Dev Server</h1>
    <p>{} template(s) found:</p>
    <ul>
{}    </ul>
    <p class="info">Serving on port {}. Templates auto-reload on file changes.</p>
</body>
</html>"#,
        names.len(),
        links,
        port
    )
}

fn inject_reload_script(html: &str) -> String {
    let script = r#"<script>
(function(){
  var v = 0;
  setInterval(function(){
    fetch('/_poll').then(function(r){return r.text()}).then(function(t){
      var nv = parseInt(t);
      if(v && nv !== v) location.reload();
      v = nv;
    }).catch(function(){});
  }, 500);
})();
</script>"#;

    if let Some(pos) = html.to_lowercase().rfind("</body>") {
        let mut result = String::with_capacity(html.len() + script.len() + 1);
        result.push_str(&html[..pos]);
        result.push_str(script);
        result.push('\n');
        result.push_str(&html[pos..]);
        result
    } else {
        // No </body> tag, append at the end
        format!("{}\n{}", html, script)
    }
}
