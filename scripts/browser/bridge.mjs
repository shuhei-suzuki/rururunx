import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { readFile, mkdir, rm, stat, realpath } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import { z } from 'zod';
import { invoke, readInput } from './io.mjs';
import { chromeArguments, checkChromeArguments } from './policy.mjs';

const require = createRequire(import.meta.url);
const version = name => JSON.parse(require('node:fs').readFileSync(path.join(path.dirname(fileURLToPath(import.meta.resolve(name))), '..', 'package.json'))).version;
const playwrightVersion = require('playwright-core/package.json').version;
// An absent/incompatible adaptive SDK must not disable the deterministic backend.
const stagehandVersion = (() => { try { return version('@browserbasehq/stagehand'); } catch { return null; } })();
const [nodeMajor, nodeMinor] = process.versions.node.split('.').map(Number);
const adaptiveCompatible = stagehandVersion === '4.1.0' && (nodeMajor > 22 || nodeMajor === 22 && nodeMinor >= 18);
if (process.argv.includes('--capabilities')) {
  process.stdout.write(JSON.stringify({ protocol_version: 1, playwright_version: playwrightVersion,
    stagehand_version: stagehandVersion, deterministic: true, adaptive: adaptiveCompatible,
    owned_connect: true, external_connect: false, headed: true, jev: false }));
} else {
  await main();
}

async function main() {
  let input, result, chrome, pw, browser, stagehand, profile, timer, closing;
  const cleanup = async () => {
    if (closing) return closing;
    closing = (async () => {
      // Stagehand close releases its agent; explicit browser close owns the CDP session.
      await stagehand?.close().catch(() => {});
      await browser?.close().catch(() => {});
      await pw?.close().catch(() => {});
      if (chrome && chrome.exitCode === null && chrome.signalCode === null) {
        const exited = new Promise(resolve => chrome.once('close', resolve));
        chrome.kill('SIGTERM');
        await Promise.race([exited, new Promise(resolve => setTimeout(resolve, 1000))]);
        if (chrome.exitCode === null && chrome.signalCode === null) chrome.kill('SIGKILL');
      }
      if (profile) await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
    })();
    return closing;
  };
  process.once('SIGTERM', () => { void cleanup().finally(() => process.exit(143)); });
  try {
    input = await readInput();
    const { request, config, backend } = input;
    result = { scope: request.scope, session_id: input.session_id, backend, success: false, failure: null,
      effect_possible: false, evidence: [], artifacts: [], fallback_used: false,
      usage: { native_sessions: [], llm_calls: backend === 'playwright' ? 0 : null, input_tokens: null, output_tokens: null,
        cached_input_tokens: null, cache_write_tokens: null, cost_usd: null, inference_ms: null,
        source: backend === 'playwright' ? 'deterministic-no-model' : null } };
    if (input.protocol_version !== 1 || !['playwright', 'stagehand'].includes(backend)) throw new Error('unsupported');
    if (backend === 'stagehand' && !adaptiveCompatible) throw new Error('unsupported');
    const origins = new Set(request.allowed_origins.map(origin => {
      const url = new URL(origin);
      if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password || url.origin !== origin) throw new Error('policy_hold');
      return url.origin;
    }));
    const loopback = origin => { const u = new URL(origin); return u.protocol === 'http:' && ['localhost', '127.0.0.1', '[::1]'].includes(u.hostname); };
    const permitted = url => { try { const parsed = new URL(url); return !parsed.username && !parsed.password && origins.has(parsed.origin); } catch { return false; } };
    if (!request.steps.length || request.steps.length > 64) throw new Error('policy_hold');
    const mutates = request.steps.some(s => ['click', 'fill', 'act'].includes(s.kind));
    if (mutates && (!config.allow_loopback_actions || [...origins].some(o => !loopback(o)))) throw new Error('policy_hold');
    if (request.steps.some(s => ['observe', 'act', 'extract'].includes(s.kind) && !s.scope_selector)) throw new Error('policy_hold');
    // Only selected semantic DOM is sent to the model. No workflow history or full HTML.
    const model = backend === 'stagehand' ? configuredModel(config.model, request.scope, config.step_timeout_ms, result.usage) : null;
    profile = path.join(input.artifact_dir, 'profile');
    await mkdir(profile, { mode: 0o700 });
    const executable = config.browser_executable ?? (process.platform === 'darwin'
      ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : '/usr/bin/google-chrome');
    // Chromium derives unpacked extension IDs from their canonical path (or manifest key).
    // Permit only the packaged Stagehand extension origin, never wildcard CDP origins.
    let extensionOrigin;
    if (backend === 'stagehand') {
      const directory = await realpath(path.join(path.dirname(fileURLToPath(import.meta.resolve('@browserbasehq/stagehand'))), 'extension'));
      const manifest = JSON.parse(await readFile(path.join(directory, 'manifest.json'), 'utf8'));
      const bytes = manifest.key ? Buffer.from(manifest.key, 'base64') : Buffer.from(directory);
      const id = createHash('sha256').update(bytes).digest('hex').slice(0, 32).replace(/[0-9a-f]/g, c => String.fromCharCode(97 + parseInt(c, 16)));
      extensionOrigin = `chrome-extension://${id}`;
    }
    // Own Chrome directly: SDK launch intentionally detaches, escaping the bridge's group.
    const args = chromeArguments(profile, config.headed, extensionOrigin);
    chrome = spawn(executable, args, { stdio: 'ignore', detached: false });
    let launchFailed = false;
    chrome.on('error', () => { launchFailed = true; });
    const cdp = await endpoint(profile, () => launchFailed || chrome.exitCode !== null);
    pw = await chromium.connectOverCDP(cdp);
    const context = pw.contexts()[0];
    await context.clearCookies();
    await context.route('**/*', async route => {
      const request = route.request();
      // Permit only exact authorized origins; read-only sessions also prohibit write verbs.
      if (permitted(request.url()) && (mutates || ['GET', 'HEAD', 'OPTIONS'].includes(request.method()))) await route.continue();
      else await route.abort('blockedbyclient');
    });
    context.on('page', page => { page.on('download', download => { void download.cancel(); }); });
    const page = context.pages().find(p => p.url() === 'about:blank') ?? await context.newPage();
    page.setDefaultTimeout(config.step_timeout_ms);
    const cdpSession = await context.newCDPSession(page);
    const commandLine = await cdpSession.send('Browser.getBrowserCommandLine');
    result.evidence.push(checkChromeArguments(commandLine.arguments, config.headed, extensionOrigin));
    await cdpSession.send('Browser.setDownloadBehavior', { behavior: 'deny' });
    let semanticPage;
    if (backend === 'stagehand') {
      const { Stagehand, localBrowser } = await import('@browserbasehq/stagehand');
      browser = await localBrowser.connect({ cdpUrl: cdp });
      stagehand = await Stagehand.create({ browser, model, selfHeal: false, cache: false, logging: { level: 'off' } });
      for (const candidate of await browser.context.pages()) { if (await candidate.url() === 'about:blank') { semanticPage = candidate; break; } }
      if (!semanticPage) throw new Error('protocol');
      await browser.context.setDomainPolicy({ allowedDomains: [...origins].map(o => new URL(o).hostname) });
    }
    timer = setTimeout(() => { void cleanup().finally(() => process.exit(124)); }, config.timeout_ms);
    for (let index = 0; index < request.steps.length; index++) {
      const step = request.steps[index];
      if (step.kind !== 'navigate' && !permitted(page.url())) throw new Error('policy_hold');
      if (['click', 'fill', 'act'].includes(step.kind)) result.effect_possible = true;
      let data;
      switch (step.kind) {
        case 'navigate': {
          if (!permitted(step.url)) throw new Error('policy_hold');
          const response = await page.goto(step.url, { waitUntil: 'domcontentloaded', timeout: config.step_timeout_ms });
          if (!permitted(page.url())) throw new Error('policy_hold');
          data = { status: response?.status() ?? null }; break;
        }
        case 'click': await page.locator(step.selector).click(); break;
        case 'fill': await page.locator(step.selector).fill(step.value); break;
        case 'assert_visible': if (!await page.locator(step.selector).isVisible()) throw new Error('assertion'); break;
        case 'assert_text': {
          const text = await page.locator(step.selector).innerText();
          if (text !== step.expected) throw new Error('assertion'); data = { matched: true }; break;
        }
        case 'read_text': data = { text: (await page.locator(step.selector).innerText()).slice(0, 4096) }; break;
        case 'screenshot': {
          if (!/^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,95}\.png$/.test(step.name)) throw new Error('policy_hold');
          const destination = path.join(input.artifact_dir, step.name);
          try { await stat(destination); throw new Error('policy_hold'); } catch (error) { if (error.code !== 'ENOENT') throw error; }
          await page.screenshot({ path: destination, fullPage: false, timeout: config.step_timeout_ms });
          result.artifacts.push(step.name); break;
        }
        case 'observe': data = (await stagehand.observe(step.instruction, { page: semanticPage,
          locator: semanticPage.locator(step.scope_selector), timeout: config.step_timeout_ms })).data; break;
        case 'act': {
          const acted = await stagehand.act(step.instruction, { page: semanticPage,
            locator: semanticPage.locator(step.scope_selector), timeout: config.step_timeout_ms });
          if (acted.data?.success !== true) throw new Error('operation'); data = { completed: true }; break;
        }
        case 'extract': data = (await stagehand.extract(step.instruction, z.fromJSONSchema(step.schema), { page: semanticPage,
          locator: semanticPage.locator(step.scope_selector), timeout: config.step_timeout_ms, screenshot: false })).data; break;
        default: throw new Error('unsupported');
      }
      // Preserve structured assertions and selected facts, never raw DOM or model conversation.
      if (data !== undefined && Buffer.byteLength(JSON.stringify(data)) > 8192) throw new Error('output_limit');
      result.evidence.push({ step: index, kind: step.kind, passed: true, ...(data === undefined ? {} : { data }) });
    }
    if (stagehand && !config.model?.custom_command) {
      const metrics = await stagehand.metrics();
      Object.assign(result.usage, { input_tokens: metrics.totalPromptTokens, output_tokens: metrics.totalCompletionTokens,
        cached_input_tokens: metrics.totalCachedInputTokens, inference_ms: metrics.totalInferenceTimeMs, source: 'stagehand-session-metrics' });
    }
    result.success = true;
  } catch (error) {
    if (!result) { process.exitCode = 1; return; }
    const known = ['unavailable', 'unsupported', 'policy_hold', 'timeout', 'assertion', 'output_limit', 'protocol'];
    result.failure = known.includes(error.message) ? error.message : error.name === 'TimeoutError' ? 'timeout' : 'operation';
  } finally {
    clearTimeout(timer);
    await cleanup();
    if (result) process.stdout.write(JSON.stringify(result));
  }
}
function configuredModel(config, scope, timeout, usage) {
  if (config?.custom_command?.length) {
    let calls = 0;
    return { generate: async params => {
      const output = await invoke(config.custom_command, { params, scope, timeout_ms: timeout, model_name: config.model_name }, timeout);
      if (!output.response || !output.usage) throw new Error('protocol');
      for (const key of ['llm_calls', 'input_tokens', 'output_tokens', 'cached_input_tokens', 'cache_write_tokens', 'cost_usd', 'inference_ms']) {
        const value = output.usage[key];
        usage[key] = calls === 0 ? (typeof value === 'number' ? value : null)
          : typeof value === 'number' && typeof usage[key] === 'number' ? usage[key] + value : null;
      }
      usage.source = output.usage.source ?? null;
      if (Array.isArray(output.usage.native_sessions)) usage.native_sessions.push(...output.usage.native_sessions);
      calls++;
      return output.response;
    } };
  }
  if (!config?.model_name || !config.api_key_env || !process.env[config.api_key_env]) throw new Error('unavailable');
  return { modelName: config.model_name, apiKey: process.env[config.api_key_env] };
}
async function endpoint(profile, failed) {
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    if (failed()) throw new Error('unavailable');
    try { const lines = (await readFile(path.join(profile, 'DevToolsActivePort'), 'utf8')).split('\n');
      if (/^\d+$/.test(lines[0])) return `http://127.0.0.1:${lines[0]}`;
    } catch {}
    await new Promise(resolve => setTimeout(resolve, 25));
  }
  throw new Error('timeout');
}
