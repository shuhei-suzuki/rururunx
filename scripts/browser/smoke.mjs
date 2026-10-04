// Actual SDK dogfood against a private synthetic target; no remote website/account writes.
import { createServer } from 'node:http';
import { mkdtemp, mkdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import assert from 'node:assert/strict';
import { invoke } from './io.mjs';

const adaptive = process.argv.includes('--adaptive');
const folder = await mkdtemp(path.join(tmpdir(), 'rrx-browser-smoke-'));
let actions = 0;
const server = createServer((request, response) => {
  if (request.url === '/write') { actions++; response.end('ok'); return; }
  response.setHeader('Content-Type', 'text/html');
  response.end('<!doctype html><main id="fixture"><h1>Browser fixture</h1><p id="status">Ready</p><label>Name<input id="name"></label><button id="greet" onclick="document.querySelector(\'#status\').textContent=\'Hello \'+document.querySelector(\'#name\').value">Greet</button><p id="fact">Reference number: 42</p></main>');
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const scope = { project_id: randomUUID(), goal_id: randomUUID(), task_id: randomUUID() };
const here = path.dirname(fileURLToPath(import.meta.url));
try {
  const artifactDir = path.join(folder, 'session'); await mkdir(artifactDir);
  const steps = [{ kind: 'navigate', url: origin }];
  if (adaptive) {
    steps.push({ kind: 'observe', instruction: 'Find the Greet button.', scope_selector: '#greet' },
      { kind: 'act', instruction: 'Click the Greet button.', scope_selector: '#greet' },
      { kind: 'assert_text', selector: '#status', expected: 'Hello' });
    steps.push({ kind: 'extract', instruction: 'Extract the numeric reference number displayed in this paragraph.', scope_selector: '#fact',
      schema: { type: 'object', properties: { reference: { type: 'integer' } }, required: ['reference'], additionalProperties: false } });
  } else {
    steps.push({ kind: 'fill', selector: '#name', value: 'Fixture' }, { kind: 'click', selector: '#greet' },
      { kind: 'assert_text', selector: '#status', expected: 'Hello Fixture' });
  }
  steps.push({ kind: 'screenshot', name: 'verified.png' });
  const input = { protocol_version: 1, request: { scope, allowed_origins: [origin], steps, deterministic_fallback: null },
    session_id: randomUUID(), backend: adaptive ? 'stagehand' : 'playwright', artifact_dir: artifactDir,
    config: { headed: process.argv.includes('--headed'), allow_loopback_actions: true, timeout_ms: 180000, step_timeout_ms: 120000,
      browser_executable: null, model: adaptive ? { model_name: null, api_key_env: null, custom_command: [process.execPath, path.join(here, 'native-claude.mjs')] } : null } };
  const result = await invoke([process.execPath, path.join(here, 'bridge.mjs')], input, 190000, 262144, true);
  assert.equal(result.success, true, JSON.stringify(result));
  assert.equal(result.evidence[0].headed, process.argv.includes('--headed'));
  if (adaptive) { assert.equal(result.evidence.find(e => e.kind === 'extract').data.reference, 42); assert.ok(result.usage.llm_calls > 0); }
  else assert.equal(result.usage.llm_calls, 0);
  assert.equal(actions, 0);
  console.log(JSON.stringify({ backend: result.backend, success: result.success, evidence: result.evidence,
    artifacts: result.artifacts, usage: result.usage, effects: result.effect_possible }));
} finally {
  await new Promise(resolve => server.close(resolve));
  await rm(folder, { recursive: true, force: true });
}
