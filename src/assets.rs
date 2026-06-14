//! Compile-time embedded assets: commands, agents, templates, command prompts, subagent prompts.
//!
//! Each file under `assets/` is embedded via `include_str!` so the resulting binary is
//! fully self-contained. The `pub const` items expose individual files; the `load_*`
//! helpers resolve a file name within a group.

// --- commands ---
pub const CMD_EXEC: &str = include_str!("../assets/commands/exec.md");
pub const CMD_ITER: &str = include_str!("../assets/commands/iter.md");
pub const CMD_PLAN: &str = include_str!("../assets/commands/plan.md");
pub const CMD_POST: &str = include_str!("../assets/commands/post.md");
pub const CMD_SPEC: &str = include_str!("../assets/commands/spec.md");

// --- agents ---
pub const AGENT_PHASE_EXECUTOR: &str = include_str!("../assets/agents/phase-executor.md");
pub const AGENT_PLAN_CREATOR: &str = include_str!("../assets/agents/plan-creator.md");
pub const AGENT_WEB_RESEARCHER: &str = include_str!("../assets/agents/web-researcher.md");

// --- templates ---
pub const TPL_SPEC: &str = include_str!("../assets/templates/SPEC.md");
pub const TPL_PLAN: &str = include_str!("../assets/templates/PLAN.md");

// --- command_prompts ---
pub const PROMPT_EXEC: &str = include_str!("../assets/command_prompts/exec.md");
pub const PROMPT_ITER: &str = include_str!("../assets/command_prompts/iter.md");
pub const PROMPT_PLAN: &str = include_str!("../assets/command_prompts/plan.md");
pub const PROMPT_POST: &str = include_str!("../assets/command_prompts/post.md");
pub const PROMPT_SPEC: &str = include_str!("../assets/command_prompts/spec.md");

// --- subagent_prompts ---
pub const SUB_CREATE_PLAN: &str = include_str!("../assets/subagent_prompts/create-plan.md");
pub const SUB_EXEC_PHASE: &str = include_str!("../assets/subagent_prompts/exec-phase.md");
pub const SUB_RESEARCH: &str = include_str!("../assets/subagent_prompts/research.md");

/// Resolve a command asset by file name (with leading `./commands/<name>` semantics).
pub fn load_command(name: &str) -> Option<&'static str> {
    match name {
        "exec.md" => Some(CMD_EXEC),
        "iter.md" => Some(CMD_ITER),
        "plan.md" => Some(CMD_PLAN),
        "post.md" => Some(CMD_POST),
        "spec.md" => Some(CMD_SPEC),
        _ => None,
    }
}

/// Resolve an agent asset by file name.
pub fn load_agent(name: &str) -> Option<&'static str> {
    match name {
        "phase-executor.md" => Some(AGENT_PHASE_EXECUTOR),
        "plan-creator.md" => Some(AGENT_PLAN_CREATOR),
        "web-researcher.md" => Some(AGENT_WEB_RESEARCHER),
        _ => None,
    }
}

/// Resolve a template asset by file name.
pub fn load_template(name: &str) -> Option<&'static str> {
    match name {
        "SPEC.md" => Some(TPL_SPEC),
        "PLAN.md" => Some(TPL_PLAN),
        _ => None,
    }
}

/// Resolve a command prompt asset by file name.
///
/// Mirrors Python's `_load_prompt` which calls `.strip("\n")` on the loaded content.
pub fn load_command_prompt(name: &str) -> Option<&'static str> {
    let raw = match name {
        "exec.md" => PROMPT_EXEC,
        "iter.md" => PROMPT_ITER,
        "plan.md" => PROMPT_PLAN,
        "post.md" => PROMPT_POST,
        "spec.md" => PROMPT_SPEC,
        _ => return None,
    };
    Some(raw.trim_matches('\n'))
}

/// Resolve a subagent prompt asset by file name.
///
/// Mirrors Python's `_load_prompt` which calls `.strip("\n")` on the loaded content.
pub fn load_subagent_prompt(name: &str) -> Option<&'static str> {
    let raw = match name {
        "create-plan.md" => SUB_CREATE_PLAN,
        "exec-phase.md" => SUB_EXEC_PHASE,
        "research.md" => SUB_RESEARCH,
        _ => return None,
    };
    Some(raw.trim_matches('\n'))
}
