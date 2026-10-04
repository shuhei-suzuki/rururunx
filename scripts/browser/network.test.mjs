import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { installNetworkPolicy } from './network.mjs';
import { boundedText } from './policy.mjs';

class FixtureSocket extends EventTarget {
  messages = [];
  constructor() { super(); queueMicrotask(() => this.dispatchEvent(new Event('open'))); }
  message(message) { this.dispatchEvent(new MessageEvent('message', { data: JSON.stringify(message) })); }
  send(bytes) { const message = JSON.parse(bytes); this.messages.push(message);
    queueMicrotask(() => this.message({ id: message.id, result: {} })); }
  close() { this.dispatchEvent(new Event('close')); }
}
test('network policy blocks redirects, sockets and page workers before forwarding', async () => {
  const server = createServer((_, response) => response.end(JSON.stringify({webSocketDebuggerUrl: `ws://127.0.0.1:${server.address().port}/devtools/browser/fixture`})));
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const context = { route: async (_, handler) => { context.routeHandler = handler; },
    routeWebSocket: async (_, handler) => { context.socketHandler = handler; }, on: () => {}, exposeBinding: async () => {}, addInitScript: async () => {} };
  const result = { effect_possible: false }; let cdp;
  const extension = 'chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  try {
    const check = await installNetworkPolicy(context, `http://127.0.0.1:${server.address().port}`, url => url.startsWith('http://fixture/'), true, result, 1000, extension,
      () => { cdp = new FixtureSocket(); return cdp; });
    assert.equal(cdp.messages[0].params.waitForDebuggerOnStart, true);
    const route = (url, method = 'GET', status = 200, headers = {}) => ({
      request: () => ({url: () => url, method: () => method, headers: () => headers, resourceType: () => 'document'}),
      fetch: async options => { assert.equal(options.maxRedirects, 0); return { status: () => status, headers: () => ({}) }; },
      abort: async () => { result.aborted = true; }, fulfill: async () => { result.fulfilled = true; },
      continue: async () => { result.continued = true; }
    });
    const trace = route('https://unlisted/v1/traces', 'POST');
    trace.request = () => ({url:()=> 'https://unlisted/v1/traces', method:()=> 'POST', frame:()=>({url:()=> `${extension}/wake-service-worker.html`})});
    await context.routeHandler(trace);
    assert.equal(result.aborted, true);
    assert.equal(result.effect_possible, false);
    assert.doesNotThrow(check); // Optional export is aborted without invalidating the primary operation.
    result.aborted = false;
    await context.routeHandler(route('http://fixture/write', 'POST', 307));
    assert.ok(result.effect_possible && result.aborted);
    assert.equal(result.fulfilled, undefined);
    assert.throws(check, /policy_hold/);
    result.aborted = false;
    await context.routeHandler(route('http://unlisted/write', 'POST'));
    assert.equal(result.aborted, true);
    result.aborted = false;
    await context.routeHandler(route('http://fixture/worker.js', 'GET', 200, {'sec-fetch-dest': 'worker'}));
    assert.equal(result.aborted, true);
    result.aborted = false;
    await context.routeHandler(route('http://fixture/sw.js', 'GET', 200, {'service-worker': 'script'}));
    assert.equal(result.aborted, true);
    await context.routeHandler(route(`${extension}/wake-service-worker.html`));
    assert.equal(result.continued, true);
    let closed = false;
    context.socketHandler({ close: () => { closed = true; }, connectToServer: () => assert.fail('socket forwarded') });
    assert.ok(closed);
    cdp.message({method: 'Target.attachedToTarget', params: {sessionId: 'worker-session', targetInfo: {targetId: 'page-worker', type: 'service_worker', url: 'http://fixture/sw.js'}}});
    cdp.message({method: 'Target.attachedToTarget', params: {sessionId: 'extension-session', targetInfo: {targetId: 'trusted-worker', type: 'service_worker', url: `${extension}/service-worker.js`}}});
    await new Promise(resolve => setImmediate(resolve));
    assert.ok(cdp.messages.some(m => m.method === 'Target.closeTarget' && m.params.targetId === 'page-worker'));
    assert.ok(cdp.messages.some(m => m.method === 'Runtime.runIfWaitingForDebugger' && m.sessionId === 'extension-session'));
    assert.ok(!cdp.messages.some(m => m.method === 'Runtime.runIfWaitingForDebugger' && m.sessionId === 'worker-session'));
  } finally { cdp?.close(); await new Promise(resolve => server.close(resolve)); }
});
test('selected read text stays within the serialized UTF-8 budget', () => {
  for (const text of ['日本語'.repeat(2000), '😀'.repeat(3000), '\"\\\n'.repeat(2000)]) {
    const data = boundedText(text);
    assert.ok(Buffer.byteLength(JSON.stringify(data)) <= 8192);
    assert.ok(!/[\uD800-\uDBFF]$/.test(data.text));
    assert.ok(text.startsWith(data.text));
  }
});
