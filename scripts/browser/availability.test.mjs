import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtemp, mkdir, copyFile, symlink, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { invoke } from './io.mjs';

test('unavailable adaptive SDK preserves deterministic discovery and safe typed fallback', async () => {
  const folder = await mkdtemp(path.join(tmpdir(), 'rrx-browser-sdk-contract-'));
  try {
    for (const file of ['bridge.mjs', 'io.mjs', 'policy.mjs']) await copyFile(new URL(file, import.meta.url), path.join(folder, file));
    const modules = path.join(folder, 'node_modules');
    await mkdir(path.join(modules, '@browserbasehq', 'stagehand'), {recursive: true});
    // Mask any ancestor installation without modifying installed SDK packages.
    await writeFile(path.join(modules, '@browserbasehq', 'stagehand', 'package.json'),
      JSON.stringify({name: '@browserbasehq/stagehand', exports: {'.': null}}));
    const require = createRequire(import.meta.url);
    for (const name of ['playwright-core', 'zod']) await symlink(path.dirname(require.resolve(`${name}/package.json`)), path.join(modules, name), 'dir');
    const command = [process.execPath, path.join(folder, 'bridge.mjs')];
    const capabilities = await invoke([...command, '--capabilities'], {}, 5000, 262144, true);
    assert.equal(capabilities.deterministic, true);
    assert.equal(capabilities.adaptive, false);
    assert.equal(capabilities.stagehand_version, null);
    const scope = {project_id: 'fixture-project', task_id: 'fixture-task'};
    const result = await invoke(command, {protocol_version: 1, backend: 'stagehand', session_id: 'fixture-session',
      request: {scope}, config: {}, artifact_dir: path.join(folder, 'artifacts')}, 5000, 262144, true);
    assert.equal(result.success, false);
    assert.equal(result.failure, 'unsupported');
    assert.equal(result.effect_possible, false);
    assert.deepEqual(result.scope, scope);
  } finally { await rm(folder, {recursive: true, force: true}); }
});
