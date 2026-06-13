<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { Database, Github, KeyRound, Network, Orbit, Server, ShieldCheck, TerminalSquare, X } from 'lucide-vue-next'

defineProps<{
    open: boolean
}>()

const emit = defineEmits<{
    (event: 'close'): void
}>()

const version = ref('0.1.0')

const technologies = [
    { name: 'Tauri 2', detail: 'Desktop runtime', icon: Orbit },
    { name: 'Vue 3', detail: 'User interface', icon: Server },
    { name: 'Rust', detail: 'Native backend', icon: ShieldCheck },
    { name: 'xterm.js', detail: 'Terminal engine', icon: TerminalSquare },
    { name: 'SQLite', detail: 'Local database', icon: Database },
    { name: 'libssh2', detail: 'SSH engine', icon: Network },
]

function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') emit('close')
}

onMounted(async () => {
    window.addEventListener('keydown', handleKeydown)
    try {
        version.value = await getVersion()
    } catch {
        // Keep the bundled version when rendered outside the Tauri runtime.
    }
})

onBeforeUnmount(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
    <Teleport to="body">
        <Transition name="about">
            <div v-if="open" class="fixed inset-0 z-[70] grid place-items-center bg-black/75 p-5 backdrop-blur-md"
                @click.self="emit('close')">
                <section class="surface relative w-full max-w-lg overflow-hidden rounded-2xl shadow-[0_30px_90px_rgba(0,0,0,.55)]">
                    <div class="absolute inset-x-0 top-0 h-px bg-linear-to-r from-transparent via-cyan-300/70 to-transparent"></div>
                    <button class="icon-button absolute right-4 top-4 size-8" title="Close" @click="emit('close')">
                        <X :size="17" />
                    </button>

                    <div class="px-7 pb-6 pt-8 text-center">
                        <div class="mx-auto grid size-14 place-items-center rounded-2xl bg-cyan-400 text-slate-950 shadow-[0_0_35px_rgba(34,211,238,.22)]">
                            <Orbit :size="29" :stroke-width="2.2" />
                        </div>
                        <h2 class="mt-4 text-xl font-semibold tracking-tight text-slate-100">NetHopper</h2>
                        <p class="mt-1 text-xs text-slate-500">A focused, local-first SSH workspace.</p>
                        <span class="mt-3 inline-flex rounded-full border border-cyan-400/15 bg-cyan-400/8 px-3 py-1 text-[11px] font-semibold text-cyan-300">
                            Version {{ version }}
                        </span>
                    </div>

                    <div class="border-y border-white/6 bg-black/10 px-7 py-5">
                        <p class="mb-3 text-[10px] font-bold uppercase tracking-[.16em] text-slate-600">Built with</p>
                        <div class="grid grid-cols-2 gap-2">
                            <div v-for="technology in technologies" :key="technology.name"
                                class="flex items-center gap-3 rounded-xl border border-white/6 bg-white/[.025] p-3">
                                <div class="grid size-8 shrink-0 place-items-center rounded-lg bg-white/5 text-cyan-300">
                                    <component :is="technology.icon" :size="15" />
                                </div>
                                <div>
                                    <p class="text-xs font-semibold text-slate-300">{{ technology.name }}</p>
                                    <p class="mt-0.5 text-[10px] text-slate-600">{{ technology.detail }}</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-3 gap-3 px-7 py-5 text-center">
                        <div><TerminalSquare :size="15" class="mx-auto text-slate-500" /><p class="mt-1.5 text-[10px] text-slate-500">SSH terminal</p></div>
                        <div><KeyRound :size="15" class="mx-auto text-slate-500" /><p class="mt-1.5 text-[10px] text-slate-500">Encrypted vault</p></div>
                        <div><Network :size="15" class="mx-auto text-slate-500" /><p class="mt-1.5 text-[10px] text-slate-500">Secure tunnels</p></div>
                    </div>

                    <footer class="flex items-center justify-between border-t border-white/6 px-7 py-4 text-[10px] text-slate-600">
                        <span>Data stays on your device</span>
                        <span class="flex items-center gap-1.5"><Github :size="12" /> NetHopper Desktop</span>
                    </footer>
                </section>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
.about-enter-active,
.about-leave-active {
    transition: opacity .18s ease;
}

.about-enter-active section,
.about-leave-active section {
    transition: transform .18s ease, opacity .18s ease;
}

.about-enter-from,
.about-leave-to {
    opacity: 0;
}

.about-enter-from section,
.about-leave-to section {
    opacity: 0;
    transform: translateY(8px) scale(.98);
}
</style>
