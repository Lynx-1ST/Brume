import { appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

// One explicit release authorization. Do not turn this into auto-merge for all PRs.
export const approval = Object.freeze({
  repository: 'Lynx-1ST/Serein', pr: 10,
  head: 'c9257897b3dd6dec470624b713d9a0b4f6326fde',
});
const requiredJobs = ['verified-source', 'preflight', 'native (windows-latest)',
  'fork-automation', 'extension-sdk', 'security', 'licenses', 'fuzz'];

export async function publishApprovedPr(api, { now = Date.now() } = {}) {
  const root = `/repos/${approval.repository}`;
  const pr = await api(`${root}/pulls/${approval.pr}`);
  if (pr.head?.sha !== approval.head || pr.head.repo?.full_name !== approval.repository
      || pr.base?.ref !== 'main' || pr.base.repo?.full_name !== approval.repository
      || pr.draft) return 'Not the approved PR source; no action.';
  let sha = pr.merge_commit_sha;
  if (!pr.merged) {
    if (pr.state !== 'open') return 'PR closed without merging; no action.';
    const workflow = await api(`${root}/actions/workflows/ci.yml`);
    const runs = await api(`${root}/actions/workflows/ci.yml/runs?head_sha=${approval.head}&event=pull_request&per_page=100`);
    const run = runs.workflow_runs.filter(run => run.head_sha === approval.head
      && run.workflow_id === workflow.id && run.path === '.github/workflows/ci.yml'
      && run.event === 'pull_request' && run.repository?.full_name === approval.repository
      && run.head_repository?.full_name === approval.repository)
      .sort((a, b) => b.id - a.id)[0];
    const age = now - Date.parse(run?.updated_at);
    if (run?.status !== 'completed' || run.conclusion !== 'success'
        || !Number.isFinite(age) || age < -300000 || age > 86400000)
      return 'Waiting for successful CI of the approved commit; no merge or release.';
    const { jobs } = await api(`${root}/actions/runs/${run.id}/jobs?filter=latest&per_page=100`);
    if (!requiredJobs.every(name => jobs.some(job => job.name === name
        && job.status === 'completed' && job.conclusion === 'success')))
      return 'Required CI jobs are incomplete; no merge or release.';
    const merged = await api(`${root}/pulls/${approval.pr}/merge`, 'PUT', {
      sha: approval.head, merge_method: 'merge',
      commit_title: 'Merge PR #10: contextual names, Vietnamese and stream PiP polish',
    });
    if (!merged.merged) throw new Error('GitHub did not merge the approved PR.');
    sha = merged.sha;
  }
  if (!/^[a-f0-9]{40}$/.test(sha ?? '')) throw new Error('Invalid merge commit.');
  const main = await api(`${root}/git/ref/heads/main`);
  if (main.object?.sha !== sha) return 'Main moved after merging; not dispatching an outdated release.';
  // GITHUB_TOKEN merges do not trigger push workflows. Dispatch CI explicitly;
  // the existing Windows Release workflow publishes only after main CI succeeds.
  const { workflow_runs: runs } = await api(`${root}/actions/workflows/ci.yml/runs?head_sha=${sha}&per_page=100`);
  if (runs.some(run => run.head_sha === sha && ['push', 'workflow_dispatch'].includes(run.event)))
    return 'Main CI already requested; Windows Release follows successful CI.';
  await api(`${root}/actions/workflows/ci.yml/dispatches`, 'POST', { ref: 'main' });
  return `Merged approved PR #${approval.pr} and requested main CI for ${sha}. Windows Release follows successful CI.`;
}

async function main() {
  if (process.env.GITHUB_REPOSITORY !== approval.repository) throw new Error('Unexpected repository.');
  const api = async (path, method = 'GET', body) => {
    const response = await fetch(`https://api.github.com${path}`, {
      method,
      headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`, Accept: 'application/vnd.github+json',
        'Content-Type': 'application/json', 'X-GitHub-Api-Version': '2022-11-28' },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: AbortSignal.timeout(30000),
    });
    if (!response.ok) throw new Error(`GitHub HTTP ${response.status}`);
    if (response.status === 204) return null;
    const text = await response.text();
    if (Buffer.byteLength(text) > 4 * 1024 * 1024) throw new Error('Oversized metadata response.');
    return JSON.parse(text);
  };
  const result = await publishApprovedPr(api);
  console.log(result);
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${result}\n`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
