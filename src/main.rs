//! Console entrypoint for the Specite CLI.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use specite::error::PromptError;
use specite::init;
use specite::iterations::IterManager;
use specite::project::{display_path, find_project_root};
use specite::prompts::{
    generate_exec_prompt, generate_iter_prompt, generate_plan_prompt, generate_post_prompt,
    generate_spec_prompt,
};

/// Spec-driven iterative development companion CLI.
#[derive(Debug, Parser)]
#[command(name = "specite", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize Specite in a project.
    Init {
        /// Project path (defaults to current directory).
        path: Option<String>,
    },
    /// Create a new iteration.
    New {
        /// Iteration name.
        name: String,
    },
    /// List iterations.
    List {
        /// Limit to the top N iterations.
        limit: Option<u32>,
    },
    /// Update an iteration stage.
    Update {
        /// Iteration ID or name (1 = most recent).
        iter_id: String,
        /// New stage.
        stage: String,
    },
    /// Generate a prompt.
    Prompt {
        /// Iteration ID/name or 'spec'.
        target: String,
        /// Prompt kind.
        kind: Option<String>,
    },
    /// Print the absolute or cwd-relative path of an iteration file (hidden helper).
    #[command(hide = true)]
    Path { iter_id: String, kind: String },
    /// Print the current stage of an iteration (hidden helper).
    #[command(hide = true)]
    Status { iter_id: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let code = run(cli).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        1
    });
    std::process::exit(code);
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    let manager_root = || -> anyhow::Result<IterManager> {
        let root = find_project_root(None)?;
        Ok(IterManager::new(&root))
    };

    match cli.command {
        Command::Init { path } => handle_init(path),
        Command::New { name } => {
            let m = manager_root()?;
            let (name, dir) = m.create_iteration(&name)?;
            println!("Created iteration: {name}");
            println!("Directory: {}", display_path(&dir));
            Ok(0)
        }
        Command::List { limit } => {
            let m = manager_root()?;
            let iterations = m.list_iterations(limit.map(|n| n as usize))?;
            if iterations.is_empty() {
                println!("No iterations found");
                return Ok(0);
            }
            for (i, iter) in iterations.iter().enumerate() {
                let name = iter.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let stage = iter.get("stage").and_then(|v| v.as_str()).unwrap_or("?");
                println!("{}. {} ({})", i + 1, name, stage);
            }
            Ok(0)
        }
        Command::Update { iter_id, stage } => {
            let m = manager_root()?;
            m.update_iteration_stage(&iter_id, &stage)?;
            println!("Updated iteration {iter_id} stage to: {stage}");
            Ok(0)
        }
        Command::Prompt { target, kind } => {
            let root = find_project_root(None)?;
            handle_prompt(&root, &target, kind.as_deref())
        }
        Command::Path { iter_id, kind } => {
            let m = manager_root()?;
            let target = match kind.as_str() {
                "spec" => m.get_spec_path(&iter_id)?,
                "plan" => m.get_plan_path(&iter_id)?,
                _ => anyhow::bail!("unknown path kind '{kind}'"),
            };
            println!("{}", display_path(&target));
            Ok(0)
        }
        Command::Status { iter_id } => {
            let m = manager_root()?;
            let stage = m.get_iteration_stage(&iter_id)?;
            println!("{stage}");
            Ok(0)
        }
    }
}

fn handle_init(path: Option<String>) -> anyhow::Result<i32> {
    let target = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let result = init::initialize_project(&target)?;
    println!(
        "Initialized Specite in {}",
        display_path(&result.project_root)
    );
    println!("Managed commands: {}", display_path(&result.commands_dir));
    println!("Managed agents: {}", display_path(&result.agents_dir));

    if result.removed_legacy_scripts {
        let legacy = result
            .legacy_scripts_dir
            .unwrap_or_else(|| result.project_root.join(".opencode").join("scripts"));
        println!(
            "Removed legacy helper scripts from {}",
            display_path(&legacy)
        );
    }

    for warning in &result.warnings {
        eprintln!("Warning: {warning}");
    }
    Ok(0)
}

fn handle_prompt(root: &std::path::Path, target: &str, kind: Option<&str>) -> anyhow::Result<i32> {
    if target == "spec" {
        if kind.is_some() {
            return Err(
                PromptError::new("`specite prompt spec` does not take a prompt kind").into(),
            );
        }
        let prompt = generate_spec_prompt(root);
        print!("{prompt}");
        Ok(0)
    } else {
        let kind = kind.ok_or_else(|| {
            PromptError::new("Prompt kind is required: plan, iter, exec, or post")
        })?;
        let prompt = match kind {
            "plan" => generate_plan_prompt(root, target)?,
            "iter" => generate_iter_prompt(root, target)?,
            "exec" => generate_exec_prompt(root, target)?,
            "post" => generate_post_prompt(root, target)?,
            other => {
                return Err(PromptError::new(format!("Unsupported prompt kind '{other}'")).into());
            }
        };
        print!("{prompt}");
        Ok(0)
    }
}
