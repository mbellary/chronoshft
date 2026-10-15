//! Offline local-link checking using HTML tokenization and hosted directory URLs.
use crate::{registry, runtime::safe_path};
use html5ever::{
    buffer_queue::BufferQueue,
    tendril::StrTendril,
    tokenizer::{states::RawKind, StartTag, Token, TokenSink, TokenSinkResult, Tokenizer},
};
use percent_encoding::percent_decode_str;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Clone, Default)]
struct Page {
    ids: BTreeSet<String>,
    links: Vec<String>,
}
#[derive(Default)]
struct Sink(RefCell<Page>);
impl TokenSink for Sink {
    type Handle = ();
    fn process_token(&self, token: Token, _line: u64) -> TokenSinkResult<()> {
        if let Token::TagToken(tag) = token {
            if tag.kind == StartTag {
                let mut p = self.0.borrow_mut();
                for attr in tag.attrs {
                    if &*attr.name.local == "id" {
                        p.ids.insert(attr.value.to_string());
                    }
                    if &*tag.name == "a" && &*attr.name.local == "href" {
                        p.links.push(attr.value.to_string());
                    }
                }
                // HTMLParser treats script/style bodies as text, not markup. Other raw-text
                // elements use HTML5 tokenization as browsers do.
                let raw = match &*tag.name {
                    "script" => Some(RawKind::ScriptData),
                    "style" | "xmp" | "iframe" | "noembed" | "noframes" => Some(RawKind::Rawtext),
                    "title" | "textarea" => Some(RawKind::Rcdata),
                    _ => None,
                };
                if let Some(kind) = raw {
                    return TokenSinkResult::RawData(kind);
                }
            }
        }
        TokenSinkResult::Continue
    }
}
fn parse(path: &Path) -> Result<Page, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tokens = Tokenizer::new(Sink::default(), Default::default());
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(text));
    let _ = tokens.feed(&input);
    tokens.end();
    Ok(tokens.sink.0.into_inner())
}
fn parsed(
    cache: &mut BTreeMap<PathBuf, Result<Page, String>>,
    path: &Path,
) -> Result<Page, String> {
    cache
        .entry(path.into())
        .or_insert_with(|| parse(path))
        .clone()
}
fn walk(root: &Path, dir: &Path, pages: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            walk(root, &path, pages)?;
        } else if path.extension().is_some_and(|e| e == "html") {
            // Do not follow directory symlinks (cycles); fail on escaping HTML symlinks.
            safe_path(root, &path)?;
            pages.push(path);
        }
    }
    Ok(())
}
#[derive(Default, Debug)]
pub struct Report {
    pub errors: Vec<String>,
    pub checked: usize,
    pub pages: usize,
}
fn decode(text: &str) -> String {
    percent_decode_str(text).decode_utf8_lossy().into_owned()
}
// urllib.urljoin removes literal dot segments before percent decoding. Preserve
// percent escapes through URL joining so filesystem confinement is checked later.
// Escape every percent once to avoid collisions with already escaped filenames.
fn protect_path(href: &str) -> String {
    let end = href.find(['?', '#']).unwrap_or(href.len());
    format!("{}{}", href[..end].replace('%', "%25"), &href[end..])
}
fn restore_path(path: &str) -> String {
    path.replace("%25", "%")
}
/// Check links without fetching external resources or modifying the build.
pub fn check(site_dir: &Path, site_url: Option<&str>) -> Result<Report, String> {
    let root = site_dir.canonicalize().map_err(|e| e.to_string())?;
    let mut pages = Vec::new();
    walk(&root, &root, &mut pages)?;
    pages.sort();
    let mut result = Report {
        pages: pages.len(),
        ..Default::default()
    };
    if pages.is_empty() {
        result
            .errors
            .push("No rendered HTML pages found; run the strict MkDocs build first.".into());
        return Ok(result);
    }
    let base = Url::parse(&format!(
        "{}/",
        site_url
            .filter(|s| !s.is_empty())
            .unwrap_or("https://documentation.invalid/")
            .trim_end_matches('/')
    ))
    .map_err(|e| format!("Invalid site_url: {e}"))?;
    if !matches!(base.scheme(), "http" | "https") || base.host_str().is_none() {
        return Err("site_url must be an absolute HTTP(S) URL".into());
    }
    let prefix = base.path();
    let mut cache = BTreeMap::new();
    for page in pages {
        let relative = page
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let page_path = relative.strip_suffix("index.html").unwrap_or(&relative);
        let page_url = base.join(page_path).map_err(|e| e.to_string())?;
        let source = match parsed(&mut cache, &page) {
            Ok(p) => p,
            Err(e) => {
                result
                    .errors
                    .push(format!("{relative}: cannot read HTML: {e}"));
                continue;
            }
        };
        for href in source.links {
            let resolved = match page_url.join(&protect_path(&href)) {
                Ok(url) => url,
                Err(e) => {
                    result
                        .errors
                        .push(format!("{relative}: invalid link {href}: {e}"));
                    continue;
                }
            };
            if !matches!(resolved.scheme(), "http" | "https")
                || resolved[url::Position::BeforeHost..url::Position::AfterPort]
                    != base[url::Position::BeforeHost..url::Position::AfterPort]
            {
                continue;
            }
            let path = restore_path(resolved.path());
            let Some(local) = path.strip_prefix(prefix) else {
                if Url::parse(&href).is_err() && !href.starts_with('/') {
                    result.errors.push(format!(
                        "{relative}: relative link escapes the site prefix: {href}"
                    ));
                }
                continue;
            };
            let mut target = match safe_path(&root, decode(local)) {
                Ok(p) => p,
                Err(_) => {
                    result.errors.push(format!(
                        "{relative}: link escapes the rendered site: {href}"
                    ));
                    continue;
                }
            };
            if path.ends_with('/') {
                target = match safe_path(&root, target.join("index.html")) {
                    Ok(p) => p,
                    Err(_) => {
                        result.errors.push(format!(
                            "{relative}: link escapes the rendered site: {href}"
                        ));
                        continue;
                    }
                };
            }
            result.checked += 1;
            if !target.is_file() {
                result
                    .errors
                    .push(format!("{relative}: missing local file: {href}"));
            } else if let Some(fragment) = resolved.fragment().filter(|f| !f.is_empty()) {
                if target
                    .extension()
                    .is_some_and(|s| s.eq_ignore_ascii_case("html"))
                {
                    match parsed(&mut cache, &target) {
                        Ok(p) if !p.ids.contains(&decode(fragment)) => result
                            .errors
                            .push(format!("{relative}: missing local anchor: {href}")),
                        Err(e) => result
                            .errors
                            .push(format!("{relative}: cannot read link target {href}: {e}")),
                        _ => (),
                    }
                }
            }
        }
    }
    Ok(result)
}
/// Read the project's MkDocs build location and hosted prefix. No Python process
/// is needed. Unsupported inheritance/custom tags fail instead of guessing.
pub fn configured(root: &Path) -> Result<Report, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let text = fs::read_to_string(safe_path(&root, "mkdocs.yml")?).map_err(|e| e.to_string())?;
    let config = registry::yaml::parse_mkdocs(&text)?;
    if !config.is_object() {
        return Err("mkdocs.yml must be a mapping".into());
    }
    if config.get("INHERIT").is_some() {
        return Err("MkDocs INHERIT is not supported by the native link checker; use an explicit site_dir/site_url configuration".into());
    }
    let directory = match config.get("site_dir") {
        None => "site",
        Some(v) => v
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("MkDocs site_dir must be a nonempty string")?,
    };
    let site = match config.get("site_url") {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => Some(
            v.as_str()
                .ok_or("MkDocs site_url must be a string or null")?,
        ),
    };
    // site_dir is an explicit operator-selected build directory and may be outside
    // the repository, as allowed by MkDocs. Links remain confined to that directory.
    let directory = root.join(directory);
    check(&directory, site)
}
