import { ref } from 'vue'

export type ToastType = 'success' | 'error' | 'info'

export interface Toast {
  id: number
  message: string
  type: ToastType
}

const toasts = ref<Toast[]>([])

export function useToast() {
  function showToast(message: string, type: ToastType = 'info', duration = 5000) {
    const id = Date.now()
    toasts.value.push({ id, message, type })
    
    setTimeout(() => {
      removeToast(id)
    }, duration)
  }
  
  function removeToast(id: number) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  function error(message: string) {
    showToast(message, 'error')
  }

  function success(message: string) {
    showToast(message, 'success')
  }

  function info(message: string) {
    showToast(message, 'info')
  }

  return {
    toasts,
    showToast,
    removeToast,
    error,
    success,
    info
  }
}
