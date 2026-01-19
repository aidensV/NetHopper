<script setup lang="ts">
import type { Group } from "../types"

defineProps<{
    items: Group[]
}>()

const emit = defineEmits<{
    (e: "navigate", index: number): void
}>()
</script>

<template>
    <div class="flex items-center gap-2 text-sm">
        <span class="cursor-pointer text-blue-600" @click="emit('navigate', -1)">
            Root
        </span>

        <template v-for="(g, i) in items" :key="g.id">
            <span>/</span>
            <span class="cursor-pointer text-blue-600" @click="emit('navigate', i)">
                {{ g.name }}
            </span>
        </template>
    </div>
</template>
