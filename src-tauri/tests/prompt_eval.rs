//! Integration test that evaluates the production STT-postprocessing prompts
//! against a 200-case dataset by hitting the real OpenAI API. See
//! `docs/feature-prompt-eval-suite.md` for design rationale.
//!
//! The full eval is `#[ignore]` because it (a) costs real money, (b) needs an
//! `api_key` configured in the app's config.toml, and (c) is non-deterministic
//! enough that we don't want it gating CI. Run with:
//!
//!     cargo test --test prompt_eval -- --include-ignored full_prompt_eval --nocapture

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EvalCase {
    pub id: String,
    pub scenario: String,         // "mixed" | "stutter" | "structure"
    pub language: String,         // "zh" | "en"
    pub text_structuring: bool,
    pub input: String,
    pub checks: Checks,
    #[serde(default)]
    pub judge_rubric: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Checks {
    #[serde(default)]
    pub must_contain: Vec<String>,
    #[serde(default)]
    pub must_not_contain: Vec<String>,
    #[serde(default)]
    pub must_match_regex: Vec<String>,
    #[serde(default)]
    pub no_markdown: bool,
    #[serde(default = "default_form")]
    pub form: Form,
    #[serde(default)]
    pub min_chars: Option<usize>,
    #[serde(default)]
    pub max_chars: Option<usize>,
}

fn default_form() -> Form { Form::Auto }

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    Plain,         // must NOT look like a numbered list
    NumberedList,  // must contain "\n1. " AND "\n2. " (≥2 items)
    Auto,          // no form check
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicResult {
    pub pass: bool,
    pub failed_checks: Vec<String>,
}

/// Run all heuristic checks declared in `checks` against `output`. Returns a
/// list of human-readable failure descriptions (empty Vec = all checks passed).
pub fn run_heuristics(output: &str, checks: &Checks) -> HeuristicResult {
    let mut failed = Vec::new();

    for needle in &checks.must_contain {
        if !output.contains(needle.as_str()) {
            failed.push(format!("must_contain '{}' missing", needle));
        }
    }

    for forbidden in &checks.must_not_contain {
        if output.contains(forbidden.as_str()) {
            failed.push(format!("must_not_contain '{}' present", forbidden));
        }
    }

    for pat in &checks.must_match_regex {
        match regex::Regex::new(pat) {
            Ok(re) => {
                if !re.is_match(output) {
                    failed.push(format!("must_match_regex '{}' did not match", pat));
                }
            }
            Err(e) => {
                failed.push(format!("invalid regex '{}': {}", pat, e));
            }
        }
    }

    if checks.no_markdown && contains_markdown(output) {
        failed.push("no_markdown violated (found markdown syntax)".to_string());
    }

    match checks.form {
        Form::Plain => {
            if looks_like_numbered_list(output) {
                failed.push("form=plain but output looks like a numbered list".to_string());
            }
        }
        Form::NumberedList => {
            if !looks_like_numbered_list(output) {
                failed.push("form=numbered_list but output is not a numbered list".to_string());
            }
        }
        Form::Auto => {}
    }

    if let Some(min) = checks.min_chars {
        if output.chars().count() < min {
            failed.push(format!("min_chars={} but output has {} chars", min, output.chars().count()));
        }
    }
    if let Some(max) = checks.max_chars {
        if output.chars().count() > max {
            failed.push(format!("max_chars={} but output has {} chars", max, output.chars().count()));
        }
    }

    HeuristicResult {
        pass: failed.is_empty(),
        failed_checks: failed,
    }
}

/// Detect markdown syntax that the prompt explicitly forbids in plain mode.
/// Numbered lists ("1. foo") are NOT counted here — that's the form check's job.
fn contains_markdown(text: &str) -> bool {
    // Heading
    if text.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("# ") || t.starts_with("## ") || t.starts_with("### ")
    }) { return true; }
    // Bullet list
    if text.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ")
    }) { return true; }
    // Code fence
    if text.contains("```") { return true; }
    // Bold/italic markers around words (rough heuristic)
    if text.contains("**") || text.contains("__") { return true; }
    false
}

/// True iff the output looks like a numbered list per our prompt rules:
/// has at least two of "1. ", "2. ", "3. " at line starts.
fn looks_like_numbered_list(text: &str) -> bool {
    let mut count = 0;
    for marker in ["1. ", "2. ", "3. ", "4. ", "5. "] {
        if text.contains(&format!("\n{}", marker))
            || text.starts_with(marker)
        {
            count += 1;
        }
    }
    count >= 2
}

#[cfg(test)]
mod heuristic_tests {
    use super::*;

    fn checks_with_must_contain(needles: &[&str]) -> Checks {
        Checks {
            must_contain: needles.iter().map(|s| s.to_string()).collect(),
            must_not_contain: vec![],
            must_match_regex: vec![],
            no_markdown: false,
            form: Form::Auto,
            min_chars: None,
            max_chars: None,
        }
    }

    #[test]
    fn must_contain_passes_when_all_present() {
        let c = checks_with_must_contain(&["React", "API"]);
        let r = run_heuristics("we use React with the API layer", &c);
        assert!(r.pass);
        assert!(r.failed_checks.is_empty());
    }

    #[test]
    fn must_contain_fails_with_missing_needle() {
        let c = checks_with_must_contain(&["React", "API"]);
        let r = run_heuristics("we use React only", &c);
        assert!(!r.pass);
        assert_eq!(r.failed_checks.len(), 1);
        assert!(r.failed_checks[0].contains("API"));
    }

    #[test]
    fn must_not_contain_fails_when_forbidden_present() {
        let mut c = checks_with_must_contain(&[]);
        c.must_not_contain = vec!["呃".to_string()];
        let r = run_heuristics("呃我觉得吧", &c);
        assert!(!r.pass);
        assert!(r.failed_checks[0].contains("呃"));
    }

    #[test]
    fn no_markdown_catches_headings_bullets_fences_bold() {
        let mut c = checks_with_must_contain(&[]);
        c.no_markdown = true;
        for bad in ["# heading", "- bullet", "```code", "**bold**"] {
            let r = run_heuristics(bad, &c);
            assert!(!r.pass, "should flag markdown in: {bad}");
        }
    }

    #[test]
    fn no_markdown_allows_clean_prose() {
        let mut c = checks_with_must_contain(&[]);
        c.no_markdown = true;
        let r = run_heuristics("这是一段正常的文本，包含标点。Just text.", &c);
        assert!(r.pass);
    }

    #[test]
    fn form_numbered_list_requires_two_items() {
        let mut c = checks_with_must_contain(&[]);
        c.form = Form::NumberedList;
        let r = run_heuristics("总起句\n1. 第一点\n2. 第二点\n3. 第三点", &c);
        assert!(r.pass);

        let r2 = run_heuristics("just one line, no list", &c);
        assert!(!r2.pass);
    }

    #[test]
    fn form_plain_rejects_numbered_list() {
        let mut c = checks_with_must_contain(&[]);
        c.form = Form::Plain;
        let r = run_heuristics("总起句\n1. 第一点\n2. 第二点", &c);
        assert!(!r.pass);
    }

    #[test]
    fn must_match_regex_works() {
        let mut c = checks_with_must_contain(&[]);
        c.must_match_regex = vec![r"\d+%".to_string()];
        assert!(run_heuristics("增长 15% 用户", &c).pass);
        assert!(!run_heuristics("增长百分之十五", &c).pass);
    }
}
