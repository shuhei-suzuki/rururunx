import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromeArguments, checkChromeArguments } from './policy.mjs';

test('adaptive browser grants only its owned extension CDP origin', () => {
  const origin = 'chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const args = chromeArguments('/private/owned/profile', true, origin);
  assert.equal(args.filter(a => a.startsWith('--remote-allow-origins=')).join(), `--remote-allow-origins=${origin}`);
  assert.deepEqual(checkChromeArguments(args, true, origin), {session: 'browser_mode', headed: true});
  for (const unexpected of ['*', 'http://remote.example', `${origin},http://remote.example`]) {
    const untrusted = args.filter(a => !a.startsWith('--remote-allow-origins=')).concat(`--remote-allow-origins=${unexpected}`);
    assert.throws(() => checkChromeArguments(untrusted, true, origin), /protocol/);
  }
});
test('deterministic browser has no CDP origin grant and preserves native security', () => {
  const args = chromeArguments('/private/owned/profile', false);
  assert.deepEqual(checkChromeArguments(args, false), {session: 'browser_mode', headed: false});
  for (const unsafe of ['--no-sandbox', '--disable-web-security', '--ignore-certificate-errors', '--remote-allow-origins=*']) {
    assert.throws(() => checkChromeArguments([...args, unsafe], false), /protocol/);
  }
  assert.throws(() => checkChromeArguments(args, true), /protocol/);
});
