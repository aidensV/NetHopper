<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import TerminalTab from './TerminalTab.vue'
import { nanoid } from 'nanoid'
import { invoke } from '@tauri-apps/api/core'

type Host = {
    id: number
}

type Tab = {
    id: string      // ← task_id (nanoid yang unique)
    hostId: number  // ← ID host dari database
}

const props = defineProps<{
    host: Host
}>()

const tabs = ref<Tab[]>([])
const activeTab = ref<string | null>(null)

function newTab() {
    const id = nanoid()  // ← Generate unique task_id
    tabs.value.push({
        id,
        hostId: props.host.id
    })
    activeTab.value = id
}

async function closeTab(id: string) {
    try {
        await invoke('ssh_exec_cancel', { taskId: id })
    } catch (e) {
        console.error('Failed to cancel SSH:', e)
    }
    tabs.value = tabs.value.filter(t => t.id !== id)
    if (activeTab.value === id) {
        activeTab.value = tabs.value[0]?.id ?? null
    }
}

onBeforeUnmount(async () => {
    for (const tab of tabs.value) {
        console.log("BUKTINYA");

        try {
            await invoke('ssh_exec_cancel', { taskId: tab.id })
        } catch (e) { }
    }
})
newTab()
</script>

<template>
    <div class="flex flex-col h-full">
        <div class="flex items-center gap-1 bg-zinc-900 text-white p-1">

            <button @click="newTab"
                class="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 rounded text-sm transition cursor-pointer">
                + New
            </button>

            <div v-for="(tab, index) in tabs" :key="tab.id" @click="activeTab = tab.id"
                class="flex items-center gap-2 px-1 py-0.5 rounded cursor-pointer text-sm transition" :class="tab.id === activeTab
                    ? 'bg-blue-600'
                    : 'bg-zinc-800 hover:bg-zinc-700'">
                <span>
                    Terminal {{ index + 1 }}
                </span>

                <span @click.stop="closeTab(tab.id)" class="text-lg font-bold opacity-70 hover:opacity-100">
                    ×
                </span>
            </div>
        </div>

        <div class="flex-1 overflow-hidden bg-[#1E1E1E]">
            <TerminalTab v-for="tab in tabs" v-show="tab.id === activeTab" :key="tab.id" :task-id="tab.id"
                :host-id="tab.hostId" />
        </div>
    </div>
</template>
