import { watch, onBeforeUnmount } from 'vue'

export function useEscapeKey(isActive: () => boolean, onEscape: () => void) {
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isActive()) onEscape()
  }

  watch(isActive, (active) => {
    if (active) {
      window.addEventListener('keydown', handleKeydown)
    } else {
      window.removeEventListener('keydown', handleKeydown)
    }
  }, { immediate: true })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', handleKeydown)
  })
}
