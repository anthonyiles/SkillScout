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
  body: string | null
  draft: boolean
  prerelease: boolean
  published_at: string
  assets: GitHubAsset[]
}

interface PlatformTarget {
  url: string
  signature: string
}

interface UpdateManifest {
  version: string
  notes: string
  pub_date: string
  platforms: Record<string, PlatformTarget>
}

// Maps asset filename suffixes to Tauri platform keys.
// The universal macOS binary covers both x86_64 and aarch64.
const PLATFORM_PATTERNS: Array<{ platforms: string[]; suffix: string }> = [
  { platforms: ['darwin-x86_64', 'darwin-aarch64'], suffix: '_universal.dmg.tar.gz' },
  { platforms: ['linux-x86_64'], suffix: '_amd64.AppImage.tar.gz' },
  { platforms: ['windows-x86_64'], suffix: '_x64-setup.nsis.zip' },
]

const JSON_HEADERS = {
  'Content-Type': 'application/json',
  'Cache-Control': 'public, max-age=300',
  'Access-Control-Allow-Origin': '*',
}

// No-update responses must not be cached — a release could be published at any moment
const NO_UPDATE_HEADERS = {
  'Cache-Control': 'no-store',
  'Access-Control-Allow-Origin': '*',
}

async function fetchRelease(repo: string, channel: string, githubHeaders: Record<string, string>): Promise<GitHubRelease | null> {
  if (channel === 'beta') {
    // Beta: pick the most recent non-draft release (prereleases included)
    const res = await fetch(
      `https://api.github.com/repos/${repo}/releases?per_page=10`,
      { headers: githubHeaders },
    )
    if (!res.ok) throw new Error(`GitHub API error: ${res.status}`)
    const releases = await res.json() as GitHubRelease[]
    return releases.find(r => !r.draft) ?? null
  } else {
    // Stable: /releases/latest only returns non-prerelease, non-draft releases.
    // GitHub returns 404 when no stable release exists yet — treat that as no update.
    const res = await fetch(
      `https://api.github.com/repos/${repo}/releases/latest`,
      { headers: githubHeaders },
    )
    if (res.status === 404) return null
    if (!res.ok) throw new Error(`GitHub API error: ${res.status}`)
    const release = await res.json() as GitHubRelease
    if (release.draft || release.prerelease) return null
    return release
  }
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    // X-Channel header takes priority — allows clients to switch channels at runtime
    // without a config change. Falls back to the ?channel query param for curl testing.
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
        JSON.stringify({ error: 'Failed to reach GitHub API', detail: String(e) }),
        { status: 502, headers: JSON_HEADERS },
      )
    }

    if (!release) {
      // 204 tells Tauri's updater there is no update available
      return new Response(null, { status: 204, headers: NO_UPDATE_HEADERS })
    }

    const version = release.tag_name.replace(/^v/, '')
    const platforms: Record<string, PlatformTarget> = {}

    for (const { platforms: keys, suffix } of PLATFORM_PATTERNS) {
      const asset = release.assets.find(a => a.name.endsWith(suffix))
      const sigAsset = release.assets.find(a => a.name.endsWith(suffix + '.sig'))
      if (!asset || !sigAsset) continue

      let signature: string
      try {
        const sigRes = await fetch(sigAsset.browser_download_url, {
          headers: { 'User-Agent': 'SkillScout-Updater/1.0' },
        })
        if (!sigRes.ok) continue
        signature = (await sigRes.text()).trim()
      } catch {
        continue
      }

      for (const platform of keys) {
        platforms[platform] = { url: asset.browser_download_url, signature }
      }
    }

    if (Object.keys(platforms).length === 0) {
      // Release exists but has no updater assets — no update to offer
      return new Response(null, { status: 204, headers: NO_UPDATE_HEADERS })
    }

    const manifest: UpdateManifest = {
      version,
      notes: release.body ?? '',
      pub_date: release.published_at,
      platforms,
    }

    return new Response(JSON.stringify(manifest), { headers: JSON_HEADERS })
  },
}
