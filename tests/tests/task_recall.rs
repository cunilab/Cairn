use cairn_e2e::feature005::Pg;
use cairn_e2e::{get_json_status_bearer, post_json_status_bearer};
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
fn task_recall_ranks_overlap_before_the_candidate_limit_and_returns_excerpts() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let exact = "The team chose a bounded parser for configuration imports. The release workflow also uploads archives.";
    let mut values = Vec::new();
    for i in 0..100 {
        let id = Uuid::now_v7();
        let content = if i == 0 {
            exact.to_string()
        } else {
            format!("bounded parser alternative {i}")
        };
        values.push(format!(
            "('{id}', '{}', 'decision', 'project', '{}', '{content}', '{session}', now() + interval '{i} seconds')",
            pg.project, pg.project
        ));
    }
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id, updated_at) VALUES {}",
        values.join(",")
    ));
    pg.server.execute(&format!(
        "INSERT INTO project_memory_attestations (memory_id, actor_user_id, basis, support_summary)
         SELECT id, '{}', 'user_report', 'The fixture reports this entire record.' FROM memories WHERE project_id = '{}'",
        pg.owner.id, pg.project
    ));
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser configuration [[selector:first-sentence]]"}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert_eq!(
        context["sections"]["project_memory"][0]["content"],
        "The team chose a bounded parser for configuration imports.",
        "{context}"
    );
    assert!(!context
        .to_string()
        .contains("release workflow also uploads archives"));
    assert_eq!(
        context["sections"]["project_memory"][0]["selection"]["kind"],
        "extractive_excerpt"
    );
    let (search, status) = get_json_status_bearer(
        &pg.server.base,
        &format!(
            "/api/projects/{}/memories?purpose=reuse&q=bounded%20parser%20alternative&limit=100",
            pg.project
        ),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{search}");
    assert_eq!(search["limit"], 72, "{search}");
    assert_eq!(
        search["memories"].as_array().map(Vec::len),
        Some(72),
        "{search}"
    );
}

#[test]
fn task_recall_never_leaks_a_pinned_or_warned_mixed_record_outside_selected_excerpt() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let id = Uuid::now_v7();
    let selected = "Use café cache for 90 seconds; do not extend it.";
    let appendix = "The release workflow also uploads private archives.";
    let source = format!("{selected} {appendix}");
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id, pinned, verification)
         VALUES ('{id}', '{}', 'decision', 'project', '{}', '{}', '{session}', true, 'conflicted')",
        pg.project,
        pg.project,
        source.replace('\'', "''"),
    ));
    pg.attest_project_memory(id, &pg.owner, "The author reported the full record.");

    let (automatic, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "session_open"}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{automatic}");
    assert!(!automatic.to_string().contains(&source), "{automatic}");
    assert!(!automatic.to_string().contains(appendix), "{automatic}");
    // A warning is not reusable. Clear it only to exercise the later,
    // separately authorized task-recall path against the same archived source.
    pg.server.execute(&format!(
        "UPDATE memories SET verification = NULL WHERE id = '{id}'"
    ));

    let query = "café cache 90 seconds [[selector:first-sentence]]";
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": query}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert_eq!(
        context["sections"]["project_memory"][0]["content"], selected,
        "{context}"
    );
    assert!(!context.to_string().contains(appendix), "{context}");
    assert_eq!(
        context["sections"]["project_memory"][0]["selection"]["spans"][0]["start_byte"],
        0
    );
    assert_eq!(
        context["sections"]["project_memory"][0]["selection"]["spans"][0]["end_byte"],
        selected.len()
    );
    let trace_id = context["trace_id"].as_str().expect("trace id");
    let (trace, status) = get_json_status_bearer(
        &pg.server.base,
        &format!("/api/retrieval-traces/{trace_id}"),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{trace}");
    let trace_item = trace["items"]
        .as_array()
        .expect("trace items")
        .iter()
        .find(|item| item["knowledge_id"] == id.to_string())
        .expect("selected project trace item");
    assert_eq!(
        trace_item["selection"], context["sections"]["project_memory"][0]["selection"],
        "trace must carry the exact delivered excerpt provenance"
    );
    for invalid in [
        "jsonb_set(selection_provenance, '{kind}', 'null'::jsonb)",
        "selection_provenance - 'spans'",
    ] {
        pg.server.execute(&format!(
            "DO $$ BEGIN
               BEGIN
                 UPDATE retrieval_trace_items SET selection_provenance = {invalid}
                  WHERE trace_id = '{trace_id}' AND knowledge_id = '{id}';
                 RAISE EXCEPTION 'invalid excerpt provenance was accepted';
               EXCEPTION WHEN check_violation THEN NULL;
               END;
             END $$;"
        ));
    }

    let encoded_query = "caf%C3%A9%20cache%2090%20seconds%20%5B%5Bselector%3Afirst-sentence%5D%5D";
    let (search, status) = get_json_status_bearer(
        &pg.server.base,
        &format!(
            "/api/projects/{}/memories?purpose=reuse&q={encoded_query}",
            pg.project
        ),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{search}");
    assert_eq!(search["memories"][0]["content"], selected, "{search}");
    assert!(
        search["memories"][0]["reuse"]["attestation"]
            .get("support_summary")
            .is_none(),
        "{search}"
    );
    assert_eq!(
        search["memories"][0]["selection"]["kind"],
        "extractive_excerpt"
    );

    let (inspect, status) = get_json_status_bearer(
        &pg.server.base,
        &format!("/api/memories/{id}"),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{inspect}");
    assert_eq!(inspect["memory"]["content"], source, "{inspect}");
    let (detail, status) = get_json_status_bearer(
        &pg.server.base,
        &format!("/api/memories/{id}?purpose=reuse&q={encoded_query}"),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{detail}");
    assert_eq!(detail["memory"]["content"], selected, "{detail}");
    assert!(!detail.to_string().contains(appendix), "{detail}");
}

#[test]
fn selector_refusal_paths_do_not_fall_back_to_project_source_bodies() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let source =
        "A bounded parser accepts TOML only. Private appendix must never leave archive inspection.";
    let id = Uuid::now_v7();
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id)
         VALUES ('{id}', '{}', 'decision', 'project', '{}', '{}', '{session}')",
        pg.project, pg.project, source
    ));
    pg.attest_project_memory(id, &pg.owner, "fixture");
    for marker in ["[[selector:outage]]", "[[selector:invalid]]"] {
        let (body, status) = post_json_status_bearer(
            &pg.server.base,
            "/api/retrieve",
            &json!({"session_id": session, "trigger": "explicit", "query": format!("bounded parser {marker}")}),
            &pg.owner.token,
        );
        assert_eq!(status, 503, "{body}");
        assert!(!body.to_string().contains(source), "{body}");
    }
    let (empty, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser [[selector:empty]]"}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{empty}");
    assert!(empty["sections"].get("project_memory").is_none(), "{empty}");
    assert!(!empty.to_string().contains(source), "{empty}");

    let Some(unconfigured) = Pg::start_without_selector() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = unconfigured.session_for(&unconfigured.owner);
    let id = Uuid::now_v7();
    unconfigured.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id)
         VALUES ('{id}', '{}', 'decision', 'project', '{}', 'bounded parser private appendix', '{session}')",
        unconfigured.project, unconfigured.project
    ));
    unconfigured.attest_project_memory(id, &unconfigured.owner, "fixture");
    let (body, status) = post_json_status_bearer(
        &unconfigured.server.base,
        "/api/retrieve",
        &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser"}),
        &unconfigured.owner.token,
    );
    assert_eq!(status, 503, "{body}");
    assert!(!body.to_string().contains("private appendix"), "{body}");
}

#[test]
fn selector_rechecks_source_and_membership_after_provider_waits() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let id = Uuid::now_v7();
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id)
         VALUES ('{id}', '{}', 'decision', 'project', '{}', 'bounded parser source revision', '{session}')",
        pg.project, pg.project
    ));
    pg.attest_project_memory(id, &pg.owner, "fixture");
    let request = paused_retrieve(&pg, session);
    pg.server.wait_for_selector();
    pg.server.execute(&format!(
        "UPDATE memories SET content = 'bounded parser changed revision' WHERE id = '{id}'"
    ));
    pg.server.release_selector();
    let (body, status) = request.join().expect("paused request joins");
    assert_eq!(status, 409, "{body}");
    assert!(!body.to_string().contains("source revision"), "{body}");

    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let id = Uuid::now_v7();
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id)
         VALUES ('{id}', '{}', 'decision', 'project', '{}', 'bounded parser membership source', '{session}')",
        pg.project, pg.project
    ));
    pg.attest_project_memory(id, &pg.owner, "fixture");
    let request = paused_retrieve(&pg, session);
    pg.server.wait_for_selector();
    pg.server.execute(&format!(
        "DELETE FROM project_members WHERE project_id = '{}' AND user_id = '{}'",
        pg.project, pg.owner.id
    ));
    pg.server.release_selector();
    let (body, status) = request.join().expect("paused request joins");
    assert_eq!(status, 403, "{body}");
    assert!(!body.to_string().contains("membership source"), "{body}");
}

fn paused_retrieve(pg: &Pg, session: Uuid) -> std::thread::JoinHandle<(serde_json::Value, u16)> {
    let base = pg.server.base.clone();
    let token = pg.owner.token.clone();
    std::thread::spawn(move || {
        post_json_status_bearer(
            &base,
            "/api/retrieve",
            &json!({"session_id": session, "trigger": "explicit", "query": "bounded parser [[selector:pause]]"}),
            &token,
        )
    })
}
