<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useTunnelStore } from '../../stores/tunnelStore'
import { useToastStore } from '../../stores/toastStore'
import { Play, Square, Pencil, Trash2, Plus, X } from 'lucide-vue-next'
import { createTunnel, deleteTunnel, listActiveTunnels, listHosts, listTunnels, startTunnel, stopTunnel, updateTunnel } from '../../api'
import { Host } from '../../types'
import { Tunnel } from '../../types'

const tunnelStore = useTunnelStore()
const toast = useToastStore()

// ── Sidebar form ──────────────────────────────────────────────────────────────
const showForm = ref(false)
const isEditing = ref(false)
const editingId = ref<number | null>(null)

const form = ref<Tunnel>({
    id: 0,
    created_at: '',
    name: '',
    type: 'local',
    host_id: 0,
    local_port: 0,
    remote_host: null,
    remote_port: null,
})

function openCreate() {
    isEditing.value = false
    editingId.value = null
    form.value = {
        id: 0,
        created_at: '',
        name: '',
        type: 'local',
        host_id: 0,
        local_port: 0,
        remote_host: null,
        remote_port: null
    }
    showForm.value = true
}

function openEdit(tunnel: Tunnel) {
    if (tunnelStore.isActive(tunnel.id)) {
        toast.warning('Stop tunnel dulu sebelum edit')
        return
    }
    isEditing.value = true
    editingId.value = tunnel.id
    form.value = { ...tunnel }
    showForm.value = true
}

function closeForm() {
    showForm.value = false
}

async function submitForm() {
    try {
        if (form.value.host_id === 0) {
            toast.error('Pilih host dulu')
            return
        }
        if (isEditing.value && editingId.value !== null) {
            await updateTunnel(form.value)
            toast.success('Tunnel updated!')
        } else {
            await createTunnel(form.value)
            toast.success('Tunnel created!')
        }
        closeForm()

    } catch (e) {
        toast.error(`Gagal: ${e}`)
    }
    tunnelStore.tunnels = await listTunnels()
}

// ── Actions ───────────────────────────────────────────────────────────────────
async function toggleTunnel(tunnel: Tunnel) {
    try {
        if (tunnelStore.isActive(tunnel.id)) {
            try {
                await stopTunnel(tunnel.id)
                toast.info(`Tunnel "${tunnel.name}" stopped`)
            } catch (e) {
                toast.error(`Gagal stop: ${e}`)
            }
        } else {
            try {
                await startTunnel(tunnel.id)
                toast.success(`Tunnel "${tunnel.name}" started`)
            } catch (e) {
                toast.error(`Gagal start: ${e}`)
            }
        }

        tunnelStore.activeTunnelIds = await listActiveTunnels()
    } catch (e) {
        toast.error(`Gagal: ${e}`)
    }
}

async function removeTunnel(tunnel: Tunnel) {
    if (!confirm(`Hapus tunnel "${tunnel.name}"?`)) return
    try {
        await deleteTunnel(tunnel.id)
        toast.success('Tunnel dihapus')
    } catch (e) {
        toast.error(`Gagal: ${e}`)
    }
}

// ── Hosts untuk dropdown ──────────────────────────────────────────────────────


const hosts = ref<Host[]>([])


onMounted(async () => {
    tunnelStore.tunnels = await listTunnels()
    tunnelStore.activeTunnelIds = await listActiveTunnels()
    hosts.value = await listHosts(null)
})

// Nama host untuk ditampilkan di list
function hostName(host_id: number) {
    const h = hosts.value.find(h => h.id === host_id)
    return h ? `${h.name} (${h.host})` : `Host #${host_id}`
}
</script>

<template>
    <div class="flex min-h-[420px]">

        <!-- ── List ── -->
        <div class="min-w-0 flex-1 overflow-y-auto">

            <!-- Toolbar -->
            <div class="mb-5 flex items-center justify-between">
                <div><h2 class="text-sm font-semibold text-slate-200">Saved tunnels</h2><p class="mt-1 text-xs text-slate-600">{{ tunnelStore.activeTunnelIds.length }} currently active</p></div>
                <button @click="openCreate"
                    class="primary-button flex items-center gap-2 rounded-lg px-3 py-2 text-xs">
                    <Plus :size="16" />
                    New Tunnel
                </button>
            </div>

            <!-- Empty state -->
            <div v-if="tunnelStore.tunnels.length === 0" class="surface grid min-h-56 place-items-center rounded-2xl border-dashed text-center text-xs text-slate-600">
                Belum ada tunnel. Klik "New Tunnel" untuk membuat.
            </div>

            <!-- Tunnel list -->
            <div class="flex flex-col gap-3">
                <div v-for="tunnel in tunnelStore.tunnels" :key="tunnel.id"
                    class="surface surface-hover flex items-center justify-between rounded-xl px-4 py-4">

                    <!-- Info -->
                    <div class="flex items-center gap-3">
                        <!-- Status indicator -->
                        <div class="w-2 h-2 rounded-full"
                            :class="tunnelStore.isActive(tunnel.id) ? 'bg-green-400' : 'bg-gray-500'" />

                        <div>
                            <div class="flex items-center gap-2">
                                <span class="text-sm font-semibold text-slate-200">{{ tunnel.name }}</span>
                                <span class="text-xs px-1.5 py-0.5 rounded" :class="tunnel.type === 'local'
                                    ? 'bg-cyan-400/10 text-cyan-300'
                                    : 'bg-violet-400/10 text-violet-300'">
                                    {{ tunnel.type === 'local' ? 'Local Forward' : 'SOCKS5' }}
                                </span>
                            </div>
                            <div class="mt-1 font-mono text-[11px] text-slate-600">
                                {{ hostName(tunnel.host_id) }}
                                · port {{ tunnel.local_port }}
                                <template v-if="tunnel.type === 'local'">
                                    → {{ tunnel.remote_host }}:{{ tunnel.remote_port }}
                                </template>
                            </div>
                        </div>
                    </div>

                    <!-- Actions -->
                    <div class="flex items-center gap-2">
                        <button @click="toggleTunnel(tunnel)" class="icon-button size-8"
                            :title="tunnelStore.isActive(tunnel.id) ? 'Stop' : 'Start'">
                            <Square v-if="tunnelStore.isActive(tunnel.id)" :size="16" class="text-red-400" />
                            <Play v-else :size="16" class="text-green-400" />
                        </button>
                        <button @click="openEdit(tunnel)" class="icon-button size-8"
                            title="Edit">
                            <Pencil :size="16" class="text-gray-400 hover:text-white" />
                        </button>
                        <button @click="removeTunnel(tunnel)" class="icon-button size-8 hover:!text-rose-400"
                            title="Delete">
                            <Trash2 :size="16" class="text-gray-400 hover:text-red-400" />
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- ── Sidebar Form ── -->
        <Transition name="slide">
            <aside v-if="showForm" class="ml-6 w-[320px] shrink-0 overflow-y-auto border-l border-white/8 bg-[#0b111a] p-5">

                <!-- Header -->
                <div class="flex items-center bg-gray-800 p-2 rounded-lg mb-6">
                    <button @click="closeForm" class="text-gray-500 hover:text-white cursor-pointer">
                        <X :size="20" />
                    </button>
                    <span class="flex-1 text-center font-semibold">
                        {{ isEditing ? 'Edit Tunnel' : 'New Tunnel' }}
                    </span>
                </div>

                <!-- Form -->
                <div class="space-y-4">
                    <div>
                        <label class="block mb-1 text-sm text-gray-300">Name</label>
                        <input v-model="form.name" type="text" placeholder="My Tunnel"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-sm text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-600" />
                    </div>

                    <div>
                        <label class="block mb-1 text-sm text-gray-300">Type</label>
                        <select v-model="form.type"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-sm text-white focus:ring-2 focus:ring-blue-600">
                            <option value="local">Local Port Forwarding</option>
                            <option value="socks5">SOCKS5 Proxy</option>
                        </select>
                    </div>

                    <div>
                        <label class="block mb-1 text-sm text-gray-300">Host</label>
                        <select v-model="form.host_id"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-sm text-white focus:ring-2 focus:ring-blue-600">
                            <option :value="0" disabled>Pilih host...</option>
                            <option v-for="h in hosts" :key="h.id" :value="h.id">
                                {{ h.name }}
                            </option>
                        </select>
                    </div>

                    <div>
                        <label class="block mb-1 text-sm text-gray-300">Local Port</label>
                        <input v-model.number="form.local_port" type="number" placeholder="1080"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-sm text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-600" />
                    </div>

                    <!-- Hanya untuk local forwarding -->
                    <template v-if="form.type === 'local'">
                        <div>
                            <label class="block mb-1 text-sm text-gray-300">Remote Host</label>
                            <input v-model="form.remote_host" type="text" placeholder="192.168.1.1"
                                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-sm text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-600" />
                        </div>
                        <div>
                            <label class="block mb-1 text-sm text-gray-300">Remote Port</label>
                            <input v-model.number="form.remote_port" type="number" placeholder="3306"
                                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-sm text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-600" />
                        </div>
                    </template>

                    <div class="pt-2 flex justify-center">
                        <button @click="submitForm"
                            class="px-6 py-1.5 bg-green-700 hover:bg-green-800 border border-gray-600 rounded-md text-white text-sm cursor-pointer">
                            {{ isEditing ? 'Update' : 'Create' }}
                        </button>
                    </div>
                </div>
            </aside>
        </Transition>
    </div>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
    transition: all 0.22s ease;
}

aside input,
aside select {
    width: 100%;
    border: 1px solid rgba(148, 163, 184, .15);
    border-radius: .65rem;
    background: #080d15;
    color: #edf5fc;
    padding: .7rem .8rem;
}

aside input:focus,
aside select:focus {
    border-color: rgba(34, 211, 238, .65);
    box-shadow: 0 0 0 3px rgba(34, 211, 238, .08);
}

.slide-enter-active,
.slide-leave-active {
    transition: all 0.22s ease;
}

.slide-enter-from,
.slide-leave-to {
    transform: translateX(100%);
    opacity: 0;
}
</style>
