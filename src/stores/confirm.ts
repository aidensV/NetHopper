import { defineStore } from 'pinia'
import { ref } from 'vue'

type ConfirmOptions = {
    title?: string
    message?: string
}

export const useConfirmStore = defineStore('confirm', () => {
    const open = ref(false)

    const title = ref('')
    const message = ref('')

    let resolver: ((value: boolean) => void) | null = null

    function confirm(options: ConfirmOptions) {
        title.value = options.title || 'Confirm'
        message.value = options.message || ''

        open.value = true

        return new Promise<boolean>((resolve) => {
            resolver = resolve
        })
    }

    function onConfirm() {
        open.value = false
        resolver?.(true)
    }

    function onCancel() {
        open.value = false
        resolver?.(false)
    }

    return {
        open,
        title,
        message,
        confirm,
        onConfirm,
        onCancel
    }
})