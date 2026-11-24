import { appendFileSync, readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const allowedRepositories = ['Lynx-1ST/Brume', 'Lynx-1ST/Serein'];
const requiredJobs = ['verified-source', 'preflight', 'native (windows-latest)',
  'fork-automation', 'extension-sdk', 'security', 'licenses', 'fuzz'];
const validSha = sha => /^[a-f0-9]{40}$/.test(sha ?? '');
const stopped = note => ({ ready: false, verified: false, note });

// Only trusted default-branch code runs here. Never execute a PR script with merge permissions.
export async function releaseSource(api, { repo, eventName, ref, sha, run, now = Date.now() }) {
  if (!allowedRepositories.includes(repo)) return stopped('Unexpected repository.');
  if (eventName !== 'workflow_run') {
    if (!validSha(sha) || !(ref === 'refs/heads/main' || ref?.startsWith('refs/tags/v')))
      return stopped('Manual releases require main or a version tag.');
    return { ready: true, verified: false, sha, note: 'Manual release: workspace validation is required.' };
  }
  const root = `/repos/${repo}`;
  const workflow = await api(`${root}/actions/workflows/ci.yml`);
  if (run?.workflow_id !== workflow.id || run.path !== '.github/workflows/ci.yml'
      || !allowedRepositories.includes(run.repository?.full_name) || !allowedRepositories.includes(run.head_repository?.full_name)
      || !validSha(run.head_sha) || run.status !== 'completed' || run.conclusion !== 'success')
    return stopped('No successful CI from this repository.');
  let source = run.head_sha;
  if (run.event === 'pull_request' || (run.event === 'workflow_dispatch' && run.head_branch !== 'main')) {
    const { workflow_runs: runs } = await api(`${root}/actions/workflows/ci.yml/runs?head_sha=${source}&event=${run.event}&per_page=100`);
    const latest = runs.filter(item => item.workflow_id === workflow.id && item.head_sha === source)
      .sort((a, b) => b.id - a.id)[0];
    if (latest?.id !== run.id || latest.status !== 'completed' || latest.conclusion !== 'success')
      return stopped('A newer CI run supersedes this result.');
    const { jobs } = await api(`${root}/actions/runs/${run.id}/jobs?filter=latest&per_page=100`);
    if (!requiredJobs.every(name => jobs.some(job => job.name === name
        && job.status === 'completed' && job.conclusion === 'success')))
      return stopped('Required PR checks are incomplete.');
    const age = now - Date.parse(run.updated_at);
    if (!Number.isFinite(age) || age < -300000 || age > 86400000)
      return stopped('PR checks are stale.');
    const pulls = await api(`${root}/commits/${source}/pulls?per_page=100`);
    const candidate = pulls.find(pr => pr.head?.sha === source && pr.base?.ref === 'main'
      && allowedRepositories.includes(pr.head.repo?.full_name) && allowedRepositories.includes(pr.base.repo?.full_name)
      && pr.user?.login?.toLowerCase() === 'lynx-1st' && !pr.draft);
    if (!candidate) return stopped('No owner-authored PR from this fork targets main.');
    const pr = await api(`${root}/pulls/${candidate.number}`);
    if (pr.head?.sha !== source || !allowedRepositories.includes(pr.head.repo?.full_name)
        || pr.base?.ref !== 'main' || !allowedRepositories.includes(pr.base.repo?.full_name)
        || pr.user?.login?.toLowerCase() !== 'lynx-1st' || pr.draft)
      return stopped('PR source or scope changed.');
    if (pr.merged) {
      source = pr.merge_commit_sha;
    } else {
      if (pr.state !== 'open') return stopped('PR was closed without merging.');
      const merged = await api(`${root}/pulls/${pr.number}/merge`, 'PUT', {
        sha: run.head_sha, merge_method: 'merge', commit_title: `Merge PR #${pr.number} after successful CI`,
      });
      if (!merged.merged) throw new Error('GitHub did not merge the checked PR.');
      source = merged.sha;
    }
    if (!validSha(source)) throw new Error('Invalid merge commit.');
    const main = await api(`${root}/git/ref/heads/main`);
    if (main.object?.sha !== source) return stopped('Main moved after the merge.');
    const [head, merged] = await Promise.all([
      api(`${root}/git/commits/${run.head_sha}`), api(`${root}/git/commits/${source}`),
    ]);
    if (!validSha(head.tree?.sha) || !validSha(merged.tree?.sha)) throw new Error('Missing source tree evidence.');
    if (head.tree.sha !== merged.tree.sha) {
      // Main changed independently: validate the actual merged tree before publishing.
      // GITHUB_TOKEN merges do not start push workflows, so request CI explicitly.
      const { workflow_runs: existing } = await api(`${root}/actions/workflows/ci.yml/runs?head_sha=${source}&per_page=100`);
      if (!existing.some(item => item.head_sha === source && ['push', 'workflow_dispatch'].includes(item.event)))
        await api(`${root}/actions/workflows/ci.yml/dispatches`, 'POST', { ref: 'main' });
      return stopped('Merged tree differs from checked PR; main CI will trigger the release after success.');
    }
  } else if (!['push', 'workflow_dispatch'].includes(run.event) || run.head_branch !== 'main') {
    return stopped('CI does not belong to a release source.');
  }
  const main = await api(`${root}/git/ref/heads/main`);
  if (main.object?.sha !== source) return stopped('Not the current main commit.');
  const releases = await api(`${root}/releases?per_page=100`);
  if (releases.some(release => !release.draft && release.target_commitish === source))
    return stopped('This source already has a published release.');
  return { ready: true, verified: true, sha: source, note: `Build and publish verified main commit ${source}.` };
}

async function main() {
  const api = async (path, method = 'GET', body) => {
    const response = await fetch(`https://api.github.com${path}`, {
      method, headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`,
        Accept: 'application/vnd.github+json', 'Content-Type': 'application/json',
        'X-GitHub-Api-Version': '2022-11-28' },
      body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(30000),
    });
    if (!response.ok) throw new Error(`GitHub HTTP ${response.status}`);
    if (response.status === 204) return null;
    const text = await response.text();
    if (Buffer.byteLength(text) > 4 * 1024 * 1024) throw new Error('Oversized GitHub metadata.');
    return JSON.parse(text);
  };
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
  const run = process.env.GITHUB_EVENT_NAME === 'workflow_run'
    ? await api(`/repos/${repository}/actions/runs/${event.workflow_run.id}`) : undefined;
  const result = await releaseSource(api, { repo: process.env.GITHUB_REPOSITORY,
    eventName: process.env.GITHUB_EVENT_NAME, ref: process.env.GITHUB_REF,
    sha: process.env.GITHUB_SHA, run });
  appendFileSync(process.env.GITHUB_OUTPUT,
    `ready=${result.ready}\nverified=${result.verified}\nsha=${result.sha ?? ''}\n`);
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${result.note}\n`);
  console.log(result.note);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
