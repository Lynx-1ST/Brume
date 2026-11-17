import assert from 'node:assert/strict';
import test from 'node:test';
import { GitHubError, syncUpstream } from '../.github/scripts/sync-upstream.mjs';

const before = 'a'.repeat(40), source = 'b'.repeat(40), merged = 'c'.repeat(40);

function mock({ ahead = 0, after = before, checked = true, conflict = false } = {}) {
  const writes = [];
  let branchReads = 0;
  return { writes, api: async (method, path, body) => {
    if (method === 'POST') {
      writes.push({ path, body });
      if (path.endsWith('/merge-upstream') && conflict) throw new GitHubError(409, 'Merge conflict');
      return null;
    }
    if (path === '/repos/Lynx-1ST/Serein') return { fork: true, parent: { full_name: 'ViceVerse-cz/Serein' } };
    if (path === '/repos/Lynx-1ST/Serein/branches/main') return { commit: { sha: branchReads++ === 0 ? before : after } };
    if (path === '/repos/ViceVerse-cz/Serein/branches/main') return { commit: { sha: source } };
    if (path.includes('/compare/')) return { ahead_by: ahead };
    if (path.includes('/actions/workflows/')) return { workflow_runs: checked ? [
      { head_sha: after, head_branch: 'main', event: 'workflow_dispatch' },
    ] : [] };
    throw new Error(`Unexpected request: ${method} ${path}`);
  } };
}

test('unchanged checked fork never writes or creates another build', async () => {
  const { api, writes } = mock();
  assert.deepEqual(await syncUpstream(api), { changed: false, sha: before, ciDispatched: false });
  assert.deepEqual(writes, []);
});

test('clean upstream merge explicitly dispatches CI without force or reset', async () => {
  const { api, writes } = mock({ ahead: 3, after: merged, checked: false });
  assert.deepEqual(await syncUpstream(api), { changed: true, sha: merged, ciDispatched: true });
  assert.deepEqual(writes, [
    { path: '/repos/Lynx-1ST/Serein/merge-upstream', body: { branch: 'main' } },
    { path: '/repos/Lynx-1ST/Serein/actions/workflows/ci.yml/dispatches', body: { ref: 'main' } },
  ]);
});

test('conflicting upstream merge stops before CI and never falls back to force', async () => {
  const { api, writes } = mock({ ahead: 1, conflict: true });
  await assert.rejects(syncUpstream(api), /conflicts with fork changes/);
  assert.deepEqual(writes, [{ path: '/repos/Lynx-1ST/Serein/merge-upstream', body: { branch: 'main' } }]);
});

test('next scheduled run recovers a missing CI dispatch after a previous merge', async () => {
  const { api, writes } = mock({ checked: false });
  assert.equal((await syncUpstream(api)).ciDispatched, true);
  assert.deepEqual(writes, [{ path: '/repos/Lynx-1ST/Serein/actions/workflows/ci.yml/dispatches', body: { ref: 'main' } }]);
});
