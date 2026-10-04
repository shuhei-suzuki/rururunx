// Optional real Chrome policy regressions. Never invokes a model or remote site.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { mkdtemp, mkdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { invoke } from './io.mjs';

test('actual Chrome holds redirect, websocket and worker egress with no receiver contact', {skip: process.env.RRX_BROWSER_REAL_TESTS !== '1'}, async () => {
  const folder = await mkdtemp(path.join(tmpdir(), 'rrx-network-real-'));
  let received = 0;
  const denied = createServer((_, response) => { received++; response.end('unexpected'); });
  denied.on('upgrade', (_, socket) => { received++; socket.destroy(); });
  await new Promise(resolve => denied.listen(0, '127.0.0.1', resolve));
  const deniedOrigin = `http://127.0.0.1:${denied.address().port}`;
  const server = createServer((request, response) => {
    if (request.url === '/redirect') { response.writeHead(307, {Location: `${deniedOrigin}/write`}); response.end(); return; }
    if (request.url === '/sw.js' || request.url === '/worker.js') {
      response.setHeader('Content-Type', 'text/javascript'); response.end(request.url === '/worker.js' ? `new WebSocket('${deniedOrigin.replace('http:', 'ws:')}/write')` : `fetch('${deniedOrigin}/write',{method:'POST'})`); return;
    }
    if (request.url === '/text') { response.setHeader('Content-Type', 'text/html; charset=utf-8'); response.end('<main id="status">'+'日'.repeat(4096)+'</main>'); return; }
    const operation = request.url === '/blobworker' ? `new Worker(URL.createObjectURL(new Blob(["new WebSocket('${deniedOrigin.replace('http:', 'ws:')}/write')"],{type:'text/javascript'})))` : request.url === '/socket' ? `new WebSocket('${deniedOrigin.replace('http:', 'ws:')}/write')`
      : request.url === '/serviceworker' ? `navigator.serviceWorker.register('/sw.js').catch(()=>{})` : `new Worker('/worker.js')`;
    response.setHeader('Content-Type', 'text/html');
    response.end(`<main id="status">Fixture</main><script type="module">try {${operation};}catch{} await new Promise(r=>setTimeout(r,500));</script>`);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  try {
    for (const kind of ['redirect', 'socket', 'serviceworker', 'worker', 'blobworker']) {
      const artifact = path.join(folder, kind); await mkdir(artifact);
      const result = await invoke([process.execPath, path.join(path.dirname(fileURLToPath(import.meta.url)), 'bridge.mjs')], {
        protocol_version: 1, backend: 'playwright', session_id: randomUUID(), artifact_dir: artifact,
        config: {headed: false, allow_loopback_actions: false, timeout_ms: 15000, step_timeout_ms: 10000},
        request: {scope: {project_id: randomUUID(), goal_id: randomUUID(), task_id: randomUUID()}, allowed_origins: [origin],
          steps: [{kind:'navigate',url:`${origin}/${kind}`},{kind:'read_text',selector:'#status'}]}}, 20000, 262144, true);
      assert.equal(received, 0, `unlisted receiver contacted: ${kind}`);
      assert.equal(result.failure, 'policy_hold', JSON.stringify({kind,result}));
      assert.equal(result.success, false);
      assert.equal(result.usage.llm_calls, 0);
      assert.equal(result.effect_possible, false);
    }
    const artifact = path.join(folder, 'budget'); await mkdir(artifact);
    const result = await invoke([process.execPath, path.join(path.dirname(fileURLToPath(import.meta.url)), 'bridge.mjs')], {
      protocol_version: 1, backend: 'playwright', session_id: randomUUID(), artifact_dir: artifact,
      config: {headed: false, allow_loopback_actions: false, timeout_ms: 15000, step_timeout_ms: 10000},
      request: {scope: {project_id: randomUUID(), goal_id: randomUUID(), task_id: randomUUID()}, allowed_origins: [origin],
        steps: [{kind:'navigate',url:`${origin}/text`}, ...Array.from({length:63},()=>({kind:'read_text',selector:'#status'}))]}}, 20000, 262144, true);
    assert.equal(result.failure, 'output_limit');
    assert.ok(Buffer.byteLength(JSON.stringify(result)) < 65536);
    assert.ok(result.evidence.some(e => e.kind === 'read_text' && e.data.text.startsWith('日')));
    assert.equal(result.usage.llm_calls, 0);
    assert.equal(result.effect_possible, false);
  } finally {
    await Promise.all([new Promise(resolve => server.close(resolve)), new Promise(resolve => denied.close(resolve))]);
    await rm(folder, {recursive: true, force: true});
  }
});
