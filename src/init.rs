//! Project initialization: copy embedded assets, set up `.specite/`, optional git init.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::assets;

const MANAGED_COMMAND_FILES: &[&str] = &["exec.md", "iter.md", "plan.md", "post.md", "spec.md"];

const MANAGED_AGENT_FILES: &[&str] = &["phase-executor.md", "plan-creator.md", "web-researcher.md"];

const OBSOLETE_COMMAND_FILES: &[&str] = &["agentsmd.md", "list-iters.md"];

const MANAGED_TEMPLATE_FILES: &[&str] = &["SPEC.md", "PLAN.md"];

const LEGACY_SCRIPT_FILES: &[&str] = &[
    "iter_manager.py",
    "prompt-agentsmd.py",
    "prompt-exec.py",
    "prompt-plan.py",
    "prompt-post.py",
];

#[derive(Debug)]
pub struct InitResult {
    pub project_root: PathBuf,
    pub commands_dir: PathBuf,
    pub agents_dir: PathBuf,
    pub removed_legacy_scripts: bool,
    pub legacy_scripts_dir: Option<PathBuf>,
    pub warnings: Vec<String>,
}

/// Initialize Specite in `target_path`.
pub fn initialize_project(target_path: &Path) -> Result<InitResult, std::io::Error> {
    let project_root = expand_user(target_path);
    fs::create_dir_all(&project_root)?;
    let project_root = project_root.canonicalize().unwrap_or(project_root);

    let mut warnings: Vec<String> = Vec::new();
    ensure_specite_state(&project_root)?;
    install_managed_templates(&project_root)?;
    let commands_dir = install_managed_commands(&project_root)?;
    let agents_dir = install_managed_agents(&project_root)?;
    cleanup_obsolete_commands(&project_root);
    append_gitignore_entries(
        &project_root.join(".gitignore"),
        &[".opencode/commands/", ".opencode/agents/"],
    );
    let (removed_legacy_scripts, legacy_scripts_dir) =
        cleanup_legacy_scripts(&project_root, &mut warnings);
    maybe_init_git(&project_root, &mut warnings);

    Ok(InitResult {
        project_root,
        commands_dir,
        agents_dir,
        removed_legacy_scripts,
        legacy_scripts_dir,
        warnings,
    })
}

fn expand_user(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    } else if s == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }
    p.to_path_buf()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn ensure_specite_state(project_root: &Path) -> Result<(), std::io::Error> {
    let specite = project_root.join(".specite");
    fs::create_dir_all(specite.join("iterations"))?;
    fs::create_dir_all(specite.join("docs"))?;
    let iters_file = specite.join("iters.json");
    if !iters_file.exists() {
        fs::write(&iters_file, "{\"iterations\": []}\n")?;
    }
    Ok(())
}

fn install_managed_templates(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    let templates_dir = project_root.join(".specite").join("templates");
    fs::create_dir_all(&templates_dir)?;
    for name in MANAGED_TEMPLATE_FILES {
        let content = assets::load_template(name)
            .unwrap_or_else(|| panic!("missing embedded template {name}"));
        fs::write(templates_dir.join(name), content)?;
    }
    Ok(templates_dir)
}

fn install_managed_commands(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    let commands_dir = project_root.join(".opencode").join("commands");
    fs::create_dir_all(&commands_dir)?;
    for name in MANAGED_COMMAND_FILES {
        let content =
            assets::load_command(name).unwrap_or_else(|| panic!("missing embedded command {name}"));
        fs::write(commands_dir.join(name), content)?;
    }
    Ok(commands_dir)
}

fn install_managed_agents(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    let agents_dir = project_root.join(".opencode").join("agents");
    fs::create_dir_all(&agents_dir)?;
    for name in MANAGED_AGENT_FILES {
        let content =
            assets::load_agent(name).unwrap_or_else(|| panic!("missing embedded agent {name}"));
        fs::write(agents_dir.join(name), content)?;
    }
    Ok(agents_dir)
}

fn cleanup_obsolete_commands(project_root: &Path) {
    let commands_dir = project_root.join(".opencode").join("commands");
    for name in OBSOLETE_COMMAND_FILES {
        let target = commands_dir.join(name);
        if target.is_file() {
            let _ = fs::remove_file(&target);
        }
    }
}

fn append_gitignore_entries(gitignore: &Path, entries: &[&str]) {
    let existing = if gitignore.exists() {
        fs::read_to_string(gitignore).unwrap_or_default()
    } else {
        String::new()
    };
    let existing_lines: std::collections::HashSet<&str> = existing.lines().collect();
    let mut to_add: Vec<&str> = entries
        .iter()
        .filter(|e| !existing_lines.contains(**e))
        .copied()
        .collect();
    if to_add.is_empty() {
        return;
    }
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(gitignore)
        .expect("failed to open .gitignore");
    if !existing.is_empty() && !existing.ends_with('\n') {
        file.write_all(b"\n").expect("gitignore write");
    }
    to_add.sort();
    for entry in to_add {
        file.write_all(entry.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .expect("gitignore write");
    }
}

fn cleanup_legacy_scripts(
    project_root: &Path,
    warnings: &mut Vec<String>,
) -> (bool, Option<PathBuf>) {
    let scripts_dir = project_root.join(".opencode").join("scripts");
    if !scripts_dir.exists() || !scripts_dir.is_dir() {
        return (false, None);
    }

    let mut unknown: Vec<String> = Vec::new();
    let entries = match fs::read_dir(&scripts_dir) {
        Ok(e) => e,
        Err(_) => return (false, Some(scripts_dir)),
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if entry.path().is_dir() && name == "__pycache__" {
            continue;
        }
        if entry.path().is_file() && LEGACY_SCRIPT_FILES.contains(&name.as_str()) {
            continue;
        }
        unknown.push(name);
    }

    if !unknown.is_empty() {
        unknown.sort();
        warnings.push(format!(
            ".opencode/scripts contains unmanaged files; leaving it in place ({})",
            unknown.join(", ")
        ));
        return (false, Some(scripts_dir));
    }

    let _ = fs::remove_dir_all(&scripts_dir);
    (true, Some(scripts_dir))
}

fn maybe_init_git(project_root: &Path, warnings: &mut Vec<String>) {
    if project_root.join(".git").exists() {
        return;
    }
    let output = Command::new("git")
        .arg("init")
        .current_dir(project_root)
        .output();
    match output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let detail = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let detail = if detail.is_empty() {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            } else {
                detail
            };
            let detail = if detail.is_empty() {
                "unknown error".to_string()
            } else {
                detail
            };
            warnings.push(format!("`git init` failed; continuing ({detail})"));
        }
        Err(_) => {
            warnings.push("git is not available; skipped `git init`".to_string());
        }
    }
}
