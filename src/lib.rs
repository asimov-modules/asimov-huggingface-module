// This is free and unencumbered software released into the public domain.

//! ASIMOV module for ingesting Hugging Face Hub repository metadata as RDF/JSON-LD.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::Deserialize;

/// The kind of Hugging Face repository being ingested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoKind {
    Model,
    Dataset,
    Space,
}

impl RepoKind {
    /// The segment used in the Hub API path (`/api/<segment>/<id>`).
    pub fn api_segment(self) -> &'static str {
        match self {
            RepoKind::Model => "models",
            RepoKind::Dataset => "datasets",
            RepoKind::Space => "spaces",
        }
    }

    /// The prefix used in the public URL (`https://huggingface.co/<prefix><id>`).
    pub fn url_prefix(self) -> &'static str {
        match self {
            RepoKind::Model => "",
            RepoKind::Dataset => "datasets/",
            RepoKind::Space => "spaces/",
        }
    }

    /// A short human-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            RepoKind::Model => "model",
            RepoKind::Dataset => "dataset",
            RepoKind::Space => "space",
        }
    }
}

/// A parsed reference to a Hugging Face repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoRef {
    pub kind: RepoKind,
    /// The canonical repo id, e.g. `google-bert/bert-base-uncased` or `squad`.
    pub id: String,
}

/// Path segments that terminate a repo id when parsing a URL.
const RESERVED: &[&str] = &[
    "tree",
    "blob",
    "resolve",
    "commit",
    "commits",
    "raw",
    "discussions",
    "edit",
];

/// Parse a model/dataset/space reference from either a bare id
/// (`org/name`, `name`) or a full `huggingface.co` / `hf.co` URL.
///
/// The kind is inferred from a `datasets/` or `spaces/` prefix, defaulting to
/// [`RepoKind::Model`].
pub fn parse_ref(input: &str) -> Result<RepoRef, String> {
    let mut s = input.trim();

    // Strip a URL scheme and host, if present.
    if let Some(pos) = s.find("://") {
        s = &s[pos + 3..];
        // Drop the host component (`huggingface.co`, `hf.co`, …).
        s = s.split_once('/').map(|x| x.1).unwrap_or("");
    } else if let Some(rest) = s
        .strip_prefix("huggingface.co/")
        .or_else(|| s.strip_prefix("hf.co/"))
    {
        s = rest;
    }

    let s = s.trim_matches('/');

    // Infer the kind from a leading `datasets/` or `spaces/` segment.
    let (kind, rest) = if let Some(rest) = s.strip_prefix("datasets/") {
        (RepoKind::Dataset, rest)
    } else if let Some(rest) = s.strip_prefix("spaces/") {
        (RepoKind::Space, rest)
    } else {
        (RepoKind::Model, s)
    };

    // Collect up to two id segments, stopping at a reserved sub-path.
    let mut segments = Vec::new();
    for seg in rest.split('/') {
        if seg.is_empty() || RESERVED.contains(&seg) || segments.len() == 2 {
            break;
        }
        segments.push(seg);
    }

    if segments.is_empty() {
        return Err(format!("could not parse a repository id from {input:?}"));
    }

    Ok(RepoRef {
        kind,
        id: segments.join("/"),
    })
}

/// Repo-level base IRI: `https://huggingface.co/<prefix><id>`.
pub fn base_uri(r: &RepoRef) -> String {
    format!("https://huggingface.co/{}{}", r.kind.url_prefix(), r.id)
}

/// Metadata for a single repository, as returned by `/api/<segment>/<id>`.
#[derive(Debug, Clone, Deserialize)]
pub struct RepoInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub sha: Option<String>,
    #[serde(default, rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub pipeline_tag: Option<String>,
    #[serde(default)]
    pub library_name: Option<String>,
    #[serde(default)]
    pub downloads: Option<u64>,
    #[serde(default)]
    pub likes: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub siblings: Vec<Sibling>,
    #[serde(default, rename = "cardData")]
    pub card_data: Option<CardData>,
}

impl RepoInfo {
    /// The declared license, taken from the model card or a `license:<id>` tag.
    pub fn license(&self) -> Option<String> {
        self.card_data
            .as_ref()
            .and_then(|c| c.license.as_ref())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                self.tags
                    .iter()
                    .find_map(|t| t.strip_prefix("license:").map(|s| s.to_string()))
            })
    }

    /// The tracked file names for this repository.
    pub fn files(&self) -> Vec<String> {
        self.siblings.iter().map(|s| s.rfilename.clone()).collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sibling {
    pub rfilename: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardData {
    #[serde(default)]
    pub license: Option<serde_json::Value>,
}

/// One commit entry from `/api/<segment>/<id>/commits/<revision>`.
#[derive(Debug, Clone, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub authors: Vec<CommitAuthor>,
    #[serde(default)]
    pub date: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

impl CommitAuthor {
    /// A stable handle for this author (username, else display name).
    pub fn handle(&self) -> String {
        self.user
            .clone()
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// The access token for gated/private repositories, taken from (in order)
/// `ASIMOV_HUGGINGFACE_TOKEN`, `HF_TOKEN`, or `HUGGING_FACE_HUB_TOKEN`.
fn hf_token() -> Option<String> {
    [
        "ASIMOV_HUGGINGFACE_TOKEN",
        "HF_TOKEN",
        "HUGGING_FACE_HUB_TOKEN",
    ]
    .into_iter()
    .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
}

/// Perform an (optionally authenticated) GET against the Hub API.
fn hf_get(url: &str) -> Result<String, String> {
    let mut req = ureq::get(url);
    if let Some(token) = hf_token() {
        req = req.set("Authorization", &format!("Bearer {token}"));
    }
    match req.call() {
        Ok(resp) => resp
            .into_string()
            .map_err(|e| format!("failed to read response body: {e}")),
        Err(ureq::Error::Status(code, resp)) => Err(format!(
            "Hugging Face API returned HTTP {code}: {}",
            resp.into_string().unwrap_or_default()
        )),
        Err(e) => Err(format!("request to {url} failed: {e}")),
    }
}

/// Fetch repository metadata from the Hub API.
pub fn fetch_repo(r: &RepoRef) -> Result<RepoInfo, String> {
    let url = format!(
        "https://huggingface.co/api/{}/{}",
        r.kind.api_segment(),
        r.id
    );
    let body = hf_get(&url)?;
    serde_json::from_str(&body).map_err(|e| format!("failed to parse repo metadata: {e}"))
}

/// Fetch commit history from the Hub API (newest first). `max` caps the count.
pub fn fetch_commits(r: &RepoRef, max: Option<usize>) -> Result<Vec<CommitInfo>, String> {
    let mut url = format!(
        "https://huggingface.co/api/{}/{}/commits/main",
        r.kind.api_segment(),
        r.id
    );
    if let Some(n) = max {
        url.push_str(&format!("?limit={n}"));
    }
    let body = hf_get(&url)?;
    let mut commits: Vec<CommitInfo> =
        serde_json::from_str(&body).map_err(|e| format!("failed to parse commits: {e}"))?;
    if let Some(n) = max {
        commits.truncate(n);
    }
    Ok(commits)
}

/// IRI for a Hub user/author: `https://huggingface.co/<handle>`.
fn person_iri(handle: &str) -> String {
    let slug: String = handle
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("https://huggingface.co/{slug}")
}

// Adopts the `git:` vocabulary from lex-o for the commit terms shared with the
// ASIMOV Git module (Commit, author, parent, authoredDate, message), plus
// `schema:` for the repository/person facets the Hub exposes natively
// (author, license, keywords, dateModified, downloads/likes via `hf:`).
//
// TODO: `Repository`, `inRepository`, `hasCommit`, and the `hf:` counters are
// net-new placeholders; upstream them into a shared namespace once stabilized.
fn context() -> serde_json::Value {
    serde_json::json!({
        "git": "https://repolex.ai/ontology/git-lex/git/",
        "hf": "https://huggingface.co/vocab#",
        "asimov-hf": "https://asimov.systems/vocab/huggingface#",
        "schema": "https://schema.org/",
        "xsd": "http://www.w3.org/2001/XMLSchema#",
        "Repository": "asimov-hf:Repository",
        "Commit": "git:Commit",
        "Person": "schema:Person",
        "name": "schema:name",
        "kind": "hf:type",
        "author": { "@id": "schema:author", "@type": "@id" },
        "license": "schema:license",
        "keywords": "schema:keywords",
        "downloads": "hf:downloads",
        "likes": "hf:likes",
        "pipelineTag": "hf:pipelineTag",
        "library": "hf:library",
        "files": "hf:file",
        "dateModified": { "@id": "schema:dateModified", "@type": "xsd:dateTime" },
        "message": "git:message",
        "authoredBy": { "@id": "git:author", "@type": "@id" },
        "parent": { "@id": "git:parent", "@type": "@id" },
        "authorDate": { "@id": "git:authoredDate", "@type": "xsd:dateTime" },
        "inRepository": { "@id": "asimov-hf:inRepository", "@type": "@id" },
        "commits": { "@id": "asimov-hf:hasCommit", "@type": "@id" }
    })
}

/// Serialize a repository and its commit history as a JSON-LD document.
///
/// One `Repository` node carries the Hub metadata; one `Person` node is emitted
/// per distinct author handle; one `Commit` node is emitted per commit, chained
/// linearly via `parent` (newest first, as returned by the Hub).
pub fn to_jsonld(r: &RepoRef, repo: &RepoInfo, commits: &[CommitInfo]) -> Result<String, String> {
    let base = base_uri(r);

    let mut graph: Vec<serde_json::Value> = Vec::new();

    // Repository node.
    let mut repo_node = serde_json::json!({
        "@id": base,
        "@type": "Repository",
        "name": if repo.id.is_empty() { r.id.clone() } else { repo.id.clone() },
        "kind": r.kind.as_str(),
        "commits": commits.iter().map(|c| format!("{base}/commit/{}", c.id)).collect::<Vec<_>>()
    });
    if let Some(author) = &repo.author {
        repo_node["author"] = serde_json::json!(person_iri(author));
    }
    if let Some(v) = repo.downloads {
        repo_node["downloads"] = serde_json::json!(v);
    }
    if let Some(v) = repo.likes {
        repo_node["likes"] = serde_json::json!(v);
    }
    if let Some(v) = &repo.pipeline_tag {
        repo_node["pipelineTag"] = serde_json::json!(v);
    }
    if let Some(v) = &repo.library_name {
        repo_node["library"] = serde_json::json!(v);
    }
    if let Some(v) = repo.license() {
        repo_node["license"] = serde_json::json!(v);
    }
    if let Some(v) = &repo.last_modified {
        repo_node["dateModified"] = serde_json::json!(v);
    }
    if !repo.tags.is_empty() {
        repo_node["keywords"] = serde_json::json!(repo.tags);
    }
    let files = repo.files();
    if !files.is_empty() {
        repo_node["files"] = serde_json::json!(files);
    }
    graph.push(repo_node);

    // Person nodes, keyed on handle (repo author + all commit authors).
    let mut people: BTreeMap<String, ()> = BTreeMap::new();
    if let Some(author) = &repo.author {
        people.insert(author.clone(), ());
    }
    for c in commits {
        for a in &c.authors {
            people.insert(a.handle(), ());
        }
    }
    for handle in people.keys() {
        graph.push(serde_json::json!({
            "@id": person_iri(handle),
            "@type": "Person",
            "name": handle
        }));
    }

    // Commit nodes, chained linearly by parent.
    for (i, c) in commits.iter().enumerate() {
        let authored_by: Vec<String> = c.authors.iter().map(|a| person_iri(&a.handle())).collect();
        let mut node = serde_json::json!({
            "@id": format!("{base}/commit/{}", c.id),
            "@type": "Commit",
            "message": c.title,
            "authorDate": c.date,
            "authoredBy": authored_by,
            "inRepository": base
        });
        if let Some(next) = commits.get(i + 1) {
            node["parent"] = serde_json::json!([format!("{base}/commit/{}", next.id)]);
        }
        graph.push(node);
    }

    let doc = serde_json::json!({
        "@context": context(),
        "@graph": graph
    });
    serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bare_model_id() {
        let r = parse_ref("bert-base-uncased").unwrap();
        assert_eq!(r.kind, RepoKind::Model);
        assert_eq!(r.id, "bert-base-uncased");
    }

    #[test]
    fn parse_namespaced_model_id() {
        let r = parse_ref("google-bert/bert-base-uncased").unwrap();
        assert_eq!(r.kind, RepoKind::Model);
        assert_eq!(r.id, "google-bert/bert-base-uncased");
    }

    #[test]
    fn parse_model_url_with_subpath() {
        let r = parse_ref("https://huggingface.co/meta-llama/Llama-3-8B/tree/main").unwrap();
        assert_eq!(r.kind, RepoKind::Model);
        assert_eq!(r.id, "meta-llama/Llama-3-8B");
    }

    #[test]
    fn parse_dataset_url() {
        let r = parse_ref("https://huggingface.co/datasets/rajpurkar/squad").unwrap();
        assert_eq!(r.kind, RepoKind::Dataset);
        assert_eq!(r.id, "rajpurkar/squad");
        assert_eq!(
            base_uri(&r),
            "https://huggingface.co/datasets/rajpurkar/squad"
        );
    }

    #[test]
    fn parse_space_shorthand() {
        let r = parse_ref("spaces/foo/bar").unwrap();
        assert_eq!(r.kind, RepoKind::Space);
        assert_eq!(r.id, "foo/bar");
    }

    #[test]
    fn empty_input_is_an_error() {
        assert!(parse_ref("https://huggingface.co/").is_err());
    }

    #[test]
    fn license_falls_back_to_tag() {
        let repo = RepoInfo {
            id: "x".into(),
            author: None,
            sha: None,
            last_modified: None,
            pipeline_tag: None,
            library_name: None,
            downloads: None,
            likes: None,
            tags: vec!["license:apache-2.0".into()],
            siblings: vec![],
            card_data: None,
        };
        assert_eq!(repo.license().as_deref(), Some("apache-2.0"));
    }
}
