use std::collections::BTreeMap;
use std::path::PathBuf;
use tokei::{Config, Languages};

#[derive(Copy, Clone, Default)]
struct Stats {
    code: usize,
    comments: usize,
    blanks: usize,
}

#[derive(Clone)]
struct StatsNode {
    stats: BTreeMap<String, Stats>,
    path: Vec<String>,
    children: BTreeMap<String, Stats>,
}

struct StatsEntry {
    language: String,
    path: String,
    code: usize,
    comments: usize,
    blanks: usize,
}

impl StatsNode {
    pub fn new(path: &[String]) -> Self {
        Self {
            stats: BTreeMap::new(),
            path: path.to_vec(),
            children: BTreeMap::new(),
        }
    }
}

fn add_to_node(node: &mut StatsNode, stats: Stats, language: &str, path: &[String]) {
    let stats = node.stats.entry(language.to_string()).or_insert(Stats::default());
    stats.code += stats.code;
    stats.comments += stats.comments;
    stats.blanks += stats.blanks;

    if path.len() > 0 {
        node.path = Vec::new();
        node.path.push(path[0].clone());
        // node.children.insert(path[0].clone(), add_to_node(
    }
}

fn main() {

    let mut tree = StatsNode::new(&Vec::new());
    let mut entries = Vec::new();

    let paths = &["orca"];
    let excluded = &[];
    let config = Config::default();
    let mut languages = Languages::new();
    languages.get_statistics(paths, excluded, &config);

    for (language, language_stats) in &languages {
        // println!("{}", language);
        for report in &language_stats.reports {

            entries.push(json::object!{
                language: language.name().to_string(),
                path: report.name.display().to_string(),
                code: report.stats.code,
                comments: report.stats.comments,
                blanks: report.stats.blanks,
            });
            let pathvec: Vec<String> = report.name.iter().collect::<Vec<_>>().into_iter().map(|p| p.display().to_string()).collect();

            let stats = Stats {
                code: report.stats.code,
                comments: report.stats.comments,
                blanks: report.stats.blanks,
            };
            add_to_node(&mut tree, stats, language.name(), &pathvec);

            // let mut stats_tree = build_tree(&mut stats, &pathvec);

            // println!("  {} {}: {} LOC, {} comments, {} blanks", stats.language, stats.name, stats.code, stats.comments, stats.blanks);
            // let mut statref = &mut stats;
            // for p in &report.name {
            //     println!("    {}", p.display());
            // }
        }
    }

    println!("var data = {};", json::stringify(entries))
}
