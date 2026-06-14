<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { ArrowDownToLine, CheckCircle2, RefreshCw, Sparkles, X } from 'lucide-vue-next'

const update = ref<Update | null>(null)
const open = ref(false)
const checking = ref(false)
const installing = ref(false)
const installed = ref(false)
const progress = ref(0)
const error = ref('')

async function checkForUpdate(showNoUpdate = false) {
    if (checking.value || installing.value) return

    checking.value = true
    error.value = ''

    try {
        update.value = await check()
        if (update.value) {
            open.value = true
        } else if (showNoUpdate) {
            error.value = 'You are already using the latest version.'
            open.value = true
        }
    } catch (reason) {
        console.error('[Updater] Failed to check for updates:', reason)
        if (showNoUpdate) {
            error.value = `Unable to check for updates: ${String(reason)}`
            open.value = true
        }
    } finally {
        checking.value = false
    }
}

async function installUpdate() {
    if (!update.value || installing.value) return

    installing.value = true
    error.value = ''
    progress.value = 0
    let downloaded = 0
    let contentLength = 0

    try {
        await update.value.downloadAndInstall((event) => {
            if (event.event === 'Started') {
                contentLength = event.data.contentLength ?? 0
            } else if (event.event === 'Progress') {
                downloaded += event.data.chunkLength
                progress.value = contentLength > 0
                    ? Math.min(100, Math.round((downloaded / contentLength) * 100))
                    : progress.value
            } else if (event.event === 'Finished') {
                progress.value = 100
            }
        })
        installed.value = true
    } catch (reason) {
        error.value = `Update failed: ${String(reason)}`
    } finally {
        installing.value = false
    }
}

onMounted(() => {
    window.setTimeout(() => checkForUpdate(), 1800)
})

defineExpose({ checkForUpdate })
</script>

<template>
    <Teleport to="body">
        <Transition name="update-dialog">
            <div v-if="open" class="fixed inset-0 z-[80] grid place-items-center bg-black/75 p-5 backdrop-blur-md"
                @click.self="!installing && !installed && (open = false)">
                <section class="surface relative w-full max-w-md overflow-hidden rounded-2xl shadow-[0_30px_90px_rgba(0,0,0,.6)]">
                    <div class="h-1 bg-linear-to-r from-cyan-400 via-blue-400 to-violet-400"></div>
                    <button v-if="!installing && !installed" class="icon-button absolute right-4 top-5 size-8"
                        title="Not now" @click="open = false">
                        <X :size="17" />
                    </button>

                    <div class="p-7">
                        <div class="grid size-12 place-items-center rounded-xl"
                            :class="installed ? 'bg-emerald-400/10 text-emerald-300' : 'bg-cyan-400/10 text-cyan-300'">
                            <CheckCircle2 v-if="installed" :size="23" />
                            <Sparkles v-else :size="23" />
                        </div>

                        <template v-if="installed">
                            <h2 class="mt-5 text-lg font-semibold text-slate-100">Update ready</h2>
                            <p class="mt-2 text-sm leading-relaxed text-slate-500">
                                NetHopper {{ update?.version }} has been installed. Restart to finish the update.
                            </p>
                            <button class="primary-button mt-6 flex w-full items-center justify-center gap-2 rounded-lg py-2.5 text-sm"
                                @click="relaunch">
                                <RefreshCw :size="16" /> Restart NetHopper
                            </button>
                        </template>

                        <template v-else-if="update">
                            <p class="text-[10px] font-bold uppercase tracking-[.16em] text-cyan-300">Update available</p>
                            <h2 class="mt-2 text-lg font-semibold text-slate-100">NetHopper {{ update.version }}</h2>
                            <p class="mt-1 text-xs text-slate-600">A newer version is ready to install.</p>

                            <div v-if="update.body" class="mt-5 max-h-32 overflow-y-auto rounded-xl border border-white/6 bg-black/15 p-4 text-xs leading-relaxed text-slate-400">
                                {{ update.body }}
                            </div>

                            <div v-if="installing" class="mt-6">
                                <div class="mb-2 flex justify-between text-[11px] text-slate-500">
                                    <span>Downloading update...</span><span>{{ progress }}%</span>
                                </div>
                                <div class="h-1.5 overflow-hidden rounded-full bg-white/6">
                                    <div class="h-full rounded-full bg-cyan-400 transition-all duration-200" :style="{ width: `${progress}%` }"></div>
                                </div>
                            </div>

                            <p v-if="error" class="mt-4 rounded-lg border border-rose-400/15 bg-rose-400/5 px-3 py-2 text-xs text-rose-300">{{ error }}</p>

                            <div class="mt-6 flex gap-2">
                                <button class="secondary-button flex-1 rounded-lg py-2.5 text-sm" :disabled="installing" @click="open = false">Later</button>
                                <button class="primary-button flex flex-1 items-center justify-center gap-2 rounded-lg py-2.5 text-sm disabled:cursor-wait disabled:opacity-60"
                                    :disabled="installing" @click="installUpdate">
                                    <ArrowDownToLine :size="16" /> {{ installing ? 'Updating...' : 'Update now' }}
                                </button>
                            </div>
                        </template>

                        <template v-else>
                            <h2 class="mt-5 text-lg font-semibold text-slate-100">Software update</h2>
                            <p class="mt-2 text-sm text-slate-500">{{ error }}</p>
                            <button class="secondary-button mt-6 w-full rounded-lg py-2.5 text-sm" @click="open = false">Close</button>
                        </template>
                    </div>
                </section>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
.update-dialog-enter-active,
.update-dialog-leave-active {
    transition: opacity .18s ease;
}

.update-dialog-enter-from,
.update-dialog-leave-to {
    opacity: 0;
}
</style>
