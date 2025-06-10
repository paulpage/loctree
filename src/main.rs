use tokei::{Config, Languages};

struct Stats {
    name: String,
    code: usize,
    comments: usize,
    blanks: usize,
    children: Vec<Stats>,
}

impl Stats {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            code: 0,
            comments: 0,
            blanks: 0,
            children: Vec::new(),
        }
    }
}

fn main() {
    let paths = &["."];
    let excluded = &[];
    let config = Config::default();
    let mut languages = Languages::new();
    languages.get_statistics(paths, excluded, &config);

    for (language_name, language_stats) in &languages {
        println!("{}", language_name);
        for report in &language_stats.reports {
            let mut stats = Stats::new(&report.name.display().to_string());
            stats.code = report.stats.code;
            stats.comments = report.stats.comments;
            stats.blanks = report.stats.blanks;

            println!("  {}: {} LOC, {} comments, {} blanks", stats.name, stats.code, stats.comments, stats.blanks);
        }
    }
}
