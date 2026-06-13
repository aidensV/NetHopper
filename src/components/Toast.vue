<script setup lang="ts">
import { useToastStore } from '../stores/toastStore'
import { X, CheckCircle, XCircle, AlertTriangle, Info } from 'lucide-vue-next'

const toast = useToastStore()

const icons = {
    success: CheckCircle,
    error: XCircle,
    warning: AlertTriangle,
    info: Info,
}

const styles = {
    success: 'bg-[#101923] border-emerald-500/30 text-slate-200',
    error: 'bg-[#1c1218] border-rose-500/30 text-slate-200',
    warning: 'bg-[#1b1911] border-amber-500/30 text-slate-200',
    info: 'bg-[#101923] border-cyan-500/30 text-slate-200',
}

const iconStyles = {
    success: 'text-green-400',
    error: 'text-red-400',
    warning: 'text-yellow-400',
    info: 'text-blue-400',
}
</script>

<template>
    <Teleport to="body">
        <div class="fixed top-4 right-4 z-50 flex flex-col gap-2 w-80">
            <TransitionGroup name="toast">
                <div v-for="t in toast.toasts" :key="t.id"
                    class="flex items-start gap-3 px-4 py-3 rounded-lg border text-sm shadow-lg"
                    :class="styles[t.type]">
                    <component :is="icons[t.type]" :size="18" class="shrink-0 mt-0.5" :class="iconStyles[t.type]" />
                    <span class="flex-1">{{ t.message }}</span>
                    <button @click="toast.remove(t.id)" class="shrink-0 opacity-60 hover:opacity-100 cursor-pointer">
                        <X :size="16" />
                    </button>
                </div>
            </TransitionGroup>
        </div>
    </Teleport>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
    transition: all 0.25s ease;
}

.toast-enter-from {
    opacity: 0;
    transform: translateX(100%);
}

.toast-leave-to {
    opacity: 0;
    transform: translateX(100%);
}
</style>
