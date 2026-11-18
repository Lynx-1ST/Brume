import { test } from 'node:test';
import assert from 'node:assert/strict';
import { approval, publishApprovedPr } from '../.github/scripts/publish-approved-pr.mjs';

function fixture(change = {}) {
  const sha = 'a'.repeat(40), now = Date.now(), writes = [];
  const pr = { state: 'open', merged: false, draft: false,
    head: { sha: approval.head, repo: { full_name: approval.repository } },
    base: { ref: 'main', repo: { full_name: approval.repository } } };
  const run = { id: 1, workflow_id: 2, path: '.github/workflows/ci.yml',
    event: 'pull_request', head_sha: approval.head, repository: { full_name: approval.repository },
    head_repository: { full_name: approval.repository }, status: 'completed', conclusion: 'success',
    updated_at: new Date(now).toISOString() };
  const jobs = ['verified-source', 'preflight', 'native (windows-latest)', 'fork-automation',
    'extension-sdk', 'security', 'licenses', 'fuzz'].map(name => ({ name, status: 'completed', conclusion: 'success' }));
  const state = { pr, run, jobs, main: sha, dispatched: false, ...change };
  const api = async (path, method = 'GET', body) => {
    if (method !== 'GET') {
      writes.push({ path, method, body });
      if (method === 'PUT') { assert.equal(body.sha, approval.head); return { merged: true, sha }; }
      return null;
    }
    if (path.endsWith(`/pulls/${approval.pr}`)) return state.pr;
    if (path.endsWith('/workflows/ci.yml')) return { id: 2 };
    if (path.includes('/jobs?')) return { jobs: state.jobs };
    if (path.endsWith('/git/ref/heads/main')) return { object: { sha: state.main } };
    if (path.includes('event=pull_request')) return { workflow_runs: [state.run] };
    if (path.includes('/runs?head_sha=')) return { workflow_runs: state.dispatched ? [{ head_sha: sha, event: 'workflow_dispatch' }] : [] };
    throw new Error(`Unexpected fixture read ${path}`);
  };
  return { api, state, writes, now, sha };
}

test('successful exact-source CI merges then dispatches main CI', async () => {
  const f = fixture();
  await publishApprovedPr(f.api, { now: f.now });
  assert.deepEqual(f.writes.map(write => write.method), ['PUT', 'POST']);
  assert.equal(f.writes[1].body.ref, 'main');
});
test('changed source, fork, failed/stale CI or skipped native job cannot merge', async () => {
  for (const mutate of [
    f => { f.state.pr.head.sha = 'b'.repeat(40); },
    f => { f.state.pr.head.repo.full_name = 'someone/else'; },
    f => { f.state.run.conclusion = 'failure'; },
    f => { f.state.run.updated_at = new Date(f.now - 86400001).toISOString(); },
    f => { f.state.jobs[2].conclusion = 'skipped'; },
  ]) {
    const f = fixture(); mutate(f);
    await publishApprovedPr(f.api, { now: f.now });
    assert.deepEqual(f.writes, []);
  }
});
test('retry after merge dispatches missing CI once and never repeats existing CI', async () => {
  const f = fixture(); f.state.pr.merged = true; f.state.pr.merge_commit_sha = f.sha;
  await publishApprovedPr(f.api, { now: f.now });
  assert.deepEqual(f.writes.map(write => write.method), ['POST']);
  f.writes.length = 0; f.state.dispatched = true;
  await publishApprovedPr(f.api, { now: f.now });
  assert.deepEqual(f.writes, []);
  f.state.dispatched = false; f.state.main = 'c'.repeat(40);
  await publishApprovedPr(f.api, { now: f.now });
  assert.deepEqual(f.writes, []);
});
