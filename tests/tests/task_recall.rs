use cairn_e2e::feature005::Pg;
use cairn_e2e::post_json_status_bearer;
use serde_json::json;
use uuid::Uuid;

#[test]
fn context_without_a_task_keeps_findings_available_without_delivering_them() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let content = "The team selected a bounded parser for configuration imports.";
    let (capture, status) = post_json_status_bearer(
        &pg.server.base,
        &format!("/api/projects/{}/captures", pg.project),
        &json!({
            "type": "decision", "scope": "project", "content": content,
            "capture_attestation": {
                "basis": "user_report", "support_summary": "The team reported its parser choice."
            }
        }),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{capture}");
    for trigger in ["session_open", "prompt_submit", "explicit"] {
        let (context, status) = post_json_status_bearer(
            &pg.server.base,
            "/api/retrieve",
            &json!({"session_id": session, "trigger": trigger}),
            &pg.owner.token,
        );
        assert_eq!(status, 200, "{context}");
        assert!(
            !context.to_string().contains(content),
            "{trigger} delivered a project finding without a task: {context}"
        );
        assert_eq!(context["project_memory_available"], true, "{context}");
    }
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser"}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert!(context.to_string().contains(content), "{context}");
    for query in ["", "   ", "unrelated satellite orbit", "the and"] {
        let (context, status) = post_json_status_bearer(
            &pg.server.base,
            "/api/retrieve",
            &json!({"session_id": session, "trigger": "explicit", "query": query}),
            &pg.owner.token,
        );
        assert_eq!(status, 200, "{context}");
        assert!(!context.to_string().contains(content), "{context}");
    }
    for query in [
        "x".repeat(257),
        "OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz0123".into(),
    ] {
        let (context, status) = post_json_status_bearer(
            &pg.server.base,
            "/api/retrieve",
            &json!({"session_id": session, "trigger": "explicit", "query": query}),
            &pg.owner.token,
        );
        assert_eq!(status, 400, "{context}");
        assert!(!context.to_string().contains(content), "{context}");
    }
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser"}),
        &pg.member.token,
    );
    assert_eq!(status, 403, "{context}");
    assert!(
        context.get("project_memory_available").is_none(),
        "{context}"
    );
}

#[test]
fn task_recall_ranks_overlap_before_the_candidate_limit_and_keeps_whole_records() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let exact = "The team chose a bounded parser for configuration imports. The release workflow also uploads archives.";
    for i in 0..26 {
        let id = Uuid::now_v7();
        let content = if i == 0 {
            exact.to_string()
        } else {
            format!("bounded parser alternative {i}")
        };
        pg.server.execute(&format!(
            "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id, updated_at)
             VALUES ('{id}', '{}', 'decision', 'project', '{}', '{content}', '{session}', now() + interval '{i} seconds')",
            pg.project, pg.project
        ));
        pg.attest_project_memory(id, &pg.owner, "The fixture reports this entire record.");
    }
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser configuration"}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert_eq!(
        context["sections"]["project_memory"][0]["content"], exact,
        "{context}"
    );
    // Matching a record does not filter appended claims; semantic quality remains an evaluation gate.
    assert!(context
        .to_string()
        .contains("release workflow also uploads archives"));
}
