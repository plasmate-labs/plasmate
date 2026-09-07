//! Dry-run validation for independently versioned release artifacts.

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub artifacts: Vec<ReleaseArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub id: String,
    pub package: String,
    pub version: String,
    pub publications: Vec<Publication>,
    pub sources: Vec<VersionSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Publication {
    CratesIo,
    GithubRelease,
    Ghcr,
    Npm,
    Pypi,
    GoModule,
    McpRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionSource {
    pub path: String,
    pub kind: SourceKind,
    pub selector: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    JsonPointer,
    TomlKey,
    /// Selects the version from a `[[package]]` table with the given package name.
    TomlPackage,
    /// The first capture group from a regular expression over a UTF-8 source file.
    TextRegex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub schema_version: u32,
    pub manifest: String,
    pub valid: bool,
    pub artifacts_total: usize,
    pub sources_total: usize,
    pub sources_valid: usize,
    pub public_identities_total: usize,
    pub public_identities_declared: usize,
    pub errors: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub artifact: String,
    pub path: String,
    pub expected: String,
    pub actual: Option<String>,
    pub message: String,
}

pub fn validate(
    repository_root: &Path,
    manifest_path: impl AsRef<Path>,
) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    let manifest_path = resolve(repository_root, manifest_path.as_ref());
    let manifest: ReleaseManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    if manifest.schema_version != 1 {
        return Err(format!(
            "unsupported release manifest schema {}; expected 1",
            manifest.schema_version
        )
        .into());
    }

    let mut errors = Vec::new();
    let mut artifact_ids = HashSet::new();
    let mut declared_sources = HashSet::new();
    let mut sources_total: usize = 0;
    let mut source_errors: usize = 0;
    for artifact in &manifest.artifacts {
        if !artifact_ids.insert(artifact.id.as_str()) {
            errors.push(ValidationIssue {
                artifact: artifact.id.clone(),
                path: String::new(),
                expected: artifact.version.clone(),
                actual: None,
                message: "artifact id is duplicated".to_string(),
            });
        }
        if artifact.publications.is_empty() {
            errors.push(ValidationIssue {
                artifact: artifact.id.clone(),
                path: String::new(),
                expected: artifact.version.clone(),
                actual: None,
                message: "artifact has no publication destination".to_string(),
            });
        }
        if artifact.sources.is_empty() {
            errors.push(ValidationIssue {
                artifact: artifact.id.clone(),
                path: String::new(),
                expected: artifact.version.clone(),
                actual: None,
                message: "artifact has no checked-in version source".to_string(),
            });
        }
        for source in &artifact.sources {
            sources_total += 1;
            if !declared_sources.insert(source.clone()) {
                source_errors += 1;
                errors.push(ValidationIssue {
                    artifact: artifact.id.clone(),
                    path: source.path.clone(),
                    expected: artifact.version.clone(),
                    actual: None,
                    message: "version source is declared more than once".to_string(),
                });
                continue;
            }
            match read_version(repository_root, source) {
                Ok(actual) if actual == artifact.version => {}
                Ok(actual) => {
                    source_errors += 1;
                    errors.push(ValidationIssue {
                        artifact: artifact.id.clone(),
                        path: source.path.clone(),
                        expected: artifact.version.clone(),
                        actual: Some(actual),
                        message: "checked-in version differs from release manifest".to_string(),
                    });
                }
                Err(message) => {
                    source_errors += 1;
                    errors.push(ValidationIssue {
                        artifact: artifact.id.clone(),
                        path: source.path.clone(),
                        expected: artifact.version.clone(),
                        actual: None,
                        message,
                    });
                }
            }
        }
    }

    let public_identities = discover_public_version_identities(repository_root)?;
    let mut public_identities_declared = 0;
    for identity in &public_identities {
        if declared_sources.contains(identity) {
            public_identities_declared += 1;
        } else {
            errors.push(ValidationIssue {
                artifact: "release-manifest".to_string(),
                path: identity.path.clone(),
                expected: "an explicit release-manifest version source".to_string(),
                actual: read_version(repository_root, identity).ok(),
                message: format!(
                    "public version identity is not declared (kind={}, selector={})",
                    source_kind_name(&identity.kind),
                    identity.selector
                ),
            });
        }
    }

    validate_mcp_runtime_declarations(repository_root, &manifest, &mut errors);
    validate_tracked_generated_artifacts(repository_root, &mut errors);

    let sources_valid = sources_total.saturating_sub(source_errors);
    Ok(ValidationReport {
        schema_version: 1,
        manifest: manifest_path.display().to_string(),
        valid: errors.is_empty(),
        artifacts_total: manifest.artifacts.len(),
        sources_total,
        sources_valid,
        public_identities_total: public_identities.len(),
        public_identities_declared,
        errors,
    })
}

fn discover_public_version_identities(
    repository_root: &Path,
) -> Result<Vec<VersionSource>, Box<dyn std::error::Error>> {
    let mut identities = Vec::new();
    for identity in [
        VersionSource {
            path: "Cargo.toml".to_string(),
            kind: SourceKind::TomlKey,
            selector: "package.version".to_string(),
        },
        VersionSource {
            path: "Cargo.lock".to_string(),
            kind: SourceKind::TomlPackage,
            selector: "plasmate".to_string(),
        },
        VersionSource {
            path: "server.json".to_string(),
            kind: SourceKind::JsonPointer,
            selector: "/version".to_string(),
        },
    ] {
        if repository_root.join(&identity.path).is_file() {
            identities.push(identity);
        }
    }
    let mut files = Vec::new();
    for root in ["sdk", "packages", "integrations", "tools"] {
        let path = repository_root.join(root);
        if path.is_dir() {
            collect_public_source_files(repository_root, &path, &mut files)?;
        }
    }
    files.sort();

    for path in files {
        let relative = path
            .strip_prefix(repository_root)?
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        match name {
            "pyproject.toml" => {
                let content = fs::read_to_string(&path)?;
                let value: toml::Value = toml::from_str(&content)?;
                if dotted_toml_value(&value, "project.version").is_some() {
                    identities.push(VersionSource {
                        path: relative,
                        kind: SourceKind::TomlKey,
                        selector: "project.version".to_string(),
                    });
                }
            }
            "package.json" => {
                let content = fs::read_to_string(&path)?;
                let value: serde_json::Value = serde_json::from_str(&content)?;
                if value.pointer("/version").is_some() {
                    identities.push(VersionSource {
                        path: relative,
                        kind: SourceKind::JsonPointer,
                        selector: "/version".to_string(),
                    });
                }
            }
            "package-lock.json" => {
                let content = fs::read_to_string(&path)?;
                let value: serde_json::Value = serde_json::from_str(&content)?;
                for selector in ["/version", "/packages//version"] {
                    if value.pointer(selector).is_some() {
                        identities.push(VersionSource {
                            path: relative.clone(),
                            kind: SourceKind::JsonPointer,
                            selector: selector.to_string(),
                        });
                    }
                }
            }
            "__init__.py" => {
                let content = fs::read_to_string(&path)?;
                if content.contains("__version__") {
                    identities.push(VersionSource {
                        path: relative,
                        kind: SourceKind::TextRegex,
                        selector: r#"__version__ = "([^"]+)""#.to_string(),
                    });
                }
            }
            "client.py" => {
                let content = fs::read_to_string(&path)?;
                if content.contains("SDK_VERSION") {
                    identities.push(VersionSource {
                        path: relative,
                        kind: SourceKind::TextRegex,
                        selector: r#"SDK_VERSION = "([^"]+)""#.to_string(),
                    });
                }
            }
            "index.ts" => {
                let content = fs::read_to_string(&path)?;
                if content.contains("const SDK_VERSION") {
                    identities.push(VersionSource {
                        path: relative,
                        kind: SourceKind::TextRegex,
                        selector: "const SDK_VERSION = '([^']+)'".to_string(),
                    });
                }
            }
            "client.go" => {
                let content = fs::read_to_string(&path)?;
                if content.contains("SDKVersion") {
                    identities.push(VersionSource {
                        path: relative,
                        kind: SourceKind::TextRegex,
                        selector: r#"const SDKVersion = "([^"]+)""#.to_string(),
                    });
                }
            }
            "VERSION" => {
                fs::read_to_string(&path)?;
                identities.push(VersionSource {
                    path: relative,
                    kind: SourceKind::TextRegex,
                    selector: r#"^([^\r\n]+)"#.to_string(),
                });
            }
            _ => {}
        }
    }
    identities.sort_by(|left, right| {
        (&left.path, source_kind_name(&left.kind), &left.selector).cmp(&(
            &right.path,
            source_kind_name(&right.kind),
            &right.selector,
        ))
    });
    identities.dedup();
    Ok(identities)
}

fn collect_public_source_files(
    repository_root: &Path,
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(
                name.as_ref(),
                "node_modules" | "dist" | "target" | ".venv" | "__pycache__"
            ) || name.ends_with(".egg-info")
            {
                continue;
            }
            collect_public_source_files(repository_root, &path, files)?;
        } else if file_type.is_file() && path.starts_with(repository_root) {
            let is_candidate =
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        matches!(
                            name,
                            "pyproject.toml"
                                | "package.json"
                                | "package-lock.json"
                                | "__init__.py"
                                | "client.py"
                                | "index.ts"
                                | "client.go"
                                | "VERSION"
                        )
                    });
            if is_candidate {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn source_kind_name(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::JsonPointer => "json_pointer",
        SourceKind::TomlKey => "toml_key",
        SourceKind::TomlPackage => "toml_package",
        SourceKind::TextRegex => "text_regex",
    }
}

fn validate_mcp_runtime_declarations(
    repository_root: &Path,
    manifest: &ReleaseManifest,
    errors: &mut Vec<ValidationIssue>,
) {
    let mut metadata_paths = HashSet::new();
    for artifact in &manifest.artifacts {
        if !artifact.publications.contains(&Publication::McpRegistry) {
            continue;
        }
        let mut has_server_metadata_source = false;
        for source in &artifact.sources {
            if source.kind == SourceKind::JsonPointer
                && source.selector == "/version"
                && source.path.ends_with("server.json")
            {
                has_server_metadata_source = true;
                metadata_paths.insert((artifact.id.as_str(), source.path.as_str()));
            }
        }
        if !has_server_metadata_source {
            errors.push(ValidationIssue {
                artifact: artifact.id.clone(),
                path: String::new(),
                expected: "an explicit server.json /version source".to_string(),
                actual: None,
                message: "MCP Registry artifact has no server metadata source".to_string(),
            });
        }
    }

    for (artifact_id, relative_path) in metadata_paths {
        let path = match secure_source_path(repository_root, Path::new(relative_path)) {
            Ok(path) => path,
            Err(_) => continue,
        };
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let metadata: serde_json::Value = match serde_json::from_str(&content) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let server_name = metadata.get("name").and_then(|value| value.as_str());
        let Some(packages) = metadata.get("packages") else {
            errors.push(ValidationIssue {
                artifact: artifact_id.to_string(),
                path: relative_path.to_string(),
                expected: "at least one runnable package declaration".to_string(),
                actual: None,
                message: "MCP registry metadata has no runnable package".to_string(),
            });
            continue;
        };
        let Some(packages) = packages.as_array() else {
            errors.push(ValidationIssue {
                artifact: artifact_id.to_string(),
                path: relative_path.to_string(),
                expected: "packages to be an array when present".to_string(),
                actual: Some(packages.to_string()),
                message: "MCP runtime declarations are malformed".to_string(),
            });
            continue;
        };
        if packages.is_empty() {
            errors.push(ValidationIssue {
                artifact: artifact_id.to_string(),
                path: relative_path.to_string(),
                expected: "at least one runnable package declaration".to_string(),
                actual: Some("[]".to_string()),
                message: "MCP registry metadata has no runnable package".to_string(),
            });
        }
        for package in packages {
            validate_mcp_runtime_package(
                repository_root,
                manifest,
                artifact_id,
                relative_path,
                server_name,
                package,
                errors,
            );
        }
    }
}

fn validate_mcp_runtime_package(
    repository_root: &Path,
    manifest: &ReleaseManifest,
    artifact_id: &str,
    metadata_path: &str,
    server_name: Option<&str>,
    package: &serde_json::Value,
    errors: &mut Vec<ValidationIssue>,
) {
    let registry = package.get("registryType").and_then(|value| value.as_str());
    let identifier = package.get("identifier").and_then(|value| value.as_str());
    let version = package.get("version").and_then(|value| value.as_str());
    let (Some(registry), Some(identifier), Some(version)) = (registry, identifier, version) else {
        errors.push(ValidationIssue {
            artifact: artifact_id.to_string(),
            path: metadata_path.to_string(),
            expected: "registryType, identifier, and an exact version".to_string(),
            actual: Some(package.to_string()),
            message: "MCP runtime package declaration is incomplete".to_string(),
        });
        return;
    };
    let publication = match registry {
        "npm" => Publication::Npm,
        "pypi" => Publication::Pypi,
        "oci" => Publication::Ghcr,
        _ => {
            errors.push(ValidationIssue {
                artifact: artifact_id.to_string(),
                path: metadata_path.to_string(),
                expected: "a statically verifiable oci, npm, or pypi runtime package".to_string(),
                actual: Some(format!("{registry}:{identifier}@{version}")),
                message: "MCP runtime package registry is not supported by release validation"
                    .to_string(),
            });
            return;
        }
    };
    let runtime_artifact = if publication == Publication::Ghcr {
        manifest.artifacts.iter().find(|artifact| {
            artifact.id == artifact_id
                && artifact.version == version
                && artifact.publications.contains(&Publication::Ghcr)
        })
    } else {
        manifest.artifacts.iter().find(|artifact| {
            artifact.package == identifier
                && artifact.version == version
                && artifact.publications.contains(&publication)
        })
    };
    let Some(runtime_artifact) = runtime_artifact else {
        errors.push(ValidationIssue {
            artifact: artifact_id.to_string(),
            path: metadata_path.to_string(),
            expected: "a matching versioned release artifact".to_string(),
            actual: Some(format!("{registry}:{identifier}@{version}")),
            message: "MCP runtime package is not covered by the release manifest".to_string(),
        });
        return;
    };

    if publication == Publication::Ghcr {
        validate_oci_runtime_package(
            repository_root,
            runtime_artifact,
            server_name,
            metadata_path,
            identifier,
            package,
            errors,
        );
        return;
    }

    let runnable = match publication {
        Publication::Npm => npm_artifact_has_executable(repository_root, runtime_artifact),
        Publication::Pypi => pypi_artifact_has_executable(repository_root, runtime_artifact),
        _ => false,
    };
    if !runnable {
        errors.push(ValidationIssue {
            artifact: runtime_artifact.id.clone(),
            path: metadata_path.to_string(),
            expected: "a package that installs a runnable MCP server command".to_string(),
            actual: Some(format!("{registry}:{identifier}@{version}")),
            message: "MCP runtime package has no executable entry point".to_string(),
        });
    }
}

fn validate_oci_runtime_package(
    repository_root: &Path,
    artifact: &ReleaseArtifact,
    server_name: Option<&str>,
    metadata_path: &str,
    identifier: &str,
    package: &serde_json::Value,
    errors: &mut Vec<ValidationIssue>,
) {
    let expected_identifier = server_name
        .and_then(|name| name.strip_prefix("io.github."))
        .map(|repository| format!("ghcr.io/{repository}:v{}", artifact.version));
    match expected_identifier {
        Some(expected) if identifier != expected => errors.push(ValidationIssue {
            artifact: artifact.id.clone(),
            path: metadata_path.to_string(),
            expected,
            actual: Some(identifier.to_string()),
            message: "MCP OCI package must identify the declared GHCR artifact with an exact v<version> tag"
                .to_string(),
        }),
        None => errors.push(ValidationIssue {
            artifact: artifact.id.clone(),
            path: metadata_path.to_string(),
            expected: "an io.github.<owner>/<repository> server name".to_string(),
            actual: server_name.map(str::to_string),
            message: "MCP server name cannot be mapped to its GHCR image".to_string(),
        }),
        _ => {}
    }

    if package
        .pointer("/transport/type")
        .and_then(|value| value.as_str())
        != Some("stdio")
    {
        errors.push(ValidationIssue {
            artifact: artifact.id.clone(),
            path: metadata_path.to_string(),
            expected: "transport.type=stdio".to_string(),
            actual: package.get("transport").map(ToString::to_string),
            message: "MCP OCI package must expose the stdio transport".to_string(),
        });
    }

    let has_required_mcp_argument = package
        .get("packageArguments")
        .and_then(|arguments| arguments.as_array())
        .is_some_and(|arguments| {
            arguments.iter().any(|argument| {
                argument.get("type").and_then(|value| value.as_str()) == Some("positional")
                    && argument.get("value").and_then(|value| value.as_str()) == Some("mcp")
                    && argument.get("isRequired").and_then(|value| value.as_bool()) == Some(true)
            })
        });
    if !has_required_mcp_argument {
        errors.push(ValidationIssue {
            artifact: artifact.id.clone(),
            path: metadata_path.to_string(),
            expected: "a required positional package argument with value mcp".to_string(),
            actual: package.get("packageArguments").map(ToString::to_string),
            message: "MCP OCI package does not invoke the server's mcp subcommand".to_string(),
        });
    }

    let dockerfile_path = repository_root.join("Dockerfile");
    let dockerfile = fs::read_to_string(&dockerfile_path).ok();
    let ownership_label =
        server_name.map(|name| format!("LABEL io.modelcontextprotocol.server.name=\"{name}\""));
    if !dockerfile
        .as_ref()
        .zip(ownership_label.as_ref())
        .is_some_and(|(contents, label)| contents.lines().any(|line| line.trim() == label))
    {
        errors.push(ValidationIssue {
            artifact: artifact.id.clone(),
            path: "Dockerfile".to_string(),
            expected: ownership_label.unwrap_or_else(|| {
                "LABEL io.modelcontextprotocol.server.name=\"<server name>\"".to_string()
            }),
            actual: None,
            message: "GHCR image is missing the MCP Registry ownership label".to_string(),
        });
    }
    if !dockerfile.as_ref().is_some_and(|contents| {
        contents
            .lines()
            .any(|line| line.trim() == "ENTRYPOINT [\"plasmate\"]")
    }) {
        errors.push(ValidationIssue {
            artifact: artifact.id.clone(),
            path: "Dockerfile".to_string(),
            expected: "ENTRYPOINT [\"plasmate\"]".to_string(),
            actual: None,
            message: "GHCR image does not launch the Plasmate executable".to_string(),
        });
    }
}

fn npm_artifact_has_executable(repository_root: &Path, artifact: &ReleaseArtifact) -> bool {
    let Some(source) = artifact.sources.iter().find(|source| {
        source.kind == SourceKind::JsonPointer
            && source.selector == "/version"
            && source.path.ends_with("package.json")
    }) else {
        return false;
    };
    let Ok(path) = secure_source_path(repository_root, Path::new(&source.path)) else {
        return false;
    };
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(package): Result<serde_json::Value, _> = serde_json::from_str(&content) else {
        return false;
    };
    match package.get("bin") {
        Some(serde_json::Value::String(command)) => !command.trim().is_empty(),
        Some(serde_json::Value::Object(commands)) => commands.values().any(|command| {
            command
                .as_str()
                .is_some_and(|command| !command.trim().is_empty())
        }),
        _ => false,
    }
}

fn pypi_artifact_has_executable(repository_root: &Path, artifact: &ReleaseArtifact) -> bool {
    let Some(source) = artifact.sources.iter().find(|source| {
        source.kind == SourceKind::TomlKey
            && source.selector == "project.version"
            && source.path.ends_with("pyproject.toml")
    }) else {
        return false;
    };
    let Ok(path) = secure_source_path(repository_root, Path::new(&source.path)) else {
        return false;
    };
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(package): Result<toml::Value, _> = toml::from_str(&content) else {
        return false;
    };
    dotted_toml_value(&package, "project.scripts")
        .and_then(|scripts| scripts.as_table())
        .is_some_and(|scripts| {
            scripts.values().any(|command| {
                command
                    .as_str()
                    .is_some_and(|command| !command.trim().is_empty())
            })
        })
}

fn validate_tracked_generated_artifacts(repository_root: &Path, errors: &mut Vec<ValidationIssue>) {
    let output = match Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["ls-files", "-z", "--", "integrations/browser-use/dist"])
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            errors.push(ValidationIssue {
                artifact: "release-manifest".to_string(),
                path: "integrations/browser-use/dist".to_string(),
                expected: "a successful git ls-files inventory".to_string(),
                actual: Some(error.to_string()),
                message: "cannot verify tracked generated artifacts".to_string(),
            });
            return;
        }
    };
    validate_tracked_artifact_output(
        repository_root,
        output.status.success(),
        output.status.code(),
        &output.stdout,
        &output.stderr,
        errors,
    );
}

fn validate_tracked_artifact_output(
    repository_root: &Path,
    success: bool,
    status_code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
    errors: &mut Vec<ValidationIssue>,
) {
    if !success {
        let diagnostic = String::from_utf8_lossy(stderr)
            .chars()
            .take(512)
            .collect::<String>();
        errors.push(ValidationIssue {
            artifact: "release-manifest".to_string(),
            path: "integrations/browser-use/dist".to_string(),
            expected: "git ls-files to exit successfully".to_string(),
            actual: Some(format!(
                "status={}; stderr={diagnostic}",
                status_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "signal".to_string())
            )),
            message: "cannot verify tracked generated artifacts".to_string(),
        });
        return;
    }
    for raw_path in stdout.split(|byte| *byte == 0) {
        if raw_path.is_empty() {
            continue;
        }
        let path = match std::str::from_utf8(raw_path) {
            Ok(path) => path,
            Err(error) => {
                errors.push(ValidationIssue {
                    artifact: "release-manifest".to_string(),
                    path: "integrations/browser-use/dist".to_string(),
                    expected: "UTF-8 repository-relative tracked paths".to_string(),
                    actual: Some(format!("invalid UTF-8 at byte {}", error.valid_up_to())),
                    message: "git returned an invalid tracked-artifact path".to_string(),
                });
                continue;
            }
        };
        let relative = Path::new(path);
        let has_invalid_component = relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            });
        if has_invalid_component
            || !relative.starts_with(Path::new("integrations/browser-use/dist"))
        {
            errors.push(ValidationIssue {
                artifact: "release-manifest".to_string(),
                path: "integrations/browser-use/dist".to_string(),
                expected: "repository-relative paths under integrations/browser-use/dist"
                    .to_string(),
                actual: Some(path.chars().take(512).collect()),
                message: "git returned an invalid tracked-artifact path".to_string(),
            });
            continue;
        }
        if !repository_root.join(relative).exists() {
            continue;
        }
        errors.push(ValidationIssue {
            artifact: "browser-use-adapter".to_string(),
            path: path.to_string(),
            expected: "generated wheels and sdists to be untracked".to_string(),
            actual: Some("tracked generated artifact".to_string()),
            message: "tracked Browser Use build artifact can drift from source metadata"
                .to_string(),
        });
    }
}

fn read_version(repository_root: &Path, source: &VersionSource) -> Result<String, String> {
    let path = secure_source_path(repository_root, Path::new(&source.path))?;
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    match source.kind {
        SourceKind::JsonPointer => {
            let value: serde_json::Value =
                serde_json::from_str(&content).map_err(|error| format!("invalid JSON: {error}"))?;
            value
                .pointer(&source.selector)
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .ok_or_else(|| format!("JSON pointer {} is not a string", source.selector))
        }
        SourceKind::TomlKey => {
            let value: toml::Value =
                toml::from_str(&content).map_err(|error| format!("invalid TOML: {error}"))?;
            dotted_toml_value(&value, &source.selector)
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .ok_or_else(|| format!("TOML key {} is not a string", source.selector))
        }
        SourceKind::TomlPackage => {
            let value: toml::Value =
                toml::from_str(&content).map_err(|error| format!("invalid TOML: {error}"))?;
            value
                .get("package")
                .and_then(|packages| packages.as_array())
                .and_then(|packages| {
                    packages.iter().find(|package| {
                        package.get("name").and_then(|name| name.as_str())
                            == Some(source.selector.as_str())
                    })
                })
                .and_then(|package| package.get("version"))
                .and_then(|version| version.as_str())
                .map(str::to_string)
                .ok_or_else(|| format!("TOML package {} has no string version", source.selector))
        }
        SourceKind::TextRegex => {
            let pattern = regex::Regex::new(&source.selector)
                .map_err(|error| format!("invalid version regex: {error}"))?;
            pattern
                .captures(&content)
                .and_then(|captures| captures.get(1))
                .map(|capture| capture.as_str().to_string())
                .ok_or_else(|| "version regex did not produce capture group 1".to_string())
        }
    }
}

fn secure_source_path(repository_root: &Path, path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Err("release source path must be repository-relative, not absolute".to_string());
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir => {
                return Err(
                    "release source path must not contain parent-directory traversal".to_string(),
                );
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("release source path must be repository-relative".to_string());
            }
        }
    }

    let canonical_root = repository_root
        .canonicalize()
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    let candidate = canonical_root.join(path);
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", candidate.display()))?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err("release source path resolves outside the repository".to_string());
    }
    Ok(canonical_candidate)
}

fn dotted_toml_value<'a>(value: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    key.split('.')
        .try_fold(value, |current, component| current.get(component))
}

fn resolve(repository_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_release_manifest_is_coherent() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = validate(root, "release-manifest.json").expect("validate release manifest");
        assert!(report.valid, "{:?}", report.errors);
        assert_eq!(report.sources_total, report.sources_valid);
        assert_eq!(
            report.public_identities_total,
            report.public_identities_declared
        );
    }

    #[test]
    fn checked_in_registry_metadata_declares_runnable_oci_package() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let cargo: toml::Value = toml::from_str(
            &fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo metadata"),
        )
        .expect("parse Cargo metadata");
        assert_eq!(cargo["package"]["version"].as_str(), Some("0.6.0"));

        let manifest: ReleaseManifest = serde_json::from_str(
            &fs::read_to_string(root.join("release-manifest.json")).expect("read manifest"),
        )
        .expect("parse manifest");
        let engine = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.id == "plasmate-rust")
            .expect("Rust engine artifact");
        assert_eq!(engine.version, "0.6.0");
        assert!(engine.publications.contains(&Publication::Ghcr));
        assert!(engine.publications.contains(&Publication::McpRegistry));

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join("server.json")).expect("read server metadata"),
        )
        .expect("parse server metadata");
        let packages = metadata["packages"].as_array().expect("package array");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0]["registryType"], "oci");
        assert_eq!(
            packages[0]["identifier"],
            "ghcr.io/plasmate-labs/plasmate:v0.6.0"
        );
        assert_eq!(metadata["version"], "0.6.0");
        assert_eq!(packages[0]["version"], "0.6.0");
        assert_eq!(packages[0]["transport"]["type"], "stdio");
        assert_eq!(packages[0]["packageArguments"][0]["type"], "positional");
        assert_eq!(packages[0]["packageArguments"][0]["value"], "mcp");
        assert_eq!(packages[0]["packageArguments"][0]["isRequired"], true);
        assert!(metadata.get("remotes").is_none());
        assert_eq!(metadata["websiteUrl"], "https://plasmate.app");
    }

    #[test]
    fn next_registry_candidate_waits_for_new_labeled_image() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dockerfile = fs::read_to_string(root.join("Dockerfile")).expect("read Dockerfile");
        assert!(dockerfile.lines().any(|line| {
            line.trim()
                == "LABEL io.modelcontextprotocol.server.name=\"io.github.plasmate-labs/plasmate\""
        }));

        let guide = fs::read_to_string(root.join("docs/publish-to-mcp-registry.md"))
            .expect("read MCP publishing guide");
        assert!(guide.contains("v0.5.1 image was built before the MCP ownership label"));
        assert!(guide.contains("must remain\nimmutable"));
        let anonymous_image_check = guide
            .find("docker manifest inspect ghcr.io/plasmate-labs/plasmate:v0.6.0")
            .expect("anonymous v0.6.0 image check");
        let registry_publish = guide
            .find("mcp-publisher publish")
            .expect("Registry publication command");
        assert!(anonymous_image_check < registry_publish);
    }

    #[test]
    fn public_identity_discovery_ignores_arbitrary_binary_files() {
        let directory = tempfile::tempdir().expect("temporary repository");
        fs::create_dir_all(directory.path().join("sdk/assets")).expect("create SDK assets");
        fs::write(
            directory.path().join("sdk/assets/browser.wasm"),
            [0xff, 0xfe, 0xfd, 0x00],
        )
        .expect("write non-UTF-8 binary");

        let identities = discover_public_version_identities(directory.path())
            .expect("ignore unrelated binary source");
        assert!(identities.is_empty());
    }

    #[test]
    fn active_public_truth_surfaces_reject_obsolete_links_and_universal_claims() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut paths = [
            "AGENTS.md",
            "Cargo.toml",
            "README.md",
            "MCP-SPEC.md",
            "server.json",
            ".claude/skills/plasmate/SKILL.md",
            "docs/claude-desktop-config.md",
            "integrations/browser-use/README.md",
            "integrations/browser-use/pyproject.toml",
            "integrations/langchain/README.md",
            "sdk/node/package.json",
            "sdk/python/pyproject.toml",
            "website/index.html",
            "website/compare.html",
            "website/.well-known/som.json",
            "website/docs/executive-guide.html",
            "website/docs/log.html",
            "website/docs/openclaw-guide.html",
            "website/llms.txt",
            "website/llms-full.txt",
        ]
        .into_iter()
        .map(|path| root.join(path))
        .collect::<Vec<_>>();
        for directory in ["website/docs/src", "website/compare", "website/blog"] {
            collect_truth_markdown(&root.join(directory), &mut paths)
                .expect("collect public truth sources");
        }

        let banned = [
            "github.com/nicepkg/plasmate",
            "10-800x",
            "10–800x",
            "10-100x fewer tokens",
            "10–100x fewer tokens",
            "17x fewer tokens",
            "17x token compression",
            "17.5x average token compression",
            "10x fewer tokens",
            "10x less tokens",
            "10x faster",
            "compress web pages 10x",
            "50x faster",
            "25x faster",
            "94% reduction in token",
            "94% token savings",
            "60+ integrations",
            "230 tests",
            "98.9% site coverage",
            "94/98 top sites",
            "first browser tool",
            "13 mcp tools",
            "26 tools are available",
            "7 methods. that's the protocol",
            "awp has 7 methods",
            "awp (native, 7 methods)",
        ];
        for path in paths {
            // The registry must name prohibited wording so automated and human
            // reviewers can distinguish retired claims from allowed claims.
            if path.ends_with("website/docs/src/claims.md") {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!(
                    "read active public truth surface {}: {error}",
                    path.display()
                )
            });
            let normalized = content.to_lowercase();
            for phrase in banned {
                assert!(
                    !normalized.contains(phrase),
                    "active public truth surface {} contains banned phrase {phrase:?}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn openclaw_docs_do_not_point_at_missing_in_tree_files_or_a_closed_tool_list() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for rel in [
            "website/docs/src/integration-openclaw.md",
            "website/docs/integration-openclaw.html",
        ] {
            let content = fs::read_to_string(root.join(rel))
                .unwrap_or_else(|error| panic!("read OpenClaw docs {rel}: {error}"));
            assert!(
                !content.contains("integrations/openclaw/"),
                "{rel} still points at missing in-tree OpenClaw files"
            );
            assert!(
                !content.contains("Available MCP tools:"),
                "{rel} still hard-codes a closed MCP tool list"
            );
            assert!(
                content.contains("tools/list"),
                "{rel} should tell agents to discover MCP tools"
            );
            assert!(
                content.contains("inspect") && content.contains("ARD"),
                "{rel} should name inspection and ARD instead of omitting them"
            );
        }
    }

    #[test]
    fn public_claim_registry_matches_retained_coverage_evidence() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let registry: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join("claims/evidence.v1.json")).expect("read claim registry"),
        )
        .expect("parse claim registry");
        assert_eq!(
            registry["schema_version"], "plasmate.claim-evidence.v1",
            "claim registry schema"
        );

        let coverage_cases = [
            (
                "public-web-output-size-html-v051",
                "website/docs/coverage.json",
            ),
            (
                "public-web-output-size-js-v051",
                "website/docs/coverage-js.json",
            ),
        ];
        let claims = registry["claims"]
            .as_array()
            .expect("claim registry claims array");
        for (claim_id, evidence_path) in coverage_cases {
            let claim = claims
                .iter()
                .find(|claim| claim["id"] == claim_id)
                .unwrap_or_else(|| panic!("claim registry contains {claim_id}"));
            let evidence: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(root.join(evidence_path))
                    .unwrap_or_else(|error| panic!("read {evidence_path}: {error}")),
            )
            .unwrap_or_else(|error| panic!("parse {evidence_path}: {error}"));

            assert_eq!(
                claim["metric"]["value"], evidence["summary"]["median_ratio"],
                "{claim_id} median ratio"
            );
            assert_eq!(
                claim["metric"]["successful_inputs"], evidence["summary"]["ok"],
                "{claim_id} successful denominator"
            );
            assert_eq!(
                claim["metric"]["attempted_inputs"], evidence["summary"]["urls_total"],
                "{claim_id} attempted denominator"
            );
            assert_eq!(
                claim["metric"]["blocked_inputs"], evidence["summary"]["blocked"],
                "{claim_id} blocked denominator"
            );
            assert_eq!(
                claim["metric"]["failed_inputs"], evidence["summary"]["failed"],
                "{claim_id} failed denominator"
            );
        }
    }

    fn collect_truth_markdown(directory: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_dir() {
                collect_truth_markdown(&path, paths)?;
            } else if file_type.is_file()
                && path.extension().and_then(|extension| extension.to_str()) == Some("md")
            {
                paths.push(path);
            }
        }
        Ok(())
    }

    #[test]
    fn rejects_oci_registry_drift_and_missing_runtime_contract() {
        let directory = tempfile::tempdir().expect("temporary repository");
        init_git_repository(directory.path());
        fs::write(
            directory.path().join("Dockerfile"),
            "FROM scratch\nLABEL io.modelcontextprotocol.server.name=\"io.github.example/plasmate\"\nENTRYPOINT [\"plasmate\"]\n",
        )
        .expect("write Dockerfile");
        fs::write(
            directory.path().join("server.json"),
            r#"{
              "name":"io.github.example/plasmate",
              "description":"example",
              "version":"1.2.3",
              "packages":[{
                "registryType":"oci",
                "identifier":"ghcr.io/example/plasmate:v1.2.3",
                "version":"1.2.3",
                "transport":{"type":"stdio"},
                "packageArguments":[{"type":"positional","value":"mcp","isRequired":true}]
              }]
            }"#,
        )
        .expect("write server metadata");
        fs::write(
            directory.path().join("release-manifest.json"),
            r#"{
              "schema_version":1,
              "artifacts":[{
                "id":"engine",
                "package":"plasmate",
                "version":"1.2.3",
                "publications":["ghcr","mcp_registry"],
                "sources":[{"path":"server.json","kind":"json_pointer","selector":"/version"}]
              }]
            }"#,
        )
        .expect("write manifest");

        let valid = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(valid.valid, "{:?}", valid.errors);

        fs::write(
            directory.path().join("Dockerfile"),
            "FROM scratch\nLABEL io.modelcontextprotocol.server.name=\"io.github.other/plasmate\"\nENTRYPOINT [\"other\"]\n",
        )
        .expect("write invalid Dockerfile");
        fs::write(
            directory.path().join("server.json"),
            r#"{
              "name":"io.github.example/plasmate",
              "description":"example",
              "version":"1.2.3",
              "packages":[{
                "registryType":"oci",
                "identifier":"ghcr.io/example/plasmate:1.2.3",
                "version":"1.2.3",
                "transport":{"type":"http"},
                "packageArguments":[]
              }]
            }"#,
        )
        .expect("write invalid server metadata");

        let invalid = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!invalid.valid);
        for expected_message in [
            "exact v<version> tag",
            "stdio transport",
            "mcp subcommand",
            "ownership label",
            "Plasmate executable",
        ] {
            assert!(
                invalid
                    .errors
                    .iter()
                    .any(|issue| issue.message.contains(expected_message)),
                "missing {expected_message:?} in {:?}",
                invalid.errors
            );
        }
    }

    #[test]
    fn checked_in_go_sdk_has_explicit_module_version_policy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest: ReleaseManifest = serde_json::from_str(
            &fs::read_to_string(root.join("release-manifest.json")).expect("read manifest"),
        )
        .expect("parse manifest");
        let go = manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.id == "plasmate-go-sdk")
            .expect("Go SDK release artifact");
        assert_eq!(go.version, "0.1.0");
        assert!(go.publications.contains(&Publication::GoModule));
        assert!(go
            .sources
            .iter()
            .any(|source| source.path == "sdk/go/VERSION"));
        assert!(go
            .sources
            .iter()
            .any(|source| source.path == "sdk/go/client.go"));
    }

    #[test]
    fn detects_version_drift_without_publishing() {
        let directory = tempfile::tempdir().expect("temporary repository");
        init_git_repository(directory.path());
        fs::write(
            directory.path().join("package.json"),
            r#"{"name":"example","version":"1.1.0"}"#,
        )
        .expect("write package");
        fs::write(
            directory.path().join("release-manifest.json"),
            r#"{
              "schema_version": 1,
              "artifacts": [{
                "id": "example",
                "package": "example",
                "version": "1.2.0",
                "publications": ["npm"],
                "sources": [{
                  "path": "package.json",
                  "kind": "json_pointer",
                  "selector": "/version"
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].expected, "1.2.0");
        assert_eq!(report.errors[0].actual.as_deref(), Some("1.1.0"));
    }

    #[test]
    fn rejects_undeclared_public_python_version_identity() {
        let directory = tempfile::tempdir().expect("temporary repository");
        let package = directory.path().join("sdk/python");
        fs::create_dir_all(package.join("src/example")).expect("create package");
        fs::write(
            package.join("pyproject.toml"),
            "[project]\nname = \"example\"\nversion = \"1.2.0\"\n",
        )
        .expect("write pyproject");
        fs::write(
            package.join("src/example/__init__.py"),
            "__version__ = \"1.1.0\"\n",
        )
        .expect("write public identity");
        fs::write(
            directory.path().join("release-manifest.json"),
            r#"{
              "schema_version": 1,
              "artifacts": [{
                "id": "example",
                "package": "example",
                "version": "1.2.0",
                "publications": ["pypi"],
                "sources": [{
                  "path": "sdk/python/pyproject.toml",
                  "kind": "toml_key",
                  "selector": "project.version"
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report.errors.iter().any(|issue| {
            issue.path == "sdk/python/src/example/__init__.py"
                && issue.message.contains("not declared")
        }));
        assert_eq!(report.public_identities_total, 2);
        assert_eq!(report.public_identities_declared, 1);
    }

    #[test]
    fn rejects_sdk_only_package_as_mcp_runtime() {
        let directory = tempfile::tempdir().expect("temporary repository");
        fs::create_dir_all(directory.path().join("sdk/node")).expect("create package");
        fs::write(
            directory.path().join("sdk/node/package.json"),
            r#"{"name":"plasmate","version":"0.4.0","main":"dist/index.js"}"#,
        )
        .expect("write package");
        fs::write(
            directory.path().join("server.json"),
            r#"{
              "name":"io.example/plasmate",
              "description":"example",
              "version":"0.5.1",
              "packages":[{
                "registryType":"npm",
                "identifier":"plasmate",
                "version":"0.4.0",
                "transport":{"type":"stdio"}
              }]
            }"#,
        )
        .expect("write server metadata");
        fs::write(
            directory.path().join("release-manifest.json"),
            r#"{
              "schema_version": 1,
              "artifacts": [
                {
                  "id":"engine",
                  "package":"plasmate",
                  "version":"0.5.1",
                  "publications":["mcp_registry"],
                  "sources":[{"path":"server.json","kind":"json_pointer","selector":"/version"}]
                },
                {
                  "id":"node-sdk",
                  "package":"plasmate",
                  "version":"0.4.0",
                  "publications":["npm"],
                  "sources":[{"path":"sdk/node/package.json","kind":"json_pointer","selector":"/version"}]
                }
              ]
            }"#,
        )
        .expect("write manifest");

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.message.contains("no executable entry point")));
    }

    #[test]
    fn rejects_mcp_registry_artifact_without_server_metadata_source() {
        let directory = tempfile::tempdir().expect("temporary repository");
        fs::write(directory.path().join("VERSION"), "1.2.3\n").expect("write version");
        fs::write(
            directory.path().join("release-manifest.json"),
            r#"{
              "schema_version":1,
              "artifacts":[{
                "id":"engine",
                "package":"plasmate",
                "version":"1.2.3",
                "publications":["mcp_registry"],
                "sources":[{
                  "path":"VERSION",
                  "kind":"text_regex",
                  "selector":"^([^\\r\\n]+)"
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report.errors.iter().any(|issue| {
            issue.artifact == "engine"
                && issue.message == "MCP Registry artifact has no server metadata source"
                && issue.expected == "an explicit server.json /version source"
        }));
    }

    #[test]
    fn rejects_tracked_browser_use_distribution_artifacts() {
        let directory = tempfile::tempdir().expect("temporary repository");
        let package = directory.path().join("integrations/browser-use");
        fs::create_dir_all(package.join("dist")).expect("create package");
        fs::write(
            package.join("pyproject.toml"),
            "[project]\nname = \"plasmate-browser-use\"\nversion = \"0.5.0\"\n",
        )
        .expect("write pyproject");
        fs::write(package.join("dist/stale.whl"), b"stale").expect("write artifact");
        fs::write(
            directory.path().join("release-manifest.json"),
            r#"{
              "schema_version": 1,
              "artifacts": [{
                "id":"browser-use-adapter",
                "package":"plasmate-browser-use",
                "version":"0.5.0",
                "publications":["pypi"],
                "sources":[{
                  "path":"integrations/browser-use/pyproject.toml",
                  "kind":"toml_key",
                  "selector":"project.version"
                }]
              }]
            }"#,
        )
        .expect("write manifest");
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(directory.path())
            .status()
            .expect("run git init")
            .success());
        assert!(Command::new("git")
            .args(["add", "integrations/browser-use/dist/stale.whl"])
            .current_dir(directory.path())
            .status()
            .expect("run git add")
            .success());

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report.errors.iter().any(|issue| {
            issue.path == "integrations/browser-use/dist/stale.whl"
                && issue.message.contains("can drift")
        }));
    }

    #[test]
    fn tracked_artifact_inventory_fails_closed_on_git_error() {
        let directory = tempfile::tempdir().expect("temporary repository");
        let mut errors = Vec::new();
        validate_tracked_artifact_output(
            directory.path(),
            false,
            Some(128),
            &[],
            b"fatal: not a git repository\nsecret suffix that must remain bounded",
            &mut errors,
        );

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].message,
            "cannot verify tracked generated artifacts"
        );
        assert!(errors[0]
            .actual
            .as_deref()
            .is_some_and(|actual| actual.contains("status=128")));
    }

    #[test]
    fn tracked_artifact_inventory_rejects_invalid_paths_without_lossy_decoding() {
        let directory = tempfile::tempdir().expect("temporary repository");
        let mut errors = Vec::new();
        validate_tracked_artifact_output(
            directory.path(),
            true,
            Some(0),
            b"\xff\xfe\0../outside.whl\0",
            &[],
            &mut errors,
        );

        assert_eq!(errors.len(), 2);
        assert!(errors
            .iter()
            .all(|issue| { issue.message == "git returned an invalid tracked-artifact path" }));
        assert!(errors.iter().any(|issue| {
            issue
                .actual
                .as_deref()
                .is_some_and(|actual| actual.contains("invalid UTF-8"))
        }));
        assert!(errors
            .iter()
            .any(|issue| { issue.actual.as_deref() == Some("../outside.whl") }));
    }

    fn write_manifest_with_source(directory: &Path, source_path: &str) {
        let manifest = serde_json::json!({
            "schema_version": 1,
            "artifacts": [{
                "id": "example",
                "package": "example",
                "version": "1.2.0",
                "publications": ["npm"],
                "sources": [{
                    "path": source_path,
                    "kind": "json_pointer",
                    "selector": "/version"
                }]
            }]
        });
        fs::write(
            directory.join("release-manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");
    }

    #[test]
    fn rejects_absolute_release_source_paths() {
        let directory = tempfile::tempdir().expect("temporary repository");
        let outside = tempfile::NamedTempFile::new().expect("outside file");
        write_manifest_with_source(
            directory.path(),
            outside.path().to_str().expect("UTF-8 path"),
        );

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report.errors[0].message.contains("not absolute"));
    }

    #[test]
    fn rejects_parent_directory_traversal() {
        let directory = tempfile::tempdir().expect("temporary repository");
        write_manifest_with_source(directory.path(), "../outside.json");

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report.errors[0]
            .message
            .contains("parent-directory traversal"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_that_resolve_outside_repository() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary repository");
        let outside = tempfile::NamedTempFile::new().expect("outside file");
        symlink(outside.path(), directory.path().join("version.json")).expect("create symlink");
        write_manifest_with_source(directory.path(), "version.json");

        let report = validate(directory.path(), "release-manifest.json").expect("validate");
        assert!(!report.valid);
        assert!(report.errors[0].message.contains("outside the repository"));
    }

    fn init_git_repository(directory: &Path) {
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(directory)
            .status()
            .expect("run git init")
            .success());
    }
}
