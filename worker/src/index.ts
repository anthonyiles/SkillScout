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

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const githubHeaders: Record<string, string> = {
      Accept: 'application/vnd.github+json',
      'User-Agent': 'SkillScout-Updater/1.0',
      'X-GitHub-Api-Version': '2022-11-28',
    }
    if (env.GITHUB_TOKEN) {
      githubHeaders['Authorization'] = `Bearer ${env.GITHUB_TOKEN}`
    }

    let release: GitHubRelease
    try {
      const res = await fetch(
        `https://api.github.com/repos/${env.GITHUB_REPO}/releases/latest`,
        { headers: githubHeaders },
      )
      if (!res.ok) {
        return new Response(
          JSON.stringify({ error: `GitHub API error: ${res.status}` }),
          { status: 502, headers: JSON_HEADERS },
        )
      }
      release = await res.json() as GitHubRelease
    } catch (e) {
      return new Response(
        JSON.stringify({ error: 'Failed to reach GitHub API', detail: String(e) }),
        { status: 502, headers: JSON_HEADERS },
      )
    }

    if (release.draft || release.prerelease) {
      // 204 tells Tauri's updater there is no update available
      return new Response(null, { status: 204, headers: JSON_HEADERS })
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
      return new Response(null, { status: 204, headers: JSON_HEADERS })
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
