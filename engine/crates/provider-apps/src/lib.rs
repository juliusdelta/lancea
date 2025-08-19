use anyhow::{Context, Result};
use deunicode::deunicode;
use dirs;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ini::Ini;
use lancea_model::{Preview, ResultItem, Provider};
use serde::Serialize;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::{env, fs};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

const PROVIDER_ID: &str = "apps";

#[derive(Debug, Clone, Serialize)]
pub struct AppRecord {
    pub desktop_id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: Option<String>,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub nodisplay: bool,
    pub desktop_path: PathBuf,
    pub search_blob: String,
}

impl AppRecord {
    fn title(&self) -> &str {
        &self.name
    }
    fn subtitle(&self) -> Option<&str> {
        return self.generic_name.as_deref().or(self.comment.as_deref());
    }
}

pub struct AppsProvider {
    apps: Vec<AppRecord>,
}

impl AppsProvider {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut apps = vec![];

        for p in application_dirs() {
            if !p.exists() {
                continue;
            }

            for entry in WalkDir::new(p)
                .min_depth(0)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }

                if let Some(app) = parse_desktop_file(path).ok().flatten() {
                    if app.nodisplay {
                        continue;
                    }
                    apps.push(app);
                }
            }
        }

        apps.sort_by(|a, b| a.desktop_id.cmp(&b.desktop_id));
        apps.dedup_by(|a, b| a.desktop_id == b.desktop_id);

        Ok(Self { apps })
    }

    pub fn search(&self, raw_query: &str) -> Vec<ResultItem> {
        let q = normalize_query(raw_query);
        let q = q
            .strip_prefix("/apps")
            .or_else(|| q.strip_prefix("/ap"))
            .map(|s| s.trim())
            .unwrap_or(&q);

        dbg!("[AppsProvder#search - Search initiated with query: {}", &q);

        if q.is_empty() {
            return Vec::new();
        }
        if q.starts_with('/') {
            return Vec::new();
        }

        let matcher = SkimMatcherV2::default();

        let mut scored: Vec<(f32, ResultItem)> = Vec::new();

        for app in &self.apps {
            let hay = app.search_blob.as_str();
            let mut best: Option<f32> = None;

            if starts_with_token(&app.name, &q)
                || app
                    .generic_name
                    .as_ref()
                    .map_or(false, |g| starts_with_token(g, &q))
                || app
                    .comment
                    .as_ref()
                    .map_or(false, |c| starts_with_token(c, &q))
            {
                dbg!("Best found. Score of 1.0 emitted.");
                best = Some(1.0);
            }

            if best.is_none() {
                if let Some(score) = matcher.fuzzy_match(hay, &q) {
                    let s = (score as f32 / 100.0).clamp(0.1, 0.7);
                    best = Some(s)
                }
            }

            if best.is_none() && hay.contains(&q) {
                best = Some(0.35);
            }

            if let Some(score) = best {
                let item = to_result_item(app, score);
                scored.push((score, item));
            }
        }

        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| natord(&a.1.title, &b.1.title))
        });

        let scored_results: Vec<ResultItem> =
            scored.into_iter().map(|(_, it)| it).take(25).collect();

        dbg!(
            "[AppsProvider#search] Found {} apps that scored.",
            &scored_results.first().unwrap()
        );

        return scored_results;
    }

    pub fn preview(&self, key: &str) -> Option<Preview> {
        let id = key.strip_prefix("apps:").unwrap_or(key);
        self.apps.iter().find(|a| a.desktop_id == id).map(|a| {
            let data = serde_json::json!({
                "iconRef": a.icon,
                "title": a.name,
                "comment": a.comment,
                "categories": a.categories,
                "desktopId": a.desktop_id,
                "path": a.desktop_path,
            });

            Preview {
                preview_kind: "card".into(),
                data,
            }
        })
    }

    pub fn execute_launch(&self, key: &str) -> Result<(), anyhow::Error> {
        let id = key.strip_prefix("apps:").unwrap_or(key);

        if !self.apps.iter().any(|a| a.desktop_id == id) {
            anyhow::bail!("Unknown desktop-id: {id}");
        }

        let status = std::process::Command::new("gtk-launch")
            .arg(id)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context("failed to spawn gtk-lauch")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("gtk-launch exited with status {status}");
        }
    }
}

fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(mut user) = dirs::data_dir() {
        user.push("applications");
        dirs.push(user);
    }

    let system_list = env::var_os("XDG_DATA_DIRS")
        .map(|v| env::split_paths(&v).collect::<Vec<PathBuf>>())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });

    for mut base in system_list {
        base.push("applications");
        dirs.push(base);
    }

    dirs.sort();
    dirs.dedup();
    dirs.retain(|p| p.exists());

    return dirs;
}

fn parse_desktop_file(path: &Path) -> Result<Option<AppRecord>, anyhow::Error> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let txt = String::from_utf8_lossy(&bytes);
    let ini = Ini::load_from_str(&txt).context("ini parse")?;

    let sec = ini
        .section(Some("Desktop Entry"))
        .cloned()
        .unwrap_or_default();
    let ty = get_best_locale(&sec, "Type");
    if ty.as_deref() != Some("Application") {
        return Ok(None);
    }

    let name = get_best_locale(&sec, "Name").unwrap_or_else(|| desktop_id_from_path(path));
    let generic = get_best_locale(&sec, "GenericName");
    let comment = get_best_locale(&sec, "Comment");
    let exec = get_best_locale(&sec, "Exec");
    let icon = get_best_locale(&sec, "Icon");

    let nodisplay = get_best_locale(&sec, "NoDisplay")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    let categories = get_best_locale(&sec, "Categories")
        .map(|s| {
            s.split(';')
                .filter(|t| !t.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_else(Vec::new);

    let keywords = get_best_locale(&sec, "Keywords")
        .map(|s| {
            s.split(';')
                .filter(|t| !t.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_else(Vec::new);

    let desktop_id = desktop_id_from_path(path);

    let mut blob = String::new();
    push_norm(&mut blob, &name);

    if let Some(g) = &generic {
        blob.push(' ');
        push_norm(&mut blob, g);
    }
    if let Some(c) = &comment {
        blob.push(' ');
        push_norm(&mut blob, c);
    }
    for k in &keywords {
        blob.push(' ');
        push_norm(&mut blob, k);
    }
    for c in &categories {
        blob.push(' ');
        push_norm(&mut blob, c);
    }

    Ok(Some(AppRecord {
        desktop_id,
        name: name.to_string(),
        generic_name: generic,
        comment,
        exec,
        icon,
        categories,
        keywords,
        nodisplay,
        desktop_path: path.to_path_buf(),
        search_blob: blob,
    }))
}

fn push_norm(blob: &mut String, s: &str) {
    let normed = norm(s);
    if !normed.is_empty() {
        blob.push_str(&normed);
        blob.push(' ');
    }
}

fn desktop_id_from_path(path: &Path) -> String {
    let stem = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();

    return stem.strip_suffix(".desktop").unwrap_or(stem).to_string();
}

fn get_best_locale(sec: &ini::Properties, key: &str) -> Option<String> {
    let lang = std::env::var("LANG").ok().unwrap_or_default();
    let lang_main = lang.split('.').next().unwrap_or("").to_string(); // en_US
    let (l1, l2) = if !lang_main.is_empty() {
        let mut parts = lang_main.split('_');
        let a = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or("");
        (
            Some(lang_main.clone()),
            if b.is_empty() {
                Some(a.to_string())
            } else {
                Some(a.to_string())
            },
        )
    } else {
        (None, None)
    };

    // candidates in order
    let mut keys = vec![key.to_string()];
    if let Some(k) = &l1 {
        keys.insert(0, format!("{key}[{k}]"));
    }
    if let Some(a) = &l2 {
        keys.insert(1, format!("{key}[{}]", a));
    }

    for k in keys {
        if let Some(v) = sec.get(&k) {
            return Some(v.to_string());
        }
    }
    None
}

fn normalize_query<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref().trim();
    if s.is_empty() {
        return String::new();
    }
    return norm(s);
}

fn norm(s: &str) -> String {
    let folded: Cow<'_, str> = Cow::Owned(
        deunicode(&s.nfkd().collect::<String>())
            .to_lowercase()
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect::<String>(),
    );

    let mut out = String::with_capacity(folded.len());
    let mut prev_space = false;
    for ch in folded.chars() {
        let space = ch.is_whitespace();
        if space && prev_space {
            continue;
        }
        out.push(if space { ' ' } else { ch });
        prev_space = space
    }

    return out.trim().to_string();
}

fn starts_with_token(hay: &str, q: &str) -> bool {
    return norm(hay).starts_with(&norm(q));
}

fn natord(a: &str, b: &str) -> std::cmp::Ordering {
    let la = a.len().min(64);
    let lb = b.len().min(64);

    return a[..la].cmp(&b[..lb]);
}

fn to_result_item(app: &AppRecord, score: f32) -> ResultItem {
    // let subtitle = app.subtitle().unwrap_or_default();
    let extras = serde_json::json!({
        "desktopId": app.desktop_id,
        "iconRef": app.icon,
        "exec": app.exec,
    });

    ResultItem {
        key: format!("apps:{}", app.desktop_id),
        title: app.title().to_string(),
        provider_id: PROVIDER_ID.into(),
        score,
        extras: Some(extras),
    }
}

impl Provider for AppsProvider {
    fn id(&self) -> &str {
        PROVIDER_ID
    }

    fn search(&self, query: &str) -> Vec<ResultItem> {
        self.search(query)
    }

    fn preview(&self, key: &str) -> Option<Preview> {
        self.preview(key)
    }

    fn execute(&self, action: &str, key: &str) -> bool {
        match action {
            "launch" => self.execute_launch(key).is_ok(),
            _ => false,
        }
    }
}
