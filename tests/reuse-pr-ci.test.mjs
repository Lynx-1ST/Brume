import assert from 'node:assert/strict';
import test from 'node:test';
import { reusableCi } from '../.github/scripts/reuse-pr-ci.mjs';

const sha = 'a'.repeat(40), head = 'b'.repeat(40), tree = 'c'.repeat(40);
const repo = { full_name: 'Lynx-1ST/Serein' };
const context = { eventName: 'push', ref: 'refs/heads/main', sha, runId: 999, now: Date.parse('2026-11-19T01:00:00Z') };
function fixture({ differentTree = false, pr = {}, run = {}, jobs: suppliedJobs } = {}) {
  const jobs = ['verified-source', 'preflight', 'native (windows-latest)', 'fork-automation', 'extension-sdk', 'security', 'licenses', 'fuzz']
    .map(name => ({ name, status: 'completed', conclusion: 'success' }));
  return async path => {
    if (path.endsWith(`/git/commits/${sha}`)) return { tree: { sha: tree } };
    if (path.endsWith(`/git/commits/${head}`)) return { tree: { sha: differentTree ? 'd'.repeat(40) : tree } };
    if (path.includes('/pulls?')) return [{ number: 5, merged_at: '2026-11-19T00:00:00Z', merge_commit_sha: sha,
      base: { ref: 'main', repo }, head: { sha: head, repo }, ...pr }];
    if (path.endsWith('/workflows/ci.yml')) return { id: 42 };
    if (path.includes('/runs?')) return { workflow_runs: [{ id: 123, workflow_id: 42,
      path: '.github/workflows/ci.yml', event: 'pull_request', head_sha: head,
      repository: repo, head_repository: repo, status: 'completed', conclusion: 'success', updated_at: '2026-11-19T00:59:00Z', ...run }] };
    if (path.includes('/jobs?')) return { jobs: suppliedJobs ?? jobs };
    throw new Error(`Unexpected fixture request ${path}`);
  };
}

test('merged main tree can reuse complete successful same-repository PR checks', async () => {
  assert.deepEqual(await reusableCi(fixture(), context), { pr: 5, run: 123, tree });
});

test('direct pushes, changed merge trees, unmerged PRs and another merge cannot reuse', async () => {
  for (const options of [{ differentTree: true }, { pr: { merged_at: null } },
    { pr: { merge_commit_sha: 'e'.repeat(40) } }, { pr: { base: { ref: 'other', repo } } }]) {
    assert.equal(await reusableCi(fixture(options), context), null);
  }
  assert.equal(await reusableCi(async () => { throw Error('must not query'); }, { ...context, eventName: 'pull_request' }), null);
  assert.equal(await reusableCi(fixture(), { ...context, ref: 'refs/heads/feature' }), null);
});

test('foreign code, wrong workflow, stale head and unfinished/failed runs cannot authorize reuse', async () => {
  for (const run of [{ head_repository: { full_name: 'other/fork' } }, { repository: { full_name: 'other/repo' } },
    { workflow_id: 8 }, { path: '.github/workflows/other.yml' }, { head_sha: sha },
    { event: 'workflow_dispatch' }, { conclusion: 'failure' }, { status: 'in_progress' }, { id: 999 },
    { updated_at: '2026-11-16T00:00:00Z' }, { updated_at: 'garbage' }, { updated_at: '2026-11-20T00:00:00Z' }]) {
    assert.equal(await reusableCi(fixture({ run }), context), null);
  }
  assert.equal(await reusableCi(fixture({ pr: { head: { sha: head, repo: { full_name: 'other/fork' } } } }), context), null);
});

test('missing, failed or skipped required jobs force a full run', async () => {
  for (const conclusion of ['skipped', 'failure', 'cancelled']) {
    const jobs = ['verified-source', 'preflight', 'native (windows-latest)', 'fork-automation', 'extension-sdk', 'security', 'licenses', 'fuzz']
      .map(name => ({ name, status: 'completed', conclusion: name === 'native (windows-latest)' ? conclusion : 'success' }));
    assert.equal(await reusableCi(fixture({ jobs }), context), null);
  }
  assert.equal(await reusableCi(fixture({ jobs: [] }), context), null);
});
