import { appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const repository = 'Lynx-1ST/Serein';
const shaPattern = /^[a-f0-9]{40}$/;
const requiredJobs = ['verified-source', 'preflight', 'native (windows-latest)', 'fork-automation', 'extension-sdk', 'security', 'licenses', 'fuzz'];

// Only main pushes from an actual merged PR may reuse checks. PR jobs explicitly
// check out the head SHA, so equality of full Git trees proves identical inputs.
export async function reusableCi(api, { eventName, ref, sha, runId, now = Date.now() }) {
  if (eventName !== 'push' || ref !== 'refs/heads/main' || !shaPattern.test(sha ?? '')) return null;
  const root = `/repos/${repository}`;
  const [commit, pulls, workflow] = await Promise.all([
    api(`${root}/git/commits/${sha}`),
    api(`${root}/commits/${sha}/pulls?per_page=100`),
    api(`${root}/actions/workflows/ci.yml`),
  ]);
  if (!shaPattern.test(commit.tree?.sha ?? '') || !Number.isSafeInteger(workflow.id)) return null;
  for (const pr of pulls) {
    if (!pr.merged_at || pr.merge_commit_sha !== sha || pr.base?.ref !== 'main'
      || pr.base.repo?.full_name !== repository || pr.head?.repo?.full_name !== repository
      || !shaPattern.test(pr.head.sha ?? '')) continue;
    const head = await api(`${root}/git/commits/${pr.head.sha}`);
    if (head.tree?.sha !== commit.tree.sha) continue;
    const { workflow_runs: runs } = await api(`${root}/actions/workflows/ci.yml/runs?head_sha=${pr.head.sha}&event=pull_request&per_page=100`);
    for (const run of runs) {
      const age = now - Date.parse(run.updated_at);
      if (String(run.id) === String(runId) || run.workflow_id !== workflow.id
        || run.path !== '.github/workflows/ci.yml' || run.event !== 'pull_request'
        || run.head_sha !== pr.head.sha || run.repository?.full_name !== repository
        || run.head_repository?.full_name !== repository
        || run.status !== 'completed' || run.conclusion !== 'success'
        || !Number.isFinite(age) || age < -300000 || age > 24 * 60 * 60 * 1000) continue;
      const { jobs } = await api(`${root}/actions/runs/${run.id}/jobs?filter=latest&per_page=100`);
      if (!requiredJobs.every(name => jobs.some(job => job.name === name
        && job.status === 'completed' && job.conclusion === 'success'))) continue;
      return { pr: pr.number, run: run.id, tree: commit.tree.sha };
    }
  }
  return null;
}

async function main() {
  let result = null;
  try {
    if (process.env.GITHUB_REPOSITORY !== repository) throw new Error('Unexpected repository');
    const api = async path => {
      const response = await fetch(`https://api.github.com${path}`, {
        headers: { Authorization: `Bearer ${process.env.GITHUB_TOKEN}`, Accept: 'application/vnd.github+json' },
        signal: AbortSignal.timeout(20000),
      });
      if (!response.ok) throw new Error(`GitHub HTTP ${response.status}`);
      const text = await response.text();
      if (Buffer.byteLength(text) > 4 * 1024 * 1024) throw new Error('Oversized response');
      return JSON.parse(text);
    };
    result = await reusableCi(api, {
      eventName: process.env.GITHUB_EVENT_NAME, ref: process.env.GITHUB_REF,
      sha: process.env.GITHUB_SHA, runId: process.env.GITHUB_RUN_ID,
    });
  } catch {
    // API outages, unknown metadata or incomplete checks require a full CI run.
    console.log('Cannot establish reusable CI evidence; running all checks.');
  }
  appendFileSync(process.env.GITHUB_OUTPUT, `reuse=${result !== null}\n`);
  if (result) appendFileSync(process.env.GITHUB_STEP_SUMMARY,
    `Reused successful PR #${result.pr} CI run ${result.run}. Full source tree: ${result.tree}. Windows Release still builds the current main SHA.\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
