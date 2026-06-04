use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestEnv {
    root: PathBuf,
    home: PathBuf,
    xdg_config_home: PathBuf,
}

impl TestEnv {
    fn new(test_name: &str) -> Self {
        let unique = format!(
            "proompt-cli-{}-{}-{}",
            test_name,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let home = root.join("home");
        let xdg_config_home = root.join("xdg-config");
        std::fs::create_dir_all(&home).expect("create temp home");
        std::fs::create_dir_all(&xdg_config_home).expect("create temp xdg config home");

        Self {
            root,
            home,
            xdg_config_home,
        }
    }

    fn history_path(&self) -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            self.home
                .join("Library")
                .join("Application Support")
                .join("proompt")
                .join("history.json")
        }

        #[cfg(target_os = "windows")]
        {
            self.xdg_config_home.join("proompt").join("history.json")
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            self.xdg_config_home.join("proompt").join("history.json")
        }
    }

    fn write_history_fixture(&self) {
        let path = self.history_path();
        std::fs::create_dir_all(path.parent().expect("history path should have parent"))
            .expect("create history dir");
        std::fs::write(
            path,
            r#"[
  {
    "id": "hist_test",
    "original_prompt": "rough prompt",
    "enhanced_prompt": "enhanced prompt",
    "enhancement_type": "text",
    "platform": "claude",
    "provider": "openai",
    "model": "gpt-4o",
    "created_at_ms": 1700000000000,
    "favorite": false
  }
]"#,
        )
        .expect("write history fixture");
    }

    fn proompt(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_proompt"));
        command
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.xdg_config_home)
            .env("APPDATA", &self.xdg_config_home)
            .env("NO_COLOR", "1")
            .env("PROOMPT_DISABLE_KEYCHAIN", "1")
            .env_remove("OPENAI_API_KEY")
            .env_remove("ANTHROPIC_API_KEY")
            .env_remove("GEMINI_API_KEY")
            .env_remove("GOOGLE_API_KEY")
            .env_remove("OPENROUTER_API_KEY")
            .env_remove("SUPERMEMORY_API_KEY");
        command
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn config_provider_switch_to_openrouter_sets_default_model() {
    let env = TestEnv::new("openrouter-provider");

    assert_success(
        env.proompt()
            .args(["config", "set", "byok.provider", "openrouter"])
            .output()
            .unwrap(),
    );

    let output = env.proompt().args(["config", "show"]).output().unwrap();
    assert_success(output.clone());
    let stderr = stderr(&output);

    assert!(stderr.contains("provider:"), "stderr was:\n{stderr}");
    assert!(stderr.contains("openrouter"), "stderr was:\n{stderr}");
    assert!(
        stderr.contains("openai/gpt-4o-mini"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn config_provider_alias_is_normalized() {
    let env = TestEnv::new("provider-alias");

    assert_success(
        env.proompt()
            .args(["config", "set", "byok.provider", "claude"])
            .output()
            .unwrap(),
    );

    let output = env.proompt().args(["config", "show"]).output().unwrap();
    assert_success(output.clone());
    let stderr = stderr(&output);

    assert!(stderr.contains("anthropic"), "stderr was:\n{stderr}");
    assert!(
        stderr.contains("claude-sonnet-4-20250514"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn config_save_history_preference_can_be_disabled() {
    let env = TestEnv::new("save-history-preference");

    assert_success(
        env.proompt()
            .args(["config", "set", "preferences.save_history", "false"])
            .output()
            .unwrap(),
    );

    let output = env.proompt().args(["config", "show"]).output().unwrap();
    assert_success(output.clone());
    let stderr = stderr(&output);

    assert!(stderr.contains("save history:"), "stderr was:\n{stderr}");
    assert!(stderr.contains("disabled"), "stderr was:\n{stderr}");
}

#[test]
fn config_rejects_image_platform_as_text_default() {
    let env = TestEnv::new("invalid-text-platform");

    let output = env
        .proompt()
        .args(["config", "set", "default_platform", "midjourney"])
        .output()
        .unwrap();

    assert_failure(output.clone());
    let stderr = stderr(&output);
    assert!(
        stderr.contains("Invalid default platform"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn history_list_shows_saved_records() {
    let env = TestEnv::new("history-list");
    env.write_history_fixture();

    let output = env.proompt().args(["history", "list"]).output().unwrap();

    assert_success(output.clone());
    let stderr = stderr(&output);
    assert!(stderr.contains("hist_test"), "stderr was:\n{stderr}");
    assert!(stderr.contains("rough prompt"), "stderr was:\n{stderr}");
    assert!(stderr.contains("enhanced prompt"), "stderr was:\n{stderr}");
}

#[test]
fn history_favorite_and_delete_update_saved_records() {
    let env = TestEnv::new("history-favorite-delete");
    env.write_history_fixture();

    assert_success(
        env.proompt()
            .args(["history", "favorite", "hist_test"])
            .output()
            .unwrap(),
    );
    let content = std::fs::read_to_string(env.history_path()).unwrap();
    assert!(
        content.contains("\"favorite\": true"),
        "history was:\n{content}"
    );

    assert_success(
        env.proompt()
            .args(["history", "delete", "hist_test"])
            .output()
            .unwrap(),
    );
    let content = std::fs::read_to_string(env.history_path()).unwrap();
    assert!(!content.contains("hist_test"), "history was:\n{content}");
}

#[test]
fn enhance_missing_openrouter_key_prints_actionable_message() {
    let env = TestEnv::new("missing-openrouter-key");

    assert_success(
        env.proompt()
            .args(["config", "set", "byok.provider", "openrouter"])
            .output()
            .unwrap(),
    );

    let output = env.proompt().arg("make this clearer").output().unwrap();

    assert_failure(output.clone());
    let stderr = stderr(&output);
    assert!(
        stderr.contains("openrouter.api_key"),
        "stderr was:\n{stderr}"
    );
    assert!(
        stderr.contains("OPENROUTER_API_KEY"),
        "stderr was:\n{stderr}"
    );
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "expected success\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout(&output),
        stderr(&output)
    );
}

fn assert_failure(output: Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout(&output),
        stderr(&output)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}
