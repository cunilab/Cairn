// Run only against a disposable alpha.9 deployment and a packaged candidate.
// Required: ALPHA9_BASE, ALPHA9_BIN_DIR, ALPHA9_EMAIL, ALPHA9_PASSWORD.
const { chromium } = require("@playwright/test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const cp = require("node:child_process");
const crypto = require("node:crypto");

const base = process.env.ALPHA9_BASE;
const bin = process.env.ALPHA9_BIN_DIR;
for (const key of ["ALPHA9_BASE", "ALPHA9_BIN_DIR", "ALPHA9_EMAIL", "ALPHA9_PASSWORD"]) {
  assert.ok(process.env[key], `${key} is required`);
}
const work = fs.mkdtempSync(path.join(os.tmpdir(), "cairn-alpha9-fresh-"));
const repo = path.join(work, "repo");
const home = path.join(work, "home");
fs.mkdirSync(repo);
fs.mkdirSync(home);
const remote = `https://github.com/example/fresh-${crypto.randomUUID()}.git`;
const projectName = `Fresh ${crypto.randomUUID().slice(0, 8)}`;
const actorA = `fresh-a-${crypto.randomUUID()}`;
const actorB = `fresh-b-${crypto.randomUUID()}`;
const fact = `alpha9 fresh journey ${crypto.randomUUID()} recalls its fact`;

function run(file, args, env, input) {
  const result = cp.spawnSync(file, args, {
    cwd: repo, env, input, encoding: "utf8", timeout: 15000,
  });
  assert.equal(result.status, 0, `${file} ${args.join(" ")}: ${result.stderr}`);
  return result.stdout;
}

function mcp(name, args, env) {
  const request = {
    jsonrpc: "2.0", id: 1, method: "tools/call",
    params: { name, arguments: { ...args, cwd: repo } },
  };
  const output = run(path.join(bin, "cairn"), ["mcp"], env, JSON.stringify(request) + "\n");
  const reply = JSON.parse(output.trim());
  assert.equal(reply.result?.isError, false, JSON.stringify(reply));
  return reply.result.content[0].text;
}

(async () => {
  const started = Date.now();
  let browser;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`${base}/login`);
    await page.getByTestId("email").fill(process.env.ALPHA9_EMAIL);
    await page.getByTestId("password").fill(process.env.ALPHA9_PASSWORD);
    await page.getByTestId("submit").click();
    await page.getByRole("heading", { name: "Projects" }).waitFor();
    await page.reload();
    await page.getByRole("heading", { name: "Projects" }).waitFor();

    await page.goto(`${base}/settings`);
    await page.getByLabel("Project name", { exact: true }).fill(projectName);
    await page.getByLabel("Repository remote", { exact: true }).fill(remote);
    const projectReply = page.waitForResponse((response) =>
      new URL(response.url()).pathname === "/api/projects" &&
      response.request().method() === "POST");
    await page.getByRole("button", { name: "Create project", exact: true }).click();
    const response = await projectReply;
    assert.equal(response.status(), 200);
    const project = await response.json();
    await page.getByTestId("project-members").locator("ul li").first().waitFor();

    await page.getByTestId("new-token").click();
    await page.getByTestId("token-name").fill(`fresh-${crypto.randomUUID().slice(0, 8)}`);
    await page.getByTestId("create-token").click();
    const credentials = JSON.parse(await page.getByTestId("token-plaintext").innerText());
    assert.equal(credentials.server_url, process.env.ALPHA9_EXPECT_API ?? base,
      "token must name the browser-reachable API");
    assert.equal(credentials.web_url, base);

    if (!process.env.ALPHA9_EXPECT_API) {
      const denied = await page.request.get(`${base}/api/health`, {
        headers: { origin: "https://unrelated.example.test" },
      });
      assert.equal(denied.status(), 200);
      assert.notEqual(denied.headers()["access-control-allow-origin"],
        "https://unrelated.example.test");
      assert.equal(denied.headers()["access-control-allow-origin"], base);
    }

    run("git", ["init", "-q"], process.env);
    run("git", ["remote", "add", "origin", remote], process.env);
    const env = {
      ...process.env,
      HOME: home,
      XDG_CONFIG_HOME: path.join(home, ".config"),
      CAIRN_HOME: home,
      CAIRND_BIN: path.join(bin, "cairnd"),
      CAIRN_SERVER_URL: credentials.server_url,
      CAIRN_SERVER_TOKEN: credentials.server_token,
      CAIRN_WEB_URL: credentials.web_url,
    };
    const setup = JSON.parse(run(path.join(bin, "cairn"), ["--json", "setup"], env));
    assert.equal(setup.data.project.linked, true);
    for (const actor of [actorA, actorB]) {
      const output = run(path.join(bin, "cairn"), ["hook", "SessionStart", "--agent", "codex"],
        env, JSON.stringify({ session_id: actor, source: "startup", cwd: repo }));
      assert.equal(JSON.parse(output).hookSpecificOutput.hookEventName, "SessionStart");
    }
    const accepted = JSON.parse(mcp("cairn_remember", {
      action: "create", agent_session_key: actorA, type: "fact",
      topic_key: "alpha9.fresh", value_key: "remembered", content: fact,
      capture_attestation: {
        basis: "user_report",
        support_summary: "The authenticated operator authored this synthetic journey fact for later recall.",
      },
    }, env));
    assert.equal(accepted.accepted_for_delivery, true);

    let recalled = "";
    for (let attempt = 0; attempt < 50; attempt++) {
      recalled = mcp("cairn_search", {
        action: "search", agent_session_key: actorB, query: fact,
      }, env);
      if (recalled.includes(fact)) break;
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    assert.ok(recalled.includes(fact), "later caller did not recall the accepted fact");
    await page.goto(`${base}/projects/${project.id}/memory`);
    await page.getByTestId("memory-list").getByText(fact).waitFor({ timeout: 15000 });
    await page.reload();
    await page.getByTestId("memory-list").getByText(fact).waitFor({ timeout: 15000 });
    console.log(JSON.stringify({
      status: "PASS", project: project.id,
      elapsed_seconds: Math.round((Date.now() - started) / 1000),
    }));
  } finally {
    try {
      if (browser) await browser.close();
    } finally {
      fs.rmSync(work, { recursive: true });
    }
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
