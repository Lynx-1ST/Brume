import assert from 'node:assert/strict';
import test from 'node:test';
import { releaseSource } from '../.github/scripts/release-source.mjs';

const repo = 'Lynx-1ST/Serein', head = 'a'.repeat(40), merge = 'b'.repeat(40);
const now = Date.parse('2026-11-19T12:00:00Z');
function fixture() {
  const run = { id: 42, workflow_id: 9, path: '.github/workflows/ci.yml',
    repository: { full_name: repo }, head_repository: { full_name: repo },
    head_sha: head, status: 'completed', conclusion: 'success', event: 'pull_request',
    updated_at: new Date(now).toISOString() };
  const pr = { number: 11, state: 'open', draft: false, merged: false,
    user: { login: 'Lynx-1ST' }, head: { sha: head, repo: { full_name: repo } },
    base: { ref: 'main', repo: { full_name: repo } } };
  const jobs = ['verified-source', 'preflight', 'native (windows-latest)',
    'fork-automation', 'extension-sdk', 'security', 'licenses', 'fuzz']
    .map(name => ({ name, status: 'completed', conclusion: 'success' }));
  const state = { run, pr, jobs, latest: run, main: merge, differentTree: false,
    releases: [], existing: [], writes: [] };
  const api = async (path, method = 'GET', body) => {
    if (method !== 'GET') {
      state.writes.push({ path, method, body });
      if (method === 'PUT') return { merged: true, sha: merge };
      return null;
    }
    if (path.endsWith('/workflows/ci.yml')) return { id: 9 };
    if (path.includes('/runs?') && /event=(pull_request|workflow_dispatch)/.test(path)) return { workflow_runs: [state.latest] };
    if (path.includes('/jobs?')) return { jobs: state.jobs };
    if (path.endsWith('/pulls?per_page=100')) return [state.pr];
    if (path.endsWith('/pulls/11')) return state.pr;
    if (path.endsWith('/git/ref/heads/main')) return { object: { sha: state.main } };
    if (path.includes('/git/commits/')) return { tree: { sha:
      (state.differentTree && path.endsWith(merge) ? 'd' : 'c').repeat(40) } };
    if (path.includes('/runs?')) return { workflow_runs: state.existing };
    if (path.endsWith('/releases?per_page=100')) return state.releases;
    throw new Error(`Unexpected request: ${path}`);
  };
  return { state, check: () => releaseSource(api, { repo, eventName: 'workflow_run', run, now }) };
}

test('checked owner PR merges atomically and releases the identical merged tree', async () => {
  const { state, check } = fixture();
  assert.deepEqual(await check(), { ready: true, verified: true, sha: merge,
    note: `Build and publish verified main commit ${merge}.` });
  assert.deepEqual(state.writes, [{ path: `/repos/${repo}/pulls/11/merge`, method: 'PUT',
    body: { sha: head, merge_method: 'merge', commit_title: 'Merge PR #11 after successful CI' } }]);
});

test('failed, stale, superseded, skipped and foreign PR evidence never authorizes a merge', async () => {
  const mutations = [
    s => { s.run.conclusion = 'failure'; },
    s => { s.run.updated_at = '2026-11-17T12:00:00Z'; },
    s => { s.latest = { ...s.run, id: 43, status: 'in_progress' }; },
    s => { s.jobs.find(j => j.name === 'security').conclusion = 'skipped'; },
    s => { s.run.head_repository.full_name = 'someone/Serein'; },
    s => { s.pr.head.sha = 'e'.repeat(40); },
    s => { s.pr.user.login = 'someone'; },
    s => { s.pr.draft = true; },
    s => { s.pr.state = 'closed'; },
    s => { s.run.workflow_id = 10; },
  ];
  for (const change of mutations) {
    const { state, check } = fixture();
    change(state);
    assert.equal((await check()).ready, false);
    assert.deepEqual(state.writes, []);
  }
});

test('independent main changes require merged-source CI instead of immediate publication', async () => {
  const { state, check } = fixture();
  state.differentTree = true;
  assert.equal((await check()).ready, false);
  assert.equal(state.writes.length, 2);
  assert.deepEqual(state.writes[1], { path: `/repos/${repo}/actions/workflows/ci.yml/dispatches`,
    method: 'POST', body: { ref: 'main' } });
  state.pr.merged = true;
  state.pr.merge_commit_sha = merge;
  state.existing = [{ head_sha: merge, event: 'workflow_dispatch' }];
  state.writes.length = 0;
  assert.equal((await check()).ready, false);
  assert.deepEqual(state.writes, []);
});

test('rerun cannot merge twice or publish stale/already-published main', async () => {
  const { state, check } = fixture();
  state.pr.merged = true;
  state.pr.merge_commit_sha = merge;
  state.releases = [{ draft: false, target_commitish: merge }];
  assert.equal((await check()).ready, false);
  state.releases = [];
  state.main = 'f'.repeat(40);
  assert.equal((await check()).ready, false);
  assert.deepEqual(state.writes, []);
});

test('successful main CI releases without merging; foreign events and stale main stop', async () => {
  const { state, check } = fixture();
  state.run.event = 'workflow_dispatch';
  state.run.head_branch = 'main';
  state.main = head;
  assert.equal((await check()).sha, head);
  state.main = merge;
  assert.equal((await check()).ready, false);
  state.run.event = 'schedule';
  assert.equal((await check()).ready, false);
  assert.deepEqual(state.writes, []);
});

test('queued branch workflow_dispatch must meet the same exact-head owner PR checks', async () => {
  const { state, check } = fixture();
  state.run.event = 'workflow_dispatch';
  state.run.head_branch = 'fix/account-profile-decoration';
  assert.equal((await check()).sha, merge);
  assert.equal(state.writes[0].body.sha, head);
  state.pr.head.sha = 'f'.repeat(40);
  state.writes.length = 0;
  assert.equal((await check()).ready, false);
  assert.deepEqual(state.writes, []);
});

test('manual main and version tags still require full build validation', async () => {
  const api = () => { throw new Error('Manual source admission needs no API'); };
  for (const ref of ['refs/heads/main', 'refs/tags/v0.1.1']) {
    const result = await releaseSource(api, { repo, eventName: 'workflow_dispatch', ref, sha: head });
    assert.equal(result.ready, true);
    assert.equal(result.verified, false);
  }
  assert.equal((await releaseSource(api, { repo, eventName: 'workflow_dispatch',
    ref: 'refs/heads/feature', sha: head })).ready, false);
});
