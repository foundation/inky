use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use inky_core::Config;

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
    data_source: crate::builder::DataSource,
    host: String,
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
        bulletproof_buttons: build_ctx.bulletproof_buttons,
        ..Config::default()
    };

    // Dev server serves HTML only.
    let builder = crate::builder::Builder::new(config, build_ctx.pipeline_options(), false);

    // Build all templates into memory
    let templates: Arc<RwLock<HashMap<String, RenderedTemplate>>> =
        Arc::new(RwLock::new(HashMap::new()));

    build_all_templates(&input, &builder, &data_source, &templates);

    let addr = format!("{}:{}", host, port);
    let server = tiny_http::Server::http(&addr).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to start server on {}: {}",
            "error:".red().bold(),
            addr,
            e
        );
        std::process::exit(1);
    });

    let display_host = if host == "0.0.0.0" { "localhost" } else { host.as_str() };
    eprintln!(
        "\n  {} http://{}:{}",
        "serving".green().bold(),
        display_host,
        port
    );
    if host == "0.0.0.0" {
        eprintln!(
            "  {} listening on all interfaces — templates are visible to your network",
            "note:".yellow().bold()
        );
    }
    eprintln!("  {} {}", "watching".cyan().bold(), input.display());
    eprintln!("  press {} to stop\n", "Ctrl+C".bold());

    // Spawn file watcher thread
    let watcher_templates = Arc::clone(&templates);
    let watcher_input = input.clone();
    let watcher_data_path = data_path.clone();

    std::thread::spawn(move || {
        run_file_watcher(
            watcher_input,
            builder,
            watcher_data_path,
            data_source,
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
    builder: &crate::builder::Builder,
    data_source: &crate::builder::DataSource,
    templates: &Arc<RwLock<HashMap<String, RenderedTemplate>>>,
) {
    let files = crate::builder::find_template_files(input, None);

    // Templates that no longer exist on disk stop being served; templates
    // that fail to build this round keep serving their last good version.
    let current: HashSet<String> = files.iter().map(|f| template_name(f, input)).collect();

    let mut state = templates.write().unwrap();
    state.retain(|name, _| current.contains(name));

    for file in &files {
        let name = template_name(file, input);
        match builder.build_file(file, input, data_source) {
            Ok(built) => {
                for w in &built.warnings {
                    eprintln!("  {} {}", "warning:".yellow().bold(), w);
                }
                for d in &built.diagnostics {
                    let label = match d.severity {
                        inky_core::validate::Severity::Warning => "warn".yellow().bold(),
                        inky_core::validate::Severity::Error => "error".red().bold(),
                    };
                    eprintln!("  {} {} [{}] {}", label, name, d.rule, d.message);
                }
                eprintln!("  {} {}", "built".green().bold(), name);
                state.insert(name, RenderedTemplate { html: built.html });
            }
            Err(e) => {
                eprintln!("  {} {}: {}", "error:".red().bold(), name, e);
                // keep the previous rendered version, if any
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
    builder: crate::builder::Builder,
    data_path: Option<PathBuf>,
    mut data_source: crate::builder::DataSource,
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

    // Watch the data path (file or directory) for changes
    if let Some(ref data_file) = data_path {
        let watch_target = if data_file.is_dir() {
            data_file.clone()
        } else {
            data_file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| data_file.clone())
        };
        let canonical = std::fs::canonicalize(&watch_target).unwrap_or(watch_target);
        eprintln!("  {} {} (data)", "watching".cyan().bold(), data_file.display());
        debouncer
            .watcher()
            .watch(&canonical, notify::RecursiveMode::Recursive)
            .unwrap_or_else(|e| {
                eprintln!(
                    "  {} Failed to watch data path '{}': {}",
                    "warning:".yellow().bold(),
                    canonical.display(),
                    e
                );
            });
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

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let mut needs_rebuild = false;
                let mut data_changed = false;

                for event in &events {
                    let path = &event.path;

                    // Check if the data path (file or directory) changed
                    if let Some(ref data_file) = data_path {
                        let canonical_data =
                            std::fs::canonicalize(data_file).unwrap_or(data_file.clone());
                        let canonical_event = std::fs::canonicalize(path).unwrap_or(path.clone());
                        let is_data_event = canonical_event == canonical_data
                            || (canonical_data.is_dir() && canonical_event.starts_with(&canonical_data));
                        if is_data_event {
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
                    data_source = match data_path.as_deref() {
                        Some(p) if p.is_dir() => {
                            crate::builder::DataSource::Directory(p.to_path_buf())
                        }
                        Some(p) => match crate::util::load_json_data(Some(p)) {
                            Some(v) => crate::builder::DataSource::File(v),
                            None => crate::builder::DataSource::None,
                        },
                        None => crate::builder::DataSource::None,
                    };
                    needs_rebuild = true;
                }

                if needs_rebuild {
                    eprintln!("  rebuilding templates...");
                    build_all_templates(&input, &builder, &data_source, &templates);
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

    let body_re = regex::Regex::new(r"(?i)</body>").unwrap();
    if let Some(pos) = body_re.find_iter(html).last().map(|m| m.start()) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_reload_script_multibyte_body_offset() {
        let html = "<html><body>İİİ content</body></html>";
        let out = inject_reload_script(html);
        let script_pos = out.find("<script>").unwrap();
        let body_close = out.find("</body>").unwrap();
        assert!(script_pos < body_close);
        assert!(out.contains("İİİ content"));
    }
}
