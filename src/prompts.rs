//! Prompt generation for specite commands.
//!
//! Load embedded templates, render `{{var}}` substitutions,
//! capture git status / diff output, and assemble the five prompt kinds.

use std::path::Path;
use std::process::Command;

use crate::assets;
use crate::error::PromptError;
use crate::iterations::IterManager;
use crate::project::{display_path, display_path_with_base};

/// Replace every `{{key}}` occurrence with `value`; unknown keys are left intact.
pub fn render_template(template: &str, variables: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in variables {
        let token = format!("{{{{{key}}}}}");
        out = out.replace(&token, value);
    }
    out
}

fn git_output(project_root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(project_root)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let trimmed = stdout.trim();
            if trimmed.is_empty() {
                "No changes".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Ok(_) => {
            if args.len() >= 2 && args[0] == "status" && args[1] == "--short" {
                "Unable to get git status".to_string()
            } else {
                "Unable to get git diff".to_string()
            }
        }
        Err(_) => "Git not found".to_string(),
    }
}

pub fn generate_plan_prompt(project_root: &Path, iter_id: &str) -> Result<String, PromptError> {
    let manager = IterManager::new(project_root);
    let resolved = manager
        .resolve_iteration_id(iter_id)
        .map_err(|e| PromptError::new(e.to_string()))?;
    let iteration_path = manager.iterations_dir.join(&resolved);
    let spec_path = iteration_path.join("SPEC.md");

    if !spec_path.exists() {
        return Err(PromptError::new(format!(
            "SPEC.md not found at {}. Check path with `specite path {} spec`, then tell the user to run `/spec` to create SPEC.md first.",
            display_path_with_base(&spec_path, project_root),
            iter_id
        )));
    }

    let plan_path = iteration_path.join("PLAN.md");

    Ok(render_template(
        assets::load_command_prompt("plan.md").expect("embedded plan.md prompt missing"),
        &[
            ("plan_path", &display_path(&plan_path)),
            ("spec_path", &display_path(&spec_path)),
            ("iter_id", &resolved),
        ],
    ))
}

pub fn generate_iter_prompt(project_root: &Path, iter_id: &str) -> Result<String, PromptError> {
    let manager = IterManager::new(project_root);
    let resolved = manager
        .resolve_iteration_id(iter_id)
        .map_err(|e| PromptError::new(e.to_string()))?;
    let iteration_path = manager.iterations_dir.join(&resolved);
    let spec_path = iteration_path.join("SPEC.md");
    let plan_path = iteration_path.join("PLAN.md");

    if !spec_path.exists() {
        return Err(PromptError::new(format!(
            "SPEC.md not found at {}. Check path with `specite path {} spec`, then tell the user to run `/spec` to create SPEC.md first.",
            display_path_with_base(&spec_path, project_root),
            iter_id
        )));
    }

    let create_plan_prompt = render_template(
        assets::load_subagent_prompt("create-plan.md").expect("embedded create-plan.md missing"),
        &[
            ("plan_path", &display_path(&plan_path)),
            ("spec_path", &display_path(&spec_path)),
            ("iter_id", &resolved),
        ],
    );
    let exec_phase_prompt = render_template(
        assets::load_subagent_prompt("exec-phase.md").expect("embedded exec-phase.md missing"),
        &[
            ("plan_path", &display_path(&plan_path)),
            ("spec_path", &display_path(&spec_path)),
        ],
    );

    Ok(render_template(
        assets::load_command_prompt("iter.md").expect("embedded iter.md prompt missing"),
        &[
            ("plan_path", &display_path(&plan_path)),
            ("spec_path", &display_path(&spec_path)),
            ("iter_id", &resolved),
            ("create_plan_prompt", &create_plan_prompt),
            ("exec_phase_prompt", &exec_phase_prompt),
        ],
    ))
}

pub fn generate_exec_prompt(project_root: &Path, iter_id: &str) -> Result<String, PromptError> {
    let manager = IterManager::new(project_root);
    let resolved = manager
        .resolve_iteration_id(iter_id)
        .map_err(|e| PromptError::new(e.to_string()))?;
    let iteration_path = manager.iterations_dir.join(&resolved);
    let spec_path = iteration_path.join("SPEC.md");
    let plan_path = iteration_path.join("PLAN.md");

    if !plan_path.exists() {
        return Err(PromptError::new(format!(
            "PLAN.md not found at {}. Check path with `specite path {} plan`, then tell the user to run `/plan` to create PLAN.md first.",
            display_path_with_base(&plan_path, project_root),
            iter_id
        )));
    }

    let exec_phase_prompt = render_template(
        assets::load_subagent_prompt("exec-phase.md").expect("embedded exec-phase.md missing"),
        &[
            ("plan_path", &display_path(&plan_path)),
            ("spec_path", &display_path(&spec_path)),
        ],
    );

    Ok(render_template(
        assets::load_command_prompt("exec.md").expect("embedded exec.md prompt missing"),
        &[
            ("plan_path", &display_path(&plan_path)),
            ("spec_path", &display_path(&spec_path)),
            ("iter_id", &resolved),
            ("exec_phase_prompt", &exec_phase_prompt),
        ],
    ))
}

pub fn generate_post_prompt(project_root: &Path, iter_id: &str) -> Result<String, PromptError> {
    let manager = IterManager::new(project_root);
    let resolved = manager
        .resolve_iteration_id(iter_id)
        .map_err(|e| PromptError::new(e.to_string()))?;
    let iteration_path = manager.iterations_dir.join(&resolved);
    let spec_path = iteration_path.join("SPEC.md");

    let git_status = git_output(project_root, &["status", "--short"]);
    let git_diff = git_output(project_root, &["diff", "--stat"]);

    Ok(render_template(
        assets::load_command_prompt("post.md").expect("embedded post.md prompt missing"),
        &[
            ("spec_path", &display_path(&spec_path)),
            ("git_status", &git_status),
            ("git_diff", &git_diff),
            (
                "finished_path",
                &display_path(&iteration_path.join("FINISHED.md")),
            ),
            ("iter_id", &resolved),
        ],
    ))
}

pub fn generate_spec_prompt(project_root: &Path) -> String {
    let agentsmd_step = if !project_root.join("AGENTS.md").is_file() {
        "\n\n6. Create a minimal AGENTS.md:\n   - Do not use `agents-md` skill or other similar skills, create directly with only following two components:\n   - Project overview (by iteration goal and current status)\n   - Tech stack"
    } else {
        ""
    };

    render_template(
        assets::load_command_prompt("spec.md").expect("embedded spec.md prompt missing"),
        &[
            ("agentsmd_step", agentsmd_step),
            (
                "research_prompt",
                assets::load_subagent_prompt("research.md").expect("embedded research.md missing"),
            ),
        ],
    )
}
