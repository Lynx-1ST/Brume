import { appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const repository = 'Lynx-1ST/Serein';
const root = `/repos/${repository}`;
const validSha = value => /^[a-f0-9]{40}$/.test(value ?? '');
const owned = pr => pr.user?.login?.toLowerCase() === 'lynx-1st'
  && pr.head?.repo?.full_name === repository && pr.base?.repo?.full_name === repository;
const stopped = note => ({ advanced: false, note });

// A one-off delivery queue explicitly requested by the owner: release PR 12, then check PR 13.
export async function advanceQueue(api, { repo, source, tag }) {
  if (repo !== repository || !validSha(source) || !/^v\d+\.\d+\.\d+(?:-nightly\.\d+\.\d+)?$/.test(tag ?? ''))
    return stopped('Invalid release queue source.');
  const previous = await api(`${root}/pulls/12`);
  if (!owned(previous) || !previous.merged || previous.base.ref !== 'main'
      || previous.merge_commit_sha !== source)
    return stopped('This release is not the merged PR #12 source.');
  const releases = await api(`${root}/releases?per_page=100`);
  const release = releases.find(item => item.tag_name === tag && !item.draft
    && item.published_at && item.target_commitish === source);
  const prefix = `serein-${tag}-Windows-X64`;
  const assets = [`${prefix}-Setup.exe`, `${prefix}.exe`, `${prefix}.zip`, 'SHA256SUMS.txt'];
  if (!release || !assets.every(name => release.assets?.some(asset =>
    asset.name === name && asset.state === 'uploaded' && asset.size > 0)))
    return stopped('PR #12 has no complete published Windows release yet.');
  const main = await api(`${root}/git/ref/heads/main`);
  if (main.object?.sha !== source) return stopped('Main moved after the PR #12 release.');
  const next = await api(`${root}/pulls/13`);
  if (!owned(next) || next.state !== 'open' || next.merged || next.draft
      || next.head.ref !== 'fix/account-profile-decoration' || !validSha(next.head.sha)
      || !['main', 'feat/windows-chat-stream-polish'].includes(next.base.ref))
    return stopped('PR #13 is no longer the queued owner decoration PR.');
  // Normal merge only: never force, reset, or overwrite either branch on a conflict.
  await api(`${root}/merges`, 'POST', { base: next.head.ref, head: source,
    commit_message: 'Merge released PR #12 into the decoration PR' });
  if (next.base.ref !== 'main') await api(`${root}/pulls/13`, 'PATCH', { base: 'main' });
  const branch = await api(`${root}/git/ref/heads/${next.head.ref}`);
  const sha = branch.object?.sha;
  if (!validSha(sha)) throw new Error('Missing updated decoration source.');
  const { workflow_runs: runs } = await api(`${root}/actions/workflows/ci.yml/runs?head_sha=${sha}&event=workflow_dispatch&per_page=100`);
  if (!runs.some(run => run.head_sha === sha && run.event === 'workflow_dispatch'))
    await api(`${root}/actions/workflows/ci.yml/dispatches`, 'POST', { ref: next.head.ref });
  return { advanced: true, sha, note: `PR #12 Windows release is published; PR #13 CI requested at ${sha}.` };
}

async function main() {
  const api = async (path, method = 'GET', body) => {
    const response = await fetch(`https://api.github.com${path}`, {
      method, headers: { Authorization: `Bearer ${process.env.GH_TOKEN}`,
        Accept: 'application/vnd.github+json', 'Content-Type': 'application/json',
        'X-GitHub-Api-Version': '2022-11-28' },
      body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(30000),
    });
    if (!response.ok) throw new Error(`GitHub HTTP ${response.status}; release queue was not forced.`);
    if (response.status === 204) return null;
    const text = await response.text();
    if (Buffer.byteLength(text) > 4 * 1024 * 1024) throw new Error('Oversized GitHub metadata.');
    return JSON.parse(text);
  };
  const result = await advanceQueue(api, { repo: process.env.GITHUB_REPOSITORY,
    source: process.env.SOURCE_SHA, tag: process.env.RELEASE_TAG });
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${result.note}\n`);
  console.log(result.note);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
