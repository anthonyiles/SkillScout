import { ref } from 'vue'
import { createSharedComposable } from '@vueuse/core'

export type ToastType = 'success' | 'error' | 'info'

export interface Toast {
  id: number
  message: string
  type: ToastType
}

let nextToastId = 0

export const useToast = createSharedComposable(() => {
  const toasts = ref<Toast[]>([])

  function showToast(message: string, type: ToastType = 'info', duration = 5000) {
    const id = ++nextToastId
    toasts.value.push({ id, message, type })
    setTimeout(() => removeToast(id), duration)
  }

  function removeToast(id: number) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  function error(message: string) { showToast(message, 'error') }
  function success(message: string) { showToast(message, 'success') }
  function info(message: string) { showToast(message, 'info') }

  return { toasts, showToast, removeToast, error, success, info }
})
