<template>
    <div class="flex h-screen overflow-hidden">

        <!-- LEFT (dynamic) -->
        <div class="flex-1 bg-gray-100 p-4">
            <button class="px-4 py-2 bg-blue-600 text-white rounded" @click="showForm = true">
                Add
            </button>

            <!-- list / table -->
        </div>

        <!-- RIGHT (fixed, stable) -->
        <Transition name="slide">
            <aside v-if="showForm" class="w-[320px] shrink-0 bg-white border-l p-4">
                <button class="text-sm text-gray-500 mb-2" @click="showForm = false">
                    Close
                </button>

                <!-- form -->
            </aside>
        </Transition>

    </div>
</template>


<script setup>
import { ref } from 'vue'

const showForm = ref(false)
</script>
<style>
.slide-enter-active,
.slide-leave-active {
    transition: all 0.3s ease;
}

.slide-enter-from {
    transform: translateX(100%);
    opacity: 0;
}

.slide-leave-to {
    transform: translateX(100%);
    opacity: 0;
}
</style>
