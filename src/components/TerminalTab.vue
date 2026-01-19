<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
    taskId: string
    hostId: number
}>()

const terminalEl = ref<HTMLElement | null>(null)
let term: Terminal
let fitAddon: FitAddon
let unlistenStdout: any
let unlistenDone: any
let unlistenProgress: any
let isReady = ref(false)  // Track if SSH is ready

onMounted(async () => {
    console.log('[Terminal] Mounting with taskId:', props.taskId)

    // Setup xterm.js
    term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: 'Consolas, "Courier New", monospace',
        theme: {
            background: '#1e1e1e',
            foreground: '#d4d4d4',
        },
        scrollback: 5000,
        convertEol: true,
    })

    fitAddon = new FitAddon()
    term.loadAddon(fitAddon)
    term.open(terminalEl.value!)

    // Small delay before fit to ensure proper rendering
    setTimeout(() => {
        fitAddon.fit()
        term.focus()
    }, 100)

    // Handle window resize
    const handleResize = () => {
        try {
            fitAddon.fit()
        } catch (e) {
            console.error('Fit error:', e)
        }
    }
    window.addEventListener('resize', handleResize)

    // ⬇️ KIRIM INPUT KE BACKEND
    term.onData(async (data) => {
        // Don't send input if SSH is not ready yet
        if (!isReady.value) {
            console.log('[Terminal] SSH not ready yet, ignoring input')
            return
        }

        const charCode = data.charCodeAt(0)
        const isCtrlC = charCode === 3
        console.log('[Terminal] onData triggered:', {
            charCode,
            char: data,
            length: data.length,
            isCtrlC,
            taskId: props.taskId
        })

        try {
            console.log('[Terminal] Calling ssh_exec_input...')
            await invoke('ssh_exec_input', {
                taskId: props.taskId,
                data
            })
            console.log('[Terminal] ssh_exec_input SUCCESS')
        } catch (err) {
            console.error('[Terminal] ssh_exec_input FAILED:', err)
            // Don't show error in terminal, just log it
        }
    })

    // Listen to progress events
    unlistenProgress = await listen<any>('ssh:progress', (e) => {
        console.log('[Terminal] Progress event:', e.payload)
        if (e.payload.task_id === props.taskId) {
            const status = e.payload.status
            if (status === 'running') {
                // SSH is ready, wait a bit more for shell init
                setTimeout(() => {
                    isReady.value = true
                    console.log('[Terminal] SSH is ready for input')
                }, 500)
            } else if (status === 'error') {
                term.writeln('\r\n\x1b[31m[ERROR]\x1b[0m')
            } else if (status === 'cancelled') {
                term.writeln('\r\n\x1b[33m[CANCELLED]\x1b[0m')
            }
        }
    })

    // Listen SSH output
    unlistenStdout = await listen<any>('ssh:stdout', (e) => {
        console.log('[Terminal] Stdout event for task:', e.payload.task_id, 'my task:', props.taskId)
        if (e.payload.task_id === props.taskId) {
            console.log('[Terminal] Writing to terminal:', e.payload.data.length, 'bytes')
            term.write(e.payload.data)
        }
    })

    // Listen SSH done/closed
    unlistenDone = await listen<any>('ssh:done', (e) => {
        console.log('[Terminal] Done event:', e.payload)
        if (e.payload.task_id === props.taskId) {
            const code = e.payload.exit_code
            if (code === 0) {
                term.writeln('\r\n\x1b[32m[Connection closed]\x1b[0m')
            } else {
                term.writeln(`\r\n\x1b[31m[Connection closed with exit code ${code}]\x1b[0m`)
            }
        }
    })

    // Start SSH connection
    console.log('[Terminal] Starting SSH connection...')
    try {
        await invoke('ssh_exec_start', {
            taskId: props.taskId,
            hostId: props.hostId,
        })
        console.log('[Terminal] SSH started successfully')
    } catch (err) {
        console.error('[Terminal] SSH start error:', err)
        term.writeln(`\x1b[31mConnection error: ${err}\x1b[0m`)
    }
})

onBeforeUnmount(() => {
    console.log('[Terminal] Unmounting, cleaning up...')
    unlistenProgress?.()
    unlistenStdout?.()
    unlistenDone?.()
    term?.dispose()

    // Cancel SSH session on unmount
    invoke('ssh_exec_cancel', { taskId: props.taskId }).catch(() => { })
})
</script>

<template>
    <div class="terminal-wrapper">
        <div ref="terminalEl" class="terminal"></div>
    </div>
</template>

<style scoped>
.terminal-wrapper {
    width: 100%;
    height: 100%;
    background: #1e1e1e;
}

.terminal {
    width: 100%;
    height: 100%;
    padding: 8px;
}
</style>