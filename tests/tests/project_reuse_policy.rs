use cairn_e2e::feature005::{Account, Pg};
use cairn_e2e::{get_json_status_bearer, post_json_status_bearer};
use serde_json::{json, Value};
use uuid::Uuid;

macro_rules! pg {
    () => {
        match Pg::start() {
            Some(pg) => pg,
            None => {
                eprintln!("skipped: CAIRN_TEST_DATABASE_URL is not set");
                return;
            }
        }
    };
}

fn create(pg: &Pg, who: &Account, content: &str, attestation: Value) -> Uuid {
    let (body, status) = post_json_status_bearer(
        &pg.server.base,
        &format!("/api/projects/{}/captures", pg.project),
        &json!({
            "type": "fact",
            "scope": "project",
            "content": content,
            "capture_attestation": attestation,
        }),
        &who.token,
    );
    assert_eq!(status, 200, "{body}");
    body["id"].as_str().unwrap().parse().unwrap()
}

#[test]
fn attested_commands_require_support_and_supersession_preserves_new_authority() {
    let pg = pg!();
    let first = create(
        &pg,
        &pg.owner,
        "first captured claim",
        json!({
            "basis": "user_report", "support_summary": "The user supplied the first claim."
        }),
    );
    let command_id = Uuid::now_v7();
    let envelope = json!({
        "command_id": command_id, "kind": "supersede_attested", "target_id": first,
        "project_id": pg.project, "payload": {
            "type": "fact", "scope": "project", "content": "replacement captured claim",
            "capture_attestation": {"basis": "user_report", "support_summary": "The user corrected the claim."}
        }
    });
    let (result, status) =
        post_json_status_bearer(&pg.server.base, "/api/commands", &envelope, &pg.owner.token);
    assert_eq!(status, 200, "{result}");
    let replacement = result["id"].as_str().unwrap();
    let (retry, status) =
        post_json_status_bearer(&pg.server.base, "/api/commands", &envelope, &pg.owner.token);
    assert_eq!(status, 200, "{retry}");
    assert!(retry.to_string().contains(replacement));
    let reuse = memories(&pg, "reuse");
    assert!(reuse["memories"].to_string().contains(replacement));
    assert!(!reuse["memories"].to_string().contains(&first.to_string()));
    for kind in ["remember_attested", "supersede_attested"] {
        let mut invalid = envelope.clone();
        invalid["command_id"] = json!(Uuid::now_v7());
        invalid["kind"] = json!(kind);
        invalid["payload"]["capture_attestation"] = Value::Null;
        let (_, status) =
            post_json_status_bearer(&pg.server.base, "/api/commands", &invalid, &pg.owner.token);
        assert_eq!(status, 400);
    }
    assert_eq!(pg.server.count("SELECT count(*) FROM memories"), 2);
}

fn memories(pg: &Pg, purpose: &str) -> Value {
    let (body, status) = get_json_status_bearer(
        &pg.server.base,
        &format!(
            "/api/projects/{}/memories?domain=project&purpose={purpose}",
            pg.project
        ),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{body}");
    body
}

#[test]
fn zero_observation_manual_attestation_is_reusable_and_legacy_stays_inspectable() {
    let pg = pg!();
    let supported = create(
        &pg,
        &pg.owner,
        "the user selected the bounded parser",
        json!({
            "basis": "user_report",
            "support_summary": "The user explicitly selected this parser."
        }),
    );
    let legacy = Uuid::now_v7();
    pg.server.execute(&format!(
        "INSERT INTO memories
             (id, project_id, type, scope, scope_key, content, origin_session_id, pinned)
         VALUES ('{legacy}', '{}', 'fact', 'project', '{}',
                 'unsupported legacy seed', '{}', true)",
        pg.project,
        pg.project,
        Uuid::nil(),
    ));

    let reuse = memories(&pg, "reuse");
    assert_eq!(reuse["reuse_policy"], "project_attestation_v1");
    assert!(reuse["memories"]
        .to_string()
        .contains(&supported.to_string()));
    assert!(!reuse["memories"].to_string().contains(&legacy.to_string()));
    assert_eq!(
        pg.server.count(&format!(
            "SELECT jsonb_array_length(observation_ids)::bigint FROM memories WHERE id = '{supported}'"
        )),
        0
    );
    assert_eq!(
        pg.server.count(&format!(
            "SELECT count(*) FROM memories WHERE id = '{supported}' AND verification_authority IS NULL"
        )),
        1,
        "capture attestation manufactured verification authority"
    );

    let inspect = memories(&pg, "inspect");
    let archived = inspect["memories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == legacy.to_string())
        .expect("legacy archive row");
    assert_eq!(archived["reuse"]["eligible"], false);
    assert_eq!(archived["reuse"]["reason"], "unattributed_legacy");
    assert!(inspect["inspection_instruction"]
        .as_str()
        .unwrap()
        .contains("objective proof"));

    let session = pg.session_for(&pg.owner);
    let (context, status) = post_json_status_bearer(
        &pg.server.base,
        "/api/retrieve",
        &json!({ "session_id": session, "trigger": "explicit", "query": "bounded parser" }),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{context}");
    assert!(context
        .to_string()
        .contains("the user selected the bounded parser"));
    assert!(!context.to_string().contains("unsupported legacy seed"));
    assert!(!context["continuity"]["pins"]
        .to_string()
        .contains(&legacy.to_string()));
}

#[test]
fn dependency_revision_and_conflict_invalidate_reuse_but_not_archive() {
    let pg = pg!();
    let dependency = create(
        &pg,
        &pg.owner,
        "dependency source",
        json!({
            "basis": "user_report",
            "support_summary": "The user supplied this dependency."
        }),
    );
    let dependent = create(
        &pg,
        &pg.owner,
        "claim based on inspected dependency",
        json!({
            "basis": "inspected_source",
            "support_summary": "Inspected the named configuration revision.",
            "source_reference": "config/runtime.toml",
            "source_revision": "git:abc123",
            "dependency_memory_id": dependency,
        }),
    );
    assert!(memories(&pg, "reuse")["memories"]
        .to_string()
        .contains(&dependent.to_string()));

    pg.server.execute(&format!(
        "UPDATE memories SET updated_at = updated_at + interval '1 second' WHERE id = '{dependency}'"
    ));
    assert!(!memories(&pg, "reuse")["memories"]
        .to_string()
        .contains(&dependent.to_string()));
    let inspect = memories(&pg, "inspect");
    let row = inspect["memories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == dependent.to_string())
        .unwrap();
    assert_eq!(row["reuse"]["reason"], "source_changed");

    let other = create(
        &pg,
        &pg.owner,
        "conflicting claim",
        json!({
            "basis": "user_report",
            "support_summary": "The user supplied the conflicting report."
        }),
    );
    pg.server.execute(&format!(
        "INSERT INTO memory_relations
             (from_memory_id, to_memory_id, kind, project_id, decided_by_session, basis)
         VALUES ('{dependent}', '{other}', 'conflicts_with', '{}', '{}', 'explicit')",
        pg.project,
        Uuid::nil(),
    ));
    assert!(!memories(&pg, "reuse")["memories"]
        .to_string()
        .contains(&other.to_string()));
}

#[test]
fn dependency_conflict_is_transitive_and_capture_dependencies_are_one_hop() {
    let pg = pg!();
    let source = create(
        &pg,
        &pg.owner,
        "supported dependency",
        json!({
            "basis": "user_report", "support_summary": "The user supplied this source."
        }),
    );
    let dependent = create(
        &pg,
        &pg.owner,
        "supported dependent",
        json!({
            "basis": "user_report", "support_summary": "The user supplied this dependent claim.",
            "dependency_memory_id": source
        }),
    );
    assert!(memories(&pg, "reuse")["memories"]
        .to_string()
        .contains(&dependent.to_string()));
    let (body, status) = post_json_status_bearer(
        &pg.server.base,
        &format!("/api/projects/{}/captures", pg.project),
        &json!({"type": "fact", "content": "second dependency hop", "capture_attestation": {
            "basis": "user_report", "support_summary": "The user supplied the chain.",
            "dependency_memory_id": dependent
        }}),
        &pg.owner.token,
    );
    assert_eq!(status, 400, "{body}");
    assert!(body.to_string().contains("one hop maximum"));
    let other = create(
        &pg,
        &pg.owner,
        "conflicting dependency",
        json!({
            "basis": "user_report", "support_summary": "The user supplied the alternative."
        }),
    );
    pg.server.execute(&format!(
        "INSERT INTO memory_relations
            (from_memory_id, to_memory_id, kind, project_id, decided_by_session, basis)
         VALUES ('{source}', '{other}', 'conflicts_with', '{}', '{}', 'explicit')",
        pg.project,
        Uuid::nil()
    ));
    assert!(!memories(&pg, "reuse")["memories"]
        .to_string()
        .contains(&dependent.to_string()));
    assert!(memories(&pg, "inspect")["memories"]
        .to_string()
        .contains(&dependent.to_string()));
}

#[test]
fn graph_reuse_withholds_an_ineligible_endpoint_while_inspection_keeps_it() {
    let pg = pg!();
    let seed = create(
        &pg,
        &pg.owner,
        "eligible graph seed",
        json!({
            "basis": "user_report",
            "support_summary": "The user supplied the graph seed."
        }),
    );
    let legacy = Uuid::now_v7();
    pg.server.execute(&format!(
        "INSERT INTO memories
             (id, project_id, type, scope, scope_key, content, origin_session_id)
         VALUES ('{legacy}', '{}', 'fact', 'project', '{}', 'legacy graph neighbor', '{}')",
        pg.project,
        pg.project,
        Uuid::nil(),
    ));
    pg.server.execute(&format!(
        "INSERT INTO memory_relations
             (from_memory_id, to_memory_id, kind, project_id, decided_by_session, basis)
         VALUES ('{seed}', '{legacy}', 'narrows', '{}', '{}', 'explicit')",
        pg.project,
        Uuid::nil(),
    ));

    for (purpose, expected) in [("reuse", 0), ("inspect", 1)] {
        let (body, status) = get_json_status_bearer(
            &pg.server.base,
            &format!(
                "/api/projects/{}/graph?memory_id={seed}&hops=1&purpose={purpose}",
                pg.project
            ),
            &pg.owner.token,
        );
        assert_eq!(status, 200, "{body}");
        assert_eq!(body["edges"].as_array().unwrap().len(), expected, "{body}");
    }
}

#[test]
fn forged_capture_authority_invalid_kind_and_incomplete_source_are_refused() {
    let pg = pg!();
    for body in [
        json!({
            "type": "fact", "content": "forged actor",
            "capture_attestation": {
                "basis": "user_report", "support_summary": "claimed",
                "actor_user_id": pg.outsider.id
            }
        }),
        json!({
            "type": "fact", "content": "forged authority",
            "capture_attestation": {
                "basis": "user_report", "support_summary": "claimed",
                "verification_authority": "attested"
            }
        }),
        json!({
            "type": "claim", "content": "invented type",
            "capture_attestation": {
                "basis": "user_report", "support_summary": "claimed"
            }
        }),
        json!({
            "type": "fact", "content": "incomplete source",
            "capture_attestation": {
                "basis": "inspected_source", "support_summary": "read it",
                "source_reference": "config/runtime.toml"
            }
        }),
    ] {
        let (reply, status) = post_json_status_bearer(
            &pg.server.base,
            &format!("/api/projects/{}/captures", pg.project),
            &body,
            &pg.owner.token,
        );
        assert_eq!(status, 400, "{reply}");
    }
}

#[test]
fn legacy_writes_refuse_capture_and_forget_erases_authored_support() {
    let pg = pg!();
    let support = "Bounded authored support that must disappear on forget";
    let body = json!({"type":"fact", "scope":"project", "content":"claim to forget",
        "capture_attestation":{"basis":"inspected_source", "support_summary":support,
            "source_reference":"private-fixture.toml", "source_revision":"fixture-revision"}});
    let (capture, status) = post_json_status_bearer(
        &pg.server.base,
        &format!("/api/projects/{}/captures", pg.project),
        &body,
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{capture}");
    let id = capture["id"].as_str().unwrap();
    for route in [
        format!("/api/projects/{}/memories", pg.project),
        format!("/api/memories/{id}/supersede"),
    ] {
        let (_, status) = post_json_status_bearer(&pg.server.base, &route, &body, &pg.owner.token);
        assert_eq!(status, 400);
    }
    for kind in ["remember", "supersede"] {
        let (_, status) = post_json_status_bearer(
            &pg.server.base,
            "/api/commands",
            &json!({"command_id":Uuid::now_v7(), "project_id":pg.project,
                "target_id":id,"kind":kind,"payload":body}),
            &pg.owner.token,
        );
        assert_eq!(status, 400);
    }
    assert_eq!(pg.server.count("SELECT count(*) FROM memories"), 1);
    let (reply, status) = post_json_status_bearer(
        &pg.server.base,
        &format!("/api/memories/{id}/forget"),
        &json!({}),
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{reply}");
    assert_eq!(
        pg.server
            .count("SELECT count(*) FROM project_memory_attestations"),
        0
    );
    assert_eq!(
        pg.server
            .count("SELECT count(*) FROM memories WHERE content <> ''"),
        0
    );
    let (detail, _) = get_json_status_bearer(
        &pg.server.base,
        &format!("/api/memories/{id}"),
        &pg.owner.token,
    );
    assert!(!detail.to_string().contains(support));
    assert!(!detail.to_string().contains("private-fixture.toml"));
    pg.server.execute(&format!(
        "UPDATE users SET role='admin' WHERE id='{}'",
        pg.owner.id
    ));
    let (export, status) = get_json_status_bearer(
        &pg.server.base,
        "/api/admin/logical-export",
        &pg.owner.token,
    );
    assert_eq!(status, 200, "{export}");
    assert!(!export.to_string().contains(support));
    assert!(!export.to_string().contains("private-fixture.toml"));
}
