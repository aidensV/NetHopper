import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
    id: string
    message: string
    type: ToastType
    duration?: number
}

export const useToastStore = defineStore('toast', () => {
    const toasts = ref<Toast[]>([])

    function add(message: string, type: ToastType = 'info', duration = 3000) {
        const id = crypto.randomUUID()
        toasts.value.push({ id, message, type, duration })

        if (duration > 0) {
            setTimeout(() => remove(id), duration)
        }
    }

    function remove(id: string) {
        toasts.value = toasts.value.filter(t => t.id !== id)
    }

    // Shorthand helpers
    const success = (msg: string, duration?: number) => add(msg, 'success', duration)
    const error = (msg: string, duration?: number) => add(msg, 'error', duration)
    const warning = (msg: string, duration?: number) => add(msg, 'warning', duration)
    const info = (msg: string, duration?: number) => add(msg, 'info', duration)

    return { toasts, add, remove, success, error, warning, info }
})