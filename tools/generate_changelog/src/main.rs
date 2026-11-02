//! Generates structured release notes from git commits between two tags/commits,
//! categorized similarly to Immich releases (Features, Bug Fixes, Docs, Maintenance).

use std::env;
use std::fs;
use std::process::Command;

fn get_git_output(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut current_tag = String::new();
    let mut prev_tag = String::new();
    let mut repo = "rbm13s/QuadFurn".to_string();
    let mut output_path = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tag" => {
                if i + 1 < args.len() {
                    current_tag = args[i + 1].clone();
                    i += 1;
                }
            }
            "--prev-tag" => {
                if i + 1 < args.len() {
                    prev_tag = args[i + 1].clone();
                    i += 1;
                }
            }
            "--repo" => {
                if i + 1 < args.len() {
                    repo = args[i + 1].clone();
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if current_tag.is_empty() {
        current_tag = get_git_output(&["describe", "--tags", "--exact-match"]);
    }
    if current_tag.is_empty() {
        current_tag = get_git_output(&["describe", "--tags", "--abbrev=0"]);
    }
    if current_tag.is_empty() {
        current_tag = "HEAD".to_string();
    }

    if prev_tag.is_empty() {
        let parent = format!("{}^", current_tag);
        prev_tag = get_git_output(&["describe", "--tags", "--abbrev=0", &parent]);
    }

    let (range_spec, compare_url) = if !prev_tag.is_empty() {
        (
            format!("{}..{}", prev_tag, current_tag),
            format!("https://github.com/{}/compare/{}...{}", repo, prev_tag, current_tag),
        )
    } else {
        (
            current_tag.clone(),
            format!("https://github.com/{}/commits/{}", repo, current_tag),
        )
    };

    let log_raw = get_git_output(&["log", &range_spec, "--no-merges", "--pretty=format:%h|%s"]);

    let mut cat_feats = Vec::new();
    let mut cat_fixes = Vec::new();
    let mut cat_docs = Vec::new();
    let mut cat_others = Vec::new();

    for line in log_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() < 2 {
            continue;
        }
        let hash = parts[0].trim();
        let subject = parts[1].trim();

        let link = format!("[{hash}](https://github.com/{repo}/commit/{hash})");
        let item = format!("* {subject} in {link}");

        let s_lower = subject.to_lowercase();
        if s_lower.starts_with("feat") {
            cat_feats.push(item);
        } else if s_lower.starts_with("fix") {
            cat_fixes.push(item);
        } else if s_lower.starts_with("doc") || s_lower.contains("readme") || s_lower.contains("screenshot") {
            cat_docs.push(item);
        } else {
            cat_others.push(item);
        }
    }

    let mut out = Vec::new();
    out.push("## What's Changed\n".to_string());

    if !cat_feats.is_empty() {
        out.push("### 🚀 Features & Improvements".to_string());
        out.extend(cat_feats);
        out.push(String::new());
    }

    if !cat_fixes.is_empty() {
        out.push("### 🐛 Bug Fixes".to_string());
        out.extend(cat_fixes);
        out.push(String::new());
    }

    if !cat_docs.is_empty() {
        out.push("### 📝 Documentation & Localization".to_string());
        out.extend(cat_docs);
        out.push(String::new());
    }

    if !cat_others.is_empty() {
        out.push("### 🧰 Maintenance & Other Changes".to_string());
        out.extend(cat_others);
        out.push(String::new());
    }

    out.push(format!("**Full Changelog**: {compare_url}\n"));

    let markdown = out.join("\n");
    if !output_path.is_empty() {
        fs::write(&output_path, &markdown).expect("Failed to write changelog output file");
        println!("Changelog successfully written to {}", output_path);
    } else {
        println!("{}", markdown);
    }
}
