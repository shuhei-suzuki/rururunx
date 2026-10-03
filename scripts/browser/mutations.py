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
subprocess.run(['node', '--test', 'scripts/browser/native-claude.test.mjs', 'scripts/browser/policy.test.mjs'], cwd=ROOT, check=True, capture_output=True)
subprocess.run(['git', 'diff', '--exit-code', '--', *sorted({str(m[1].relative_to(ROOT)) for m in mutations})], cwd=ROOT, check=True, capture_output=True)
print(json.dumps({'mutations': results, 'restored_tests': 'passed'}))
