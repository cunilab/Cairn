//! Server-side retrieval and delivery, as a contract (T065,
//! `contracts/retrieval-delivery.md` §3, §4, §4.1, §5, §6, §12.0-§12.2).
//!
//! US2's independent test: knowledge is seeded directly with SQL into
//! `memories`, `personal_knowledge`, `team_knowledge` and `shared_patterns`,
//! never through the capture path, so these assertions stand on their own
//! against a server that never ran consolidation.
//!
//! What matters here is not "does a briefing come back" — it does, trivially —
//! but the properties a caller can lean on:
//!
//! - **Domain ordering never lets personal/team displace project truth**
//!   (SC-710): project sections are admitted first, from the same shared
//!   budget, so project content that alone consumes the whole budget leaves
//!   nothing for personal or team, ever.
//! - **Every briefing states, and never exceeds, its budget** (SC-709), and
//!   the sum of what it says it spent per item is the total it says it spent.
//! - **Every selection is reproducible by hand** (SC-711): the rule and the
//!   budget remaining travel with each item, and replaying costs in rank
//!   order reproduces the recorded spend exactly.
//! - **Two points, two budgets, and dedup between them** (§4, §4.1): a second
//!   delivery in one session never restates an unchanged item, an edited item
//!   re-enters, and `explicit` is exempt because it is a request to be told
//!   again.
//! - **The same UUID in four domains stays four references** (SC-767): a
//!   `reference_key` carries its domain, so identical ids never collapse and
//!   personal delivery never suppresses team.
//! - **A non-member is refused, never handed an empty briefing** (§3), and a
//!   session that is not the caller's own is refused the same way.

use cairn_e2e::feature005::{Account, Pg};
use cairn_e2e::post_json_status_bearer;
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

// ---------------------------------------------------------------------------
// Request helpers
// ---------------------------------------------------------------------------

fn retrieve(pg: &Pg, who: &Account, session: Uuid, trigger: &str) -> (Value, u16) {
    retrieve_with(
        pg,
        who,
        json!({ "session_id": session, "trigger": trigger }),
    )
}

fn retrieve_with(pg: &Pg, who: &Account, body: Value) -> (Value, u16) {
    post_json_status_bearer(&pg.server.base, "/api/retrieve", &body, &who.token)
}

fn report_transmitted(pg: &Pg, who: &Account, trace_id: &str) -> (Value, u16) {
    post_json_status_bearer(
        &pg.server.base,
        &format!("/api/retrieval-traces/{trace_id}/transmission"),
        &json!({ "outcome": "transmitted" }),
        &who.token,
    )
}

// ---------------------------------------------------------------------------
// Seeding helpers — knowledge is planted directly with SQL (US2 is
// independent of Story 1's capture path).
// ---------------------------------------------------------------------------

fn escape(s: &str) -> String {
    s.replace('\'', "''")
}

/// A row in `memories`, scoped `project` — the scope `gather()` reads under
/// `project_memory` when the session names no task and no branch match.
fn seed_project_memory(pg: &Pg, session: Uuid, content: &str) -> Uuid {
    let id = Uuid::now_v7();
    let content = escape(content);
    pg.server.execute(&format!(
        "INSERT INTO memories (id, project_id, type, scope, scope_key, content, origin_session_id)
         VALUES ('{id}', '{}', 'fact', 'project', '{}', '{content}', '{session}')",
        pg.project, pg.project
    ));
    pg.attest_project_memory(id, &pg.owner, "Fixture author reports this project fact.");
    id
}

#[test]
fn legacy_project_rows_stay_archival_and_do_not_reach_context_pins_or_warnings() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);
    let ordinary = seed_project_memory(&pg, session, "legacy durable project finding");
    let pin = seed_continuity_memory(
        &pg,
        pg.project,
        session,
        "legacy pinned constraint",
        true,
        None,
    );
    let warning = seed_continuity_memory(
        &pg,
        pg.project,
        session,
        "legacy conflicting claim",
        false,
        Some("conflicted"),
    );
    let foreign = pg.extra_project("legacy-foreign", &[&pg.owner]);
    let foreign_session = pg.session_in(foreign, &pg.owner);
    let foreign_pin = seed_continuity_memory(
        &pg,
        foreign,
        foreign_session,
        "foreign legacy secret",
        true,
        Some("drifted"),
    );
    pg.server.execute(&format!(
        "DELETE FROM project_memory_attestations WHERE memory_id IN ('{ordinary}', '{pin}', '{warning}', '{foreign_pin}')"
    ));
    pg.server.execute(&format!("UPDATE memories SET scope_key = '{}' WHERE id IN ('{ordinary}', '{pin}', '{warning}', '{foreign_pin}')", Uuid::now_v7()));
    let (context, status) = retrieve(&pg, &pg.owner, session, "explicit");
    assert_eq!(status, 200, "{context}");
    assert!(
        !context
            .to_string()
            .contains("legacy durable project finding"),
        "{context}"
    );
    assert!(
        !context["continuity"]["pins"]
            .to_string()
            .contains(&pin.to_string()),
        "{context}"
    );
    assert!(
        !context["continuity"]["warnings"]
            .to_string()
            .contains("legacy conflicting claim"),
        "{context}"
    );
    assert!(
        !context.to_string().contains("foreign legacy secret"),
        "{context}"
    );
}

fn seed_handoff(pg: &Pg, project: Uuid, session: Uuid, marker: &str) -> Uuid {
    let id = Uuid::now_v7();
    let marker = escape(marker);
    pg.server.execute(&format!(
        r#"INSERT INTO handoffs
            (id, project_id, session_id, trigger, goal, progress, completed_work,
             remaining_work, changed_files, decisions, failures, tests_executed,
             repository_state, next_step)
         VALUES ('{id}', '{project}', '{session}', 'session_end', '{marker} goal',
                 '{marker} progress', '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
                 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
                 '{{"branch":"main","commit_sha":null,"staged":0,"unstaged":0,"untracked":0}}'::jsonb,
                 '{marker} next')"#
    ));
    id
}

fn seed_continuity_memory(
    pg: &Pg,
    project: Uuid,
    session: Uuid,
    content: &str,
    pinned: bool,
    verification: Option<&str>,
) -> Uuid {
    let id = Uuid::now_v7();
    let content = escape(content);
    let verification = verification
        .map(|value| format!("'{}'", escape(value)))
        .unwrap_or_else(|| "NULL".into());
    pg.server.execute(&format!(
        "INSERT INTO memories
            (id, project_id, type, scope, scope_key, content, origin_session_id,
             topic_key, pinned, verification)
         VALUES ('{id}', '{project}', 'fact', 'project', '{project}', '{content}',
                 '{session}', 'continuity', {pinned}, {verification})"
    ));
    pg.attest_project_memory(
        id,
        &pg.owner,
        "Fixture author reports this continuity record.",
    );
    id
}

/// A row in `personal_knowledge`, owned by `owner` — always a retrieval
/// candidate for its owner, never for anyone else (§6.1).
fn seed_personal(pg: &Pg, owner: &Account, content: &str) -> Uuid {
    let id = Uuid::now_v7();
    let content = escape(content);
    pg.server.execute(&format!(
        "INSERT INTO personal_knowledge (id, owner_user_id, knowledge_type, content, writer_id, writer_seq)
         VALUES ('{id}', '{}', 'fact', '{content}', 'writer-{id}', 1)",
        owner.id
    ));
    id
}

/// A row in `team_knowledge`, already `authoritative` — `gather()` only ever
/// offers ratified guidance, never a `proposed` row (a proposal is not
/// guidance, and delivering one would make consolidation's own proposals read
/// as settled).
fn seed_team_authoritative(pg: &Pg, author: &Account, content: &str) -> Uuid {
    let id = Uuid::now_v7();
    let content = escape(content);
    pg.server.execute(&format!(
        "INSERT INTO team_knowledge
            (id, knowledge_type, content, state, proposed_by_user_id,
             ratified_by_user_id, ratified_at, writer_id, writer_seq)
         VALUES ('{id}', 'fact', '{content}', 'authoritative', '{}', '{}', now(),
                 'writer-{id}', 1)",
        author.id, author.id
    ));
    id
}

// ---------------------------------------------------------------------------
// Response helpers
// ---------------------------------------------------------------------------

/// The one item across every section carrying this `reference_key`, if the
/// briefing selected it.
fn find_item<'a>(resp: &'a Value, reference_key: &str) -> Option<&'a Value> {
    resp["sections"].as_object()?.values().find_map(|section| {
        section
            .as_array()?
            .iter()
            .find(|item| item["reference_key"] == reference_key)
    })
}

/// Every selected item, from every section, flattened into one list.
fn all_items(resp: &Value) -> Vec<Value> {
    resp["sections"]
        .as_object()
        .map(|sections| {
            sections
                .values()
                .flat_map(|arr| arr.as_array().cloned().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default()
}

/// Content sized to cost exactly `tokens` under the Cairn estimator: a single
/// whitespace-free run, so `estimate` is purely `chars/CHARS_PER_TOKEN`
/// rounded up, and `tokens * 3.5` is an exact integer for every even `tokens`
/// this file uses — no rounding to drift against.
fn content_with_cost(tag: &str, tokens: usize) -> String {
    let want_chars = (tokens as f64 * cairn_core::budget::CHARS_PER_TOKEN).ceil() as usize;
    let mut s = tag.to_string();
    while s.len() < want_chars {
        s.push('x');
    }
    s
}

// ---------------------------------------------------------------------------
// Domain-separated ordering (contract §3, SECTION_ORDER)
// ---------------------------------------------------------------------------

#[test]
fn project_sections_are_ranked_ahead_of_patterns_personal_notes_and_team_guidance() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    let project_id = seed_project_memory(&pg, session, "project truth about the sync boundary");
    let pattern_id = Uuid::now_v7();
    pg.seed_pattern_with_id(&pg.owner, pattern_id, "a recurring build-flake pattern");
    let personal_id = seed_personal(&pg, &pg.owner, "a personal note about local setup");
    let team_id = seed_team_authoritative(&pg, &pg.owner, "team guidance about release tags");

    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");

    let project_key = format!("knowledge:project:{project_id}");
    let pattern_key = format!("pattern:{pattern_id}");
    let personal_key = format!("knowledge:personal:{personal_id}");
    let team_key = format!("knowledge:team:{team_id}");

    let rank = |key: &str| -> i64 {
        find_item(&resp, key).unwrap_or_else(|| panic!("{key} was not delivered: {resp}"))["rank"]
            .as_i64()
            .expect("rank is an integer")
    };

    let project_rank = rank(&project_key);
    let pattern_rank = rank(&pattern_key);
    let personal_rank = rank(&personal_key);
    let team_rank = rank(&team_key);

    // DURABLE_SECTIONS processes task/branch/project memory before patterns,
    // and patterns before personal_notes/team_guidance — SECTION_ORDER's
    // domain-separated ordering, observable as a strict rank gradient rather
    // than as an assumption about JSON key order (an object's key order is
    // not part of the contract; `rank` is).
    assert!(
        project_rank < pattern_rank,
        "project ({project_rank}) did not precede patterns ({pattern_rank})"
    );
    assert!(
        pattern_rank < personal_rank,
        "patterns ({pattern_rank}) did not precede personal notes ({personal_rank})"
    );
    assert!(
        personal_rank < team_rank,
        "personal notes ({personal_rank}) did not precede team guidance ({team_rank})"
    );
}

// ---------------------------------------------------------------------------
// SC-710 — project reserve / non-displacement
// ---------------------------------------------------------------------------

#[test]
fn sc_710_project_knowledge_that_alone_fills_the_general_pool_leaves_personal_and_team_with_none_of_it(
) {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    // The reserve stays withheld for the whole of this module (it belongs to
    // the daemon's own Level 0, assembled elsewhere): `general_remaining()`
    // never exceeds `tokens - reserve`, and every durable section — project
    // memory included — draws from that same pool. So "project knowledge
    // alone fills the reserve['s ceiling on the general pool]" means project
    // content whose cost reaches `tokens - reserve`; nine rows at exactly 200
    // estimated tokens is exactly `3000 - floor(3000*0.40) = 1800`, with
    // three extra rows so the pool runs out from real content rather than
    // from an empty candidate list. Each row is verified against the
    // production estimator once below, so a drift in its rounding fails
    // loudly here rather than silently changing what "fills the pool" means.
    let unit = content_with_cost("proj", 200);
    assert_eq!(
        cairn_core::budget::estimate(&unit),
        200,
        "the token-cost fixture drifted from the production estimator"
    );
    for _ in 0..12 {
        seed_project_memory(&pg, session, &content_with_cost("proj", 200));
    }
    let personal_id = seed_personal(&pg, &pg.owner, &content_with_cost("pn", 20));
    let team_id = seed_team_authoritative(&pg, &pg.owner, &content_with_cost("tm", 20));

    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");

    let tokens = resp["budget"]["tokens"].as_u64().expect("tokens");
    let reserved = resp["budget"]["reserved_for_level0"]
        .as_u64()
        .expect("reserved_for_level0");
    assert_eq!(
        tokens, 3000,
        "the default budget was not what this test assumes: {resp}"
    );
    // The reserve itself, per the contract's own worked example (§4.1):
    // `floor(3000 * 0.40) = 1200`. Never spent by this module, no matter how
    // much durable content exists to want it.
    assert_eq!(reserved, 1200, "{resp}");

    let general_pool = tokens - reserved;
    assert_eq!(
        resp["budget"]["spent"].as_u64(),
        Some(general_pool),
        "project content alone did not exhaust the pool available to durable memory: {resp}"
    );

    let personal_key = format!("knowledge:personal:{personal_id}");
    let team_key = format!("knowledge:team:{team_id}");
    assert!(
        find_item(&resp, &personal_key).is_none(),
        "personal knowledge occupied pool space project truth alone had filled: {resp}"
    );
    assert!(
        find_item(&resp, &team_key).is_none(),
        "team guidance occupied pool space project truth alone had filled: {resp}"
    );

    // And every delivered item really is project-domain, never a fallback
    // that quietly widened the pool for anything else.
    for item in all_items(&resp) {
        assert_eq!(item["domain"], json!("project"), "{item}");
    }
}

// ---------------------------------------------------------------------------
// SC-709 — every briefing is within its stated budget
// ---------------------------------------------------------------------------

#[test]
fn sc_709_a_mixed_domain_briefing_never_exceeds_its_stated_budget_and_costs_sum_to_spent() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    seed_project_memory(
        &pg,
        session,
        "project note about the deploy pipeline retry policy",
    );
    seed_project_memory(&pg, session, "a second, shorter project note");
    seed_personal(&pg, &pg.owner, "a personal reminder about local dev setup");
    seed_team_authoritative(&pg, &pg.owner, "always run the full suite before merging");
    pg.seed_pattern_with_id(&pg.owner, Uuid::now_v7(), "a recurring deployment pattern");

    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");

    let tokens = resp["budget"]["tokens"].as_u64().expect("tokens");
    let spent = resp["budget"]["spent"].as_u64().expect("spent");
    assert!(
        spent <= tokens,
        "a briefing exceeded its own stated budget: spent {spent} > tokens {tokens}"
    );

    let items = all_items(&resp);
    assert!(!items.is_empty(), "the fixture seeded nothing deliverable");
    let sum_of_costs: u64 = items
        .iter()
        .map(|i| i["cost"].as_u64().expect("cost"))
        .sum();
    assert_eq!(
        sum_of_costs, spent,
        "the sum of per-item costs did not equal the reported spend"
    );
}

// ---------------------------------------------------------------------------
// SC-711 — stable explanations, reproducible by hand
// ---------------------------------------------------------------------------

#[test]
fn sc_711_replaying_selected_costs_in_rank_order_reproduces_the_reported_spend_exactly() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    seed_project_memory(&pg, session, "project note one, about the sync boundary");
    seed_project_memory(
        &pg,
        session,
        "project note two, rather longer than the first one",
    );
    seed_personal(&pg, &pg.owner, "a personal note of moderate length");
    seed_team_authoritative(&pg, &pg.owner, "team guidance, ratified and short");
    pg.seed_pattern_with_id(
        &pg.owner,
        Uuid::now_v7(),
        "a pattern with problem and approach text",
    );

    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");

    let tokens = resp["budget"]["tokens"].as_u64().expect("tokens") as i64;
    // The pool actually available to durable-section admission never includes
    // the withheld reserve (it belongs to the daemon's own Level 0) — reading
    // it straight from this response rather than recomputing the fraction
    // independently, so the reproduction cannot drift against whatever this
    // retrieval actually withheld.
    let reserved = resp["budget"]["reserved_for_level0"]
        .as_i64()
        .expect("reserved_for_level0");
    let mut items = all_items(&resp);
    assert!(
        items.len() >= 3,
        "not enough delivered items to be meaningful: {resp}"
    );

    items.sort_by_key(|i| i["rank"].as_i64().expect("rank"));

    // Dense ranks 1..n — no gaps, because `rank` is one counter shared across
    // every section as items are admitted.
    for (idx, item) in items.iter().enumerate() {
        assert_eq!(
            item["rank"].as_i64(),
            Some(idx as i64 + 1),
            "ranks were not dense at position {idx}: {resp}"
        );
        assert!(
            item["selection_rule"].as_str().is_some(),
            "a selected item carried no selection_rule: {item}"
        );
    }

    // The actual reproduction: replay costs in rank order and recompute what
    // budget_remaining must have been at each step, entirely from the
    // recorded inputs — not merely asserting the field is present.
    let mut running: i64 = 0;
    for item in &items {
        let cost = item["cost"].as_i64().expect("cost");
        running += cost;
        let expected_remaining = tokens - reserved - running;
        assert_eq!(
            item["budget_remaining"].as_i64(),
            Some(expected_remaining),
            "replaying costs in rank order did not reproduce budget_remaining for {item}"
        );
    }
    assert_eq!(
        running,
        resp["budget"]["spent"].as_i64().expect("spent"),
        "the replayed total did not equal the reported spend"
    );
}

// ---------------------------------------------------------------------------
// Full versus 25% incremental budgets (§4)
// ---------------------------------------------------------------------------

#[test]
fn session_open_reports_the_full_budget_and_prompt_submit_reports_twenty_five_percent() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    let (open, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{open}");
    assert_eq!(open["budget"]["tokens"].as_u64(), Some(3000), "{open}");

    let (prompt, status) = retrieve(&pg, &pg.owner, session, "prompt_submit");
    assert_eq!(status, 200, "{prompt}");
    assert_eq!(prompt["budget"]["tokens"].as_u64(), Some(750), "{prompt}");
}

// ---------------------------------------------------------------------------
// Dedup (§4, §4.1)
// ---------------------------------------------------------------------------

#[test]
fn an_unchanged_delivered_item_does_not_reappear_on_the_next_prompt_submit() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);
    let id = seed_project_memory(&pg, session, "an item that will be delivered once");
    let key = format!("knowledge:project:{id}");

    let (opened, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{opened}");
    assert!(
        find_item(&opened, &key).is_some(),
        "the item was not delivered at session open: {opened}"
    );
    let trace_id = opened["trace_id"].as_str().expect("trace_id");

    let (report, status) = report_transmitted(&pg, &pg.owner, trace_id);
    assert_eq!(status, 200, "{report}");
    assert_eq!(report["status"], json!("recorded"), "{report}");

    let (prompt, status) = retrieve(&pg, &pg.owner, session, "prompt_submit");
    assert_eq!(status, 200, "{prompt}");
    assert!(
        find_item(&prompt, &key).is_none(),
        "an unchanged, already-delivered item was resent.\n\
         The dedup rule is `source_updated_at <= delivered_at`, so the two \
         timestamps say whether this is a dedup defect or a genuinely changed \
         item. memories.updated_at={}, delivered_context.delivered_at={}.\n\
         response: {prompt}",
        pg.server.text(&format!(
            "SELECT COALESCE(max(updated_at)::text, '<none>') FROM memories WHERE id = '{id}'"
        )),
        pg.server.text(&format!(
            "SELECT COALESCE(max(delivered_at)::text, '<no delivery row>')
               FROM delivered_context
              WHERE session_id = '{session}' AND reference_key = '{key}'"
        )),
    );
    // Nothing was owed (contract §4.1's worked example, t1): a dedup-emptied
    // delivery spends zero, distinct from a failed retrieval. (Which
    // `degradation_level` this lands on is purely a function of wall-clock
    // latency against the soft/hard deadlines, so it is not asserted here —
    // doing so would make this test's pass/fail depend on how much load
    // happens to share the machine, rather than on retrieval's own logic.)
    assert_eq!(prompt["budget"]["spent"].as_u64(), Some(0), "{prompt}");
}

#[test]
fn an_item_edited_after_delivery_re_enters_on_the_next_retrieval() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);
    let id = seed_project_memory(&pg, session, "an item that will be edited after delivery");
    let key = format!("knowledge:project:{id}");

    let (opened, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{opened}");
    assert!(find_item(&opened, &key).is_some(), "{opened}");
    let trace_id = opened["trace_id"].as_str().expect("trace_id");
    let (report, status) = report_transmitted(&pg, &pg.owner, trace_id);
    assert_eq!(status, 200, "{report}");

    // Confirm dedup first: without the edit, it stays withheld.
    let (deduped, status) = retrieve(&pg, &pg.owner, session, "prompt_submit");
    assert_eq!(status, 200, "{deduped}");
    assert!(find_item(&deduped, &key).is_none(), "{deduped}");

    // Touch the record. `memories.updated_at` moving past its `delivered_at`
    // is exactly what re-qualifies it (§4's "PLUS any delivered item whose
    // updated_at > its delivered_at").
    pg.server.execute(&format!(
        "UPDATE memories SET updated_at = now() WHERE id = '{id}'"
    ));

    let (again, status) = retrieve(&pg, &pg.owner, session, "prompt_submit");
    assert_eq!(status, 200, "{again}");
    assert!(
        find_item(&again, &key).is_some(),
        "an item edited after delivery did not re-enter: {again}"
    );
}

#[test]
fn trigger_explicit_is_exempt_from_dedup_and_returns_an_already_delivered_item() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);
    let id = seed_project_memory(&pg, session, "an item requested again explicitly");
    let key = format!("knowledge:project:{id}");

    let (opened, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{opened}");
    let trace_id = opened["trace_id"].as_str().expect("trace_id");
    let (report, status) = report_transmitted(&pg, &pg.owner, trace_id);
    assert_eq!(status, 200, "{report}");

    // Confirm the ordinary dedup rule applies to prompt_submit first.
    let (deduped, status) = retrieve(&pg, &pg.owner, session, "prompt_submit");
    assert_eq!(status, 200, "{deduped}");
    assert!(find_item(&deduped, &key).is_none(), "{deduped}");

    // `explicit` is what `cairn_context`/`cairn_search` produce: a request to
    // be told again, so it is exempt (§3, §4).
    let (explicit, status) = retrieve(&pg, &pg.owner, session, "explicit");
    assert_eq!(status, 200, "{explicit}");
    assert_eq!(explicit["trigger"], json!("explicit"), "{explicit}");
    assert_eq!(explicit["delivery_point"], json!("explicit"), "{explicit}");
    assert!(
        find_item(&explicit, &key).is_some(),
        "an explicit retrieval was suppressed by dedup: {explicit}"
    );
    // Explicit is never a fraction: it uses the full budget, same as
    // session_open.
    assert_eq!(
        explicit["budget"]["tokens"].as_u64(),
        Some(3000),
        "{explicit}"
    );
}

// ---------------------------------------------------------------------------
// SC-767 — identical UUIDs across domains coexist
// ---------------------------------------------------------------------------

#[test]
fn sc_767_identical_uuids_across_domains_coexist_and_personal_delivery_does_not_suppress_team() {
    let pg = pg!();
    let ids = pg.seed_identical_ids(&pg.owner);
    // `seed_identical_ids` leaves the team row `proposed` (the schema's
    // default); `gather()` only ever offers `authoritative` guidance, so it
    // is ratified here to make it a real retrieval candidate.
    pg.server.execute(&format!(
        "UPDATE team_knowledge
            SET state = 'authoritative', ratified_by_user_id = '{}', ratified_at = now()
          WHERE id = '{}'",
        pg.owner.id, ids.id
    ));

    let session = pg.session_for(&pg.owner);
    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");

    let [project_key, personal_key, team_key, pattern_key] = ids.reference_keys();
    for key in [&project_key, &personal_key, &team_key, &pattern_key] {
        assert!(
            find_item(&resp, key).is_some(),
            "reference {key} did not survive alongside the other three sharing its id: {resp}"
        );
    }
    // The sharpest form of the claim: personal and team share the bare UUID
    // `ids.id`, and both are present in the same response, at the same time —
    // delivering one cannot have suppressed the other.
    assert_eq!(
        find_item(&resp, &personal_key).unwrap()["knowledge_id"],
        find_item(&resp, &team_key).unwrap()["knowledge_id"],
        "the two items were not actually the same id under different domains: {resp}"
    );
}

// ---------------------------------------------------------------------------
// Thin-edge Level 0 continuity
// ---------------------------------------------------------------------------

#[test]
fn a_fresh_session_gets_an_explicitly_empty_continuity_frame() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");
    assert_eq!(
        resp["continuity"],
        json!({ "previous_handoff": null, "warnings": [], "pins": [] }),
        "a fresh thin edge must receive an honest empty server-authorized frame"
    );
}

#[test]
fn ordinary_open_uses_the_bound_predecessor_handoff_while_compact_uses_the_current_session() {
    let pg = pg!();
    let previous = pg.session_for(&pg.owner);
    let current = pg.session_for(&pg.owner);
    pg.server.execute(&format!(
        "UPDATE sessions SET previous_session_id = '{previous}' WHERE id = '{current}'"
    ));
    let previous_handoff = seed_handoff(&pg, pg.project, previous, "previous");
    let current_handoff = seed_handoff(&pg, pg.project, current, "current");

    let (ordinary, status) = retrieve(&pg, &pg.owner, current, "session_open");
    assert_eq!(status, 200, "{ordinary}");
    assert_eq!(
        ordinary["continuity"]["previous_handoff"]["id"],
        previous_handoff.to_string(),
        "ordinary restoration must continue from the exact predecessor"
    );

    let (compact, status) = retrieve_with(
        &pg,
        &pg.owner,
        json!({
            "session_id": current,
            "trigger": "session_open",
            "open_trigger": "compact"
        }),
    );
    assert_eq!(status, 200, "{compact}");
    assert_eq!(
        compact["continuity"]["previous_handoff"]["id"],
        current_handoff.to_string(),
        "post-compaction restoration must use the current session boundary"
    );
}

#[test]
fn an_oversized_handoff_is_projected_to_bounded_context_fields_before_the_response() {
    let pg = pg!();
    let previous = pg.session_for(&pg.owner);
    let current = pg.session_for(&pg.owner);
    pg.server.execute(&format!(
        "UPDATE sessions SET previous_session_id = '{previous}' WHERE id = '{current}'"
    ));
    let handoff = seed_handoff(&pg, pg.project, previous, "oversized");
    let evidence = Uuid::now_v7();
    pg.server.execute(&format!(
        "UPDATE handoffs SET
             goal = repeat('g', 100000),
             progress = repeat('p', 100000),
             completed_work = jsonb_build_array(repeat('c', 100000)),
             remaining_work = (SELECT jsonb_agg(repeat('r', 2000) || n::text)
                                 FROM generate_series(1, 100) n),
             changed_files = (SELECT jsonb_agg(repeat('f', 2000) || n::text)
                                FROM generate_series(1, 100) n),
             decisions = (SELECT jsonb_agg(repeat('d', 2000) || n::text)
                            FROM generate_series(1, 100) n),
             failures = (SELECT jsonb_agg(repeat('x', 2000) || n::text)
                           FROM generate_series(1, 100) n),
             repository_state = jsonb_build_object('branch', repeat('b', 100000)),
             next_step = repeat('n', 100000),
             agent_note = repeat('a', 100000),
             observation_ids = jsonb_build_array('{evidence}')
          WHERE id = '{handoff}'"
    ));

    let (resp, status) = retrieve(&pg, &pg.owner, current, "session_open");
    assert_eq!(status, 200, "{resp}");
    let returned = &resp["continuity"]["previous_handoff"];
    assert_eq!(returned["goal"], "", "non-context handoff prose leaked");
    assert_eq!(returned["progress"], "", "non-context handoff prose leaked");
    assert_eq!(returned["completed_work"], json!([]));
    assert_eq!(returned["tests_executed"], json!([]));
    assert_eq!(returned["agent_note"], Value::Null);
    assert_eq!(returned["evidence"], json!([]));
    assert_eq!(returned["repository_state"]["branch"], "");
    assert!(returned["next_step"].as_str().unwrap().len() <= 256);
    for field in ["remaining_work", "changed_files", "decisions", "failures"] {
        let values = returned[field].as_array().unwrap();
        assert!(values.len() <= 9, "{field} was not count-bounded: {resp}");
        assert!(
            values
                .iter()
                .all(|value| value.as_str().is_some_and(|text| text.len() <= 256)),
            "{field} carried an oversized item: {resp}"
        );
    }
    assert!(
        serde_json::to_vec(&resp["continuity"]).unwrap().len() < 16 * 1024,
        "the Level 0 candidate itself is too large for a bounded response"
    );
}

#[test]
fn a_pin_older_than_twenty_four_newer_memories_still_reaches_level0() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);
    let pin = seed_continuity_memory(
        &pg,
        pg.project,
        session,
        "keep the release branch immutable",
        true,
        None,
    );
    pg.server.execute(&format!(
        "UPDATE memories SET updated_at = '2020-01-01T00:00:00Z' WHERE id = '{pin}'"
    ));
    for index in 0..25 {
        seed_continuity_memory(
            &pg,
            pg.project,
            session,
            &format!("newer ordinary memory {index}"),
            false,
            None,
        );
    }

    let (resp, status) = retrieve(&pg, &pg.owner, session, "session_open");
    assert_eq!(status, 200, "{resp}");
    assert_eq!(
        resp["continuity"]["pins"][0]["id"],
        pin.to_string(),
        "{resp}"
    );
    assert_eq!(
        resp["continuity"]["pins"][0]["text"], "keep the release branch immutable",
        "a Level 0 pin must not inherit the durable candidate window"
    );
}

#[test]
fn continuity_never_crosses_the_bound_account_or_project() {
    let pg = pg!();
    let other_project = pg.extra_project("other-continuity", &[&pg.owner]);
    let foreign_project_session = pg.session_in(other_project, &pg.owner);
    let foreign_project_handoff = seed_handoff(
        &pg,
        other_project,
        foreign_project_session,
        "foreign-project-secret",
    );
    seed_continuity_memory(
        &pg,
        other_project,
        foreign_project_session,
        "foreign-project-pin",
        true,
        Some("drifted"),
    );

    let cross_project = pg.session_for(&pg.owner);
    pg.server.execute(&format!(
        "UPDATE sessions SET previous_session_id = '{foreign_project_session}' \
         WHERE id = '{cross_project}'"
    ));
    let (project_resp, status) = retrieve(&pg, &pg.owner, cross_project, "session_open");
    assert_eq!(status, 200, "{project_resp}");
    assert!(project_resp["continuity"]["previous_handoff"].is_null());
    assert_eq!(project_resp["continuity"]["pins"], json!([]));
    assert_eq!(project_resp["continuity"]["warnings"], json!([]));
    assert!(!project_resp
        .to_string()
        .contains(&foreign_project_handoff.to_string()));
    assert!(!project_resp.to_string().contains("foreign-project"));

    let foreign_account_session = pg.session_for(&pg.member);
    seed_handoff(
        &pg,
        pg.project,
        foreign_account_session,
        "foreign-account-secret",
    );
    let cross_account = pg.session_for(&pg.owner);
    pg.server.execute(&format!(
        "UPDATE sessions SET previous_session_id = '{foreign_account_session}' \
         WHERE id = '{cross_account}'"
    ));
    let (account_resp, status) = retrieve(&pg, &pg.owner, cross_account, "session_open");
    assert_eq!(status, 200, "{account_resp}");
    assert!(account_resp["continuity"]["previous_handoff"].is_null());
    assert!(!account_resp.to_string().contains("foreign-account-secret"));
}

#[test]
fn a_tiny_budget_keeps_level0_candidates_bounded_and_leaves_durable_spend_at_zero() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);
    for index in 0..7 {
        seed_continuity_memory(
            &pg,
            pg.project,
            session,
            &format!("pinned constraint {index}"),
            true,
            None,
        );
        seed_continuity_memory(
            &pg,
            pg.project,
            session,
            &format!("drift warning {index}"),
            false,
            Some("drifted"),
        );
    }

    let (resp, status) = retrieve_with(
        &pg,
        &pg.owner,
        json!({
            "session_id": session,
            "trigger": "session_open",
            "budget_tokens": 1
        }),
    );
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["budget"]["tokens"], 1, "{resp}");
    assert_eq!(resp["budget"]["spent"], 0, "{resp}");
    let pins = resp["continuity"]["pins"].as_array().unwrap();
    let warnings = resp["continuity"]["warnings"].as_array().unwrap();
    assert_eq!(
        pins.len(),
        7,
        "server must not impose the daemon's default cap"
    );
    assert_eq!(
        warnings.len(),
        0,
        "invalidated claims must not enter reuse warnings"
    );
    assert!(pins.len() <= 24 && warnings.len() <= 24, "{resp}");
}

// ---------------------------------------------------------------------------
// Refusal (contract §3, §7)
// ---------------------------------------------------------------------------

#[test]
fn a_non_member_is_refused_403_and_never_handed_an_empty_briefing() {
    let pg = pg!();
    let session = pg.session_for(&pg.owner);

    let (resp, status) = retrieve(&pg, &pg.outsider, session, "session_open");
    assert_eq!(status, 403, "{resp}");
    assert_eq!(resp["error"]["code"], json!("forbidden"), "{resp}");
    // A refusal must never carry the shape of an empty success — that would
    // make a refusal and a legitimately empty briefing indistinguishable.
    assert!(resp.get("sections").is_none(), "{resp}");
    assert!(resp.get("budget").is_none(), "{resp}");
}

#[test]
fn a_session_belonging_to_another_account_is_refused() {
    let pg = pg!();
    // Both are members of the fixture project, so membership alone does not
    // close this: the session's *owner* is what is checked (FR-769a).
    let owners_session = pg.session_for(&pg.owner);

    let (resp, status) = retrieve(&pg, &pg.member, owners_session, "session_open");
    assert_eq!(status, 403, "{resp}");
    assert_eq!(resp["error"]["code"], json!("forbidden"), "{resp}");
    assert!(
        resp["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("another account"),
        "{resp}"
    );
}
