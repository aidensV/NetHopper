<script setup lang="ts">
import { useConfirmStore } from '../stores/confirm'

const confirm = useConfirmStore()
</script>

<template>
    <Teleport to="body">
        <div v-if="confirm.open"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
            <div class="surface w-full max-w-sm rounded-2xl p-6 shadow-2xl">
                <h2 class="text-lg font-semibold text-white">
                    {{ confirm.title }}
                </h2>

                <p class="mt-2 text-sm text-zinc-300">
                    {{ confirm.message }}
                </p>

                <div class="mt-6 flex justify-end gap-2">
                    <button @click="confirm.onCancel"
                    class="secondary-button rounded-lg px-4 py-2 text-sm">
                        Cancel
                    </button>

                    <button @click="confirm.onConfirm"
                    class="rounded-lg bg-rose-500 px-4 py-2 text-sm font-semibold text-white transition hover:bg-rose-400">
                        Confirm
                    </button>
                </div>
            </div>
        </div>
    </Teleport>
</template>
