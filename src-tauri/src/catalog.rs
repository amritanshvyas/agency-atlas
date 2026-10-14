use serde::Serialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub category_label: String,
    pub source_path: String,
    pub color: Option<String>,
    pub emoji: Option<String>,
    pub vibe: Option<String>,
    pub metadata_warning: Option<String>,
    pub mapping: Mapping,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    pub slug: Option<String>,
    pub conflicting_sources: Vec<String>,
    pub generated: FileStatus,
    pub installed: FileStatus,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Present,
    Missing,
    Ambiguous,
    #[default]
    NotAvailable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub agents: Vec<Agent>,
    pub source_root: String,
    pub installed_directory: Option<String>,
    pub generated_directory: String,
    pub warnings: Vec<String>,
}

fn source_id(root: &Path, path: &Path) -> io::Result<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| io::Error::other("source is outside repository"))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn slugify(name: &str) -> String {
    let mut result = String::new();
    let mut separator = false;
    for byte in name.bytes() {
        let byte = byte.to_ascii_lowercase();
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            if separator && !result.is_empty() {
                result.push('-');
            }
            result.push(char::from(byte));
            separator = false;
        } else {
            separator = true;
        }
    }
    result
}

fn string_field(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    match map.get(Value::String(key.to_owned())) {
        Some(Value::String(text)) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        _ => None,
    }
}

// None means there is no opening frontmatter fence, so a Markdown document is not an agent.
fn parse_metadata(source: &str) -> Option<(serde_yaml::Mapping, Option<String>)> {
    let mut lines = source.lines();
    if lines.next()?.trim_end_matches('\r') != "---" {
        return None;
    }
    let mut yaml = String::new();
    let mut closed = false;
    for line in lines {
        if line.trim_end_matches('\r') == "---" {
            closed = true;
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    if !closed {
        return Some((
            serde_yaml::Mapping::new(),
            Some("Unclosed frontmatter".into()),
        ));
    }
    match serde_yaml::from_str::<Value>(&yaml) {
        Ok(Value::Mapping(map)) => Some((map, None)),
        Ok(_) => Some((
            serde_yaml::Mapping::new(),
            Some("Frontmatter must be a YAML object".into()),
        )),
        Err(error) => Some((
            serde_yaml::Mapping::new(),
            Some(format!("Invalid YAML: {error}")),
        )),
    }
}

fn parse_agent(root: &Path, path: &Path, category: &str, label: &str) -> io::Result<Option<Agent>> {
    let source = fs::read_to_string(path)?;
    let Some((fields, warning)) = parse_metadata(&source) else {
        return Ok(None);
    };
    let id = source_id(root, path)?;
    let name = string_field(&fields, "name");
    let description = string_field(&fields, "description");
    let mut issues = Vec::new();
    if let Some(warning) = warning {
        issues.push(warning);
    }
    if name.is_none() {
        issues.push("Missing name".to_owned());
    }
    if description.is_none() {
        issues.push("Missing description".to_owned());
    }
    let fallback = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .replace('-', " ");
    Ok(Some(Agent {
        id: id.clone(),
        name: name.unwrap_or(fallback),
        description: description.unwrap_or_default(),
        category: category.to_owned(),
        category_label: label.to_owned(),
        source_path: id,
        color: string_field(&fields, "color"),
        emoji: string_field(&fields, "emoji"),
        vibe: string_field(&fields, "vibe"),
        metadata_warning: if issues.is_empty() {
            None
        } else {
            Some(issues.join("; "))
        },
        mapping: Mapping::default(),
    }))
}

fn collect_markdown(directory: &Path, files: &mut Vec<PathBuf>, warnings: &mut Vec<String>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!("Cannot read {}: {error}", directory.display()));
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(value) => value,
            Err(error) => {
                warnings.push(error.to_string());
                continue;
            }
        };
        let path = entry.path();
        let kind = match entry.file_type() {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        if kind.is_dir() {
            collect_markdown(&path, files, warnings);
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            files.push(path);
        }
    }
}

fn is_repo_root(path: &Path) -> bool {
    path.join("divisions.json").is_file() && path.join("scripts/convert.sh").is_file()
}

fn find_root() -> io::Result<PathBuf> {
    let mut starts = Vec::new();
    if let Some(value) = env::var_os("AGENCY_AGENTS_REPO") {
        starts.push(PathBuf::from(value));
    }
    starts.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Ok(value) = env::current_dir() {
        starts.push(value);
    }
    if let Ok(value) = env::current_exe() {
        starts.push(value);
    }
    for start in starts {
        for candidate in start.ancestors() {
            if is_repo_root(candidate) {
                return fs::canonicalize(candidate);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Repository not found. Set AGENCY_AGENTS_REPO to the repository directory.",
    ))
}

fn installed_dir() -> Option<PathBuf> {
    let home = env::var_os("USERPROFILE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            let drive = env::var_os("HOMEDRIVE")?;
            let path = env::var_os("HOMEPATH")?;
            Some(PathBuf::from(drive).join(path))
        })?;
    let dir = home.join(".codex").join("agents");
    if dir.is_dir() && fs::read_dir(&dir).is_ok() {
        Some(dir)
    } else {
        None
    }
}

fn file_status(directory: Option<&Path>, slug: &str) -> FileStatus {
    let Some(directory) = directory else {
        return FileStatus::NotAvailable;
    };
    let path = directory.join(format!("{slug}.toml"));
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => FileStatus::Present,
        Ok(_) => FileStatus::Missing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => FileStatus::Missing,
        Err(_) => FileStatus::NotAvailable,
    }
}

fn map_tomls(agents: &mut [Agent], generated: Option<&Path>, installed: Option<&Path>) {
    let mut by_slug: HashMap<String, Vec<String>> = HashMap::new();
    for agent in agents.iter() {
        if agent.metadata_warning.as_deref().is_some_and(|warning| {
            warning.contains("Missing name")
                || warning.contains("Invalid YAML")
                || warning.contains("Unclosed frontmatter")
        }) {
            continue;
        }
        let slug = slugify(&agent.name);
        if !slug.is_empty() {
            by_slug.entry(slug).or_default().push(agent.id.clone());
        }
    }
    for agent in agents {
        let slug = slugify(&agent.name);
        let sources = by_slug.get(&slug).cloned().unwrap_or_default();
        if sources.is_empty() {
            continue;
        }
        agent.mapping.slug = Some(slug.clone());
        if sources.len() > 1 {
            agent.mapping.conflicting_sources =
                sources.into_iter().filter(|id| id != &agent.id).collect();
            agent.mapping.generated = FileStatus::Ambiguous;
            agent.mapping.installed = FileStatus::Ambiguous;
        } else {
            agent.mapping.generated = file_status(generated, &slug);
            agent.mapping.installed = file_status(installed, &slug);
        }
    }
}

pub fn load() -> io::Result<Catalog> {
    let root = find_root()?;
    let divisions: Value = serde_json::from_slice(&fs::read(root.join("divisions.json"))?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let entries = divisions
        .get("divisions")
        .and_then(Value::as_mapping)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "divisions.json has no divisions object",
            )
        })?;
    let mut warnings = Vec::new();
    let mut agents = Vec::new();
    for (key, value) in entries {
        let Some(category) = key.as_str() else {
            continue;
        };
        let label = value
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(category);
        let directory = root.join(category);
        if !directory.is_dir() {
            warnings.push(format!("Missing division: {category}"));
            continue;
        }
        let mut files = Vec::new();
        collect_markdown(&directory, &mut files, &mut warnings);
        files.sort();
        for path in files {
            match parse_agent(&root, &path, category, label) {
                Ok(Some(agent)) => agents.push(agent),
                Ok(None) => {}
                Err(error) => warnings.push(format!("Cannot read {}: {error}", path.display())),
            }
        }
    }
    agents.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.id.cmp(&b.id))
    });
    let generated = root.join("integrations/codex/agents");
    let generated_accessible = generated.is_dir() && fs::read_dir(&generated).is_ok();
    let installed = installed_dir();
    map_tomls(
        &mut agents,
        generated_accessible.then_some(generated.as_path()),
        installed.as_deref(),
    );
    Ok(Catalog {
        agents,
        source_root: root.to_string_lossy().into_owned(),
        installed_directory: installed.map(|path| path.to_string_lossy().into_owned()),
        generated_directory: generated.to_string_lossy().into_owned(),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(id: &str, name: &str) -> Agent {
        Agent {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category: "engineering".into(),
            category_label: "Engineering".into(),
            source_path: id.into(),
            color: None,
            emoji: None,
            vibe: None,
            metadata_warning: None,
            mapping: Mapping::default(),
        }
    }

    #[test]
    fn parses_folded_and_quoted_metadata() {
        let (fields, warning) = parse_metadata("---\r\nname: 'Builder: One'\r\ndescription: First line\r\n  second line\r\n---\r\nBody").unwrap();
        assert!(warning.is_none());
        assert_eq!(
            string_field(&fields, "name").as_deref(),
            Some("Builder: One")
        );
        assert_eq!(
            string_field(&fields, "description").as_deref(),
            Some("First line second line")
        );
    }

    #[test]
    fn malformed_frontmatter_is_contained() {
        assert!(parse_metadata("# A document").is_none());
        let (_, warning) = parse_metadata("---\nname: [broken\n---\n").unwrap();
        assert!(warning.unwrap().contains("Invalid YAML"));
        assert!(parse_metadata("---\nname: Lost").unwrap().1.is_some());
    }

    #[test]
    fn stable_id_comes_from_relative_path() {
        let root = Path::new("repo");
        assert_eq!(
            source_id(root, Path::new("repo/engineering/nested/agent.md")).unwrap(),
            "engineering/nested/agent.md"
        );
    }

    #[test]
    fn slug_matches_converter_and_duplicate_sources_are_ambiguous() {
        assert_eq!(slugify("AEO / GEO: Specialist"), "aeo-geo-specialist");
        let mut agents = vec![
            agent("engineering/backend.md", "Backend Architect"),
            agent("specialized/another.md", "Backend Architect"),
        ];
        map_tomls(&mut agents, None, None);
        assert!(matches!(agents[0].mapping.generated, FileStatus::Ambiguous));
        assert_eq!(
            agents[0].mapping.conflicting_sources,
            vec!["specialized/another.md"]
        );
        assert!(matches!(agents[1].mapping.installed, FileStatus::Ambiguous));
    }

    #[test]
    fn unavailable_directories_do_not_claim_a_match() {
        let mut agents = vec![agent("engineering/backend.md", "Backend Architect")];
        map_tomls(&mut agents, None, None);
        assert!(matches!(
            agents[0].mapping.generated,
            FileStatus::NotAvailable
        ));
        assert!(matches!(
            agents[0].mapping.installed,
            FileStatus::NotAvailable
        ));
    }

    #[test]
    fn checks_toml_presence_by_converter_filename() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(matches!(
            file_status(Some(manifest), "Cargo"),
            FileStatus::Present
        ));
        assert!(matches!(
            file_status(Some(manifest), "not-a-real-agent"),
            FileStatus::Missing
        ));
    }

    #[test]
    fn discovers_only_canonical_divisions_in_current_checkout() {
        let catalog = load().unwrap();
        assert!(catalog.agents.len() >= 280);
        assert!(catalog
            .agents
            .iter()
            .all(|agent| !agent.id.starts_with("integrations/")));
        assert!(catalog
            .agents
            .iter()
            .all(|agent| agent.id == agent.source_path));
        assert!(catalog
            .agents
            .iter()
            .any(|agent| agent.id == "engineering/engineering-backend-architect.md"));
    }
}
