//! Iteration metadata management: kebab-case names, JSON-backed `iters.json`.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, Timelike};
use regex::Regex;
use serde_json::{Map, Value};

use crate::error::ValidationError;

pub const VALID_STAGES: &[&str] = &["new", "specified", "planned", "executed", "completed"];

/// Convert arbitrary text to lowercase kebab-case.
///
/// Mirrors Python's `re.sub(r"[^a-zA-Z0-9]+", "-", name).strip("-").lower()`.
pub fn to_kebab_case(name: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]+").expect("static regex");
    let normalized = re.replace_all(name, "-");
    normalized.trim_matches('-').to_lowercase()
}

/// Manager for Specite iteration metadata and path resolution.
pub struct IterManager {
    pub project_root: PathBuf,
    pub specite_dir: PathBuf,
    pub iters_file: PathBuf,
    pub iterations_dir: PathBuf,
}

impl IterManager {
    pub fn new(project_root: &Path) -> Self {
        let root = absolutize(project_root);
        let specite = root.join(".specite");
        Self {
            project_root: root,
            specite_dir: specite.clone(),
            iters_file: specite.join("iters.json"),
            iterations_dir: specite.join("iterations"),
        }
    }

    /// Load the raw `iters.json` document. Returns an empty document if missing.
    pub fn load_iters(&self) -> Value {
        if self.iters_file.exists() {
            let text = fs::read_to_string(&self.iters_file).expect("failed to read iters.json");
            serde_json::from_str(&text).expect("iters.json is not valid JSON")
        } else {
            Value::Object(Map::new())
        }
    }

    /// Persist the `iters.json` document with 4-space indentation and a trailing newline,
    /// matching Python's `json.dumps(data, indent=4) + "\n"`.
    pub fn save_iters(&self, data: &Value) {
        fs::create_dir_all(&self.specite_dir).expect("failed to create .specite");
        fs::create_dir_all(&self.iterations_dir).expect("failed to create .specite/iterations");
        let text = to_pretty_json_4space(data);
        fs::write(&self.iters_file, text).expect("failed to write iters.json");
    }

    pub fn verify_iters_file_exists(&self) -> Result<(), std::io::Error> {
        if !self.iters_file.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "iters.json not found. Run `specite init` to initialize the project.",
            ));
        }
        Ok(())
    }

    /// Resolve a numeric 1-based id or existing iteration name to its kebab name.
    pub fn resolve_iteration_id(&self, iter_id: &str) -> Result<String, ValidationError> {
        self.verify_iters_file_exists()
            .map_err(|e| ValidationError::new(e.to_string()))?;
        let iterations = self.iterations_array();

        let numeric_id = match iter_id.parse::<i64>() {
            Ok(n) => n,
            Err(_) => {
                if iterations
                    .iter()
                    .any(|item| item.get("name").and_then(Value::as_str) == Some(iter_id))
                {
                    return Ok(iter_id.to_string());
                }
                return Err(ValidationError::new(format!(
                    "Invalid iteration ID '{iter_id}': must be a number >= 1 or an existing iteration name"
                )));
            }
        };

        if numeric_id < 1 {
            return Err(ValidationError::new(format!(
                "Iteration ID must be >= 1, got {numeric_id}"
            )));
        }
        if numeric_id > iterations.len() as i64 {
            return Err(ValidationError::new(format!(
                "Iteration #{numeric_id} not found (only {} iterations)",
                iterations.len()
            )));
        }

        let entry = &iterations[(numeric_id - 1) as usize];
        Ok(entry
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string())
    }

    /// Create a new iteration with kebab-normalized name.
    ///
    /// Returns `(kebab_name, iteration_dir)`.
    pub fn create_iteration(&self, name: &str) -> Result<(String, PathBuf), ValidationError> {
        let kebab = to_kebab_case(name);
        if kebab.is_empty() {
            return Err(ValidationError::new(format!(
                "Invalid iteration name '{name}'"
            )));
        }

        let mut data = self.load_iters();
        let iterations = data
            .as_object_mut()
            .and_then(|m| m.get_mut("iterations"))
            .and_then(Value::as_array_mut)
            .expect("iters.json must contain an 'iterations' array");

        if iterations
            .iter()
            .any(|item| item.get("name").and_then(Value::as_str) == Some(&kebab))
        {
            return Err(ValidationError::new(format!(
                "Iteration '{kebab}' already exists"
            )));
        }

        let iteration_dir = self.iterations_dir.join(&kebab);
        fs::create_dir_all(&iteration_dir).expect("failed to create iteration dir");

        let mut entry = Map::new();
        entry.insert("time".into(), Value::String(now_iso_like_python()));
        entry.insert("name".into(), Value::String(kebab.clone()));
        entry.insert("stage".into(), Value::String("new".into()));
        iterations.push(Value::Object(entry));

        sort_iterations(iterations);
        self.save_iters(&data);
        Ok((kebab, iteration_dir))
    }

    /// List all iterations (already sorted by time desc). Optionally truncate.
    pub fn list_iterations(&self, limit: Option<usize>) -> Result<Vec<Value>, ValidationError> {
        self.verify_iters_file_exists()
            .map_err(|e| ValidationError::new(e.to_string()))?;
        let iterations = self.iterations_array();
        Ok(match limit {
            Some(n) => iterations.into_iter().take(n).collect(),
            None => iterations,
        })
    }

    pub fn get_iteration_path(&self, iter_id: &str) -> Result<PathBuf, ValidationError> {
        Ok(self
            .iterations_dir
            .join(self.resolve_iteration_id(iter_id)?))
    }

    pub fn get_spec_path(&self, iter_id: &str) -> Result<PathBuf, ValidationError> {
        Ok(self.get_iteration_path(iter_id)?.join("SPEC.md"))
    }

    pub fn get_plan_path(&self, iter_id: &str) -> Result<PathBuf, ValidationError> {
        Ok(self.get_iteration_path(iter_id)?.join("PLAN.md"))
    }

    pub fn get_iteration_stage(&self, iter_id: &str) -> Result<String, ValidationError> {
        let resolved = self.resolve_iteration_id(iter_id)?;
        for iteration in self.iterations_array() {
            if iteration.get("name").and_then(Value::as_str) == Some(&resolved) {
                if let Some(stage) = iteration.get("stage").and_then(Value::as_str) {
                    return Ok(stage.to_string());
                }
            }
        }
        Err(ValidationError::new(format!(
            "Iteration '{iter_id}' not found"
        )))
    }

    /// Update the stage of an existing iteration. Re-sorts and saves.
    pub fn update_iteration_stage(
        &self,
        iter_id: &str,
        stage: &str,
    ) -> Result<(), ValidationError> {
        self.verify_iters_file_exists()
            .map_err(|e| ValidationError::new(e.to_string()))?;
        if !VALID_STAGES.contains(&stage) {
            return Err(ValidationError::new(format!(
                "Invalid stage '{stage}'. Valid stages: {}",
                VALID_STAGES.join(", ")
            )));
        }

        let resolved = self.resolve_iteration_id(iter_id)?;
        let mut data = self.load_iters();
        let iterations = data
            .as_object_mut()
            .and_then(|m| m.get_mut("iterations"))
            .and_then(Value::as_array_mut)
            .expect("iters.json must contain an 'iterations' array");

        let mut found = false;
        for iteration in iterations.iter_mut() {
            if iteration.get("name").and_then(Value::as_str) == Some(&resolved) {
                if let Some(obj) = iteration.as_object_mut() {
                    obj.insert("stage".into(), Value::String(stage.into()));
                    obj.insert("time".into(), Value::String(now_iso_like_python()));
                }
                found = true;
                break;
            }
        }
        if !found {
            return Err(ValidationError::new(format!(
                "Iteration '{iter_id}' not found"
            )));
        }

        sort_iterations(iterations);
        self.save_iters(&data);
        Ok(())
    }

    fn iterations_array(&self) -> Vec<Value> {
        self.load_iters()
            .get("iterations")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

/// Produce a local naive ISO-8601 timestamp, matching Python's `datetime.now().isoformat()`.
///
/// Python's naive `datetime.now()` uses local time without timezone suffix and omits
/// microseconds when they are zero.
fn now_iso_like_python() -> String {
    let now = Local::now().naive_local();
    if now.nanosecond() == 0 {
        now.format("%Y-%m-%dT%H:%M:%S").to_string()
    } else {
        now.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
    }
}

fn sort_iterations(iterations: &mut [Value]) {
    iterations.sort_by(|a, b| {
        let a_time = a.get("time").and_then(Value::as_str).unwrap_or("");
        let b_time = b.get("time").and_then(Value::as_str).unwrap_or("");
        b_time.cmp(a_time)
    });
}

/// Serialize a JSON value with 4-space indentation, matching Python's `json.dumps(indent=4)`.
fn to_pretty_json_4space(value: &Value) -> String {
    let mut buf = Vec::new();
    let indent = vec![b' '; 4];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    use serde::Serialize;
    value.serialize(&mut ser).expect("failed to serialize JSON");
    String::from_utf8(buf).expect("JSON should be UTF-8") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_basic() {
        assert_eq!(to_kebab_case("Hello World!"), "hello-world");
    }

    #[test]
    fn kebab_strips_edges() {
        assert_eq!(to_kebab_case("---foo bar---"), "foo-bar");
    }

    #[test]
    fn kebab_preserves_digits() {
        assert_eq!(to_kebab_case("v2 release 42"), "v2-release-42");
    }

    #[test]
    fn kebab_all_digits() {
        assert_eq!(to_kebab_case("123"), "123");
    }

    #[test]
    fn kebab_empty() {
        assert_eq!(to_kebab_case("!!!"), "");
    }

    #[test]
    fn kebab_collapse_runs() {
        assert_eq!(to_kebab_case("a   b"), "a-b");
    }
}
