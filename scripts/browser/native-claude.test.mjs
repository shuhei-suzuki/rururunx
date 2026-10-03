import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { invoke } from './io.mjs';

test('native callback inherits owned group/cwd and preserves strict restrictions/default model', async () => {
  const folder = await mkdtemp(path.join(tmpdir(), 'rrx-native-model-contract-'));
  try {
    const script = `#!/usr/bin/env python3
import sys,json,os
a=sys.argv[1:]
assert a[a.index('--tools')+1]==''
assert a[a.index('--disallowedTools')+1]=='*'
assert '--strict-mcp-config' in a
assert json.loads(a[a.index('--mcp-config')+1])=={'mcpServers':{}}
assert a[a.index('--permission-prompts')+1]=='none'
assert not any(x in a for x in ['--bare','--dangerously-skip-permissions','--system-prompt','--setting-sources','--model','--effort'])
assert os.getpgrp()==os.getppid(), 'native child escaped its bridge-owned process group'
assert os.getcwd()==os.environ['RRX_NATIVE_TEST_CWD']
p=json.load(sys.stdin)
assert p['scope']['task_id']=='fixture-task'
assert p['browser_request']['responseFormat']['schema']['properties']['answer']['type']=='integer'
print(json.dumps({'type':'result','subtype':'success','is_error':False,'session_id':a[a.index('--session-id')+1],'result':'{\"answer\":42}','usage':{'input_tokens':7,'output_tokens':11}}))
`;
    await writeFile(path.join(folder, 'claude'), script, { mode: 0o700 });
    const params = { messages: [{ role: 'user', content: { type: 'text', text: 'The answer is 42.' } }],
      responseFormat: { type: 'json_schema', name: 'fixture', schema: { type: 'object', properties: { answer: {type: 'integer'} }, required: ['answer'], additionalProperties: false } } };
    const output = await invoke(['/usr/bin/env', `PATH=${folder}:${process.env.PATH}`, `RRX_NATIVE_TEST_CWD=${process.cwd()}`, process.execPath,
      path.join(path.dirname(fileURLToPath(import.meta.url)), 'native-claude.mjs')],
      { scope: {project_id: 'fixture-project', task_id: 'fixture-task'}, params, timeout_ms: 5000, model_name: null }, 6000, 262144, true);
    assert.deepEqual(output.response.structuredContent, {answer: 42});
    assert.equal(output.usage.input_tokens, 7);
    assert.equal(output.usage.cached_input_tokens, null);
    assert.equal(output.usage.cost_usd, null);
    assert.match(output.usage.native_sessions[0], /^[0-9a-f-]{36}$/);
  } finally { await rm(folder, {recursive: true, force: true}); }
});

test('bounded child cannot keep a callback alive after timeout', async () => {
  const start = Date.now();
  await assert.rejects(invoke(['python3', '-c', 'import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); time.sleep(20)'], {}, 100), /timeout/);
  assert.ok(Date.now() - start < 2500);
});
