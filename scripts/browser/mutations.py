#!/usr/bin/env python3
"""Targeted reversible contract mutations; never runs a real model/browser."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
RUST = ROOT / 'crates/rrx/src/browser.rs'

def rust_test(name):
    return ['cargo', 'test', '--offline', '--locked', '--test', 'browser', name, '--', '--exact']

mutations = [
    ('routing', RUST, '(Backend::Auto, false) => Ok(Backend::Playwright)', '(Backend::Auto, false) => Ok(Backend::Stagehand)', rust_test('routing_never_spends_model_tokens_for_known_flows'), 'test result: FAILED'),
    ('uncertain_replay', RUST, '&& !result.effect_possible', '&& (result.effect_possible || !result.effect_possible)', rust_test('fallback_does_not_replay_failed_assertions_or_uncertain_actions'), 'test result: FAILED'),
    ('scope', RUST, 'self.task.scope() == *scope && self.project.id == self.task.project_id', 'self.task.scope().project_id == self.project.id && self.project.id == self.task.project_id', rust_test('scope_and_policy_hold_prevent_process_launch'), 'test result: FAILED'),
    ('artifact_symlink', RUST, 'fs::symlink_metadata(&path)', 'fs::metadata(&path)', rust_test('artifact_ancestor_symlinks_cannot_cross_projects'), 'test result: FAILED'),
    ('scoped_provider_auth', RUST, '(!binding.other_project_environment.contains(key)', '(true || !binding.other_project_environment.contains(key)', rust_test('provider_baseline_does_not_leak_another_projects_scoped_api_key'), 'test result: FAILED'),
    ('output_limit', RUST, 'bytes.len() > bound', 'bytes.len() > bound + 10000', rust_test('bounded_process_failure_is_normalized_and_descendants_are_owned'), 'test result: FAILED'),
    ('deadline', RUST, 'Instant::now() >= deadline', 'Instant::now() >= deadline + Duration::from_secs(3)', rust_test('bounded_process_failure_is_normalized_and_descendants_are_owned'), 'test result: FAILED'),
    ('native_group', ROOT / 'scripts/browser/io.mjs', 'detached: ownGroup', 'detached: true', ['node', '--test', 'scripts/browser/native-claude.test.mjs'], 'not ok'),
    ('strict_mcp', ROOT / 'scripts/browser/native-claude.mjs', '{"mcpServers":{}}', '{"mcpServers":{"unexpected":{}}}', ['node', '--test', 'scripts/browser/native-claude.test.mjs'], 'not ok'),
    ('cdp_origin', ROOT / 'scripts/browser/policy.mjs', '`--remote-allow-origins=${extensionOrigin}`', '`--remote-allow-origins=*`', ['node', '--test', 'scripts/browser/policy.test.mjs'], 'not ok'),
    ('adaptive_sdk_optional', ROOT / 'scripts/browser/bridge.mjs', 'catch { return null; }', "catch { throw new Error('unavailable'); }", ['node', '--test', 'scripts/browser/availability.test.mjs'], 'not ok'),
    ('native_group_signal', RUST, 'match kill_process_group(pid, signal)', 'match Ok::<(), rustix::io::Errno>(())', rust_test('bounded_process_failure_is_normalized_and_descendants_are_owned'), 'test result: FAILED'),
    ('fallback_profile', RUST, 'profiles[0].cleanup().is_err()', 'false', rust_test('fallback_clears_primary_profile_and_retains_all_attempt_telemetry'), 'test result: FAILED'),
    ('fallback_usage', RUST, 'merge_usage(&mut result.usage, &primary.usage);', 'let _ = &primary.usage;', rust_test('fallback_clears_primary_profile_and_retains_all_attempt_telemetry'), 'test result: FAILED'),
    ('escaped_pipe_bound', RUST, 'let io_deadline = Instant::now() + Duration::from_millis(500)', 'let io_deadline = Instant::now() + Duration::from_secs(5)', rust_test('escaped_pipe_holder_cannot_wedge_the_bounded_supervisor'), 'test result: FAILED'),
    ('primary_artifact', RUST, 'validate_artifacts(&mut result, &artifacts, request);', 'let _ = &artifacts;', rust_test('fallback_authorities_and_primary_evidence_fail_closed'), 'test result: FAILED'),
    ('fallback_cookie_final', RUST, 'profiles.push(PrivateProfile(fallback_directory.join("profile")));', 'let _ = &fallback_directory;', rust_test('fallback_clears_primary_profile_and_retains_all_attempt_telemetry'), 'test result: FAILED'),
    ('timeout_metadata', RUST, '.and_then(|bytes| serde_json::from_slice::<VerificationResult>(bytes).ok())', '.and_then(|_| None::<VerificationResult>)', rust_test('timeout_retains_a_verified_graceful_terminal_result'), 'test result: FAILED'),
    ('native_inner_deadline', ROOT / 'scripts/browser/native-claude.mjs', 'input.timeout_ms - 1500', 'input.timeout_ms + 1500', ['node', '--test', 'scripts/browser/native-claude.test.mjs'], 'not ok'),
    ('utf8_text', ROOT / 'scripts/browser/policy.mjs', '<= 8192', '<= 32768', ['node', '--test', 'scripts/browser/network.test.mjs'], 'not ok'),
    ('redirect_hops', ROOT / 'scripts/browser/network.mjs', 'maxRedirects: 0', 'maxRedirects: 5', ['node', '--test', 'scripts/browser/network.test.mjs'], 'not ok'),
    ('worker_pause', ROOT / 'scripts/browser/network.mjs', 'waitForDebuggerOnStart: true', 'waitForDebuggerOnStart: false', ['node', '--test', 'scripts/browser/network.test.mjs'], 'not ok'),
    ('native_pipe_drain', ROOT / 'scripts/browser/io.mjs', 'finish(code); }, 100)', 'finish(code); }, 5000)', ['node', '--test', 'scripts/browser/native-claude.test.mjs'], 'not ok'),

]

subprocess.run(['git', 'diff', '--exit-code', '--', *sorted({str(m[1].relative_to(ROOT)) for m in mutations})], cwd=ROOT, check=True, capture_output=True)
results = []
for name, file, original, changed, command, expected in mutations:
    content = file.read_text()
    assert original in content, f'mutation target disappeared: {name}'
    try:
        file.write_text(content.replace(original, changed))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=60)
        output = result.stdout + result.stderr
        assert result.returncode != 0 and expected in output, f'mutation survived or did not reach assertions: {name}\n{output}'
        results.append({'mutation': name, 'killed': True})
        print(f'killed: {name}', flush=True)
    finally:
        file.write_text(content)

subprocess.run(['cargo', 'test', '--offline', '--locked', '--test', 'browser'], cwd=ROOT, check=True, capture_output=True)
subprocess.run(['node', '--test', *sorted(str(p.relative_to(ROOT)) for p in (ROOT / 'scripts/browser').glob('*.test.mjs'))], cwd=ROOT, check=True, capture_output=True)
subprocess.run(['git', 'diff', '--exit-code', '--', *sorted({str(m[1].relative_to(ROOT)) for m in mutations})], cwd=ROOT, check=True, capture_output=True)
print(json.dumps({'mutations': results, 'restored_tests': 'passed'}))
