import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Tunnel } from '../types'

export const useTunnelStore = defineStore('tunnel', () => {
    const tunnels = ref<Tunnel[]>([])
    const activeTunnelIds = ref<number[]>([])

    function isActive(id: number) {
        return activeTunnelIds.value.includes(id)
    }

    return {
        tunnels,
        activeTunnelIds,
        isActive,
    }
})