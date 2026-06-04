interface Env {
  GITHUB_REPO: string
  GITHUB_TOKEN?: string
}

interface GitHubAsset {
  name: string
  browser_download_url: string
}

interface GitHubRelease {
  tag_name: string
  draft: boolean
  prerelease: boolean
  assets: GitHubAsset[]
}

function semverFromTag(tag: string): [number, number, number] {
  const m = tag.match(/(\d+)\.(\d+)\.(\d+)/)
  return m ? [+m[1], +m[2], +m[3]] : [0, 0, 0]
}

function byVersionDesc(a: GitHubRelease, b: GitHubRelease): number {
  const [aMaj, aMin, aPat] = semverFromTag(a.tag_name)
  const [bMaj, bMin, bPat] = semverFromTag(b.tag_name)
  return (bMaj - aMaj) || (bMin - aMin) || (bPat - aPat)
}

const JSON_HEADERS = {
  'Content-Type': 'application/json',
  'Cache-Control': 'no-store',
  'Access-Control-Allow-Origin': '*',
}

const NO_UPDATE_HEADERS = {
  'Cache-Control': 'no-store',
  'Access-Control-Allow-Origin': '*',
}

async function fetchRelease(repo: string, channel: string, githubHeaders: Record<string, string>): Promise<GitHubRelease | null> {
  const res = await fetch(
    `https://api.github.com/repos/${repo}/releases?per_page=10`,
    { headers: githubHeaders },
  )
  if (!res.ok) throw new Error(`GitHub API error: ${res.status}`)
  const releases = await res.json() as GitHubRelease[]
  if (channel === 'beta') {
    return releases.filter(r => !r.draft).sort(byVersionDesc)[0] ?? null
  }
  return releases.filter(r => !r.draft && !r.prerelease).sort(byVersionDesc)[0] ?? null
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const channel = request.headers.get('X-Channel')
      ?? new URL(request.url).searchParams.get('channel')
      ?? 'stable'

    const githubHeaders: Record<string, string> = {
      Accept: 'application/vnd.github+json',
      'User-Agent': 'SkillScout-Updater/1.0',
      'X-GitHub-Api-Version': '2022-11-28',
    }
    if (env.GITHUB_TOKEN) {
      githubHeaders['Authorization'] = `Bearer ${env.GITHUB_TOKEN}`
    }

    let release: GitHubRelease | null
    try {
      release = await fetchRelease(env.GITHUB_REPO, channel, githubHeaders)
    } catch (e) {
      return new Response(
        JSON.stringify({ error: 'Failed to reach GitHub API', detail: e instanceof Error ? e.message : 'Unknown error' }),
        { status: 502, headers: JSON_HEADERS },
      )
    }

    if (!release) {
      return new Response(
        JSON.stringify({ error: `No releases found for channel: ${channel}` }),
        { status: 404, headers: JSON_HEADERS },
      )
    }

    const latestJson = release.assets.find(a => a.name === 'latest.json')
    if (!latestJson) {
      return new Response(null, { status: 204, headers: NO_UPDATE_HEADERS })
    }

    let manifestRes: Response
    try {
      manifestRes = await fetch(latestJson.browser_download_url, {
        headers: { 'User-Agent': 'SkillScout-Updater/1.0' },
      })
    } catch {
      return new Response(null, { status: 204, headers: NO_UPDATE_HEADERS })
    }
    if (!manifestRes.ok) {
      return new Response(null, { status: 204, headers: NO_UPDATE_HEADERS })
    }

    return new Response(await manifestRes.text(), { headers: JSON_HEADERS })
  },
}
