import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { ref } from 'vue'
import { createSharedComposable } from '@vueuse/core'
import { useToast } from './useToast'
import { getSetting, setSetting } from '../api'

export interface UpdateInfo {
  version: string
  notes: string | null
}

export const useUpdater = createSharedComposable(() => {
  const { error } = useToast()

  const updateAvailable = ref<UpdateInfo | null>(null)
  const checking = ref(false)
  const installing = ref(false)
  const installPercent = ref<number | null>(null)
  const isBetaTester = ref(false)

  async function loadChannel() {
    const val = await getSetting('betaTester').catch(() => null)
    isBetaTester.value = val === 'true'
  }

  async function setBetaTester(value: boolean) {
    await setSetting('betaTester', value ? 'true' : 'false')
    isBetaTester.value = value
  }

  function channelHeaders() {
    return { headers: { 'X-Channel': isBetaTester.value ? 'beta' : 'stable' } }
  }

  async function checkForUpdate() {
    if (checking.value) return
    checking.value = true
    try {
      await loadChannel()
      const update = await check(channelHeaders())
      if (update) {
        updateAvailable.value = { version: update.version, notes: update.body ?? null }
      } else {
        updateAvailable.value = null
      }
    } catch (e) {
      // Best-effort on startup — network unavailability is not a user-facing error
      console.warn('[updater] check failed:', e)
    } finally {
      checking.value = false
    }
  }

  async function installUpdate() {
    if (installing.value) return
    installing.value = true
    installPercent.value = null

    try {
      // Always re-check to guarantee the freshest available version is installed,
      // not the one cached when the app launched hours ago.
      const update = await check(channelHeaders())
      if (!update) {
        updateAvailable.value = null
        return
      }

      let totalLength: number | undefined
      let downloaded = 0

      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          totalLength = event.data.contentLength
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          if (totalLength && totalLength > 0) {
            installPercent.value = Math.min(100, Math.round((downloaded / totalLength) * 100))
          }
        }
      })

      await relaunch()
    } catch (e) {
      error(`Update failed: ${e instanceof Error ? e.message : String(e)}`)
    } finally {
      installing.value = false
      installPercent.value = null
    }
  }

  return { updateAvailable, checking, installing, installPercent, isBetaTester, checkForUpdate, installUpdate, setBetaTester }
})
