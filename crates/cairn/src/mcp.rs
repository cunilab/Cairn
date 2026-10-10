//! The MCP server: exactly five tools, no more.
//!
//! Speaks JSON-RPC 2.0 over stdio — `initialize`, `tools/list`, `tools/call` —
//! and forwards every call to the local daemon. Each tool takes an `action`
//! discriminator rather than exploding into one tool per database operation,
//! which is what keeps the agent's tool list short enough to be useful.

use crate::client;
use crate::render;
use cairn_core::reuse::{CaptureAttestation, ReusePurpose};
use cairn_core::wire::{ContextDepth, MemoryQuery, Request, WireError};
use cairn_core::{KnowledgeDomain, MemoryScope, MemoryType};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "2025-06-18";

/// Versions this server understands, newest first.
///
/// A client asking for something else gets our newest — echoing an unknown
/// version back would claim support Cairn does not have.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// The five agent tools.
pub const TOOL_NAMES: &[&str] = &[
    "cairn_context",
    "cairn_search",
    "cairn_remember",
    "cairn_session",
    "cairn_handoff",
];

pub async fn serve() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();
    let mut codex_identity = false;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        // Malformed input gets the JSON-RPC parse error, not silence.
        let message = match serde_json::from_str::<Value>(&line) {
            Ok(m) => m,
            Err(e) => {
                let body = serde_json::to_string(&error_response(
                    Value::Null,
                    -32700,
                    &format!("parse error: {e}"),
                ))?;
                stdout.write_all(format!("{body}\n").as_bytes()).await?;
                stdout.flush().await?;
                continue;
            }
        };
        let Some(method) = message.get("method").and_then(|m| m.as_str()) else {
            continue;
        };
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);

        // Notifications carry no id and expect no reply.
        let Some(id) = id else { continue };

        let response = match method {
            "initialize" => {
                // ponytail: bind only the observed pinned CLI profile; extend after checking another version's per-call metadata.
                codex_identity = params["clientInfo"]["name"] == "codex-mcp-client"
                    && params["clientInfo"]["version"] == "0.160.0";
                success(id, initialize(&params))
            }
            "tools/list" => success(id, json!({ "tools": tool_definitions() })),
            "tools/call" => success(id, call(&params, codex_identity).await),
            "ping" => success(id, json!({})),
            other => error_response(id, -32601, &format!("unknown method: {other}")),
        };

        let mut body = serde_json::to_string(&response)?;
        body.push('\n');
        stdout.write_all(body.as_bytes()).await?;
        stdout.flush().await?;
    }
    Ok(())
}

fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn initialize(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(|v| v.as_str());
    let negotiated = match requested {
        Some(v) if SUPPORTED_PROTOCOL_VERSIONS.contains(&v) => v,
        _ => PROTOCOL_VERSION,
    };
    // The compact universal rendering of the usage contract, delivered
    // through the protocol's own server-instructions mechanism, so a client
    // with no native adapter still receives correct behavior (FR-129).
    //
    // Delivery is best-effort: the specification calls `instructions` a hint
    // clients *may* add to the system prompt, so Cairn never reports the
    // contract as *delivered* through this path.
    json!({
        "protocolVersion": negotiated,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "cairn", "version": env!("CARGO_PKG_VERSION") },
        "instructions": cairn_integrate::render::Contract::canonical().mcp_instructions(),
    })
}

fn cwd_property() -> Value {
    json!({ "type": "string", "description": "Directory to resolve the repository from" })
}

fn capture_attestation_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "description": "Accountable support for reusing a project-memory create or replacement. This is not objective verification. The authenticated actor is added by Cairn.",
        "properties": {
            "basis": { "type": "string", "enum": ["user_report", "inspected_source"], "description": "Use user_report for a user-supplied fact or choice; inspected_source only after reading the named revision." },
            "support_summary": { "type": "string", "description": "A concise authored reason this capture is supported. Never paste prompts, transcripts, credentials, or unbounded output." },
            "source_reference": { "type": "string", "description": "Bounded source name or repository-relative locator; required for inspected_source." },
            "source_revision": { "type": "string", "description": "Revision actually inspected; required for inspected_source." },
            "dependency_memory_id": { "type": "string", "description": "Optional eligible project memory with no dependency of its own (one hop maximum). Cairn records its current revision server-side." }
        },
        "required": ["basis", "support_summary"]
    })
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "cairn_context",
            "description": "Build the bounded briefing for the current repository: project, \
                            branch, commit, working tree, previous handoff, and \
                            memory availability — plus the minimum safe continuity, drift \
                            and conflict warnings, and whether your checkpoint diverged.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": cwd_property(),
                    "query": { "type": "string", "description": "Optional task keywords (at most 256 bytes) to retrieve project findings. Omit for continuity only; never send a raw prompt or transcript." },
                    // `post_compaction` is how an agent whose adapter has no
                    // post-compaction event restores continuity itself. An
                    // unknown value still falls back to `refresh` (D57, FR-426).
                    "reason": { "type": "string", "enum": ["session_start", "continuation", "refresh", "post_compaction"] },
                    "depth": { "type": "string", "enum": ["minimum", "standard"], "description": "`minimum` is Level 0 only. Level 2 is never automatic." },
                    "include_patterns": { "type": "boolean", "description": "Signal-matched patterns from other projects, always labelled unverified here" },
                    "explain": { "type": "boolean", "description": "Return the selection diagnostics. Costs no budget when false." },
                    "token_budget": { "type": "integer", "description": "Cairn-estimated tokens" },
                    "agent_session_key": { "type": "string", "description": "Native Codex framework identity overrides this field. Otherwise use your existing vendor session key, never an invented key or Cairn UUID. Omit for one active session; ambiguity requires this key or your own session_id." },
                    "session_id": { "type": "string", "description": "Cairn session UUID, including candidates labelled session_id in a recovery error; distinct from agent_session_key" }
                },
                "required": ["cwd"]
            }
        }),
        json!({
            "name": "cairn_search",
            "description": "Search durable memory. Results are ranked scope-first — current \
                            session, then branch, then project — with lexical relevance and \
                            recency breaking ties within a scope. Filter by verification state \
                            or subject; ask for reusable patterns explicitly.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": cwd_property(),
                    "action": { "type": "string", "enum": ["search", "graph"], "description": "`search` is lexical retrieval. `graph` is a typed, capped related-result request; unavailable servers refuse it explicitly." },
                    "query": { "type": "string", "description": "Task keywords required for working recall; at most 256 bytes. Context supplies continuity; this query selects relevant findings." },
                    "memory_id": { "type": "string", "description": "Required for `graph`; seed memory id." },
                    "hops": { "type": "integer", "description": "Graph depth, capped at two." },
                    "purpose": { "type": "string", "enum": ["reuse", "inspect"], "description": "`reuse` returns currently eligible working knowledge (default). `inspect` is only for deliberate memory auditing, verification or correction and reports why records are ineligible. Never use inspection as an empty-recall fallback; inspected records are not working knowledge." },
                    "scope": { "type": "string", "enum": ["project", "branch", "session"] },
                    "scope_key": { "type": "string" },
                    "type": { "type": "string", "enum": ["fact", "decision", "convention", "failure", "procedure"] },
                    "state": { "type": "string", "enum": ["active", "stale", "superseded"] },
                    "limit": { "type": "integer" },
                    // A `drifted` memory is lifecycle-`active` and is returned
                    // by default, with its verification state visible (FR-373).
                    "verification": { "type": "string", "enum": ["unverified", "verified", "needs_recheck", "drifted", "conflicted"] },
                    "authority": { "type": "string", "enum": ["cairn", "attested", "remote_cairn", "remote_attested"], "description": "What established the verification" },
                    "corroborated": { "type": "boolean" },
                    "conflicted": { "type": "boolean" },
                    "topic_key": { "type": "string", "description": "Exact, or a prefix when it ends in a dot" },
                    "as_of": { "type": "string", "description": "What was effective at this instant, RFC 3339" },
                    "pinned": { "type": "boolean" },
                    "include_patterns": { "type": "boolean", "description": "Patterns in a separate array, never mixed into results" },
                    // Absent means all three. Each domain is ranked within its
                    // own corpus and returned in its own array; there is no
                    // comparator across them (FR-471, FR-472).
                    "domains": { "type": "array", "items": { "type": "string", "enum": ["project", "personal", "team"] }, "description": "Which knowledge domains to search. Omit for all three; personal and team return sibling arrays, never merged into results" },
                    "agent_session_key": { "type": "string", "description": "Native Codex framework identity overrides this field. Otherwise use your existing vendor session key, never an invented key or Cairn UUID. Omit for one active session; use an explicit existing identity when ambiguous." },
                    "session_id": { "type": "string", "description": "Cairn session UUID, including candidates labelled session_id in a recovery error; distinct from agent_session_key" }
                },
                "required": ["cwd"]
            }
        }),
        json!({
            "name": "cairn_remember",
            "description": "Record durable knowledge, replace it, or forget it. Preserve user \
                            choices as decisions and trials as observations, not implemented \
                            or validated behavior. For durable project findings preserve relevant \
                            requirements, counts, qualifiers, identifiers, constraints, status and \
                            lasting policies, including read-only policies. Omit temporary task \
                            requests and completion reports; never generalize a limited observation. \
                            When applying recall, preserve task-relevant details and distinguish \
                            remembered intent from current implementation. Record independently inspected findings separately \
                            with their own source attestation. Keep those findings concise; cite bounded \
                            source locators without appending implementation summaries. Give durable \
                            project facts a `topic_key` and a `value_key` specific enough to \
                            state the whole claim. Local observation IDs cannot be attached \
                            to server-owned memory; source citations are agent attestations, \
                            not server verification. If Cairn reports a corroborating member and it is the \
                            same claim, reinforce it. Record a conflict rather than overwriting.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": cwd_property(),
                    // One discriminator, and no action takes a sub-operation (D70).
                    "action": { "type": "string", "enum": [
                        "create", "supersede", "capture", "forget",
                        "reinforce", "attach_evidence", "verify", "pin",
                        "reconcile", "governance"
                    ] },
                    "type": { "type": "string", "enum": ["fact", "decision", "convention", "failure", "procedure"] },
                    "scope": { "type": "string", "enum": ["project", "branch", "session"] },
                    "scope_key": { "type": "string" },
                    "content": { "type": "string" },
                    "evidence_observation_ids": { "type": "array", "items": { "type": "string" }, "description": "Legacy local observation IDs are unsupported for server-owned memory; omit or leave empty." },
                    "local_only": { "type": "boolean" },
                    "memory_id": { "type": "string" },
                    // create / forget. `team` is deliberately absent from this
                    // enum: no MCP action authors or mutates team knowledge
                    // directly, because ratification of proposed team
                    // guidance is an administrator action with no MCP surface
                    // at all (FR-455, FR-527). Sending "team" anyway is
                    // refused, not silently accepted or coerced.
                    "domain": { "type": "string", "enum": ["project", "personal"], "description": "Whose knowledge this is (create) or which domain's tombstone-only mutation to apply (forget). Defaults to `project`." },
                    // create / supersede
                    "topic_key": { "type": "string", "description": "The subject this states something about. A key that will not normalize is reported and the memory is stored free-form." },
                    "value_key": { "type": "string", "description": "The comparable value it asserts. Needs a topic_key." },
                    "importance": { "type": "string", "enum": ["low", "normal", "high"], "description": "Ranks within a bucket, and nothing more" },
                    "capture_attestation": capture_attestation_schema(),
                    "findings": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": CAPTURE_FINDINGS_MAX,
                        "description": "For action `capture`: one to eight complete, independently reusable project findings. Keep decisions, intent, and source observations separate. Every item needs its own provenance; admission is sequential and non-transactional.",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "type": { "type": "string", "enum": ["fact", "decision", "convention", "failure", "procedure"] },
                                "scope": { "type": "string", "enum": ["project", "branch", "session"] },
                                "scope_key": { "type": "string" },
                                "content": { "type": "string", "description": "One complete finding, including its qualifiers; at most 2048 UTF-8 bytes." },
                                "topic_key": { "type": "string" },
                                "value_key": { "type": "string" },
                                "capture_attestation": capture_attestation_schema()
                            },
                            "required": ["type", "scope", "content", "topic_key", "value_key", "capture_attestation"]
                        }
                    },
                    // attach_evidence
                    "kind": { "type": "string", "enum": ["observation", "file", "git_ref", "configuration", "test_outcome", "command_outcome", "runtime_state", "schema_version"] },
                    "subject": { "type": "string" },
                    "observed_value": { "type": "string" },
                    "source_locator": { "type": "string", "description": "Repository-relative or a Git ref. Never absolute." },
                    "observation_id": { "type": "string" },
                    "role": { "type": "string", "enum": ["supports", "contradicts"] },
                    "collector": { "type": "string", "enum": ["cairn", "agent"], "description": "`agent` when you are attesting rather than Cairn checking" },
                    // pin
                    "pinned": { "type": "boolean" },
                    "reason": { "type": "string" },
                    // reconcile
                    "from_memory_id": { "type": "string" },
                    "to_memory_id": { "type": "string" },
                    "relation": { "type": "string", "enum": ["narrows", "not_applicable_to", "supersedes"] },
                    "basis": { "type": "string", "enum": ["explicit_agent", "evidence"] },
                    "basis_evidence_id": { "type": "string" },
                    "rationale": { "type": "string" },
                    "agent_session_key": { "type": "string", "description": "Native Codex framework identity overrides this field. Otherwise use your existing vendor session key, never an invented key or Cairn UUID. Omit for automatic resolution of one active session; multiple sessions require this key or your own session_id." },
                    "session_id": { "type": "string", "description": "Your own Cairn session UUID, as an alternative to agent_session_key; pass it alone when recovering from an unknown vendor key." }
                },
                "required": ["cwd", "action"]
            }
        }),
        json!({
            "name": "cairn_session",
            "description": "Inspect an existing session or write a continuity checkpoint. \
                            Native integrations manage lifecycle automatically. Generic MCP cannot start sessions; do not invent session keys. Use your existing vendor key \
                            or omit it for one active session; ambiguity requires explicit selection.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": cwd_property(),
                    "action": { "type": "string", "enum": ["current", "start", "end", "checkpoint", "capture_disposition", "capture_review", "replay"] },
                    "agent": { "type": "string" },
                    "agent_session_key": { "type": "string" },
                    "session_id": { "type": "string" },
                    "status": { "type": "string", "enum": ["completed", "interrupted"] },
                    "disposition": { "type": "string", "enum": ["no_durable_finding"], "description": "Native Codex only: state that this verified turn has no durable finding. Turn identity comes from framework metadata." },
                    // checkpoint
                    "next_action": { "type": "string", "description": "What you were about to do" },
                    "relevant_paths": { "type": "array", "items": { "type": "string" }, "description": "Repository-relative paths this work depends on" }
                },
                "required": ["cwd", "action"]
            }
        }),
        json!({
            "name": "cairn_handoff",
            "description": "Read the latest handoff, generate one at a boundary, or attach a \
                            bounded note beside the derived record — optionally with its \
                            continuity checkpoint.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": cwd_property(),
                    "action": { "type": "string", "enum": ["latest", "generate", "annotate"] },
                    "session_id": { "type": "string" },
                    "agent_session_key": { "type": "string" },
                    // `stop` is deliberately absent: a turn checkpoint is not a
                    // handoff boundary (Feature 001 D16).
                    "trigger": { "type": "string", "enum": ["pre_compact", "session_end"] },
                    "note": { "type": "string" },
                    "include_checkpoint": { "type": "boolean", "description": "Add the anchored checkpoint and its staleness assessment" }
                },
                "required": ["cwd", "action"]
            }
        }),
    ]
}

fn call_arguments(params: &Value, codex_identity: bool) -> Result<Value, WireError> {
    let mut args = params.get("arguments").cloned().unwrap_or(json!({}));
    if args
        .as_object()
        .is_some_and(|fields| fields.contains_key("__cairn_native_turn_id"))
    {
        return Err(WireError::invalid(
            "native turn identity is supplied by Codex metadata, not tool arguments",
        ));
    }
    if codex_identity {
        let thread = params["_meta"]["threadId"]
            .as_str()
            .and_then(|value| uuid::Uuid::parse_str(value).ok())
            .filter(|id| !id.is_nil())
            .ok_or_else(|| WireError::invalid("Codex call requires valid framework threadId metadata; reconnect the MCP server"))?;
        let fields = args
            .as_object_mut()
            .ok_or_else(|| WireError::invalid("tool arguments must be an object"))?;
        if let Some(id) = fields.get("session_id") {
            id.as_str()
                .and_then(|value| uuid::Uuid::parse_str(value).ok())
                .filter(|id| !id.is_nil())
                .ok_or_else(|| {
                    WireError::invalid("session_id must be a valid Cairn session UUID")
                })?;
        }
        // Client provenance selects a session; account/project/worktree authorization remains in the daemon.
        fields.insert("agent_session_key".into(), json!(thread.to_string()));
        if let Some(turn) = native_turn_metadata(params, thread) {
            fields.insert("__cairn_native_turn_id".into(), json!(turn.to_string()));
        }
    }
    Ok(args)
}

#[derive(Deserialize)]
struct NativeTurnMetadata {
    turn_id: uuid::Uuid,
    session_id: uuid::Uuid,
    thread_id: uuid::Uuid,
}

/// Pinned Codex carries these three UUIDs in a metadata object. Any
/// malformed or conflicting value simply withholds disposition credit.
fn native_turn_metadata(params: &Value, thread: uuid::Uuid) -> Option<uuid::Uuid> {
    let header_session = params["_meta"]["sessionId"]
        .as_str()
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .filter(|id| !id.is_nil())?;
    let metadata: NativeTurnMetadata =
        serde_json::from_value(params["_meta"]["x-codex-turn-metadata"].clone()).ok()?;
    (metadata.turn_id != uuid::Uuid::nil()
        && metadata.session_id != uuid::Uuid::nil()
        && metadata.thread_id != uuid::Uuid::nil()
        && header_session == thread
        && metadata.session_id == thread
        && metadata.thread_id == thread)
        .then_some(metadata.turn_id)
}

fn native_turn_id(args: &Value) -> Option<uuid::Uuid> {
    args.get("__cairn_native_turn_id")
        .and_then(|value| value.as_str())
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .filter(|id| !id.is_nil())
}

async fn call(params: &Value, codex_identity: bool) -> Value {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or_default();
    let result = match call_arguments(params, codex_identity) {
        Ok(args) => dispatch(name, &args).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        Err(e) => json!({
            "content": [{ "type": "text", "text": format!("{}: {}", e.code, e.message) }],
            "isError": true
        }),
    }
}

async fn dispatch(name: &str, args: &Value) -> Result<String, WireError> {
    let cwd = args
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(crate::cwd);
    let key = str_arg(args, "agent_session_key");

    match name {
        "cairn_context" => {
            let query = str_arg(args, "query").filter(|q| !q.trim().is_empty());
            if query.is_some() {
                cairn_core::reuse::validate_recall_query(query.as_deref())
                    .map_err(WireError::invalid)?;
            }
            let value = client::send(
                &Request::Context {
                    cwd,
                    query: query.clone(),
                    agent_session_key: key,
                    session_id: uuid_arg(args, "session_id").ok(),
                    reason: args
                        .get("reason")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    token_budget: args
                        .get("token_budget")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as usize),
                    // The five tools are fixed; diagnostics stay a web affordance.
                    explain: false,

                    // `minimum` excludes personal_notes/team_guidance entirely;
                    // absent means `standard`, today's full assembly, so a
                    // caller that has never named this sees no change (FR-481).
                    // `ContextDepth` has no `FromStr` (it derives only `Serialize`
                    // / `Deserialize`), so this goes through serde rather than
                    // `enum_arg`.
                    depth: args
                        .get("depth")
                        .and_then(|v| serde_json::from_value::<ContextDepth>(v.clone()).ok()),
                    // Absent: this tool always retrieves as an explicit pull
                    // (`contracts/retrieval-delivery.md` §3) -- FR-831's manual
                    // override, never a push, and never reported `transmitted`.
                    trigger: None,
                    open_trigger: None,
                }
                .for_project_reuse(),
            )
            .await?;
            if let Some(query) = query {
                confirm_project_recall(&value, &cairn_core::digest(query.trim()))?;
            }
            // The agent gets the rendered briefing plus the raw envelope, so it
            // can read either.
            let context = render::context(&value).map_err(WireError::invalid)?;
            Ok(format!("{}\n\n```json\n{}\n```", context, pretty(&value)))
        }

        "cairn_search" => {
            let purpose = enum_arg(args, "purpose").unwrap_or(ReusePurpose::Reuse);
            if str_arg(args, "action").as_deref() == Some("graph") {
                let request = Request::Graph {
                    cwd,
                    memory_id: uuid_arg(args, "memory_id")?,
                    hops: args.get("hops").and_then(|v| v.as_i64()),
                    purpose,
                };
                let request = if purpose == ReusePurpose::Reuse {
                    request.for_project_reuse()
                } else {
                    request
                };
                let value = client::send(&request).await?;
                return Ok(pretty(&value));
            }
            let query = MemoryQuery {
                purpose,
                query: str_arg(args, "query"),
                scope: enum_arg(args, "scope"),
                scope_key: str_arg(args, "scope_key"),
                kind: enum_arg(args, "type"),
                state: enum_arg(args, "state"),
                limit: args.get("limit").and_then(|v| v.as_i64()),
                topic_key: str_arg(args, "topic_key"),
                as_of: str_arg(args, "as_of").and_then(|t| {
                    chrono::DateTime::parse_from_rfc3339(&t)
                        .ok()
                        .map(|d| d.with_timezone(&chrono::Utc))
                }),
                conflicted: args
                    .get("conflicted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                corroborated: args
                    .get("corroborated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                include_patterns: args
                    .get("include_patterns")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                verification: enum_arg(args, "verification"),
                authority: enum_arg(args, "authority"),

                // Absent stays absent rather than becoming an explicit list of
                // all three: the daemon's default is the thing the contract
                // documents, and duplicating it here would let the two drift
                // (FR-472).
                domains: enum_list(args, "domains"),
            };
            if purpose == ReusePurpose::Reuse {
                cairn_core::reuse::validate_recall_query(query.query.as_deref())
                    .map_err(WireError::invalid)?;
            }
            let project_recall = purpose == ReusePurpose::Reuse
                && query.domains.as_ref().is_none_or(|domains| {
                    domains.contains(&cairn_core::domain::KnowledgeDomain::Project)
                });
            let query_digest = query.query.as_deref().map(|q| cairn_core::digest(q.trim()));
            let request = Request::MemorySearch {
                cwd,
                agent_session_key: key,
                session_id: uuid_arg(args, "session_id").ok(),
                query,
            };
            let request = if purpose == ReusePurpose::Reuse {
                request.for_project_reuse()
            } else {
                request
            };
            let value = client::send(&request).await?;
            if project_recall {
                confirm_project_recall(&value, query_digest.as_deref().unwrap_or_default())?;
            }
            Ok(pretty(&value))
        }

        "cairn_remember" => {
            let action = required_action(args)?;
            let value = match action.as_str() {
                "capture" => return capture_findings(cwd, key, args).await,
                "create" | "supersede" => {
                    let kind = enum_arg(args, "type")
                        .ok_or_else(|| WireError::invalid("type is required"))?;
                    let content = str_arg(args, "content")
                        .ok_or_else(|| WireError::invalid("content is required"))?;
                    if args.get("evidence_observation_ids").is_some_and(|value| {
                        !value.is_null() && value.as_array().is_none_or(|ids| !ids.is_empty())
                    }) {
                        return Err(WireError::invalid(
                            "local observation IDs cannot be attached to server-owned memory; omit evidence_observation_ids",
                        ));
                    }
                    let evidence = Vec::new();
                    let local_only = args
                        .get("local_only")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if let Some(capture_attestation) = capture_attestation_arg(args)? {
                        if matches!(
                            knowledge_domain_arg(args)?,
                            Some(KnowledgeDomain::Personal | KnowledgeDomain::Team)
                        ) {
                            return Err(WireError::invalid(
                                "capture_attestation applies only to project memory",
                            ));
                        }
                        client::send(&Request::MemoryCapture {
                            cwd,
                            agent_session_key: key,
                            session_id: uuid_opt(args, "session_id"),
                            native_turn_id: native_turn_id(args),
                            kind,
                            scope: enum_arg(args, "scope"),
                            scope_key: str_arg(args, "scope_key"),
                            content,
                            evidence_observation_ids: evidence,
                            local_only,
                            topic_key: str_arg(args, "topic_key"),
                            value_key: str_arg(args, "value_key"),
                            supersedes: if action == "supersede" {
                                Some(uuid_arg(args, "memory_id")?)
                            } else {
                                None
                            },
                            capture_attestation,
                        })
                        .await?
                    } else if action == "create" {
                        client::send(&Request::MemoryCreate {
                            cwd,
                            agent_session_key: key,
                            session_id: uuid_opt(args, "session_id"),
                            kind,
                            scope: enum_arg(args, "scope"),
                            scope_key: str_arg(args, "scope_key"),
                            content,
                            evidence_observation_ids: evidence,
                            local_only,
                            topic_key: str_arg(args, "topic_key"),
                            value_key: str_arg(args, "value_key"),
                            importance: enum_arg(args, "importance"),

                            domain: knowledge_domain_arg(args)?,
                            capture_attestation: None,
                        })
                        .await?
                    } else {
                        client::send(&Request::MemorySupersede {
                            cwd,
                            agent_session_key: key,
                            session_id: uuid_opt(args, "session_id"),
                            memory_id: uuid_arg(args, "memory_id")?,
                            kind,
                            scope: enum_arg(args, "scope"),
                            scope_key: str_arg(args, "scope_key"),
                            content,
                            evidence_observation_ids: evidence,
                            local_only,
                            topic_key: str_arg(args, "topic_key"),
                            value_key: str_arg(args, "value_key"),
                            importance: enum_arg(args, "importance"),
                            capture_attestation: None,
                        })
                        .await?
                    }
                }
                "forget" => {
                    client::send(&Request::MemoryForget {
                        cwd,
                        memory_id: uuid_arg(args, "memory_id")?,

                        domain: knowledge_domain_arg(args)?,
                    })
                    .await?
                }
                "reinforce" => {
                    client::send(&Request::MemoryReinforce {
                        cwd,
                        agent_session_key: key,
                        session_id: uuid_opt(args, "session_id"),
                        memory_id: uuid_arg(args, "memory_id")?,
                        from_memory_id: uuid_opt(args, "from_memory_id"),
                    })
                    .await?
                }
                "attach_evidence" => {
                    client::send(&Request::EvidenceAdd {
                        cwd,
                        agent_session_key: key,
                        session_id: uuid_opt(args, "session_id"),
                        kind: enum_arg(args, "kind")
                            .ok_or_else(|| WireError::invalid("kind is required"))?,
                        collector: enum_arg(args, "collector"),
                        subject: str_arg(args, "subject")
                            .ok_or_else(|| WireError::invalid("subject is required"))?,
                        observed_value: str_arg(args, "observed_value")
                            .ok_or_else(|| WireError::invalid("observed_value is required"))?,
                        source_locator: str_arg(args, "source_locator")
                            .ok_or_else(|| WireError::invalid("source_locator is required"))?,
                        observation_id: uuid_opt(args, "observation_id"),
                        memory_id: uuid_opt(args, "memory_id"),
                        role: enum_arg(args, "role"),
                    })
                    .await?
                }
                "verify" => {
                    client::send(&Request::Verify {
                        cwd,
                        memory_id: uuid_opt(args, "memory_id"),
                        all: bool_arg(args, "all"),
                        explain: bool_arg(args, "explain"),
                    })
                    .await?
                }
                "pin" => {
                    client::send(&Request::MemoryPin {
                        cwd,
                        agent_session_key: key,
                        session_id: uuid_opt(args, "session_id"),
                        memory_id: uuid_arg(args, "memory_id")?,
                        // Pinning is the default; `pinned: false` unpins.
                        pinned: args.get("pinned").and_then(|v| v.as_bool()).unwrap_or(true),
                        reason: str_arg(args, "reason"),
                    })
                    .await?
                }
                "reconcile" => {
                    client::send(&Request::MemoryReconcile {
                        cwd,
                        agent_session_key: key,
                        session_id: uuid_opt(args, "session_id"),
                        from_memory_id: uuid_arg(args, "from_memory_id")?,
                        to_memory_id: uuid_arg(args, "to_memory_id")?,
                        relation: enum_arg(args, "relation")
                            .ok_or_else(|| WireError::invalid("relation is required"))?,
                        basis: enum_arg(args, "basis")
                            .unwrap_or(cairn_core::RelationBasis::ExplicitAgent),
                        basis_evidence_id: uuid_opt(args, "basis_evidence_id"),
                        rationale: str_arg(args, "rationale"),
                    })
                    .await?
                }
                "governance" => client::send(&Request::Governance { cwd }).await?,
                other => return Err(WireError::invalid(format!("unknown action: {other}"))),
            };
            Ok(pretty(&value))
        }

        "cairn_session" => {
            let action = required_action(args)?;
            let value = match action.as_str() {
                "current" => {
                    client::send(&Request::SessionShow {
                        cwd,
                        session_id: uuid_opt(args, "session_id"),
                        agent_session_key: key,
                    })
                    .await?
                }
                "start" => {
                    return Err(WireError::new(
                        cairn_core::wire::codes::AGENT_UNSUPPORTED,
                        "MCP cannot start native lifecycle sessions; use the session opened by your native integration",
                    ));
                }
                "end" => {
                    client::send(&Request::SessionEnd {
                        cwd,
                        session_id: uuid_opt(args, "session_id"),
                        agent_session_key: key,
                        status: enum_arg(args, "status")
                            .unwrap_or(cairn_core::domain::SessionStatus::Completed),
                        reason: str_arg(args, "reason"),
                        // An agent tool call has no vendor handler deadline
                        // over it, so it keeps Feature 001's behavior and
                        // waits for the durable handoff (D22).
                        wait_for_handoff: true,
                    })
                    .await?
                }
                // The path FR-425 gives an agent whose integration cannot be
                // called back after a compaction: write the checkpoint yourself,
                // before you compact.
                "checkpoint" => {
                    client::send(&Request::SessionCheckpoint {
                        cwd,
                        agent_session_key: key,
                        session_id: uuid_opt(args, "session_id"),
                    })
                    .await?
                }
                "capture_review" => {
                    let turn = native_turn_id(args).ok_or_else(|| {
                        WireError::invalid(
                            "capture_review requires verified native Codex turn metadata",
                        )
                    })?;
                    client::send_once(&Request::CaptureReview {
                        cwd,
                        agent_session_key: key.ok_or_else(|| {
                            WireError::invalid(
                                "capture_review requires verified native Codex identity",
                            )
                        })?,
                        native_turn_id: turn,
                    })
                    .await?
                }
                "capture_disposition" => {
                    if str_arg(args, "disposition").as_deref() != Some("no_durable_finding") {
                        return Err(WireError::invalid(
                            "capture_disposition requires disposition no_durable_finding",
                        ));
                    }
                    let native_turn_id = native_turn_id(args).ok_or_else(|| {
                        WireError::invalid(
                            "capture_disposition requires verified native Codex turn metadata",
                        )
                    })?;
                    client::send_once(&Request::CaptureDisposition {
                        cwd,
                        agent_session_key: key.ok_or_else(|| {
                            WireError::invalid(
                                "capture_disposition requires verified native Codex identity",
                            )
                        })?,
                        native_turn_id,
                        no_durable_finding: true,
                    })
                    .await?
                }
                "replay" => client::send(&Request::Replay { cwd }).await?,
                other => return Err(WireError::invalid(format!("unknown action: {other}"))),
            };
            Ok(pretty(&value))
        }

        "cairn_handoff" => {
            let action = required_action(args)?;
            let value = match action.as_str() {
                "latest" => {
                    client::send(&Request::HandoffLatest {
                        cwd,
                        session_id: uuid_arg(args, "session_id").ok(),
                        agent_session_key: key,
                    })
                    .await?
                }
                "generate" => {
                    // `stop` is deliberately absent: a turn checkpoint is not a
                    // handoff boundary (FR-032).
                    client::send(&Request::HandoffGenerate {
                        cwd,
                        session_id: uuid_arg(args, "session_id").ok(),
                        agent_session_key: key,
                        trigger: enum_arg(args, "trigger")
                            .unwrap_or(cairn_core::domain::HandoffTrigger::SessionEnd),
                    })
                    .await?
                }
                "annotate" => {
                    client::send(&Request::HandoffAnnotate {
                        cwd,
                        session_id: uuid_arg(args, "session_id").ok(),
                        agent_session_key: key,
                        note: str_arg(args, "note")
                            .ok_or_else(|| WireError::invalid("note is required"))?,
                    })
                    .await?
                }
                other => return Err(WireError::invalid(format!("unknown action: {other}"))),
            };
            Ok(pretty(&value))
        }

        other => Err(WireError::invalid(format!(
            "unknown tool `{other}`; Cairn exposes {}",
            TOOL_NAMES.join(", ")
        ))),
    }
}

fn required_action(args: &Value) -> Result<String, WireError> {
    args.get("action")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| WireError::invalid("action is required"))
}

fn str_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn enum_arg<T: std::str::FromStr>(args: &Value, key: &str) -> Option<T> {
    args.get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
}

fn uuid_arg(args: &Value, key: &str) -> Result<uuid::Uuid, WireError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| WireError::invalid(format!("{key} must be a uuid")))
}

/// A uuid argument that is allowed to be absent.
fn uuid_opt(args: &Value, key: &str) -> Option<uuid::Uuid> {
    args.get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
}

fn capture_attestation_arg(args: &Value) -> Result<Option<CaptureAttestation>, WireError> {
    let Some(value) = args.get("capture_attestation") else {
        return Ok(None);
    };
    let attestation: CaptureAttestation = serde_json::from_value(value.clone())
        .map_err(|e| WireError::invalid(format!("invalid capture_attestation: {e}")))?;
    attestation.validate().map_err(WireError::invalid)?;
    Ok(Some(attestation))
}

const CAPTURE_FINDINGS_MAX: usize = 8;
const CAPTURE_FINDING_CONTENT_MAX_BYTES: usize = 2048;

/// A batch item deliberately has no defaults: a complete finding needs its own
/// scope, subject, and accountable support before any daemon request is sent.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureFinding {
    #[serde(rename = "type")]
    kind: MemoryType,
    scope: MemoryScope,
    #[serde(default)]
    scope_key: Option<String>,
    content: String,
    topic_key: String,
    value_key: String,
    capture_attestation: CaptureAttestation,
}

fn prepare_capture_findings(args: &Value) -> Result<Vec<CaptureFinding>, WireError> {
    let fields = args
        .as_object()
        .ok_or_else(|| WireError::invalid("tool arguments must be an object"))?;
    for key in fields.keys() {
        if !matches!(
            key.as_str(),
            "cwd"
                | "action"
                | "findings"
                | "agent_session_key"
                | "session_id"
                | "__cairn_native_turn_id"
        ) {
            return Err(WireError::invalid(
                "action capture accepts only its batch envelope and findings",
            ));
        }
    }
    if let Some(key) = fields.get("agent_session_key") {
        if key.as_str().is_none_or(|value| value.trim().is_empty()) {
            return Err(WireError::invalid(
                "agent_session_key must be a nonempty string",
            ));
        }
    }
    if let Some(session_id) = fields.get("session_id") {
        if session_id
            .as_str()
            .and_then(|value| uuid::Uuid::parse_str(value).ok())
            .is_none_or(|id| id.is_nil())
        {
            return Err(WireError::invalid(
                "session_id must be a valid non-nil Cairn session UUID",
            ));
        }
    }
    let findings_value = fields
        .get("findings")
        .ok_or_else(|| WireError::invalid("findings is required for action capture"))?;
    let array = findings_value
        .as_array()
        .ok_or_else(|| WireError::invalid("findings must be an array"))?;
    if array.is_empty() || array.len() > CAPTURE_FINDINGS_MAX {
        return Err(WireError::invalid(format!(
            "capture requires between 1 and {CAPTURE_FINDINGS_MAX} findings"
        )));
    }
    let mut findings: Vec<CaptureFinding> = serde_json::from_value(findings_value.clone())
        .map_err(|_| WireError::invalid("capture findings must use the documented item fields"))?;
    for (index, finding) in findings.iter_mut().enumerate() {
        if finding.content.trim().is_empty() {
            return Err(WireError::invalid(format!(
                "finding {index} content must not be blank"
            )));
        }
        finding.topic_key =
            cairn_core::knowledge::normalize_topic_key_strict(&finding.topic_key)
                .map_err(|_| WireError::invalid(format!("finding {index} topic_key is invalid")))?;
        finding.value_key =
            cairn_core::knowledge::normalize_value_key_strict(&finding.value_key)
                .map_err(|_| WireError::invalid(format!("finding {index} value_key is invalid")))?;
        if finding.content.len() > CAPTURE_FINDING_CONTENT_MAX_BYTES {
            return Err(WireError::invalid(format!(
                "finding {index} content exceeds {CAPTURE_FINDING_CONTENT_MAX_BYTES} UTF-8 bytes"
            )));
        }
        finding.capture_attestation.validate().map_err(|error| {
            WireError::invalid(format!(
                "finding {index} invalid capture_attestation: {error}"
            ))
        })?;
        // This is the same privacy/content gate used for server candidates.
        // Repository identities are daemon-owned, so that contextual part is
        // checked again by the canonical write path.
        cairn_core::validate::validate_candidate_content(
            &finding.content,
            Some(&finding.topic_key),
            Some(&finding.value_key),
            &[],
        )
        .map_err(|_| WireError::invalid(format!("finding {index} is not admissible")))?;
    }
    Ok(findings)
}

fn bounded_error(error: &WireError) -> Value {
    const ERROR_MAX_BYTES: usize = 512;
    let redacted = cairn_core::redact::redact(&error.message);
    let message = if redacted.len() <= ERROR_MAX_BYTES {
        redacted
    } else {
        let mut end = ERROR_MAX_BYTES;
        while !redacted.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &redacted[..end])
    };
    json!({ "code": error.code, "message": message })
}

async fn capture_findings(
    cwd: String,
    agent_session_key: Option<String>,
    args: &Value,
) -> Result<String, WireError> {
    // Parse and validate every item before the first request. The writes below
    // remain deliberately non-transactional because the existing wire protocol
    // has one capture per request.
    let findings = prepare_capture_findings(args)?;
    let session_id = uuid_opt(args, "session_id");
    let native_turn_id = native_turn_id(args);
    let mut receipts = Vec::with_capacity(findings.len());
    let mut unconfirmed = false;
    let mut rejected = false;
    for (index, finding) in findings.into_iter().enumerate() {
        match client::send_once(&Request::MemoryCapture {
            cwd: cwd.clone(),
            agent_session_key: agent_session_key.clone(),
            session_id,
            native_turn_id,
            kind: finding.kind,
            scope: Some(finding.scope),
            scope_key: finding.scope_key,
            content: finding.content,
            evidence_observation_ids: Vec::new(),
            local_only: false,
            topic_key: Some(finding.topic_key),
            value_key: Some(finding.value_key),
            supersedes: None,
            capture_attestation: finding.capture_attestation,
        })
        .await
        {
            Ok(receipt) => receipts
                .push(json!({ "index": index, "status": "acknowledged", "receipt": receipt })),
            Err(error) => {
                let status = if matches!(
                    error.code.as_str(),
                    cairn_core::wire::codes::DAEMON_UNAVAILABLE
                        | cairn_core::wire::codes::STORAGE_UNAVAILABLE
                ) {
                    unconfirmed = true;
                    "unconfirmed"
                } else {
                    rejected = true;
                    "rejected"
                };
                receipts.push(
                    json!({ "index": index, "status": status, "error": bounded_error(&error) }),
                );
            }
        }
    }
    Ok(pretty(&json!({
        "admission": if unconfirmed { "unconfirmed" } else if rejected { "partial" } else { "complete" },
        "retry": "none_automatic",
        "transaction": "none",
        "receipts": receipts,
    })))
}

fn bool_arg(args: &Value, key: &str) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// A list of enum-valued strings, or `None` when the caller omitted the key.
///
/// Absent and present-but-empty are different requests and stay different: an
/// omitted `domains` searches every domain, an explicitly empty one searches
/// none. An unparseable member is dropped rather than failing the call, the
/// same way [`enum_arg`] treats a scalar.
fn enum_list<T: std::str::FromStr>(args: &Value, key: &str) -> Option<Vec<T>> {
    args.get(key).and_then(|v| v.as_array()).map(|a| {
        a.iter()
            .filter_map(|v| v.as_str())
            .filter_map(|s| s.parse().ok())
            .collect()
    })
}

/// `domain` for `cairn_remember`'s `create`/`forget`: `project` (default) or
/// `personal` only. `KnowledgeDomain` still has a `Team` variant — this is
/// where it gets refused rather than forwarded. No MCP action authors or
/// mutates team knowledge directly: team guidance is proposed and then
/// ratified by a server administrator, and ratification has no MCP surface
/// at all (FR-455, FR-527). `handlers.rs` refuses `domain: "team"` again
/// daemon-side; this is the surface-level half of that same refusal, not a
/// substitute for it.
fn knowledge_domain_arg(args: &Value) -> Result<Option<cairn_core::KnowledgeDomain>, WireError> {
    let domain = enum_arg::<cairn_core::KnowledgeDomain>(args, "domain");
    if domain == Some(cairn_core::KnowledgeDomain::Team) {
        return Err(WireError::invalid(
            "domain \"team\" is refused here: team knowledge is proposed and ratified by an \
             administrator, and no MCP action authors it directly (FR-455, FR-527)",
        ));
    }
    Ok(domain)
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn confirm_project_recall(value: &Value, query_digest: &str) -> Result<(), WireError> {
    if value["project_recall_policy"] != cairn_core::reuse::TASK_QUERY_POLICY
        || value["project_query_sha256"].as_str() != Some(query_digest)
    {
        return Err(WireError::new(cairn_core::wire::codes::SERVER_UNAVAILABLE,
            "task-scoped project excerpts were not confirmed; configure server inference or upgrade the server and daemon"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn working_recall_refuses_legacy_or_different_task_responses() {
        let digest = cairn_core::digest("bounded parser");
        for response in [
            json!({}),
            json!({"project_recall_policy":"task_keywords_v1", "project_query_sha256":digest}),
            json!({"project_recall_policy":cairn_core::reuse::TASK_QUERY_POLICY}),
            json!({"project_recall_policy":cairn_core::reuse::TASK_QUERY_POLICY,
                   "project_query_sha256":cairn_core::digest("another task")}),
        ] {
            let error = confirm_project_recall(&response, &digest).unwrap_err();
            assert_eq!(error.code, cairn_core::wire::codes::SERVER_UNAVAILABLE);
        }
        assert!(confirm_project_recall(
            &json!({
                "project_recall_policy":cairn_core::reuse::TASK_QUERY_POLICY,
                "project_query_sha256":digest
            }),
            &digest
        )
        .is_ok());
    }

    #[tokio::test]
    async fn working_search_without_keywords_refuses_before_ipc() {
        for query in [Value::Null, json!(""), json!("   ")] {
            let error = dispatch("cairn_search", &json!({"cwd": "/unused", "query": query}))
                .await
                .unwrap_err();
            assert!(
                error.message.contains("requires task keywords"),
                "{error:?}"
            );
        }
    }

    #[test]
    fn exposes_exactly_five_tools() {
        let tools = tool_definitions();
        assert_eq!(tools.len(), 5, "MCP exposes exactly five tools");
        let names: Vec<&str> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap_or_default())
            .collect();
        assert_eq!(names, TOOL_NAMES);
        for name in ["cairn_search", "cairn_remember"] {
            let tool = tools
                .iter()
                .find(|tool| tool["name"] == name)
                .expect("surviving tool");
            let scopes = tool["inputSchema"]["properties"]["scope"]["enum"]
                .as_array()
                .expect("scope enum");
            assert_eq!(scopes.len(), 3);
        }
    }

    #[test]
    fn advanced_actions_stay_typed_inside_the_five_tool_surface() {
        let tools = tool_definitions();
        let action_values = |name: &str| {
            tools
                .iter()
                .find(|tool| tool["name"] == name)
                .and_then(|tool| tool["inputSchema"]["properties"]["action"]["enum"].as_array())
                .expect("typed action enum")
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
        };
        assert!(action_values("cairn_search").contains(&"graph"));
        assert!(action_values("cairn_session").contains(&"replay"));
        assert!(action_values("cairn_remember").contains(&"governance"));
        assert!(action_values("cairn_remember").contains(&"capture"));
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn every_tool_declares_an_object_schema_requiring_cwd() {
        for tool in tool_definitions() {
            let schema = &tool["inputSchema"];
            assert_eq!(schema["type"], "object", "{}", tool["name"]);
            let required = schema["required"].as_array().unwrap();
            assert!(
                required.iter().any(|r| r == "cwd"),
                "{} does not require cwd",
                tool["name"]
            );
        }
    }

    #[test]
    fn handoff_generate_cannot_be_triggered_by_a_turn_checkpoint() {
        let handoff = tool_definitions()
            .into_iter()
            .find(|t| t["name"] == "cairn_handoff")
            .expect("cairn_handoff");
        let triggers = handoff["inputSchema"]["properties"]["trigger"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap_or_default())
            .collect::<Vec<_>>();
        assert!(
            !triggers.contains(&"stop"),
            "stop is a turn boundary, not a handoff trigger"
        );
        assert!(triggers.contains(&"pre_compact"));
        assert!(triggers.contains(&"session_end"));
    }

    #[tokio::test]
    async fn mcp_cannot_start_a_session_by_claiming_a_native_agent() {
        for agent in ["generic-mcp", "codex", "claude-code", "opencode"] {
            let error = dispatch("cairn_session", &json!({
                "action":"start", "agent":agent, "agent_session_key":"invented", "cwd":"/not-a-repo"
            })).await.unwrap_err();
            assert_eq!(error.code, cairn_core::wire::codes::AGENT_UNSUPPORTED);
        }
    }

    #[test]
    fn native_call_identity_comes_from_each_framework_request() {
        for thread in [
            "00000000-0000-4000-8000-000000000001",
            "00000000-0000-4000-8000-000000000002",
        ] {
            let params = json!({"_meta":{"threadId":thread,"sessionId":"different-runtime-id"},
                "arguments":{"cwd":"/repo","agent_session_key":"invented", "session_id":"00000000-0000-4000-8000-000000000003"}});
            let args = call_arguments(&params, true).unwrap();
            assert_eq!(args["agent_session_key"], thread);
            assert_eq!(args["session_id"], "00000000-0000-4000-8000-000000000003");
            assert_eq!(args["cwd"], "/repo");
            assert_eq!(call_arguments(&params, false).unwrap(), params["arguments"]);
        }
        for params in [
            json!({"arguments":{}}),
            json!({"_meta":{"threadId":"invented"},"arguments":{}}),
            json!({"_meta":{"threadId":"00000000-0000-4000-8000-000000000001"},"arguments":[]}),
            json!({"_meta":{"threadId":"00000000-0000-4000-8000-000000000001"},"arguments":{"session_id":"invalid"}}),
        ] {
            assert!(call_arguments(&params, true).is_err());
        }
    }

    #[test]
    fn pinned_codex_turn_metadata_is_cross_checked_before_credit() {
        let thread = uuid::Uuid::now_v7();
        let session = thread;
        let turn = uuid::Uuid::now_v7();
        let metadata = json!({
            "turn_id": turn,
            "session_id": session,
            "thread_id": thread,
        });
        let params = json!({
            "_meta": { "threadId": thread, "sessionId": session, "x-codex-turn-metadata": metadata },
            "arguments": { "cwd": "/repo" },
        });
        let args = call_arguments(&params, true).unwrap();
        assert_eq!(native_turn_id(&args), Some(turn));
        assert_eq!(args["agent_session_key"], thread.to_string());

        let other_session = uuid::Uuid::now_v7();
        let distinct_pairs =
            json!({"turn_id": turn, "session_id": other_session, "thread_id": thread});
        for bad in [
            json!({ "_meta": { "threadId": thread, "sessionId": other_session, "x-codex-turn-metadata": distinct_pairs }, "arguments": { "cwd": "/repo" } }),
            json!({ "_meta": { "threadId": thread, "sessionId": uuid::Uuid::now_v7(), "x-codex-turn-metadata": params["_meta"]["x-codex-turn-metadata"] }, "arguments": { "cwd": "/repo" } }),
            json!({ "_meta": { "threadId": thread, "sessionId": session, "x-codex-turn-metadata": "not json" }, "arguments": { "cwd": "/repo" } }),
        ] {
            let args = call_arguments(&bad, true).unwrap();
            assert!(native_turn_id(&args).is_none());
        }

        let metadata_with_extras = json!({
            "turn_id": turn,
            "session_id": session,
            "thread_id": thread,
            "model": "gpt-6-luna",
            "reasoning_effort": "low",
        });
        let args = call_arguments(
            &json!({
                "_meta": { "threadId": thread, "sessionId": session, "x-codex-turn-metadata": metadata_with_extras },
                "arguments": { "cwd": "/repo" },
            }),
            true,
        )
        .unwrap();
        assert_eq!(native_turn_id(&args), Some(turn));
    }

    #[test]
    fn native_turn_identity_cannot_be_authored_in_tool_arguments() {
        let thread = uuid::Uuid::now_v7();
        let params = json!({
            "_meta": { "threadId": thread },
            "arguments": { "cwd": "/repo", "__cairn_native_turn_id": uuid::Uuid::now_v7() },
        });
        assert!(call_arguments(&params, true).is_err());
        assert!(call_arguments(&params, false).is_err());
    }

    #[test]
    fn generic_capture_remains_compatible_without_native_turn_metadata() {
        let params = json!({ "arguments": { "cwd": "/repo", "action": "capture" } });
        let args = call_arguments(&params, false).unwrap();
        assert!(native_turn_id(&args).is_none());
        let session = tool_definitions()
            .into_iter()
            .find(|tool| tool["name"] == "cairn_session")
            .unwrap();
        assert!(session["inputSchema"]["properties"]["action"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action == "capture_disposition"));
    }

    #[test]
    fn initialize_carries_the_usage_contract() {
        // FR-129, SC-107: the same rules the managed block states, in the
        // tool-facing form.
        let out = initialize(&json!({}));
        let instructions = out["instructions"].as_str().expect("instructions");
        assert!(instructions.contains("Cairn"));
        assert!(instructions.contains("1. "));
        assert_eq!(
            instructions,
            cairn_integrate::render::Contract::canonical().mcp_instructions()
        );
        // A generic client has neither hooks nor Skills, so neither is
        // mentioned.
        let lower = instructions.to_lowercase();
        assert!(!lower.contains("hook"));
        assert!(!lower.contains("skill"));
    }

    #[test]
    fn generic_mcp_instructions_do_not_offer_unsupported_lifecycle() {
        let out = initialize(&json!({}));
        let instructions = out["instructions"].as_str().unwrap();
        assert!(instructions.contains("generic MCP cannot start sessions"));
        assert!(instructions.contains("existing sessions"));
        let session = tool_definitions()
            .into_iter()
            .find(|tool| tool["name"] == "cairn_session")
            .unwrap();
        assert!(session["description"]
            .as_str()
            .unwrap()
            .contains("Generic MCP cannot start sessions"));
    }

    #[test]
    fn the_protocol_revision_does_not_change() {
        // D34, FR-130: `instructions` is a field 2025-06-18 already defines,
        // so adding it is not a protocol bump.
        assert_eq!(PROTOCOL_VERSION, "2025-06-18");
        assert_eq!(initialize(&json!({}))["protocolVersion"], PROTOCOL_VERSION);
    }

    #[test]
    fn the_surface_is_exactly_five_tools() {
        assert_eq!(TOOL_NAMES.len(), 5, "the MCP surface grew a sixth tool");
        assert_eq!(tool_definitions().len(), 5);
        for forbidden in [
            "cairn_doctor",
            "cairn_repair",
            "cairn_connect",
            "cairn_agents",
            "cairn_integration",
        ] {
            assert!(
                !TOOL_NAMES.contains(&forbidden),
                "{forbidden} is a developer operation, not an agent tool"
            );
        }
    }

    #[test]
    fn initialize_negotiates_a_supported_protocol_version() {
        // A version we support is honoured.
        let out = initialize(&json!({ "protocolVersion": "2025-03-26" }));
        assert_eq!(out["protocolVersion"], "2025-03-26");
        assert!(out["capabilities"]["tools"].is_object());
        assert_eq!(out["serverInfo"]["name"], "cairn");

        // One we do not is answered with ours, not echoed back.
        let future = initialize(&json!({ "protocolVersion": "2999-01-01" }));
        assert_eq!(future["protocolVersion"], PROTOCOL_VERSION);
        let missing = initialize(&json!({}));
        assert_eq!(missing["protocolVersion"], PROTOCOL_VERSION);
    }

    #[test]
    fn read_tools_advertise_session_identity() {
        // Without this an agent cannot scope a briefing to its own session (M1).
        for name in ["cairn_context", "cairn_search"] {
            let tool = tool_definitions()
                .into_iter()
                .find(|t| t["name"] == name)
                .expect(name);
            let props = &tool["inputSchema"]["properties"];
            assert!(
                props["agent_session_key"].is_object(),
                "{name} hides agent_session_key"
            );
            assert!(props["session_id"].is_object(), "{name} hides session_id");
        }
    }

    #[test]
    fn search_defaults_working_recall_to_reuse_and_exposes_deliberate_inspection() {
        let search = tool_definitions()
            .into_iter()
            .find(|tool| tool["name"] == "cairn_search")
            .expect("cairn_search");
        assert_eq!(
            search["inputSchema"]["properties"]["purpose"]["enum"],
            json!(["reuse", "inspect"])
        );
        assert_eq!(
            enum_arg::<ReusePurpose>(&json!({}), "purpose").unwrap_or(ReusePurpose::Reuse),
            ReusePurpose::Reuse
        );
    }

    #[test]
    fn remember_schema_exposes_bounded_accountable_capture() {
        let remember = tool_definitions()
            .into_iter()
            .find(|tool| tool["name"] == "cairn_remember")
            .expect("cairn_remember");
        let attestation = &remember["inputSchema"]["properties"]["capture_attestation"];
        assert_eq!(attestation["type"], "object");
        assert_eq!(
            attestation["properties"]["basis"]["enum"],
            json!(["user_report", "inspected_source"])
        );
        assert!(attestation["properties"].get("actor_user_id").is_none());
        assert!(attestation["properties"]
            .get("verification_authority")
            .is_none());
    }

    #[test]
    fn forged_capture_identity_is_refused_by_the_parser() {
        let args = json!({
            "capture_attestation": {
                "basis": "user_report",
                "support_summary": "The user chose this design.",
                "actor_user_id": uuid::Uuid::now_v7()
            }
        });
        assert!(capture_attestation_arg(&args).is_err());
    }

    fn capture_args(findings: Value) -> Value {
        json!({
            "cwd": "/repo",
            "action": "capture",
            "findings": findings,
        })
    }

    fn finding(content: &str) -> Value {
        json!({
            "type": "decision",
            "scope": "project",
            "scope_key": "project",
            "content": content,
            "topic_key": "capture.batch",
            "value_key": "complete-finding",
            "capture_attestation": {
                "basis": "user_report",
                "support_summary": "The user requested this durable decision."
            }
        })
    }

    #[test]
    fn capture_preflights_every_item_before_any_client_request() {
        let first = finding("The user requires one complete finding per captured record.");
        let mut invalid_later = finding("This finding must never be sent.");
        invalid_later["capture_attestation"]["actor_user_id"] = json!(uuid::Uuid::now_v7());
        let error = prepare_capture_findings(&capture_args(json!([first, invalid_later])))
            .expect_err("a later invalid item blocks the entire batch before send");
        assert!(error.message.contains("capture findings"));
    }

    #[test]
    fn capture_enforces_batch_bounds_and_strict_item_fields() {
        assert!(prepare_capture_findings(&capture_args(json!([]))).is_err());
        let nine = (0..9)
            .map(|_| finding("A complete finding remains independently reusable."))
            .collect::<Vec<_>>();
        assert!(prepare_capture_findings(&capture_args(json!(nine))).is_err());

        for field in [("type", json!("unknown")), ("scope", json!("unknown"))] {
            let mut item = finding("A complete finding remains independently reusable.");
            item[field.0] = field.1;
            assert!(prepare_capture_findings(&capture_args(json!([item]))).is_err());
        }
    }

    #[test]
    fn capture_rejects_blank_fields_and_bounds_utf8_before_sending() {
        for field in ["content", "topic_key", "value_key"] {
            let mut item = finding("A complete finding.");
            item[field] = json!(" \n\t");
            assert!(prepare_capture_findings(&capture_args(json!([item]))).is_err());
        }
        assert!(
            prepare_capture_findings(&capture_args(json!([finding(&"é".repeat(1025))]))).is_err()
        );
        assert!(
            prepare_capture_findings(&capture_args(json!([finding(&"é".repeat(1024))]))).is_ok()
        );
        let mut item = finding("A complete finding.");
        item["topic_key"] = json!("  Capture.Batch  ");
        item["value_key"] = json!("  COMPLETE-FINDING  ");
        let findings = prepare_capture_findings(&capture_args(json!([item]))).unwrap();
        assert_eq!(findings[0].topic_key, "capture.batch");
        assert_eq!(findings[0].value_key, "complete_finding");
    }

    #[test]
    fn capture_preserves_each_findings_provenance_and_multisentence_content() {
        let content = "The user requires a seven-day trial. The condition applies before release.";
        let mut second = finding("The source was inspected independently.");
        second["capture_attestation"] = json!({
            "basis": "inspected_source",
            "support_summary": "The named revision contains this observation.",
            "source_reference": "docs/product.md",
            "source_revision": "abc123"
        });
        let findings = prepare_capture_findings(&capture_args(json!([finding(content), second])))
            .expect("complete independently supported findings");
        assert_eq!(findings[0].content, content);
        assert_ne!(
            findings[0].capture_attestation.basis,
            findings[1].capture_attestation.basis
        );
    }

    #[test]
    fn capture_requires_attestation_and_refuses_root_write_fields() {
        let mut item = finding("A complete finding remains independently reusable.");
        item.as_object_mut().unwrap().remove("capture_attestation");
        assert!(prepare_capture_findings(&capture_args(json!([item]))).is_err());
        let mut args = capture_args(json!([finding(
            "A complete finding remains independently reusable."
        )]));
        args["local_only"] = json!(false);
        assert!(prepare_capture_findings(&args).is_err());
        args.as_object_mut().unwrap().remove("local_only");
        args["session_id"] = json!("not-a-cairn-session");
        assert!(prepare_capture_findings(&args).is_err());
        args["session_id"] = json!(uuid::Uuid::nil());
        assert!(prepare_capture_findings(&args).is_err());
    }

    #[test]
    fn capture_leaves_scope_key_to_existing_scope_derivation() {
        let mut item = finding("A complete finding remains independently reusable.");
        item.as_object_mut().unwrap().remove("scope_key");
        let findings = prepare_capture_findings(&capture_args(json!([item]))).unwrap();
        assert!(findings[0].scope_key.is_none());
    }

    #[test]
    fn capture_schema_requires_per_finding_provenance_but_not_scope_key() {
        let remember = tool_definitions()
            .into_iter()
            .find(|tool| tool["name"] == "cairn_remember")
            .unwrap();
        let required = remember["inputSchema"]["properties"]["findings"]["items"]["required"]
            .as_array()
            .unwrap();
        assert!(required.iter().any(|field| field == "capture_attestation"));
        assert!(!required.iter().any(|field| field == "scope_key"));
    }
}
