<script setup lang="ts">
import type { Group } from "../types"
import { ChevronRight, Home } from "lucide-vue-next"

defineProps<{
    items: Group[]
}>()

const emit = defineEmits<{
    (e: "navigate", index: number): void
}>()
</script>

<template>
    <nav class="mb-6 flex items-center gap-1 overflow-x-auto text-xs">
        <!-- HOME -->
        <button @click="emit('navigate', -1)"
            class="group flex items-center gap-2 rounded-lg px-2 py-1.5 font-medium text-slate-500 transition hover:bg-white/5 hover:text-slate-200">
            <Home :size="14" /><span>Connections</span>
        </button>

        <!-- ITEMS -->
        <template v-for="(g, i) in items" :key="g.id">
            <!-- separator -->
            <ChevronRight :size="13" class="shrink-0 text-slate-700" />

            <!-- item -->
            <button @click="emit('navigate', i)"
                class="shrink-0 rounded-lg px-2 py-1.5 font-medium transition"
                :class="i === items.length - 1
                    ? 'bg-white/5 text-slate-200'
                    : 'text-slate-500 hover:bg-white/5 hover:text-slate-200'
                    ">
                {{ g.name }}
            </button>
        </template>
    </nav>
</template>
