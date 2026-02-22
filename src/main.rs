use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use pulldown_cmark::{Options, Parser, html};
use std::fs;
use std::time::{Duration, SystemTime};

const CSS: &str = r#"
:root {
    --accent: #2563eb;
    --accent-hover: #1d4ed8;
    --text: #1e293b;
    --muted: #64748b;
    --border: #e2e8f0;
    --bg: #ffffff;
    --bg-alt: #f8fafc;
    --shadow: rgba(0,0,0,0.1);
    --btn-shadow: rgba(37,99,235,0.3);
}

[data-theme="dark"] {
    --accent: #60a5fa;
    --accent-hover: #93bbfd;
    --text: #e2e8f0;
    --muted: #94a3b8;
    --border: #334155;
    --bg: #0f172a;
    --bg-alt: #1e293b;
    --shadow: rgba(0,0,0,0.4);
    --btn-shadow: rgba(96,165,250,0.3);
}

*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    color: var(--text);
    background: var(--bg-alt);
    line-height: 1.6;
    font-size: 16px;
    transition: background 0.3s, color 0.3s;
}

.container {
    max-width: 800px;
    margin: 2rem auto;
    background: var(--bg);
    padding: 3rem;
    border-radius: 8px;
    box-shadow: 0 1px 3px var(--shadow);
    transition: background 0.3s, box-shadow 0.3s;
}

h1 {
    font-size: 2rem;
    color: var(--accent);
    border-bottom: 3px solid var(--accent);
    padding-bottom: 0.5rem;
    margin-bottom: 1.5rem;
}

h2 {
    font-size: 1.35rem;
    color: var(--accent);
    margin-top: 2rem;
    margin-bottom: 0.75rem;
    padding-bottom: 0.3rem;
    border-bottom: 1px solid var(--border);
}

h3 { font-size: 1.1rem; margin-top: 1rem; margin-bottom: 0.5rem; }

p { margin-bottom: 0.75rem; }

a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }

blockquote {
    border-left: 3px solid var(--accent);
    padding: 0.75rem 1rem;
    margin: 0.75rem 0;
    background: var(--bg-alt);
    border-radius: 0 4px 4px 0;
    transition: background 0.3s;
}

blockquote blockquote {
    border-left-color: var(--border);
    background: var(--bg);
    margin: 0.5rem 0;
}

blockquote p { margin-bottom: 0.25rem; }

strong { color: var(--text); }

ul, ol { margin: 0.5rem 0 0.75rem 1.5rem; }
li { margin-bottom: 0.25rem; }

hr { border: none; border-top: 1px solid var(--border); margin: 1.5rem 0; }

code {
    background: var(--bg-alt);
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
    font-size: 0.9em;
}

.fab {
    position: fixed;
    right: 2rem;
    background: var(--accent);
    color: white;
    border: none;
    padding: 0.6rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    box-shadow: 0 2px 8px var(--btn-shadow);
    transition: background 0.2s, transform 0.1s;
}

.fab:hover { background: var(--accent-hover); }
.fab:active { transform: scale(0.96); }

.print-btn { bottom: 2rem; }
.theme-btn { bottom: 5rem; }

@media print {
    body { background: white !important; color: #111 !important; font-size: 11pt; }
    .container { max-width: 100%; margin: 0; padding: 0; box-shadow: none; border-radius: 0; background: white !important; }
    .fab { display: none !important; }
    h1, h2 { color: #111 !important; }
    h1 { border-bottom-color: #111 !important; }
    blockquote { background: none !important; border-left-color: #999 !important; page-break-inside: avoid; }
    a { color: #111 !important; }
    a[href]::after { content: " (" attr(href) ")"; font-size: 0.85em; color: #666; }
    a[href^="mailto:"]::after { content: " (" attr(href) ")"; }
    @page { size: A4; margin: 1.5cm; }
}

@media (max-width: 640px) {
    .container { margin: 0; padding: 1.5rem; border-radius: 0; }
    .fab { right: 1rem; padding: 0.5rem 0.8rem; font-size: 0.8rem; }
    .print-btn { bottom: 1.5rem; }
    .theme-btn { bottom: 4rem; }
}
"#;

const HTML_PREFIX: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Gabriel Garrido - CV</title>
    <script>
        (function() {
            var t = localStorage.getItem('theme');
            if (t === 'dark' || (!t && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
                document.documentElement.setAttribute('data-theme', 'dark');
            }
        })();
    </script>
    <style>"#;

const HTML_MIDDLE: &str = r#"</style>
</head>
<body>
<div class="container">
"#;

const HTML_SUFFIX: &str = r#"</div>
<button class="fab theme-btn" onclick="toggleTheme()" aria-label="Toggle dark mode" id="theme-toggle"></button>
<button class="fab print-btn" onclick="window.print()">Print / Export PDF</button>
<script>
    function getTheme() {
        var t = localStorage.getItem('theme');
        if (t) return t;
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    function applyTheme(t) {
        if (t === 'dark') {
            document.documentElement.setAttribute('data-theme', 'dark');
        } else {
            document.documentElement.removeAttribute('data-theme');
        }
        document.getElementById('theme-toggle').textContent = t === 'dark' ? 'Light Mode' : 'Dark Mode';
    }
    function toggleTheme() {
        var current = getTheme();
        var next = current === 'dark' ? 'light' : 'dark';
        localStorage.setItem('theme', next);
        applyTheme(next);
    }
    applyTheme(getTheme());
</script>
</body>
</html>"#;

#[get("/")]
async fn root(req: HttpRequest) -> impl Responder {
    let con_info = req.connection_info();
    let peer_ip = &req.peer_addr().unwrap().to_string();

    let ip = con_info
        .realip_remote_addr()
        .unwrap()
        .split(':')
        .next()
        .unwrap_or(peer_ip);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));

    println!(
        "[{}][{}][{}]: Processing cv request...",
        now.as_secs(),
        ip,
        peer_ip
    );

    let data = fs::read_to_string("./cv.md").expect("Unable to read file");

    let options = Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(&data, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);

    let page = format!("{HTML_PREFIX}{CSS}{HTML_MIDDLE}{html_content}{HTML_SUFFIX}");

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(root).service(healthz))
        .workers(2)
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
