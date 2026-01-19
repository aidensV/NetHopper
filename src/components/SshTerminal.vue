<script setup lang="ts">
import { ref } from 'vue'
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

async function testInput() {
    if (!activeTab.value) {
        alert('No active tab')
        return
    }

    try {
        await invoke('ssh_exec_input', {
            taskId: activeTab.value,
            data: 'ls\n'
        })
        console.log('Test input sent!')
    } catch (err) {
        console.error('Test input failed:', err)
        alert('Test failed: ' + err)
    }
}

function newTab() {
    const id = nanoid()  // ← Generate unique task_id
    tabs.value.push({
        id,
        hostId: props.host.id
    })
    activeTab.value = id
}

function closeTab(id: string) {
    tabs.value = tabs.value.filter(t => t.id !== id)
    if (activeTab.value === id) {
        activeTab.value = tabs.value[0]?.id ?? null
    }
}
</script>

<template>
    <div class="terminal-root">
        <div class="tab-bar">
            <button @click="newTab">+ New Terminal</button>
            <button @click="testInput" style="background: #ff6600;">🧪 Test Input (ls)</button>
            <div v-for="tab in tabs" :key="tab.id" class="tab" :class="{ active: tab.id === activeTab }"
                @click="activeTab = tab.id">
                Terminal {{ tab.id.slice(0, 6) }}
                <span class="close" @click.stop="closeTab(tab.id)">×</span>
            </div>
        </div>
        <div class="terminal-area">
            <TerminalTab v-for="tab in tabs" v-show="tab.id === activeTab" :key="tab.id" :task-id="tab.id"
                :host-id="tab.hostId" />
        </div>
    </div>
</template>

<style scoped>
.terminal-root {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.tab-bar {
    display: flex;
    background: #1e1e1e;
    color: #fff;
    gap: 4px;
    padding: 4px;
}

.tab-bar button {
    padding: 6px 12px;
    background: #2d2d2d;
    border: none;
    color: #fff;
    cursor: pointer;
    border-radius: 4px;
}

.tab-bar button:hover {
    background: #3d3d3d;
}

.tab {
    padding: 6px 12px;
    cursor: pointer;
    background: #2d2d2d;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 8px;
}

.tab:hover {
    background: #3d3d3d;
}

.tab.active {
    background: #007acc;
}

.tab .close {
    font-size: 18px;
    font-weight: bold;
    opacity: 0.7;
}

.tab .close:hover {
    opacity: 1;
}

.terminal-area {
    flex: 1;
    overflow: hidden;
}
</style>