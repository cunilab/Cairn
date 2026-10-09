//! Server-authoritative retrieval, merged with the daemon's own Level 0
//! assembly, and the account-bound outage cache (T072;
//! `contracts/retrieval-delivery.md` §1–§6, §12.3).
//!
//! # One budget, two assemblers
//!
//! The server selects the durable sections — `session_memory`, `branch_memory`,
//! `project_memory`, `patterns`, `personal_notes`, `team_guidance` — and sends
//! the authenticated continuity inputs it alone owns: previous handoff,
//! warnings, and pins. It reports what durable selection spent
//! (`budget.tokens`, `budget.spent`) plus what it withheld for Level 0
//! (`budget.reserved_for_level0`). This module gives the one Level 0 assembler
//! exactly what remains — `tokens - spent` — and combines those server inputs
//! with the repository working state derived on this machine. It never
//! recomputes the reserve fraction, and it never reads historical SQLite state.
//!
//! `patterns` is taken from the server like every other durable section, and
//! for a stricter reason than the others. The server selects a canonical
//! `shared_patterns` row, budgets it, and traces its `pattern_id` as
//! *selected*; on the transmission report it copies that same id into
//! `delivered_context`. So the id the server will record as delivered is
//! fixed before this module runs, and the only way that record can be true is
//! for the content rendered here to be that exact canonical pattern. The
//! server therefore sends the pattern's own fields alongside its id
//! (`cairn-server/src/retrieve.rs::SectionPattern`) and the merge below
//! renders those fields under that id.
//!
//! # The outage cache (§12.3, FR-789, FR-790a, SC-718)
//!
//! Retrieval moved server-side, so an outage means no fresh *durable*
//! knowledge. The cache below holds the server's last answer per session,
//! bound to the account it was assembled for, and is consulted only when the
//! server cannot be reached at all this call.
//!
//! **Server-owned Level 0 is not always current, and saying so was the FR-790a
//! defect.** With no fresh response and no cache entry *for this account*, the
//! daemon serves only repository state: no handoff, warnings, pins, or durable
//! knowledge. The local edge is deliberately not a fallback authority for any
//! of those fields.

use crate::state::{Daemon, Resolved};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trigger
// ---------------------------------------------------------------------------

/// Why retrieval ran (`contracts/retrieval-delivery.md` §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    SessionOpen,
    PromptSubmit,
    Explicit,
}

impl Trigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Trigger::SessionOpen => "session_open",
            Trigger::PromptSubmit => "prompt_submit",
            Trigger::Explicit => "explicit",
        }
    }

    /// Parse the wire value `Request::Context::trigger` carries.
    ///
    /// Anything unrecognized — including absent, which is how every caller
    /// written before this field existed still parses — becomes `Explicit`,
    /// never an automatic trigger: `explicit` is the one value that asserts
    /// no push and permits no transmission report, so it is the safe
    /// direction to fall in when a value cannot be trusted (§3).
    pub fn parse(s: &str) -> Self {
        match s {
            "session_open" => Trigger::SessionOpen,
            "prompt_submit" => Trigger::PromptSubmit,
            _ => Trigger::Explicit,
        }
    }
}

// ---------------------------------------------------------------------------
// The outage cache
// ---------------------------------------------------------------------------

const CACHE_MAX_SESSIONS: usize = 200;
const CACHE_MAX_BYTES: usize = 64 * 1024;
const CACHE_TTL: Duration = Duration::from_secs(300);
const PROJECT_REUSE_POLICY: &str = "project_attestation_v1";

fn project_reuse_confirmed(response: &Value) -> bool {
    response.get("project_reuse_policy").and_then(Value::as_str) == Some(PROJECT_REUSE_POLICY)
}

/// Context carries no task query, including when talking to an older server.
fn withhold_project_bodies_without_query(response: &mut Value) {
    let mut removed = false;
    if let Some(sections) = response.get_mut("sections").and_then(Value::as_object_mut) {
        for section in ["session_memory", "branch_memory", "project_memory"] {
            removed |= sections
                .remove(section)
                .is_some_and(|v| v.as_array().is_none_or(|a| !a.is_empty()));
        }
    }
    if removed {
        if let Some(object) = response.as_object_mut() {
            // The old trace selected bodies we did not deliver; never report it transmitted.
            object.remove("trace_id");
            object.insert("project_memory_withheld".into(), json!(true));
            object.insert(
                "project_memory_withheld_reason".into(),
                json!("task_query_required"),
            );
        }
    }
}

/// Remove server project-memory claims when their current eligibility cannot
/// be established. Other domains and the prior handoff keep their existing
/// authority contracts.
fn withhold_unconfirmed_project_memory(response: &mut Value, reason: &'static str) {
    if let Some(sections) = response.get_mut("sections").and_then(Value::as_object_mut) {
        for section in ["session_memory", "branch_memory", "project_memory"] {
            sections.remove(section);
        }
    }
    if let Some(continuity) = response
        .get_mut("continuity")
        .and_then(Value::as_object_mut)
    {
        continuity.insert("warnings".into(), json!([]));
        continuity.insert("pins".into(), json!([]));
    }
    if let Some(object) = response.as_object_mut() {
        object.remove("project_memory_available");
        object.insert("project_memory_withheld".into(), json!(true));
        object.insert("project_memory_withheld_reason".into(), json!(reason));
    }
}

struct CachedResponse {
    account_id: Uuid,
    trigger: Trigger,
    open_trigger: Option<String>,
    budget_tokens: usize,
    cached_at: std::time::Instant,
    /// The server's own answer, verbatim — `sections`, `degradation_level`,
    /// `budget`, `trace_id` and all. Read back through the same parser a
    /// fresh response goes through ([`ResponseMeta::extract`]), with
    /// `from_cache: true` so its `trace_id` is discarded rather than replayed
    /// against a report the server never asked for.
    response: Value,
}

/// Last briefing per session, account- and request-bound, LRU-evicted at
/// [`CACHE_MAX_SESSIONS`] sessions, each entry capped at [`CACHE_MAX_BYTES`].
///
/// A cache, not durable state (Principle II): in-memory, lost on restart, and
/// rebuilt by the next successful retrieval. It exists solely so a server
/// outage degrades the durable half of a briefing rather than blanking it.
/// A *hit* is also the evidence that the server authorized this account for
/// this session, which is exactly what a miss does not have — see the module
/// header for what a miss may therefore serve.
#[derive(Default)]
pub struct OutageCache {
    /// Most-recently-used session id first.
    order: VecDeque<Uuid>,
    entries: HashMap<Uuid, CachedResponse>,
}

impl OutageCache {
    fn touch(&mut self, session_id: Uuid) {
        self.order.retain(|s| *s != session_id);
        self.order.push_front(session_id);
    }

    /// Refill on a successful retrieval (§12.3).
    ///
    /// An over-budget response is **rejected outright, never truncated**: a
    /// truncated durable section would misrepresent what the server actually
    /// said the last time it was reachable, which is worse than simply not
    /// caching it. The session keeps whatever entry it already had.
    #[allow(clippy::too_many_arguments)]
    fn put(
        &mut self,
        session_id: Uuid,
        account_id: Uuid,
        trigger: Trigger,
        open_trigger: Option<&str>,
        budget_tokens: usize,
        response: &Value,
    ) {
        let bytes = serde_json::to_vec(response)
            .map(|b| b.len())
            .unwrap_or(usize::MAX);
        if bytes > CACHE_MAX_BYTES {
            return;
        }
        self.entries.insert(
            session_id,
            CachedResponse {
                account_id,
                trigger,
                open_trigger: open_trigger.map(str::to_owned),
                budget_tokens,
                cached_at: std::time::Instant::now(),
                response: response.clone(),
            },
        );
        self.touch(session_id);
        while self.order.len() > CACHE_MAX_SESSIONS {
            if let Some(evicted) = self.order.pop_back() {
                self.entries.remove(&evicted);
            }
        }
    }

    /// Served only for the account and exact retrieval request it was assembled
    /// for (FR-790a). A different budget, trigger, or session-open reason may
    /// have selected different content, so it is a miss rather than a replay.
    fn get(
        &mut self,
        session_id: Uuid,
        account_id: Uuid,
        trigger: Trigger,
        open_trigger: Option<&str>,
        budget_tokens: usize,
    ) -> Option<Value> {
        let hit = self.entries.get(&session_id)?;
        if hit.account_id != account_id
            || hit.trigger != trigger
            || hit.open_trigger.as_deref() != open_trigger
            || hit.budget_tokens != budget_tokens
        {
            return None;
        }
        if hit.cached_at.elapsed() > CACHE_TTL {
            self.entries.remove(&session_id);
            self.order.retain(|id| *id != session_id);
            return None;
        }
        let mut response = hit.response.clone();
        if let Some(object) = response.as_object_mut() {
            object.insert(
                "cache_age_seconds".into(),
                json!(hit.cached_at.elapsed().as_secs()),
            );
            object.insert("cache_account_id".into(), json!(account_id));
        }
        // A cache hit cannot prove that a server-side supersession, conflict,
        // or dependency revision did not happen after this answer was stored.
        withhold_unconfirmed_project_memory(&mut response, "outage_cache_cannot_revalidate");
        self.touch(session_id);
        Some(response)
    }

    fn invalidate(&mut self, session_id: Uuid, account_id: Uuid) {
        if self
            .entries
            .get(&session_id)
            .is_some_and(|entry| entry.account_id == account_id)
        {
            self.entries.remove(&session_id);
            self.order.retain(|id| *id != session_id);
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

// ---------------------------------------------------------------------------
// Delivery
// ---------------------------------------------------------------------------

/// What one delivery produced, ready for a caller to render and inspect for a
/// fresh `trace_id` before reporting the transmission outcome.
pub struct Delivered {
    pub payload: Value,
}

/// Retrieve, merge with the daemon's own Level 0 assembly, and fall back to
/// the outage cache when the server cannot be reached within `deadline`
/// (`contracts/retrieval-delivery.md` §1–§6, §12.3). `deadline` is the
/// existing `context_deadline_ms` — this module introduces no deadline
/// constant of its own.
#[allow(clippy::too_many_arguments)]
pub async fn deliver(
    d: &Daemon,
    resolved: &Resolved,
    session_id: Uuid,
    trigger: Trigger,
    open_trigger: Option<&str>,
    // What this machine may spend. Sent to the server rather than applied
    // afterwards, because the two assemblers share one budget and only the side
    // that selects first can keep the total inside it — trimming the answer
    // here would already have exceeded it.
    budget_tokens: usize,
    deadline: Duration,
    query: Option<&str>,
) -> Delivered {
    let started = std::time::Instant::now();
    let account_id = d.account_identity().await;

    // A session created moments ago may still be in a typed durable lane.
    // Drain one bounded pass before retrieval so the server can bind it. This
    // deliberately does not invoke legacy entity sync or any pull path.
    let _ = tokio::time::timeout(
        (deadline / 2).min(remaining_deadline(deadline, started)),
        crate::sync::drain_typed_spools(d),
    )
    .await;

    // A timeout is silence, exactly as a transport failure is.
    let remote = tokio::time::timeout(
        remaining_deadline(deadline, started),
        retrieve_remote(d, session_id, trigger, open_trigger, budget_tokens, query),
    )
    .await
    .unwrap_or(Answer::Unreachable);

    let (response, served_from_cache) = match remote {
        Answer::Answered(mut response) => {
            if !response_budget_is_valid(&response, budget_tokens) {
                return unavailable_delivery(d, resolved, budget_tokens, deadline, started).await;
            }
            if let Some(query) = query {
                if response["project_recall_policy"] != cairn_core::reuse::TASK_QUERY_POLICY
                    || response["project_query_sha256"] != cairn_core::digest(query)
                {
                    return unavailable_delivery(d, resolved, budget_tokens, deadline, started)
                        .await;
                }
            }
            if query.is_none() {
                withhold_project_bodies_without_query(&mut response);
            }
            if !project_reuse_confirmed(&response) {
                withhold_unconfirmed_project_memory(
                    &mut response,
                    "server_did_not_confirm_reuse_policy",
                );
            }
            if let Some(account_id) = account_id.filter(|_| query.is_none()) {
                d.outage_cache.lock().await.put(
                    session_id,
                    account_id,
                    trigger,
                    open_trigger,
                    budget_tokens,
                    &response,
                );
            }
            (Some(response), false)
        }
        // Refused, by something that was there to refuse. No cache, because a
        // hit would claim an authorization this very call was denied.
        Answer::Refused => {
            if let Some(account_id) = account_id {
                d.outage_cache
                    .lock()
                    .await
                    .invalidate(session_id, account_id);
            }
            (None, false)
        }
        Answer::Rejected => (None, false),
        Answer::Unreachable => {
            // Explicit query results never borrow another query's cached selection.
            let cached = match account_id.filter(|_| query.is_none()) {
                Some(account_id) => d.outage_cache.lock().await.get(
                    session_id,
                    account_id,
                    trigger,
                    open_trigger,
                    budget_tokens,
                ),
                None => None,
            };
            match cached {
                Some(cached) if response_budget_is_valid(&cached, budget_tokens) => {
                    (Some(cached), true)
                }
                None => (None, false),
                Some(_) => (None, false),
            }
        }
    };

    let mut meta = ResponseMeta::extract(response.as_ref(), served_from_cache);
    let local_budget = meta.local_budget(budget_tokens);
    let continuity = response
        .as_ref()
        .and_then(|answer| answer.get("continuity"));
    let payload_result = tokio::time::timeout(
        remaining_deadline(deadline, started),
        crate::briefing::build(d, resolved, continuity, response.is_some(), local_budget),
    )
    .await;
    let mut payload = match payload_result {
        Ok(Ok(built)) => serde_json::to_value(built).unwrap_or_else(|_| json!({})),
        Ok(Err(e)) => {
            meta.degradation_level = "none".into();
            json!({ "error": e.message })
        }
        Err(_) => {
            meta.degradation_level = "none".into();
            json!({
                "fresh_knowledge_unavailable": true,
                "degradation_level": "none",
                "sections": {},
            })
        }
    };

    match response.as_ref().and_then(|answer| answer.get("sections")) {
        Some(sections) => merge_durable_sections(&mut payload, sections),
        None => {
            if let Some(object) = payload.as_object_mut() {
                object.insert("fresh_knowledge_unavailable".into(), json!(true));
            }
        }
    }
    // The local assembler reports only what it spent. Add the server's
    // already-budgeted durable spend and restore the caller-visible whole
    // budget; `local_budget = tokens - spent` keeps the sum within it.
    if meta.answered {
        let local_spent = payload
            .get("estimated_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        if let Some(object) = payload.as_object_mut() {
            object.insert(
                "estimated_tokens".into(),
                json!(local_spent.saturating_add(meta.spent)),
            );
            object.insert("budget".into(), json!(meta.tokens));
        }
    }
    if payload.get("degraded").and_then(Value::as_bool) == Some(true)
        && meta.degradation_level == "full"
    {
        meta.degradation_level = "reduced".into();
    }
    copy_response_envelope(response.as_ref(), &mut payload);
    embed_meta(&mut payload, &meta, served_from_cache);

    Delivered { payload }
}

async fn unavailable_delivery(
    d: &Daemon,
    resolved: &Resolved,
    budget_tokens: usize,
    deadline: Duration,
    started: std::time::Instant,
) -> Delivered {
    let built = tokio::time::timeout(
        remaining_deadline(deadline, started),
        crate::briefing::build(d, resolved, None, false, budget_tokens),
    )
    .await;
    let mut payload = match built {
        Ok(Ok(payload)) => serde_json::to_value(payload).unwrap_or_else(|_| json!({})),
        Ok(Err(error)) => json!({ "error": error.message }),
        Err(_) => json!({}),
    };
    if let Some(object) = payload.as_object_mut() {
        object.insert("fresh_knowledge_unavailable".into(), json!(true));
    }
    embed_meta(&mut payload, &ResponseMeta::unavailable(), false);
    Delivered { payload }
}

fn remaining_deadline(deadline: Duration, started: std::time::Instant) -> Duration {
    deadline.saturating_sub(started.elapsed())
}

/// A server answer must not be able to widen the caller's budget, and a cache
/// entry created for a larger request must not be replayed into a smaller one.
fn response_budget_is_valid(response: &Value, requested: usize) -> bool {
    let Some(budget) = response.get("budget") else {
        return false;
    };
    let (Some(tokens), Some(spent), Some(reserved)) = (
        budget.get("tokens").and_then(Value::as_u64),
        budget.get("spent").and_then(Value::as_u64),
        budget.get("reserved_for_level0").and_then(Value::as_u64),
    ) else {
        return false;
    };
    let requested = requested as u64;
    tokens <= requested && spent <= tokens && reserved <= tokens.saturating_sub(spent)
}

/// Report what actually happened to a generated briefing
/// (`contracts/retrieval-delivery.md` §3, §6.2).
///
/// Best-effort and idempotent by construction: the server answers a repeated
/// identical report with `duplicate` rather than an error (§3), so a caller
/// retrying after a dropped response needs no retry loop of its own here.
///
/// **Never call this with `transmitted: true` without having actually
/// written the context to the hook's return channel.** Generating a briefing
/// is not evidence that an agent received one (FR-843, FR-854) — that is the
/// entire reason this is a second call, made by the daemon after the caller
/// tells it what happened, rather than something `deliver` claims on its own.
pub async fn report_outcome(d: &Daemon, trace_id: Uuid, transmitted: bool, reason: Option<&str>) {
    let creds = d.server.read().await.clone();
    let (Some(base), Some(token)) = (creds.url, creds.token) else {
        return;
    };
    let Ok(http) = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    else {
        return;
    };
    let body = if transmitted {
        json!({ "outcome": "transmitted" })
    } else {
        json!({
            "outcome": "failed",
            "failure_reason": reason.unwrap_or("hook_transmission_failed"),
        })
    };
    let url = format!(
        "{}/api/retrieval-traces/{trace_id}/transmission",
        base.trim_end_matches('/')
    );
    if let Err(e) = http.post(url).bearer_auth(token).json(&body).send().await {
        tracing::debug!(error = %e, %trace_id, "transmission outcome not reported");
    }
}

/// `POST /api/retrieve`, resolved into the three outcomes the outage cache
/// turns on ([`Answer`]).
///
/// **The caller must tell these apart, and folding them together was a
/// defect.** Only [`Answer::Unreachable`] permits the cache to answer, so the
/// mapping is the whole authorization story of an outage:
///
/// - **no credential**, a client that will not build, a transport failure, or
///   a 2xx whose body will not parse → [`Answer::Unreachable`]. Nothing
///   answered, or nothing intelligible did, so a previously authorized entry
///   for this account and session may still stand in (§12.3).
/// - **401, 403, 404** → [`Answer::Refused`]. Authentication or existence was
///   denied, so matching cache is invalidated.
/// - **5xx** → [`Answer::Unreachable`]. Server failure is outage, not auth.
/// - **other 4xx** → [`Answer::Rejected`]. No cache this turn, no invalidation.
async fn retrieve_remote(
    d: &Daemon,
    session_id: Uuid,
    trigger: Trigger,
    open_trigger: Option<&str>,
    budget_tokens: usize,
    query: Option<&str>,
) -> Answer {
    let creds = d.server.read().await.clone();
    let (Some(base), Some(token)) = (creds.url, creds.token) else {
        return Answer::Unreachable;
    };
    let Ok(http) = reqwest::Client::builder().build() else {
        return Answer::Unreachable;
    };

    let mut body = json!({
        "session_id": session_id,
        "trigger": trigger.as_str(),
        // The server clamps this to its own figure, so asking is always safe
        // and never widens anything.
        "budget_tokens": budget_tokens,
    });
    if let Some(query) = query {
        body["query"] = json!(query);
    }
    // `open_trigger` belongs to a `session_open` retrieval and to no other
    // (the server refuses it otherwise) — never sent for the other two.
    if trigger == Trigger::SessionOpen {
        if let Some(ot) = open_trigger {
            body["open_trigger"] = json!(ot);
        }
    }

    let url = format!("{}/api/retrieve", base.trim_end_matches('/'));
    let Ok(response) = http.post(url).bearer_auth(token).json(&body).send().await else {
        // Nothing answered. This is the outage the cache exists for.
        return Answer::Unreachable;
    };
    if response.status().is_server_error() {
        return Answer::Unreachable;
    }
    if !response.status().is_success() {
        // **Something answered, and it refused.**
        //
        // This used to be folded into "unreachable", and the fold was a
        // cross-deployment leak: replace the server at the same address and the
        // old token authenticates against nothing there, so every retrieval
        // came back `401` — which looked exactly like silence, so the daemon
        // served the *predecessor's* cached briefing and labelled it cached, as
        // though the new deployment had authorized it. A cache hit is supposed
        // to be evidence that the server authorized this account for this
        // session; a live refusal is evidence of the opposite, and it cannot be
        // allowed to produce one.
        return match response.status().as_u16() {
            401 | 403 | 404 => Answer::Refused,
            _ => Answer::Rejected,
        };
    }
    match response.json::<Value>().await {
        Ok(value) => Answer::Answered(value),
        // A 2xx whose body will not parse is a server that answered
        // incomprehensibly, not one that declined. Treated as silence.
        Err(_) => Answer::Unreachable,
    }
}

/// What `/api/retrieve` did, distinguished because the cache turns on it.
enum Answer {
    Answered(Value),
    /// Reachable, and it declined — a wrong deployment, a revoked token, a
    /// session it does not hold. The outage cache must not answer for it.
    Refused,
    Rejected,
    /// Nothing answered at all.
    Unreachable,
}

// ---------------------------------------------------------------------------
// Reading the server's answer
// ---------------------------------------------------------------------------

/// The parts of `/api/retrieve`'s response this module reasons about,
/// pulled out of the raw `Value` once so the rest of the module never
/// re-parses it.
struct ResponseMeta {
    answered: bool,
    trace_id: Option<Uuid>,
    degradation_level: String,
    tokens: usize,
    spent: usize,
}

impl ResponseMeta {
    /// No response at all — the server was unreachable and nothing was
    /// cached for this session and account. `none` here is not a claim that
    /// the briefing is empty (Level 0 never is): it says durable retrieval
    /// produced nothing, which is true because none was attempted (§5's
    /// `none` row: "retrieval produced nothing").
    fn unavailable() -> Self {
        Self {
            answered: false,
            trace_id: None,
            degradation_level: "none".to_string(),
            tokens: 0,
            spent: 0,
        }
    }

    /// `from_cache` discards `trace_id`: a cached answer's trace was already
    /// resolved (`generated` → `transmitted` or `failed`) the call it was
    /// captured on, and replaying its id here would let a later transmission
    /// report land against a trace this call never asked the server to make
    /// (§3's idempotency is about *repeating* a report, not about reusing a
    /// stale identity for a new one).
    fn extract(response: Option<&Value>, from_cache: bool) -> Self {
        let Some(response) = response else {
            return Self::unavailable();
        };
        Self {
            answered: true,
            trace_id: if from_cache {
                None
            } else {
                response
                    .get("trace_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
            },
            degradation_level: response
                .get("degradation_level")
                .and_then(|v| v.as_str())
                .unwrap_or("none")
                .to_string(),
            tokens: response
                .get("budget")
                .and_then(|b| b.get("tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
            spent: response
                .get("budget")
                .and_then(|b| b.get("spent"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
        }
    }

    /// What the daemon's own Level 0 / local-section assembly may spend.
    ///
    /// `tokens - spent` when the server answered (fresh or cached) — a number
    /// the server guarantees is never less than what it withheld for exactly
    /// this (`budget.reserved_for_level0`), so this never recomputes that
    /// fraction itself. The whole local budget when nothing durable was
    /// retrieved at all: nothing else claimed a share of it that time.
    fn local_budget(&self, full: usize) -> usize {
        if !self.answered {
            full
        } else {
            self.tokens.saturating_sub(self.spent)
        }
    }
}

/// One durable section's admitted content, in the order the server admitted
/// it, discarding everything but the rendered text — reference keys, ranks
/// and costs are trace-only detail (`contracts/retrieval-delivery.md` §6),
/// not briefing content.
fn section_contents(sections: &Value, name: &str) -> Vec<String> {
    sections
        .get(name)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|it| {
                    it.get("content")
                        .and_then(|c| c.as_str())
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Replace the daemon's own (locally recomputed, undeduplicated) durable
/// section content with the server's. Selection and dedup happened
/// server-side against `delivered_context`, which the daemon's own read of
/// the same tables never sees (`contracts/retrieval-delivery.md` §4) — so the
/// server's answer, not the daemon's own read, is what a caller must be
/// shown. `patterns` is rendered from the canonical fields the server sent
/// with its selection, under the very id the server traced; see the module
/// docs for why that one is not merely a preference.
fn merge_durable_sections(payload: &mut Value, sections: &Value) {
    let Some(briefing) = payload.get_mut("briefing").and_then(|b| b.as_object_mut()) else {
        return;
    };
    if let Some(memory) = briefing.get_mut("memory").and_then(|m| m.as_object_mut()) {
        memory.insert(
            "session".into(),
            json!(section_contents(sections, "session_memory")),
        );
        memory.insert(
            "branch".into(),
            json!(section_contents(sections, "branch_memory")),
        );
        memory.insert(
            "project".into(),
            json!(section_contents(sections, "project_memory")),
        );
    }
    // `Briefing`'s own fields are `#[serde(skip_serializing_if =
    // "Vec::is_empty")]` (FR-481: byte-identical output for a caller with
    // nothing in either domain), which only governs serializing *from* the
    // struct. This is a raw JSON merge after that already happened, so an
    // empty section is dropped here rather than inserted as a present but
    // empty array.
    for key in ["personal_notes", "team_guidance"] {
        let items = section_contents(sections, key);
        if items.is_empty() {
            briefing.remove(key);
        } else {
            briefing.insert(key.into(), json!(items));
        }
    }

    // **The server's canonical patterns, and only those.**
    //
    // This section used not to be merged at all, and the omission was the
    // canonical pattern-delivery defect. The server selected a `PatternRef`,
    // spent budget on it and recorded it as selected; the daemon then rendered
    // whatever its *local* matcher found in `reusable_patterns` — a different
    // store, with different identities — and reported the transmission
    // successful, at which point the server copied its selected refs into
    // `delivered_context`. A reference could be recorded as delivered without
    // the pattern behind it ever having been rendered.
    //
    // Each item is rendered under the id the server traced, from the canonical
    // fields the server sent with it, so the reference, the budgeted content
    // and the text the agent reads are one record. `signal_overlap` is absent
    // because no signal comparison ran: the server selected by budget.
    let patterns: Vec<Value> = sections
        .get("patterns")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = item.get("knowledge_id")?.as_str()?;
                    let p = item.get("pattern")?;
                    Some(json!({
                        "id": id,
                        "title": p.get("title").and_then(Value::as_str).unwrap_or_default(),
                        "trust": p.get("trust").and_then(Value::as_str).unwrap_or("sanitized"),
                        "verified_in_this_project": false,
                        "applicability": p.get("applicability").cloned().unwrap_or(json!([])),
                        "approach": p.get("approach").and_then(Value::as_str).unwrap_or_default(),
                        "constraints": p.get("constraints").cloned().unwrap_or(json!([])),
                    }))
                })
                .collect()
        })
        .unwrap_or_default();
    if patterns.is_empty() {
        briefing.remove("patterns");
    } else {
        briefing.insert("patterns".into(), json!(patterns));
    }
}

/// Preserve the non-content parts of the authenticated retrieval envelope.
/// `sections` and `continuity` are deliberately excluded: both were consumed
/// through their typed/budgeted paths above rather than copied around them.
fn copy_response_envelope(response: Option<&Value>, payload: &mut Value) {
    let (Some(source), Some(target)) =
        (response.and_then(Value::as_object), payload.as_object_mut())
    else {
        return;
    };
    for key in [
        "trigger",
        "delivery_point",
        "open_trigger",
        "restored_after_compaction",
        "cache_age_seconds",
        "cache_account_id",
        "project_reuse_policy",
        "project_memory_available",
        "project_recall_policy",
        "project_query_sha256",
        "project_memory_withheld",
        "project_memory_withheld_reason",
    ] {
        if let Some(value) = source.get(key) {
            target.insert(key.into(), value.clone());
        }
    }
}

/// Add what a caller needs beyond the rendered briefing itself: whether this
/// answer is fresh or replayed, at what level, and — only when it is fresh —
/// the trace to report a transmission outcome against.
fn embed_meta(payload: &mut Value, meta: &ResponseMeta, served_from_cache: bool) {
    let Some(obj) = payload.as_object_mut() else {
        return;
    };
    obj.insert(
        "trace_id".into(),
        meta.trace_id.map(|id| json!(id)).unwrap_or(Value::Null),
    );
    obj.insert("degradation_level".into(), json!(meta.degradation_level));
    obj.insert("served_from_cache".into(), json!(served_from_cache));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ServerCredentials;
    use crate::testsupport as fx;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn older_context_cannot_restore_unqueried_project_bodies_or_claim_transmission() {
        let mut answer = json!({
            "trace_id": Uuid::now_v7(),
            "sections": {
                "session_memory": [{"content": "unqueried scratch finding"}],
                "branch_memory": [{"content": "unqueried branch finding"}],
                "project_memory": [{"content": "unqueried project finding"}],
                "personal_notes": [{"content": "authorized personal guidance"}]
            },
            "continuity": {"pins": [{"text": "explicitly pinned constraint"}]}
        });
        withhold_project_bodies_without_query(&mut answer);
        assert!(!answer.to_string().contains("unqueried"));
        assert!(answer.get("trace_id").is_none());
        assert_eq!(
            answer["project_memory_withheld_reason"],
            "task_query_required"
        );
        assert_eq!(
            answer["sections"]["personal_notes"][0]["content"],
            "authorized personal guidance"
        );
        assert_eq!(
            answer["continuity"]["pins"][0]["text"],
            "explicitly pinned constraint"
        );
    }

    fn response(trace: &str, level: &str, tokens: u64, spent: u64) -> Value {
        json!({
            "trace_id": trace,
            "project_reuse_policy": PROJECT_REUSE_POLICY,
            "degradation_level": level,
            "budget": { "tokens": tokens, "spent": spent, "reserved_for_level0": tokens * 4 / 10 },
            "sections": {
                "personal_notes": [{ "content": "p1" }],
                "team_guidance": [{ "content": "g1" }],
                "session_memory": [{ "content": "s1" }],
            },
        })
    }

    fn continuity() -> Value {
        json!({
            "previous_handoff": {
                "id": "0199b6d0-d228-7b91-a420-2f935a954731",
                "session_id": "0199b6d0-d228-7b91-a420-2f935a954732",
                "trigger": "session_end",
                "goal": "ship alpha.9",
                "progress": "candidate built",
                "completed_work": ["built candidate"],
                "remaining_work": ["publish release"],
                "changed_files": ["crates/cairnd/src/deliver.rs"],
                "decisions": ["server authorizes continuity"],
                "failures": ["old response omitted Level 0"],
                "tests_executed": [],
                "repository_state": {
                    "branch": "main",
                    "commit_sha": "abc1234",
                    "staged": 0,
                    "unstaged": 0,
                    "untracked": 0
                },
                "next_step": "publish release",
                "agent_note": null,
                "evidence": [],
                "created_at": "2026-10-04T00:00:00Z",
                "deleted_at": null
            },
            "warnings": [{
                "kind": "conflict",
                "subject": "release channel",
                "detail": "alpha and stable both recorded"
            }],
            "pins": [{
                "id": "0199b6d0-d228-7b91-a420-2f935a954733",
                "text": "never publish untested artifacts",
                "drifted": false
            }]
        })
    }

    async fn serve_once(status: &'static str, body: Value) -> String {
        let body = serde_json::to_vec(&body).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = socket.read(&mut request).await.unwrap();
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
            socket.write_all(&body).await.unwrap();
        });
        format!("http://{address}")
    }

    fn cache_put(cache: &mut OutageCache, session: Uuid, account: Uuid, response: &Value) {
        cache.put(session, account, Trigger::Explicit, None, 3000, response);
    }

    fn cache_get(cache: &mut OutageCache, session: Uuid, account: Uuid) -> Option<Value> {
        cache.get(session, account, Trigger::Explicit, None, 3000)
    }

    #[test]
    fn trigger_parses_the_three_wire_values_and_nothing_else_as_automatic() {
        assert_eq!(Trigger::parse("session_open"), Trigger::SessionOpen);
        assert_eq!(Trigger::parse("prompt_submit"), Trigger::PromptSubmit);
        // Absent, misspelled, or anything else Cairn has never declared: the
        // one direction that asserts no push (§3).
        assert_eq!(Trigger::parse("explicit"), Trigger::Explicit);
        assert_eq!(Trigger::parse("bogus"), Trigger::Explicit);
        assert_eq!(Trigger::parse(""), Trigger::Explicit);
    }

    #[test]
    fn trigger_as_str_round_trips_through_parse() {
        for t in [
            Trigger::SessionOpen,
            Trigger::PromptSubmit,
            Trigger::Explicit,
        ] {
            assert_eq!(Trigger::parse(t.as_str()), t);
        }
    }

    #[test]
    fn retrieval_receives_only_the_delivery_budget_left_after_drain() {
        let deadline = Duration::from_millis(100);
        let started = std::time::Instant::now() - Duration::from_millis(60);
        let remaining = remaining_deadline(deadline, started);
        assert!(remaining <= Duration::from_millis(40));
        assert!(remaining > Duration::ZERO);
        assert_eq!(
            remaining_deadline(
                deadline,
                std::time::Instant::now() - Duration::from_millis(101)
            ),
            Duration::ZERO
        );
    }

    #[test]
    fn retrieval_budget_metadata_cannot_widen_or_overdraw_the_request() {
        assert!(response_budget_is_valid(
            &response("t", "full", 3000, 100),
            3000
        ));
        assert!(!response_budget_is_valid(
            &response("t", "full", 3001, 100),
            3000
        ));
        assert!(!response_budget_is_valid(
            &response("t", "full", 3000, 3001),
            3000
        ));
        let mut steals_reserve = response("t", "full", 3000, 2000);
        steals_reserve["budget"]["reserved_for_level0"] = json!(1200);
        assert!(!response_budget_is_valid(&steals_reserve, 3000));
    }

    // -- OutageCache -----------------------------------------------------

    /// The invariant the caller specifically asked to see tested: a cached
    /// entry is bound to the account it was assembled for and is never
    /// served to a different one (FR-790a). Not "empty" or "an error" — the
    /// same as no entry existing at all, so a second account cannot even
    /// learn that a first account has a cached briefing here.
    #[test]
    fn a_cached_entry_never_crosses_accounts() {
        let mut cache = OutageCache::default();
        let session = Uuid::now_v7();
        let owner = Uuid::now_v7();
        let intruder = Uuid::now_v7();

        cache_put(
            &mut cache,
            session,
            owner,
            &response("t1", "full", 3000, 100),
        );

        assert!(
            cache_get(&mut cache, session, intruder).is_none(),
            "a different account must not read the owner's cached briefing"
        );
        assert!(
            cache
                .get(session, owner, Trigger::PromptSubmit, None, 3000)
                .is_none(),
            "a different delivery point must not reuse an explicit answer"
        );
        assert!(
            cache_get(&mut cache, session, owner).is_some(),
            "the owning account's own read must still succeed"
        );
    }

    /// Refilled on every successful retrieval, and the newest answer is what
    /// a later outage replays (§12.3).
    #[test]
    fn a_second_put_for_the_same_session_replaces_the_first() {
        let mut cache = OutageCache::default();
        let session = Uuid::now_v7();
        let owner = Uuid::now_v7();

        cache_put(
            &mut cache,
            session,
            owner,
            &response("t1", "full", 3000, 100),
        );
        cache_put(
            &mut cache,
            session,
            owner,
            &response("t2", "reduced", 3000, 40),
        );

        let got = cache_get(&mut cache, session, owner).expect("entry");
        assert_eq!(got["trace_id"], "t2");
        assert_eq!(got["cache_account_id"], owner.to_string());
        assert!(got["cache_age_seconds"].is_u64());
    }

    #[test]
    fn an_expired_entry_cannot_answer_an_outage() {
        let mut cache = OutageCache::default();
        let session = Uuid::now_v7();
        let owner = Uuid::now_v7();
        cache_put(
            &mut cache,
            session,
            owner,
            &response("t1", "full", 3000, 100),
        );
        cache.entries.get_mut(&session).unwrap().cached_at =
            std::time::Instant::now() - CACHE_TTL - Duration::from_secs(1);
        assert!(cache_get(&mut cache, session, owner).is_none());
    }

    #[test]
    fn a_live_refusal_removes_the_entry_before_a_later_outage() {
        let mut cache = OutageCache::default();
        let session = Uuid::now_v7();
        let owner = Uuid::now_v7();
        cache_put(
            &mut cache,
            session,
            owner,
            &response("t1", "full", 3000, 100),
        );
        cache.invalidate(session, owner);
        assert!(cache_get(&mut cache, session, owner).is_none());
    }

    #[tokio::test]
    async fn a_server_error_uses_an_eligible_cached_answer() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = Uuid::now_v7();
        let account = Uuid::now_v7();
        let mut cached = response("cached", "full", 3000, 100);
        cached["continuity"] = continuity();
        repo.daemon.outage_cache.lock().await.put(
            session,
            account,
            Trigger::Explicit,
            None,
            3000,
            &cached,
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 1024];
            let _ = socket.read(&mut request).await.unwrap();
            socket.write_all(b"HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n").await.unwrap();
        });
        *repo.daemon.server.write().await = ServerCredentials {
            url: Some(format!("http://{address}")),
            token: Some("token".into()),
            account_id: Some(account),
        };
        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session,
            Trigger::Explicit,
            None,
            3000,
            Duration::from_secs(1),
            None,
        )
        .await;
        assert_eq!(delivered.payload["served_from_cache"], true);
        assert_eq!(delivered.payload["cache_account_id"], account.to_string());
        assert!(delivered.payload["trace_id"].is_null());
        assert_eq!(
            delivered.payload["briefing"]["previous_handoff"]["next_step"],
            "publish release"
        );
        assert!(delivered.payload["briefing"]["constraints"].is_null());
        assert_eq!(delivered.payload["project_memory_withheld"], true);
        assert_eq!(
            delivered.payload["project_memory_withheld_reason"],
            "outage_cache_cannot_revalidate"
        );
    }

    #[test]
    fn cached_replay_keeps_other_domains_but_removes_project_claims() {
        let mut cached = response("cached", "full", 3000, 100);
        cached["continuity"] = continuity();
        cached["project_memory_available"] = json!(true);

        withhold_unconfirmed_project_memory(&mut cached, "outage_cache_cannot_revalidate");

        assert!(cached["sections"].get("session_memory").is_none());
        assert!(cached.get("project_memory_available").is_none());
        assert_eq!(cached["sections"]["personal_notes"][0]["content"], "p1");
        assert_eq!(cached["sections"]["team_guidance"][0]["content"], "g1");
        assert_eq!(cached["continuity"]["warnings"], json!([]));
        assert_eq!(cached["continuity"]["pins"], json!([]));
        assert_eq!(
            cached["continuity"]["previous_handoff"]["next_step"],
            "publish release"
        );
    }

    #[tokio::test]
    async fn a_live_refusal_never_replays_cached_continuity() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "refused-level0").await;
        let account = Uuid::now_v7();
        let mut cached = response("cached", "full", 3000, 100);
        cached["continuity"] = continuity();
        repo.daemon.outage_cache.lock().await.put(
            session.id,
            account,
            Trigger::Explicit,
            None,
            3000,
            &cached,
        );
        let url = serve_once("403 Forbidden", json!({ "error": "forbidden" })).await;
        *repo.daemon.server.write().await = ServerCredentials {
            url: Some(url),
            token: Some("token".into()),
            account_id: Some(account),
        };

        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session.id,
            Trigger::Explicit,
            None,
            3000,
            Duration::from_secs(1),
            None,
        )
        .await;

        assert_eq!(delivered.payload["served_from_cache"], false);
        assert_eq!(delivered.payload["fresh_knowledge_unavailable"], true);
        assert!(delivered.payload["briefing"]["previous_handoff"].is_null());
        assert!(delivered.payload["briefing"]["constraints"].is_null());
        assert!(repo
            .daemon
            .outage_cache
            .lock()
            .await
            .get(session.id, account, Trigger::Explicit, None, 3000,)
            .is_none());
    }

    /// Regression for the alpha.9 delivery-path rewrite: the server owns
    /// durable selection, but its response is not itself a complete briefing.
    /// The daemon must spend the budget the server left for Level 0 and then
    /// merge the selected durable sections into that locally assembled frame.
    #[tokio::test]
    async fn queried_context_never_replays_a_cache_or_accepts_an_unbound_answer() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "query-binding").await;
        let account = Uuid::now_v7();
        let mut cached = response("cached", "full", 3000, 100);
        cached["continuity"] = continuity();
        repo.daemon.outage_cache.lock().await.put(
            session.id,
            account,
            Trigger::Explicit,
            None,
            3000,
            &cached,
        );
        for (status, mut answer) in [
            ("500 Internal Server Error", json!({})),
            ("200 OK", response("legacy", "full", 3000, 100)),
            ("200 OK", response("other-query", "full", 3000, 100)),
        ] {
            if answer["trace_id"] == "other-query" {
                answer["project_recall_policy"] = json!(cairn_core::reuse::TASK_QUERY_POLICY);
                answer["project_query_sha256"] = json!(cairn_core::digest("different task"));
            }
            let url = serve_once(status, answer).await;
            *repo.daemon.server.write().await = ServerCredentials {
                url: Some(url),
                token: Some("token".into()),
                account_id: Some(account),
            };
            let delivered = deliver(
                &repo.daemon,
                &resolved,
                session.id,
                Trigger::Explicit,
                None,
                3000,
                Duration::from_secs(1),
                Some("bounded parser"),
            )
            .await;
            assert_ne!(delivered.payload["served_from_cache"], true);
            assert!(delivered.payload["trace_id"].is_null());
            assert!(delivered.payload["briefing"]["memory"]["session"]
                .as_array()
                .is_none_or(|items| items.is_empty()));
            assert!(delivered.payload["briefing"]["previous_handoff"].is_null());
        }
    }

    #[tokio::test]
    async fn a_fresh_server_answer_keeps_level0_and_merges_durable_sections() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "level0").await;
        let account = Uuid::now_v7();
        let mut answer = response("0199b6d0-d228-7b91-a420-2f935a95473b", "full", 3000, 100);
        answer["continuity"] = continuity();
        answer["project_recall_policy"] = json!(cairn_core::reuse::TASK_QUERY_POLICY);
        answer["project_query_sha256"] = json!(cairn_core::digest("session finding"));
        let url = serve_once("200 OK", answer).await;
        *repo.daemon.server.write().await = ServerCredentials {
            url: Some(url),
            token: Some("token".into()),
            account_id: Some(account),
        };

        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session.id,
            Trigger::Explicit,
            None,
            3000,
            Duration::from_secs(1),
            Some("session finding"),
        )
        .await;

        assert_eq!(
            delivered.payload["briefing"]["repository"]["branch"],
            "main"
        );
        assert_eq!(
            delivered.payload["briefing"]["memory"]["session"],
            json!(["s1"])
        );
        assert_eq!(
            delivered.payload["briefing"]["previous_handoff"]["next_step"],
            "publish release"
        );
        assert!(delivered.payload["briefing"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning["subject"] == "release channel"));
        assert_eq!(
            delivered.payload["briefing"]["constraints"][0]["text"],
            "never publish untested artifacts"
        );
        assert!(
            delivered.payload["estimated_tokens"].as_u64().unwrap() <= 3000,
            "local continuity plus durable selection must fit the whole budget"
        );
        assert_eq!(delivered.payload["budget"], 3000);
        assert_eq!(
            delivered.payload["trace_id"],
            "0199b6d0-d228-7b91-a420-2f935a95473b"
        );
    }

    #[tokio::test]
    async fn malformed_continuity_degrades_without_hiding_durable_sections() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "malformed-level0").await;
        let account = Uuid::now_v7();
        let mut answer = response("0199b6d0-d228-7b91-a420-2f935a95473c", "full", 3000, 100);
        answer["continuity"] = json!({ "pins": "not-an-array" });
        answer["project_recall_policy"] = json!(cairn_core::reuse::TASK_QUERY_POLICY);
        answer["project_query_sha256"] = json!(cairn_core::digest("session finding"));
        let url = serve_once("200 OK", answer).await;
        *repo.daemon.server.write().await = ServerCredentials {
            url: Some(url),
            token: Some("token".into()),
            account_id: Some(account),
        };

        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session.id,
            Trigger::Explicit,
            None,
            3000,
            Duration::from_secs(1),
            Some("session finding"),
        )
        .await;

        assert_eq!(delivered.payload["degraded"], true);
        assert_eq!(
            delivered.payload["briefing"]["memory"]["session"],
            json!(["s1"])
        );
        assert!(delivered.payload["briefing"]["constraints"].is_null());
    }

    #[tokio::test]
    async fn a_server_without_the_continuity_field_is_explicitly_degraded() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "missing-level0").await;
        let account = Uuid::now_v7();
        let mut answer = response("0199b6d0-d228-7b91-a420-2f935a95473d", "full", 3000, 100);
        answer["project_recall_policy"] = json!(cairn_core::reuse::TASK_QUERY_POLICY);
        answer["project_query_sha256"] = json!(cairn_core::digest("session finding"));
        let url = serve_once("200 OK", answer).await;
        *repo.daemon.server.write().await = ServerCredentials {
            url: Some(url),
            token: Some("token".into()),
            account_id: Some(account),
        };

        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session.id,
            Trigger::Explicit,
            None,
            3000,
            Duration::from_secs(1),
            Some("session finding"),
        )
        .await;

        assert_eq!(delivered.payload["degraded"], true);
        assert_eq!(delivered.payload["degradation_level"], "reduced");
        assert_eq!(
            delivered.payload["briefing"]["repository"]["branch"],
            "main"
        );
        assert_eq!(
            delivered.payload["briefing"]["memory"]["session"],
            json!(["s1"])
        );
    }

    #[tokio::test]
    async fn a_cached_answer_for_a_larger_budget_is_not_replayed() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "small-budget").await;
        let account = Uuid::now_v7();
        let mut cached = response("cached", "full", 3000, 100);
        cached["continuity"] = continuity();
        repo.daemon.outage_cache.lock().await.put(
            session.id,
            account,
            Trigger::Explicit,
            None,
            3000,
            &cached,
        );
        let url = serve_once("500 Internal Server Error", json!({})).await;
        *repo.daemon.server.write().await = ServerCredentials {
            url: Some(url),
            token: Some("token".into()),
            account_id: Some(account),
        };

        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session.id,
            Trigger::Explicit,
            None,
            100,
            Duration::from_secs(1),
            None,
        )
        .await;

        assert_eq!(delivered.payload["served_from_cache"], false);
        assert_eq!(delivered.payload["fresh_knowledge_unavailable"], true);
        assert!(delivered.payload["briefing"]["previous_handoff"].is_null());
        assert!(delivered.payload["estimated_tokens"].as_u64().unwrap() <= 100);
    }

    #[tokio::test]
    async fn delivery_does_not_run_past_an_exhausted_deadline() {
        let repo = fx::Repo::with(cairn_core::CairnConfig::default()).await;
        let resolved = repo.daemon.resolve(&repo.cwd).await.unwrap();
        let session = fx::session(&repo.daemon, &resolved.project, "deadline").await;
        let started = std::time::Instant::now();

        let delivered = deliver(
            &repo.daemon,
            &resolved,
            session.id,
            Trigger::Explicit,
            None,
            3000,
            Duration::ZERO,
            None,
        )
        .await;

        assert!(started.elapsed() < Duration::from_millis(100));
        assert_eq!(delivered.payload["fresh_knowledge_unavailable"], true);
        assert_eq!(delivered.payload["degradation_level"], "none");
        assert!(delivered.payload["trace_id"].is_null());
    }

    /// An over-budget entry is rejected outright, and whatever the session
    /// already had survives untouched — never silently truncated into a
    /// misrepresentation of what the server actually said.
    #[test]
    fn an_over_budget_entry_is_rejected_not_truncated() {
        let mut cache = OutageCache::default();
        let session = Uuid::now_v7();
        let owner = Uuid::now_v7();

        cache_put(
            &mut cache,
            session,
            owner,
            &response("t1", "full", 3000, 100),
        );

        let huge_note = "x".repeat(CACHE_MAX_BYTES + 1024);
        let oversized = json!({
            "trace_id": "t2",
            "degradation_level": "full",
            "budget": { "tokens": 3000, "spent": 100, "reserved_for_level0": 1200 },
            "sections": { "personal_notes": [{ "content": huge_note }] },
        });
        cache_put(&mut cache, session, owner, &oversized);

        let got = cache
            .get(session, owner, Trigger::Explicit, None, 3000)
            .expect("the original entry survives");
        assert_eq!(
            got["trace_id"], "t1",
            "the oversized put must not have landed"
        );
    }

    /// LRU eviction at the session cap: the least recently touched session is
    /// the one that goes.
    #[test]
    fn the_least_recently_used_session_is_evicted_at_the_cap() {
        let mut cache = OutageCache::default();
        let owner = Uuid::now_v7();
        let sessions: Vec<Uuid> = (0..CACHE_MAX_SESSIONS).map(|_| Uuid::now_v7()).collect();

        for s in &sessions {
            cache_put(&mut cache, *s, owner, &response("t", "full", 3000, 0));
        }
        assert_eq!(cache.len(), CACHE_MAX_SESSIONS);

        // Touch every session but the first, so it is unambiguously the
        // least recently used one when the cap is next exceeded.
        for s in &sessions[1..] {
            assert!(cache_get(&mut cache, *s, owner).is_some());
        }

        let newcomer = Uuid::now_v7();
        cache_put(&mut cache, newcomer, owner, &response("t", "full", 3000, 0));

        assert_eq!(cache.len(), CACHE_MAX_SESSIONS);
        assert!(
            cache_get(&mut cache, sessions[0], owner).is_none(),
            "the session nothing touched again must be the one evicted"
        );
        assert!(cache_get(&mut cache, newcomer, owner).is_some());
    }

    // -- ResponseMeta ------------------------------------------------------

    #[test]
    fn local_budget_is_tokens_minus_spent_when_the_server_answered() {
        let meta = ResponseMeta::extract(Some(&response("t1", "full", 3000, 700)), false);
        assert_eq!(meta.local_budget(3000), 2300);
    }

    /// Guaranteed never less than what the server withheld — this is the
    /// property the coordinator's fix (`reserved_for_level0`) exists for,
    /// checked from the daemon's side of the same arithmetic.
    #[test]
    fn local_budget_never_falls_below_the_servers_own_reserve() {
        let response = response("t1", "full", 3000, 1799); // spends right up to the edge
        let meta = ResponseMeta::extract(Some(&response), false);
        let reserved = response["budget"]["reserved_for_level0"].as_u64().unwrap() as usize;
        assert!(meta.local_budget(3000) >= reserved);
    }

    #[test]
    fn local_budget_falls_back_to_the_full_local_budget_when_nothing_was_retrieved() {
        let meta = ResponseMeta::unavailable();
        assert_eq!(meta.local_budget(3000), 3000);
    }

    #[test]
    fn a_real_zero_token_answer_does_not_become_the_full_local_budget() {
        let meta = ResponseMeta::extract(Some(&response("t1", "full", 0, 0)), false);
        assert_eq!(meta.local_budget(1), 0);
    }

    /// An empty durable selection is a complete delivery of nothing owed
    /// (§4.1's worked example), never treated as degraded here — this module
    /// only ever passes the server's own `degradation_level` through, never
    /// reinterprets it by how many items came back.
    #[test]
    fn an_empty_selection_still_reports_the_servers_own_level_untouched() {
        let empty = json!({
            "trace_id": "t1",
            "degradation_level": "full",
            "budget": { "tokens": 750, "spent": 0, "reserved_for_level0": 300 },
            "sections": {},
        });
        let meta = ResponseMeta::extract(Some(&empty), false);
        assert_eq!(meta.degradation_level, "full");
    }

    #[test]
    fn a_cached_answer_never_carries_a_reportable_trace_id() {
        let meta = ResponseMeta::extract(Some(&response("t1", "full", 3000, 0)), true);
        assert_eq!(meta.trace_id, None);
    }

    // -- merge_durable_sections --------------------------------------------

    fn bare_payload() -> Value {
        json!({
            "briefing": {
                "memory": { "session": [], "branch": [], "project": [] },
            },
            "estimated_tokens": 0,
        })
    }

    #[test]
    fn durable_sections_are_merged_into_the_matching_fields() {
        let mut payload = bare_payload();
        let sections = json!({
            "session_memory": [{ "content": "s1" }],
            "branch_memory": [{ "content": "b1" }],
            "project_memory": [{ "content": "p1" }],
            "personal_notes": [{ "content": "n1" }],
            "team_guidance": [{ "content": "g1" }],
        });
        merge_durable_sections(&mut payload, &sections);

        assert_eq!(payload["briefing"]["memory"]["session"], json!(["s1"]));
        assert_eq!(payload["briefing"]["memory"]["branch"], json!(["b1"]));
        assert_eq!(payload["briefing"]["memory"]["project"], json!(["p1"]));
        assert_eq!(payload["briefing"]["personal_notes"], json!(["n1"]));
        assert_eq!(payload["briefing"]["team_guidance"], json!(["g1"]));
    }

    /// FR-481: a caller with nothing in a global domain sees exactly what a
    /// caller who never touched that domain sees — the key absent, not
    /// present with an empty array.
    #[test]
    fn an_empty_global_section_is_dropped_not_inserted_empty() {
        let mut payload = bare_payload();
        merge_durable_sections(&mut payload, &json!({}));
        assert!(payload["briefing"].get("personal_notes").is_none());
        assert!(payload["briefing"].get("team_guidance").is_none());
    }

    #[test]
    fn embed_meta_nulls_the_trace_id_when_absent() {
        let mut payload = json!({});
        let meta = ResponseMeta::unavailable();
        embed_meta(&mut payload, &meta, false);
        assert!(payload["trace_id"].is_null());
        assert_eq!(payload["degradation_level"], "none");
        assert_eq!(payload["served_from_cache"], false);
    }
}
