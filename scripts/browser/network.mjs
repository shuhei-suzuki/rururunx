// Fail closed where ordinary Playwright routing has gaps: no redirects, sockets,
// or page workers. Only the packaged trusted Stagehand extension worker survives.
export async function installNetworkPolicy(context, endpoint, permitted, mutates, result, timeout, extensionOrigin, createSocket = address => new WebSocket(address)) {
  let violated = false;
  const hold = (reason = 'network_interception') => { if (!violated) result.evidence?.push({ policy_reason: reason }); violated = true; };
  await context.exposeBinding('__rrx_browser_policy_hold', () => hold('worker_csp'));
  await context.addInitScript(() => document.addEventListener('securitypolicyviolation', event => {
    if (event.violatedDirective === 'worker-src') void globalThis.__rrx_browser_policy_hold();
  }));
  await context.routeWebSocket('**/*', socket => { hold(); socket.close(); });
  await context.route('**/*', async route => {
    const request = route.request();
    const write = !['GET', 'HEAD', 'OPTIONS'].includes(request.method());
    let source;
    try { source = request.frame().url(); } catch { source = request.serviceWorker?.()?.url(); }
    if (extensionOrigin && source?.startsWith(`${extensionOrigin}/`)
        && new URL(request.url()).pathname === '/v1/traces' && ['POST', 'OPTIONS'].includes(request.method())) {
      // Installed v4 has an example.com OTLP default. Optional SDK traces are
      // blocked before network access; they never determine browser correctness.
      await route.abort('blockedbyclient'); return;
    }
    if (extensionOrigin && request.url().startsWith(`${extensionOrigin}/`) && !write) {
      await route.continue(); return; // Packaged, exact owned extension resources only.
    }
    if (!permitted(request.url()) || write && !mutates || request.headers()['service-worker'] === 'script' || ['worker', 'sharedworker', 'serviceworker'].includes(request.headers()['sec-fetch-dest'])) {
      hold('origin_or_method'); await route.abort('blockedbyclient'); return;
    }
    if (write) result.effect_possible = true;
    try {
      // route.continue auto-follows redirects without another handler invocation.
      // Fetch zero hops and reject every redirect before handing it to Chromium.
      const response = await route.fetch({ maxRedirects: 0, timeout });
      if (response.status() >= 300 && response.status() < 400) {
        hold('redirect'); await route.abort('blockedbyclient');
      } else {
        const headers = response.headers();
        if (request.resourceType() === 'document') headers['content-security-policy'] = `${headers['content-security-policy'] ? headers['content-security-policy'] + ', ' : ''}worker-src 'none'`;
        await route.fulfill({ response, headers });
      }
    } catch {
      await route.abort('failed').catch(() => {});
    }
  });
  const descriptor = await (await fetch(`${endpoint}/json/version`)).json();
  const address = new URL(descriptor.webSocketDebuggerUrl);
  const expected = new URL(endpoint);
  if (address.protocol !== 'ws:' || address.hostname !== expected.hostname || address.port !== expected.port
      || !address.pathname.startsWith('/devtools/browser/')) throw new Error('protocol');
  const cdp = createSocket(address);
  const replies = new Map(); let sequence = 0;
  const send = (method, params, sessionId) => new Promise((resolve, reject) => {
    const id = ++sequence; replies.set(id, { resolve, reject, method });
    cdp.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
  });
  cdp.addEventListener('message', event => {
    void (async () => {
      if (typeof event.data !== 'string' || Buffer.byteLength(event.data) > 65536) { hold(); return; }
      const message = JSON.parse(event.data);
      if (message.id) {
        const reply = replies.get(message.id); replies.delete(message.id);
        if (message.error) { reply?.reject(new Error('protocol')); } else reply?.resolve(message.result);
      } else if (message.method === 'Target.attachedToTarget') {
        const event = message.params, target = event.targetInfo;
        if (extensionOrigin && target.type === 'service_worker' && target.url.startsWith(`${extensionOrigin}/`)) {
          await send('Runtime.runIfWaitingForDebugger', {}, event.sessionId);
        } else {
          // Fresh Chrome can create component-extension workers. Close them without
          // treating an inactive browser component as a webpage policy violation.
          if (event.waitingForDebugger && !target.url.startsWith('chrome-extension://')) hold('page_worker');
          await send('Target.closeTarget', { targetId: target.targetId });
        }
      }
    })().catch(() => hold('network_interception'));
  });
  cdp.addEventListener('close', () => { for (const reply of replies.values()) reply.reject(new Error('protocol')); replies.clear(); });
  await new Promise((resolve, reject) => {
    cdp.addEventListener('open', resolve, { once: true });
    cdp.addEventListener('error', () => reject(new Error('protocol')), { once: true });
  });
  context.on('close', () => cdp.close());
  await send('Target.setAutoAttach', { autoAttach: true, waitForDebuggerOnStart: true, flatten: true,
    filter: [{ type: 'service_worker' }, { type: 'shared_worker' }, { type: 'worker' }, { exclude: true }] });
  return () => { if (violated) throw new Error('policy_hold'); };
}
