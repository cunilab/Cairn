//! Alpha 9's installed edge journey: setup links a canonical remote, two
//! callers remain distinct, and an accepted explicit memory is retrievable.

use cairn_e2e::feature005::Pg;
use cairn_e2e::{post_json_status_bearer, Mcp, Sandbox};
use serde_json::json;
use uuid::Uuid;

#[test]
fn installed_setup_remembers_and_recalls_across_callers() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let remote = format!(
        "https://github.com/example/alpha9-journey-{}.git",
        Uuid::now_v7()
    );
    let (project, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/projects",
        &json!({
            "name": "alpha9 installed journey",
            "repository_remote": remote,
        }),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "creating the linked project: {project}");

    let sandbox = Sandbox::new();
    sandbox.install_agent("codex");
    sandbox.install_agent("claude-code");
    sandbox.git(&["remote", "set-url", "origin", &remote]);
    let setup = sandbox.cairn_with_env(
        &["--json", "setup"],
        &[
            ("CAIRN_SERVER_URL", &pg.server.base),
            ("CAIRN_SERVER_TOKEN", &pg.owner.token),
        ],
    );
    assert!(setup.ok(), "setup failed: {}", setup.stderr);
    let setup_json: serde_json::Value = serde_json::from_str(&setup.stdout).expect("setup JSON");
    assert_eq!(setup_json["data"]["project"]["linked"], true);
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let instructions = std::fs::read_to_string(sandbox.repo_dir().join(name))
            .expect("setup installs the project memory contract");
        assert!(instructions.contains("cairn_remember"));
    }
    let codex_path = sandbox.fake_home().join(".codex/config.toml");
    let codex = std::fs::read_to_string(&codex_path).expect("installed Codex MCP config");
    let codex_entry = cairn_integrate::edit::toml::get(
        &codex_path.display().to_string(),
        &codex,
        &["mcp_servers", "cairn"],
    )
    .expect("valid Codex MCP config")
    .expect("Cairn MCP entry");
    assert_eq!(
        codex_entry["env"]["CAIRN_HOME"],
        sandbox.cairn_home().display().to_string()
    );
    let claude: serde_json::Value = serde_json::from_slice(
        &std::fs::read(sandbox.fake_home().join(".claude.json"))
            .expect("installed Claude MCP config"),
    )
    .expect("valid Claude MCP config");
    assert_eq!(
        claude["mcpServers"]["cairn"]["env"]["CAIRN_HOME"],
        sandbox.cairn_home().display().to_string()
    );

    for key in ["alpha9-caller-a", "alpha9-caller-b"] {
        let hook = sandbox.hook_as(
            "codex",
            "SessionStart",
            json!({"session_id": key, "source": "startup"}),
        );
        assert_eq!(hook.code, 0, "native hook for {key}: {}", hook.stderr);
    }

    let mut mcp = Mcp::start(&sandbox);
    let accepted = mcp.tool_result(
        "cairn_remember",
        json!({
            "action": "create",
            "agent_session_key": "alpha9-caller-a",
            "type": "fact",
            "topic_key": "alpha9.journey",
            "value_key": "remembered",
            "content": "the alpha9 journey remembers this durable fact",
        }),
        &sandbox.repo_dir().display().to_string(),
    );
    assert_eq!(accepted["isError"], false, "{accepted}");
    assert_eq!(
        accepted["content"][0]["text"]["accepted_for_delivery"],
        true
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while pg.server.count("SELECT count(*) FROM memories") != 1 {
        assert!(
            std::time::Instant::now() < deadline,
            "the accepted command never reached the server"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let recalled = mcp.tool(
        "cairn_search",
        json!({
            "action": "search",
            "agent_session_key": "alpha9-caller-b",
            "query": "alpha9 journey remembered durable fact",
        }),
        &sandbox.repo_dir().display().to_string(),
    );
    assert!(recalled.contains("the alpha9 journey remembers this durable fact"));
    let natural_query = mcp.tool(
        "cairn_search",
        json!({
            "action": "search",
            "agent_session_key": "alpha9-caller-b",
            "query": "what does alpha9 remember after a new session",
        }),
        &sandbox.repo_dir().display().to_string(),
    );
    assert!(natural_query.contains("the alpha9 journey remembers this durable fact"));

    let claude_first = sandbox.hook_as(
        "claude-code",
        "SessionStart",
        json!({"session_id": "alpha9-claude-first", "source": "startup"}),
    );
    assert_eq!(claude_first.code, 0, "{}", claude_first.stderr);
    let claude_end = sandbox.hook_as(
        "claude-code",
        "SessionEnd",
        json!({"session_id": "alpha9-claude-first", "reason": "other"}),
    );
    assert_eq!(claude_end.code, 0, "{}", claude_end.stderr);
    let claude_return = sandbox.hook_as(
        "claude-code",
        "SessionStart",
        json!({"session_id": "alpha9-claude-return", "source": "startup"}),
    );
    assert_eq!(claude_return.code, 0, "{}", claude_return.stderr);
    let returned_context = mcp.tool(
        "cairn_context",
        json!({"agent_session_key": "alpha9-claude-return", "reason": "session_start"}),
        &sandbox.repo_dir().display().to_string(),
    );
    assert!(returned_context.contains("Cairn context"));
    assert!(returned_context.contains("cairn_remember"));
    let returned_search = mcp.tool(
        "cairn_search",
        json!({"agent_session_key": "alpha9-claude-return", "query": "alpha9 journey remembered durable fact"}),
        &sandbox.repo_dir().display().to_string(),
    );
    assert!(returned_search.contains("the alpha9 journey remembers this durable fact"));
    assert_eq!(pg.server.count("SELECT count(*) FROM memories"), 1);
}

#[test]
fn setup_failures_explain_how_to_recover_without_installing_integrations() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };

    let missing_token = Sandbox::new();
    missing_token.install_agent("codex");
    let result = missing_token.cairn_with_env(
        &["--json", "setup"],
        &[("CAIRN_SERVER_URL", &pg.server.base)],
    );
    assert!(!result.ok());
    let body: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Settings"));
    assert!(!missing_token
        .fake_home()
        .join(".codex/config.toml")
        .exists());

    let wrong_remote = Sandbox::new();
    wrong_remote.install_agent("codex");
    let result = wrong_remote.cairn_with_env(
        &["--json", "setup"],
        &[
            ("CAIRN_SERVER_URL", &pg.server.base),
            ("CAIRN_SERVER_TOKEN", &pg.owner.token),
        ],
    );
    assert!(!result.ok());
    let body: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(body["error"]["code"], "not_linked");
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("remote") && message.contains("Settings"),
        "{body}"
    );
    assert!(!wrong_remote.fake_home().join(".codex/config.toml").exists());
    let config: serde_json::Value = serde_json::from_slice(
        &std::fs::read(wrong_remote.cairn_home().join("config.json")).unwrap(),
    )
    .unwrap();
    assert!(config["server_url"].is_null());
    assert!(!wrong_remote.cairn_home().join("token").exists());

    let nonmember = Sandbox::new();
    nonmember.install_agent("codex");
    nonmember.git(&[
        "remote",
        "set-url",
        "origin",
        "git@example.test:feature005.git",
    ]);
    let result = nonmember.cairn_with_env(
        &["--json", "setup"],
        &[
            ("CAIRN_SERVER_URL", &pg.server.base),
            ("CAIRN_SERVER_TOKEN", &pg.outsider.token),
        ],
    );
    assert!(!result.ok());
    let body: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(body["error"]["code"], "not_linked");
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("remote") && message.contains("Settings"),
        "{body}"
    );
    assert!(!nonmember.fake_home().join(".codex/config.toml").exists());
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(nonmember.cairn_home().join("config.json")).unwrap())
            .unwrap();
    assert!(config["server_url"].is_null());
    assert!(!nonmember.cairn_home().join("token").exists());

    let valid_remote = format!(
        "https://github.com/example/preflight-{}.git",
        Uuid::now_v7()
    );
    let (_, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/projects",
        &json!({"name": "preflight", "repository_remote": valid_remote}),
        &pg.owner.token,
    );
    assert_eq!(status, 200);
    let existing = Sandbox::new();
    existing.git(&["remote", "set-url", "origin", &valid_remote]);
    let credentials = [
        ("CAIRN_SERVER_URL", pg.server.base.as_str()),
        ("CAIRN_SERVER_TOKEN", pg.owner.token.as_str()),
    ];
    assert!(existing
        .cairn_with_env(&["--json", "setup"], &credentials)
        .ok());
    let config_path = existing.cairn_home().join("config.json");
    let token_path = existing.cairn_home().join("token");
    let before = (
        std::fs::read(&config_path).unwrap(),
        std::fs::read(&token_path).unwrap(),
    );
    let other_server = Pg::start().expect("second disposable server");
    let (_, status) = post_json_status_bearer(
        &other_server.server.base,
        "/api/projects",
        &json!({"name": "different project id", "repository_remote": valid_remote}),
        &other_server.owner.token,
    );
    assert_eq!(status, 200);
    assert!(!existing
        .cairn_with_env(
            &["--json", "setup"],
            &[
                ("CAIRN_SERVER_URL", &other_server.server.base),
                ("CAIRN_SERVER_TOKEN", &other_server.owner.token),
            ],
        )
        .ok());
    assert_eq!(std::fs::read(&config_path).unwrap(), before.0);
    assert_eq!(std::fs::read(&token_path).unwrap(), before.1);
    existing.git(&[
        "remote",
        "set-url",
        "origin",
        "https://github.com/example/other.git",
    ]);
    assert!(!existing
        .cairn_with_env(&["--json", "setup"], &credentials)
        .ok());
    assert_eq!(std::fs::read(config_path).unwrap(), before.0);
    assert_eq!(std::fs::read(token_path).unwrap(), before.1);
}
