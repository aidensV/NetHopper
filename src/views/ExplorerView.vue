<script setup>
import { ref, onMounted } from "vue";
import {
    listGroups,
    listHosts,
    createGroup,
    createHost
} from "../api/backend";

/* STATE */
const currentGroupId = ref(null);
const groups = ref([]);
const hosts = ref([]);

/* LOAD DATA */
async function loadGroups() {
    groups.value = await listGroups(currentGroupId.value);
}

async function loadHosts() {
    if (currentGroupId.value === null) {
        hosts.value = [];
        return;
    }
    hosts.value = await listHosts(currentGroupId.value);
}

async function reload() {
    await loadGroups();
    await loadHosts();
}

/* ACTIONS */
async function onCreateGroup() {
    const name = prompt("Group name");
    if (!name) return;

    await createGroup(name, currentGroupId.value);
    reload();
}

async function onCreateHost() {
    if (currentGroupId.value === null) {
        alert("Select group first");
        return;
    }

    const name = prompt("Host name");
    const address = prompt("Host address (IP / hostname)");

    if (!name || !address) return;

    await createHost({
        groupId: currentGroupId.value,
        name,
        address,
        port: 22
    });

    reload();
}

/* NAVIGATION */
function enterGroup(groupId) {
    currentGroupId.value = groupId;
    reload();
}

function goRoot() {
    currentGroupId.value = null;
    reload();
}

onMounted(reload);
</script>

<template>
    <div class="h-screen p-4 bg-gray-50">
        <div class="mb-4 flex items-center gap-4">
            <button class="text-sm text-blue-600" @click="goRoot">
                Root
            </button>

            <button class="px-3 py-1 bg-blue-600 text-white text-sm rounded" @click="onCreateGroup">
                + Group
            </button>

            <button class="px-3 py-1 bg-green-600 text-white text-sm rounded" @click="onCreateHost">
                + Host
            </button>
        </div>

        <div class="grid grid-cols-2 gap-4">
            <!-- GROUP LIST -->
            <div class="bg-white rounded shadow p-3">
                <h2 class="font-semibold mb-2">Groups</h2>
                <ul class="space-y-1">
                    <li v-for="g in groups" :key="g.id" class="px-2 py-1 rounded hover:bg-gray-100 cursor-pointer"
                        @click="enterGroup(g.id)">
                        📁 {{ g.name }}
                    </li>
                </ul>
            </div>

            <!-- HOST LIST -->
            <div class="bg-white rounded shadow p-3">
                <h2 class="font-semibold mb-2">Hosts</h2>
                <ul class="space-y-1">
                    <li v-for="h in hosts" :key="h.id" class="px-2 py-1 border rounded text-sm">
                        <div class="font-medium">{{ h.name }}</div>
                        <div class="text-gray-500">
                            {{ h.address }}:{{ h.port }}
                        </div>
                    </li>
                </ul>
            </div>
        </div>
    </div>
</template>
