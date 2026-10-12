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
        &json!({"type":"decision","scope":"project","content":content,
            "capture_attestation":{"basis":"user_report","support_summary":"The team reported its parser choice."}}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{capture}");
    for trigger in ["session_open", "prompt_submit", "explicit"] {
        let (context, status) = post_json_status_bearer(
            &pg.server.base, "/api/retrieve", &json!({"session_id":session,"trigger":trigger}), &pg.owner.token,
        );
        assert_eq!(status, 200, "{context}");
        assert!(!context.to_string().contains(content), "{trigger}: {context}");
        assert_eq!(context["project_memory_available"], true, "{context}");
    }
    let (context, status) = post_json_status_bearer(
        &pg.server.base, "/api/retrieve", &json!({"session_id":session,"trigger":"explicit","query":"bounded parser"}), &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert!(context.to_string().contains(content), "{context}");
    for query in ["", "   ", "unrelated satellite orbit", "the and"] {
        let (context, status) = post_json_status_bearer(
            &pg.server.base, "/api/retrieve", &json!({"session_id":session,"trigger":"explicit","query":query}), &pg.owner.token,
        );
        assert_eq!(status, 200, "{context}");
        assert!(!context.to_string().contains(content), "{context}");
    }
    let (context, status) = post_json_status_bearer(
        &pg.server.base, "/api/retrieve", &json!({"session_id":session,"trigger":"explicit","query":"bounded parser"}), &pg.member.token,
    );
    assert_eq!(status, 403, "{context}");
}

#[test]
fn lexical_recall_ranks_overlap_before_recency_and_delivers_complete_records() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let exact = "The team chose a bounded parser for configuration imports. The release workflow also uploads archives.";
    let mut values = Vec::new();
    for i in 0..100 {
        let id = Uuid::now_v7();
        let content = if i == 0 { exact.to_owned() } else { format!("bounded parser alternative {i}") };
        values.push(format!("('{id}', '{}', 'decision', 'project', '{}', '{content}', '{session}', now() + interval '{i} seconds')", pg.project, pg.project));
    }
    pg.server.execute(&format!("INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id, updated_at) VALUES {}", values.join(",")));
    pg.server.execute(&format!("INSERT INTO project_memory_attestations (memory_id, actor_user_id, basis, support_summary) SELECT id, '{}', 'user_report', 'fixture' FROM memories WHERE project_id = '{}'", pg.owner.id, pg.project));
    let (context, status) = post_json_status_bearer(
        &pg.server.base, "/api/retrieve", &json!({"session_id":session,"trigger":"explicit","query":"bounded parser configuration"}), &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    let item = &context["sections"]["project_memory"][0];
    assert_eq!(item["content"], exact, "{context}");
    assert_eq!(item["selection"]["kind"], "lexical_full_record", "{context}");
    assert!(item["selection"].get("spans").is_none(), "{context}");
    assert!(item["selection"].get("selected_content_sha256").is_none(), "{context}");
    let (search, status) = get_json_status_bearer(&pg.server.base, &format!("/api/projects/{}/memories?purpose=reuse&q=bounded%20parser%20configuration&limit=100", pg.project), &pg.owner.token);
    assert_eq!(status, 200, "{search}");
    assert_eq!(search["memories"][0]["content"], exact, "{search}");
    assert_eq!(search["memories"][0]["selection"]["kind"], "lexical_full_record", "{search}");
    assert_eq!(search["limit"], 72, "{search}");
}

#[test]
fn lexical_detail_uses_the_same_predicate_and_full_record_provenance() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let source = "Use café cache for 90 seconds; do not extend it. The release workflow also uploads private archives.";
    let id = Uuid::now_v7();
    pg.server.execute(&format!("INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id) VALUES ('{id}', '{}', 'decision', 'project', '{}', '{}', '{session}')", pg.project, pg.project, source.replace('\'', "''")));
    pg.attest_project_memory(id, &pg.owner, "fixture");
    let (detail, status) = get_json_status_bearer(&pg.server.base, &format!("/api/memories/{id}?purpose=reuse&q=caf%C3%A9%20cache%2090%20seconds"), &pg.owner.token);
    assert_eq!(status, 200, "{detail}");
    assert_eq!(detail["memory"]["content"], source, "{detail}");
    assert_eq!(detail["memory"]["selection"]["kind"], "lexical_full_record", "{detail}");
    let (missing, status) = get_json_status_bearer(&pg.server.base, &format!("/api/memories/{id}?purpose=reuse&q=unrelated%20satellite"), &pg.owner.token);
    assert_eq!(status, 404, "{missing}");
}

#[test]
fn lexical_recall_refuses_non_members() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let id = Uuid::now_v7();
    pg.server.execute(&format!("INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id) VALUES ('{id}', '{}', 'decision', 'project', '{}', 'bounded parser source', '{session}')", pg.project, pg.project));
    pg.attest_project_memory(id, &pg.owner, "fixture");
    pg.server.execute(&format!("DELETE FROM project_members WHERE project_id = '{}' AND user_id = '{}'", pg.project, pg.owner.id));
    let (body, status) = post_json_status_bearer(&pg.server.base, "/api/retrieve", &json!({"session_id":session,"trigger":"explicit","query":"bounded parser"}), &pg.owner.token);
    assert_eq!(status, 403, "{body}");
    assert!(!body.to_string().contains("bounded parser source"), "{body}");
}


#[test]
fn lexical_recall_omits_an_overbudget_record_without_cropping_it() {
    let Some(pg) = Pg::start() else {
        eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
        return;
    };
    let session = pg.session_for(&pg.owner);
    let id = Uuid::now_v7();
    let prefix = "bounded parser must retain every qualifier. ";
    let source = format!("{prefix}{}", "x".repeat(20_000));
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id) VALUES ('{id}', '{}', 'decision', 'project', '{}', '{}', '{session}')",
        pg.project,
        pg.project,
        source.replace('\'', "''"),
    ));
    pg.attest_project_memory(id, &pg.owner, "fixture");
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({"session_id":session,"trigger":"explicit","query":"bounded parser","budget_tokens":64}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert!(context["sections"]["project_memory"].as_array().map_or(true, Vec::is_empty), "{context}");
    assert!(!context.to_string().contains(prefix), "overbudget record was cropped rather than omitted: {context}");
    let trace_id = context["trace_id"].as_str().expect("trace id");
    let (trace, status) = get_json_status_bearer(&pg.server.base, &format!("/api/retrieval-traces/{trace_id}"), &pg.owner.token);
    assert_eq!(status, 200, "{trace}");
    assert!(trace["items"].as_array().is_some_and(|items| items.iter().any(|item| item["knowledge_id"] == id.to_string() && item["status"] == "considered" && item["selection_rule"] == "budget_exhausted")), "{trace}");
}
