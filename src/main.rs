use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use tokei::{Config, Languages};
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    extract::{Path, State},
    response::Html,
};
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tokio::sync::Mutex;

// #[derive(Copy, Clone, Default)]
// struct Stats {
//     code: usize,
//     comments: usize,
//     blanks: usize,
// }

// #[derive(Clone)]
// struct StatsNode {
//     stats: BTreeMap<String, Stats>,
//     path: Vec<String>,
//     children: BTreeMap<String, Stats>,
// }

// struct StatsEntry {
//     language: String,
//     path: String,
//     code: usize,
//     comments: usize,
//     blanks: usize,
// }

// impl StatsNode {
//     pub fn new(path: &[String]) -> Self {
//         Self {
//             stats: BTreeMap::new(),
//             path: path.to_vec(),
//             children: BTreeMap::new(),
//         }
//     }
// }

// fn add_to_node(node: &mut StatsNode, stats: Stats, language: &str, path: &[String]) {
//     let stats = node.stats.entry(language.to_string()).or_insert(Stats::default());
//     stats.code += stats.code;
//     stats.comments += stats.comments;
//     stats.blanks += stats.blanks;

//     if path.len() > 0 {
//         node.path = Vec::new();
//         node.path.push(path[0].clone());
//         // node.children.insert(path[0].clone(), add_to_node(
//     }
// }


// ============================================================

use axum::{body::Body, http::{Request}};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use std::pin::Pin;
use std::future::Future;

#[derive(Clone)]
struct LogMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for LogMiddleware<S>
where
    S: Service<Request<Body>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();
        let fut = self.inner.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let status = res.status();
            let duration = start.elapsed();
            println!(
                "{} {} {} - {}ms",
                method,
                uri,
                status.as_u16(),
                duration.as_millis()
            );
            Ok(res)
        })
    }
}

#[derive(Clone)]
struct LogLayer;

impl<S> Layer<S> for LogLayer {
    type Service = LogMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogMiddleware { inner }
    }
}

// ============================================================

#[derive(Clone, Default)]
struct Stats {
    code: usize,
    comments: usize,
    blanks: usize,
}

#[derive(Clone, Default)]
struct Node {
    name: String,
    stats: BTreeMap<String, Stats>,
    children: BTreeMap<String, Node>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stats: BTreeMap::new(),
            children: BTreeMap::new(),
        }
    }
}

// #[derive(Clone)]
struct AppState {
    tree: Node,
    filters: BTreeMap<String, bool>,
    expanded: HashSet<String>,
}

fn add_to_node(node: &mut Node, lang: String, path: &[String], stats: Stats) {
    let s = node.stats.entry(lang.clone()).or_insert(Stats::default());
    s.code += stats.code;
    s.comments += stats.comments;
    s.blanks += stats.blanks;

    if path.len() > 0 {
        let child = node.children.entry(path[0].clone()).or_insert(Node::new(&path[0]));
        add_to_node(child, lang, &path[1..], stats);
    }
}

fn html_build_filters(root_node: &Node) -> String {
    let mut html = String::from(r#"<div id="filters">"#);

    for (lang, _) in &root_node.stats {
        // let checked = filters[lang];
        html.push_str(
            &format!(r#"<div class="checkbox-wrapper">
<input type="checkbox" id="chk-{lang}" data-form-type=other>
<label for="chk-{lang}">{lang}</label>
</div>"#
            )
        ); // TODO replace event listener with hx-thing
    }
    html.push_str("</div>");
    html
}

#[tokio::main]
async fn main() {

    // let mut tree = StatsNode::new(&Vec::new());
    let mut entries = Vec::new();

    let t1 = Instant::now();
    let paths = &["orca"];
    let excluded = &[];
    let config = Config::default();
    let mut languages = Languages::new();
    let t2 = Instant::now();
    languages.get_statistics(paths, excluded, &config);
    let t3 = Instant::now();

    let mut tree = Node {
        name: String::new(),
        stats: BTreeMap::new(),
        children: BTreeMap::new(),
    };

    let mut filters = BTreeMap::new();

    for (language, language_stats) in &languages {

        filters.insert(language.name().to_string(), true);

        for report in &language_stats.reports {

            let stats = Stats {
                code: report.stats.code,
                comments: report.stats.comments,
                blanks: report.stats.blanks,
            };
            add_to_node(&mut tree, language.name().to_string(), &report.name.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(), stats);

            entries.push(json::array!{
                language.name().to_string(),
                report.name.display().to_string(),
                report.stats.code,
                report.stats.comments,
                report.stats.blanks,
            });
            let pathvec: Vec<String> = report.name.iter().collect::<Vec<_>>().into_iter().map(|p| p.display().to_string()).collect();

            let stats = Stats {
                code: report.stats.code,
                comments: report.stats.comments,
                blanks: report.stats.blanks,
            };
        }
    }

    let t4 = Instant::now();

    let json_str = json::stringify(entries);
    let mut file = File::create("list.json").expect("Failed to create list.json");
    file.write_all(json_str.as_bytes()).expect("Failed to write to list.json");

    let data_js_str = format!("var data = {}", json_str);
    file = File::create("data.js").expect("Failed to create data.js");
    file.write_all(data_js_str.as_bytes()).expect("Failed to write to data.js");

    let t5 = Instant::now();

    println!("config: {:?}", t2 - t1);
    println!("tokei: {:?}", t3 - t2);
    println!("tree: {:?}", t4 - t3);
    println!("write: {:?}", t5 - t4);

    // Router ============================================================

    let app_state = AppState {
        tree,
        filters,
        expanded: HashSet::new(),
    };
    let state = Arc::new(Mutex::new(app_state));

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(get_root))
        .route("/tree", get(get_tree))
        .route("/path/{path}", get(get_path))
        .route("/filters/{language}", post(toggle_filter))
        .route("/expand/{path}", post(expand_path))
        .route("/collapse/{path}", post(collapse_path))
        .with_state(state)
        .layer(LogLayer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

fn collect_stats(n: &Node, filters: &BTreeMap<String, bool>) -> Stats {
    let mut stats = Stats::default();
    for (lang, lang_stats) in &n.stats {
        if *filters.get(lang).unwrap_or(&false) {
            stats.code += lang_stats.code;
            stats.comments += lang_stats.comments;
            stats.blanks += lang_stats.blanks;
        }
    }
    stats
}

fn get_html_for_node(path: &str, node: &Node, filters: &BTreeMap<String, bool>, expanded: &HashSet<String>) -> String {
    let start = Instant::now();
    let mut node_stats = collect_stats(node, filters);

    let node_summary = format!("<b>{}</b>: {} code, {} comments, {} blanks", node.name, node_stats.code, node_stats.comments, node_stats.blanks);

    if node.children.len() == 0 {
        return format!("<p>{}</p>\n", node_summary);
    } else {
        let mut s = String::new();
        // s.push_str(&format!(r#"<details open="true" hx-post="/collapse/{path}"><summary>{}</summary>{}"#, node_summary, "\n"));
        s.push_str(&format!(r#"<details open="true"><summary>{}</summary>{}"#, node_summary, "\n"));

        let mut sorted_children: Vec<Node> = node.children.values().cloned().collect();
        sorted_children.sort_by(|a, b| {
            let astats = collect_stats(a, filters);
            let bstats = collect_stats(b, filters);
            bstats.code.cmp(&astats.code)
        });

        for child in &sorted_children {
            let new_path = if path == "" {
                &child.name
            } else {
                &format!("{}/{}", path, child.name)
            };
            let new_path_escaped = new_path.replace("/", "%2F");
            let new_id = format!("details-{}", new_path).replace("/", "____");

            let mut stats = collect_stats(child, filters);

            let summary = format!("<b>{}</b>: {} code, {} comments, {} blanks", child.name, stats.code, stats.comments, stats.blanks);

            if child.children.len() == 0 {
                s.push_str(&format!(r###"<p>{summary}</p>{}"###, "\n"));
            } else {
                if expanded.contains(new_path) {
                    s.push_str(&get_html_for_node(new_path, child, filters, expanded));
                    // s.push_str(&format!(r###"<details open="true" hx-post="/collapse/{new_path_escaped}" hx-trigger="toggle"><summary>{summary}</summary><span id="{new_id}"></span></details>{}"###, "\n"));
                } else {
                    s.push_str(&format!(r###"<details id="{new_id}" hx-get="/path/{new_path_escaped}" hx-trigger="toggle" hx-swap="outerHTML"><summary>{summary}</summary></details>{}"###, "\n"));
                }
            }

        }
        s.push_str("</details>");
        // s.push_str(r###"
// <script>
  // document.querySelectorAll('details').forEach(el => {
    // el.addEventListener('toggle', function(event) {
      // event.stopPropagation();
    // });
  // });
// </script>
        //   "###);
        let end = Instant::now();
        println!("get_node {:?}", end - start);
        s
    }
}

fn get_html_for_filters(filters: &BTreeMap<String, bool>) -> String {
    let mut s = String::new();
    s.push_str(r#"<div id="filters">"#);
    s.push_str(&format!(r###"<div class="checkbox-wrapper"><input type="checkbox" id="chk-all" checked data-form-type="other" hx-post="/filters/all" hx-target="#tree"></input><label for="chk-all">(All)</label></div>{}"###, "\n"));
    for (language, enabled) in filters {
        let id = format!("chk-{language}");
        let checked = if *enabled { "checked" } else { "" };
        s.push_str(&format!(r###"<div class="checkbox-wrapper"><input type="checkbox" id="{id}" {checked} data-form-type="other" hx-post="/filters/{language}" hx-target="#tree"></input><label for="{id}">{language}</label></div>{}"###, "\n"));
    }
    s.push_str("</div>");
    s
}

async fn toggle_filter(State(state): State<Arc<Mutex<AppState>>>, Path(language): Path<String>) -> Html<String> {
    {
        let mut state = state.lock().await;
        {
            if &language == "all" {
                let mut toggle_direction = true;
                for (_, val) in state.filters.iter() {
                    if *val {
                        toggle_direction = false;
                    }
                }
                for (lang, val) in state.filters.iter_mut() {
                    *val = toggle_direction;
                }
            } else {
                if let Some(val) = state.filters.get_mut(&language) {
                    *val = !(*val);
                }
            }
        }

        {
            // let filters = state.filters.lock().await;
            // for (lang, enabled) in state.filters.iter() {
            //     println!("{lang}: {enabled}");
            // }
        }
    }

    get_tree(State(state)).await
}

async fn get_path(State(state): State<Arc<Mutex<AppState>>>, Path(path): Path<PathBuf>) -> Result<Html<String>, String> {
    let mut result = {
        let mut state = state.lock().await;

        let t1 = Instant::now();
        let mut node = &state.tree;
        for p in &path {
            if let Some(n) = &node.children.get(&p.display().to_string()) {
                node = n;
            } else {
                return Err("not found".to_string());
            }
        }
        let t2 = Instant::now();
        println!("prep path {:?}", t2 - t1);

        // let filters = state.filters.lock().await;

        get_html_for_node(&path.display().to_string(), &node, &state.filters, &state.expanded)
    };

    result.push_str(r###"
        <script>
document.querySelectorAll('details').forEach(details => {
        details.addEventListener('toggle', event => {
        const el = event.target
            console.log(el, el.hasAttribute('open'))
            if (el.hasAttribute('open')) {
                if (el.id) {
                    htmx.ajax('POST', `/expand/${el.id}`)
                }
            } else {
                if (el.id) {
                    htmx.ajax('POST', `/collapse/${el.id}`)
                }
            }
          event.stopPropagation();
        })
})
</script>
    "###);

    // println!("about to expand {path:?}");
    // expand_path(State(state), Path(path)).await;

    // let t3 = Instant::now();
    // println!("outside get_node {:?}", t3 - t2);

    Ok(Html(result))
}

async fn expand_path(State(state): State<Arc<Mutex<AppState>>>, Path(path): Path<String>) {
    let mut state = state.lock().await;
    state.expanded.insert(path);

    println!("============= expand");
    for s in &state.expanded {
        println!("expanded {s}");
    }
    println!("=============");
}

async fn collapse_path(State(state): State<Arc<Mutex<AppState>>>, Path(path): Path<String>) {
    let mut state = state.lock().await;
    state.expanded.remove(&path);

    println!("============= collapse");
    for s in &state.expanded {
        println!("expanded {s}");
    }
    println!("=============");
}


async fn get_root() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn get_tree(State(state): State<Arc<Mutex<AppState>>>) -> Html<String> {
    let mut state = state.lock().await;
    let mut node = &state.tree.children.first_key_value().unwrap().1;
    let mut s = String::new();
    // let filters = state.filters.lock().await;
    s.push_str(&get_html_for_filters(&state.filters));
    s.push_str(&get_html_for_node(&node.name, node, &state.filters, &state.expanded));
    Html(s)
}
