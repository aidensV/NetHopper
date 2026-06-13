import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Host } from '../types'

export const useTabStore = defineStore('tabs', () => {
    const tabs = ref<{ id: number; host: Host }[]>([])
    const activeTabId = ref<number | null>(null)

    function openTab(host: Host) {
        const existing = tabs.value.find(t => t.id === host.id)
        if (existing) {
            activeTabId.value = host.id
            return
        }
        tabs.value.push({ id: host.id, host })
        activeTabId.value = host.id
    }

    function closeTab(hostId: number) {
        tabs.value = tabs.value.filter(t => t.id !== hostId)
        if (activeTabId.value === hostId) {
            activeTabId.value = tabs.value.length > 0
                ? tabs.value[tabs.value.length - 1].id
                : null
        }
    }

    return { tabs, activeTabId, openTab, closeTab }
})