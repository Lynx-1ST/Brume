import assert from 'node:assert/strict';
import test from 'node:test';
import { advanceQueue } from '../.github/scripts/advance-release-queue.mjs';

const repo = 'Lynx-1ST/Serein', source = 'a'.repeat(40), nextSha = 'b'.repeat(40);
const root = `/repos/${repo}`, tag = 'v0.1.0-nightly.20261119.20';
function fixture() {
  const scoped = { user: { login: 'Lynx-1ST' },
    head: { sha: nextSha, ref: 'fix/account-profile-decoration', repo: { full_name: repo } },
    base: { ref: 'main', repo: { full_name: repo } } };
  const state = {
    previous: { ...structuredClone(scoped), merged: true, merge_commit_sha: source },
    next: { ...structuredClone(scoped), state: 'open', draft: false, merged: false },
    main: source, writes: [], existing: [], conflict: false,
    releases: [{ tag_name: tag, target_commitish: source, draft: false, published_at: '2026-11-19',
      assets: [`serein-${tag}-Windows-X64-Setup.exe`, `serein-${tag}-Windows-X64.exe`,
        `serein-${tag}-Windows-X64.zip`, 'SHA256SUMS.txt']
        .map(name => ({ name, state: 'uploaded', size: 10 })) }],
  };
  state.next.base.ref = 'feat/windows-chat-stream-polish';
  const api = async (path, method = 'GET', body) => {
    if (method !== 'GET') {
      state.writes.push({ path, method, body });
      if (path.endsWith('/merges') && state.conflict) throw new Error('GitHub HTTP 409');
      return null;
    }
    if (path.endsWith('/pulls/12')) return state.previous;
    if (path.endsWith('/pulls/13')) return state.next;
    if (path.endsWith('/releases?per_page=100')) return state.releases;
    if (path.endsWith('/git/ref/heads/main')) return { object: { sha: state.main } };
    if (path.endsWith('/git/ref/heads/fix/account-profile-decoration')) return { object: { sha: nextSha } };
    if (path.includes('/runs?')) return { workflow_runs: state.existing };
    throw new Error(`Unexpected request: ${path}`);
  };
  return { state, check: () => advanceQueue(api, { repo, source, tag }) };
}

test('only a complete published PR 12 release merges main into PR 13, retargets and starts CI', async () => {
  const { state, check } = fixture();
  assert.equal((await check()).advanced, true);
  assert.deepEqual(state.writes, [
    { path: `${root}/merges`, method: 'POST', body: { base: 'fix/account-profile-decoration',
      head: source, commit_message: 'Merge released PR #12 into the decoration PR' } },
    { path: `${root}/pulls/13`, method: 'PATCH', body: { base: 'main' } },
    { path: `${root}/actions/workflows/ci.yml/dispatches`, method: 'POST', body: { ref: 'fix/account-profile-decoration' } },
  ]);
});

test('missing, draft, incomplete and wrong-source releases cannot start the next PR', async () => {
  for (const mutate of [s => { s.releases = []; }, s => { s.releases[0].draft = true; },
    s => { s.releases[0].assets.pop(); }, s => { s.previous.merge_commit_sha = nextSha; },
    s => { s.main = nextSha; }, s => { s.next.head.repo.full_name = 'stranger/Serein'; }]) {
    const { state, check } = fixture();
    mutate(state);
    assert.equal((await check()).advanced, false);
    assert.deepEqual(state.writes, []);
  }
});

test('branch conflicts stop without retargeting, dispatching or force fallback', async () => {
  const { state, check } = fixture();
  state.conflict = true;
  await assert.rejects(check, /409/);
  assert.equal(state.writes.length, 1);
  assert.equal(state.writes[0].path, `${root}/merges`);
});

test('recovery after a completed retarget never dispatches a second CI for the same head', async () => {
  const { state, check } = fixture();
  state.next.base.ref = 'main';
  state.existing = [{ head_sha: nextSha, event: 'workflow_dispatch' }];
  assert.equal((await check()).advanced, true);
  assert.equal(state.writes.length, 1); // Idempotent Git merge: GitHub returns 204 if already contained.
});
