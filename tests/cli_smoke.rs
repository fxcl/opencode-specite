//! End-to-end CLI smoke tests using `assert_cmd`.
//!
//! Each test stages a fresh tempdir, runs the binary as a subprocess, and
//! asserts on the resulting filesystem state.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn specite() -> Command {
    Command::cargo_bin("specite").expect("specite binary")
}

/// Initialize a fresh project in `dir` and run the given closure after `cd`.
fn init_project(dir: &TempDir) -> &std::path::Path {
    specite().arg("init").arg(dir.path()).assert().success();
    dir.path()
}

#[test]
fn init_creates_specite_state_and_assets() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let root = dir.path();
    assert!(root.join(".specite").is_dir());
    assert!(root.join(".specite/iterations").is_dir());
    assert!(root.join(".specite/docs").is_dir());
    assert!(root.join(".specite/templates").is_dir());
    assert!(root.join(".specite/templates/SPEC.md").is_file());
    assert!(root.join(".specite/templates/PLAN.md").is_file());
    assert_eq!(
        fs::read_to_string(root.join(".specite/iters.json")).unwrap(),
        "{\"iterations\": []}\n"
    );

    // Commands
    for f in ["exec.md", "iter.md", "plan.md", "post.md", "spec.md"] {
        assert!(root.join(".opencode/commands").join(f).is_file(), "{f}");
    }
    // Agents
    for f in ["phase-executor.md", "plan-creator.md", "web-researcher.md"] {
        assert!(root.join(".opencode/agents").join(f).is_file(), "{f}");
    }

    // gitignore entries
    let gi = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(gi.contains(".opencode/commands/"));
    assert!(gi.contains(".opencode/agents/"));
}

#[test]
fn init_removes_obsolete_command_files() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".opencode/commands")).unwrap();
    fs::write(root.join(".opencode/commands/agentsmd.md"), "old").unwrap();
    fs::write(root.join(".opencode/commands/list-iters.md"), "old").unwrap();

    specite().arg("init").arg(root).assert().success();

    assert!(!root.join(".opencode/commands/agentsmd.md").exists());
    assert!(!root.join(".opencode/commands/list-iters.md").exists());
}

#[test]
fn init_cleans_legacy_scripts_dir() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let scripts = root.join(".opencode").join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("iter_manager.py"), "# old").unwrap();

    let output = specite()
        .current_dir(root)
        .arg("init")
        .arg(".")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        !scripts.exists(),
        "legacy scripts dir should be removed when fully legacy"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Removed legacy helper scripts"),
        "stdout: {stdout}"
    );
}

#[test]
fn init_warns_on_legacy_scripts_with_unknown_files() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let scripts = root.join(".opencode").join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("user_helper.py"), "# keep me").unwrap();

    let output = specite()
        .current_dir(root)
        .arg("init")
        .arg(".")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        scripts.exists(),
        "scripts dir should be kept when unknowns present"
    );
    assert!(scripts.join("user_helper.py").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unmanaged files"), "stderr: {stderr}");
}

#[test]
fn init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    // Run again; should still succeed and overwrite assets.
    specite().arg("init").arg(dir.path()).assert().success();
}

#[test]
fn new_creates_iteration_and_lists_it() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    specite()
        .current_dir(dir.path())
        .arg("new")
        .arg("My Iteration!!")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created iteration: my-iteration"));

    let iters = dir.path().join(".specite/iters.json");
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&iters).unwrap()).unwrap();
    let arr = parsed.get("iterations").and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "my-iteration");
    assert_eq!(arr[0]["stage"], "new");

    specite()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("1. my-iteration (new)"));
}

#[test]
fn update_changes_stage() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    specite()
        .current_dir(dir.path())
        .arg("new")
        .arg("a")
        .assert()
        .success();

    specite()
        .current_dir(dir.path())
        .args(["update", "1", "planned"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Updated iteration 1 stage to: planned",
        ));

    specite()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("(planned)"));
}

#[test]
fn update_rejects_invalid_stage() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    specite()
        .current_dir(dir.path())
        .arg("new")
        .arg("a")
        .assert()
        .success();

    specite()
        .current_dir(dir.path())
        .args(["update", "1", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid stage 'bogus'"));
}

#[test]
fn prompt_spec_emits_non_empty_output() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    specite()
        .current_dir(dir.path())
        .args(["prompt", "spec"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SPEC document"));
}

#[test]
fn prompt_plan_without_spec_fails() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    specite()
        .current_dir(dir.path())
        .arg("new")
        .arg("a")
        .assert()
        .success();

    specite()
        .current_dir(dir.path())
        .args(["prompt", "1", "plan"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SPEC.md not found"));
}

#[test]
fn prompt_plan_with_spec_succeeds() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    specite()
        .current_dir(dir.path())
        .arg("new")
        .arg("a")
        .assert()
        .success();
    let iter_dir = dir.path().join(".specite/iterations/a");
    fs::write(iter_dir.join("SPEC.md"), "# spec").unwrap();

    let output = specite()
        .current_dir(dir.path())
        .args(["prompt", "1", "plan"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PLAN.md"));
    assert!(stdout.contains("SPEC.md"));
}
