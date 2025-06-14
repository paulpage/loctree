use std::collections::BTreeMap;
use std::path::PathBuf;
use tokei::{Config, Languages};
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use axum::{
    routing::get,
    Router,
    http::StatusCode,
    extract::{Path, State},
    response::Html,
};
use tower_http::services::ServeDir;

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

#[derive(Clone)]
struct AppState {
    html: String,
    tree: Node,
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

fn html_write_node(html: &mut String, node: &Node, level: usize, key: String) {
    for (name, child) in &node.children {
        let mut total_stats = Stats::default();
        for (_lang, stats) in &child.stats {
            if true { // filters
                total_stats.code += stats.code;
                total_stats.comments += stats.comments;
                total_stats.blanks += stats.blanks;
            }
        }
        if total_stats.code + total_stats.comments + total_stats.blanks == 0 {
            continue;
        }

        let msg = format!("<span><b>{}: </b>{} code, {} comments, {} blanks</span>", name, total_stats.code, total_stats.comments, total_stats.blanks);
        if child.children.len() > 0 {
            let child_key = format!("{key}/{name}");
            let is_open = if level == 0 { "open=\"true\"" } else { "" }; 
            html.push_str(&format!(r#"<details id="{child_key}" {is_open}><summary>{msg}</summary>"#));
            html_write_node(html, child, level + 1, child_key);
            html.push_str("</details>");
        } else {
            html.push_str(&format!("<p>{msg}</p>"));

        }
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

    for (language, language_stats) in &languages {
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
            // add_to_node(&mut tree, stats, language.name(), &pathvec);

            // let mut stats_tree = build_tree(&mut stats, &pathvec);

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

    let mut html = String::new();

    html.push_str(&html_build_filters(&tree));
    html_write_node(&mut html, &tree, 0, String::from("node::"));



    println!("config: {:?}", t2 - t1);
    println!("tokei: {:?}", t3 - t2);
    println!("tree: {:?}", t4 - t3);
    println!("write: {:?}", t5 - t4);

    // Router ============================================================

    let state = AppState {
        html,
        tree,
    };

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(get_root))
        .route("/tree", get(get_tree))
        .route("/numbers/{n}", get(number))
        .route("/path/{path}", get(get_path))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

fn collect_stats(n: &Node) -> Stats {
    let mut stats = Stats::default();
    for (lang, lang_stats) in &n.stats {
        stats.code += lang_stats.code;
        stats.comments += lang_stats.comments;
        stats.blanks += lang_stats.blanks;
    }
    stats
}

fn get_html_for_node(path: &str, node: &Node) -> String {
    let mut node_stats = collect_stats(node);

    let node_summary = format!("<b>{}</b>: {} code, {} comments, {} blanks", node.name, node_stats.code, node_stats.comments, node_stats.blanks);

    if node.children.len() == 0 {
        return format!("<p>{}</p>\n", node_summary);
    } else {
        let mut s = String::new();
        s.push_str(&format!(r#"<details open="true"><summary>{}</summary>{}"#, node_summary, "\n"));

        let mut sorted_children: Vec<Node> = node.children.values().cloned().collect();
        sorted_children.sort_by(|a, b| {
            let astats = collect_stats(a);
            let bstats = collect_stats(b);
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

            let mut stats = collect_stats(child);

            let summary = format!("<b>{}</b>: {} code, {} comments, {} blanks", child.name, stats.code, stats.comments, stats.blanks);

            if child.children.len() == 0 {
                s.push_str(&format!(r###"<p>{summary}</p>{}"###, "\n"));
            } else {
                s.push_str(&format!(r###"<details hx-get="/path/{new_path_escaped}" hx-trigger="toggle" hx-swap="outerHTML"><summary>{summary}</summary><span id="{new_id}"></span></details>{}"###, "\n"));
            }

        }
        s.push_str("</details>");
        s
    }
}

async fn get_path(State(state): State<AppState>, Path(path): Path<PathBuf>) -> Result<Html<String>, String> {

    let mut node = &state.tree;
    for p in &path {
        if let Some(n) = &node.children.get(&p.display().to_string()) {
            node = n;
        } else {
            return Err("not found".to_string());
        }
    }

    Ok(Html(get_html_for_node(&path.display().to_string(), &node)))
}

async fn number(Path(n): Path<i32>) -> String {
    n.to_string()
}

async fn get_root() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn get_tree(State(state): State<AppState>) -> Html<String> {
    
    let mut node = &state.tree.children.first_key_value().unwrap().1;
    Html(get_html_for_node(&node.name, node))
}
