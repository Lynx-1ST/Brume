import { appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const repository = 'Lynx-1ST/Serein';
const upstream = 'ViceVerse-cz/Serein';
const branch = 'main';

export class GitHubError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

function commitSha(value) {
  if (!/^[a-f0-9]{40}$/.test(value ?? '')) throw new Error('Invalid GitHub commit SHA');
  return value;
}

export async function syncUpstream(api) {
  const metadata = await api('GET', `/repos/${repository}`);
  if (!metadata.fork || metadata.parent?.full_name !== upstream) {
    throw new Error(`Expected a direct fork of ${upstream}; refusing to sync another source.`);
  }
  const before = commitSha((await api('GET', `/repos/${repository}/branches/${branch}`)).commit.sha);
  const source = commitSha((await api('GET', `/repos/${upstream}/branches/${branch}`)).commit.sha);
  const comparison = await api('GET', `/repos/${repository}/compare/${before}...${source}`);
  if (!Number.isSafeInteger(comparison.ahead_by) || comparison.ahead_by < 0) {
    throw new Error('GitHub returned invalid comparison metadata');
  }
  if (comparison.ahead_by > 0) {
    try {
      // GitHub performs a merge, preserving fork commits. Never force or reset main.
      await api('POST', `/repos/${repository}/merge-upstream`, { branch });
    } catch (error) {
      if (error.status === 409) {
        throw new Error('Upstream conflicts with fork changes. Resolve the merge manually; main was not force-updated.');
      }
      throw error;
    }
  }
  const after = commitSha((await api('GET', `/repos/${repository}/branches/${branch}`)).commit.sha);
  // GITHUB_TOKEN merges do not trigger push workflows. Dispatch CI explicitly.
  // Also recover a previous sync whose merge succeeded but CI dispatch failed.
  const runs = await api('GET', `/repos/${repository}/actions/workflows/ci.yml/runs?head_sha=${after}&per_page=100`);
  if (!Array.isArray(runs.workflow_runs)) throw new Error('GitHub returned invalid workflow metadata');
  const alreadyChecked = runs.workflow_runs.some(run => run.head_sha === after
    && run.head_branch === branch && ['push', 'workflow_dispatch'].includes(run.event));
  if (!alreadyChecked) {
    await api('POST', `/repos/${repository}/actions/workflows/ci.yml/dispatches`, { ref: branch });
  }
  return { changed: before !== after, sha: after, ciDispatched: !alreadyChecked };
}

async function main() {
  if (process.env.GITHUB_REPOSITORY !== repository) throw new Error('This workflow only syncs the configured fork');
  const token = process.env.GH_TOKEN;
  if (!token) throw new Error('GH_TOKEN is required');
  const api = async (method, path, body) => {
    const headers = { Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2026-03-10', ...(body ? { 'Content-Type': 'application/json' } : {}) };
    // The token belongs to the fork; public upstream reads need no cross-repo grant.
    if (path === `/repos/${repository}` || path.startsWith(`/repos/${repository}/`)) {
      headers.Authorization = `Bearer ${token}`;
    }
    const response = await fetch(`https://api.github.com${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
      signal: AbortSignal.timeout(60000),
    });
    if (!response.ok) throw new GitHubError(response.status, `GitHub ${method} ${path} failed (HTTP ${response.status})`);
    return response.status === 204 ? null : response.json();
  };
  const result = await syncUpstream(api);
  const summary = `Upstream sync: ${result.changed ? 'merged new commits' : 'already current'}.\n\nMain: ${result.sha}\n\nCI: ${result.ciDispatched ? 'requested; Windows release follows successful CI' : 'already requested for this commit'}.\n`;
  console.log(summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
