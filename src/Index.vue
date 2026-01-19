<script setup lang="ts">
import { ref, onMounted, reactive } from "vue"
import Breadcrumb from "./components/Breadcrumb.vue"
import { listGroups, listHosts, createGroup, createHost, updateHost, renameGroup, deleteGroup, sshExec } from "./api"
import type { Group, Host } from "./types"
import SshTerminal from "./components/sshterminal.vue";

const currentParentId = ref<number | null>(null)
const breadcrumb = ref<Group[]>([])
const groups = ref<Group[]>([])
const hosts = ref<Host[]>([])
const newGroupName = ref("")

const selectedHost = ref<Host | null>(null)
const command = ref("")
const output = ref("")
const running = ref(false)

async function load() {
    groups.value = await listGroups(currentParentId.value)
    hosts.value = await listHosts(currentParentId.value)
}
const editingGroupId = ref<number | null>(null)
const editGroupName = ref("")

async function enterGroup(group: Group) {
    breadcrumb.value.push(group)
    currentParentId.value = group.id
    await load()
}

async function goBack() {
    breadcrumb.value.pop()
    currentParentId.value =
        breadcrumb.value.length > 0
            ? breadcrumb.value[breadcrumb.value.length - 1].id
            : null
    await load()
}

async function navigateBreadcrumb(index: number) {
    if (index === -1) {
        breadcrumb.value = []
        currentParentId.value = null
    } else {
        breadcrumb.value = breadcrumb.value.slice(0, index + 1)
        currentParentId.value = breadcrumb.value[index].id
    }
    await load()
}

async function addGroup() {
    if (!newGroupName.value) return
    await createGroup(newGroupName.value, currentParentId.value)
    newGroupName.value = ""
    await load()
}

const newHost = ref({
    name: "",
    host: "",
    port: 22,
    username: "",
    authType: "password",
    password: "",
})

async function addHost() {
    await createHost({
        ...newHost.value,
        groupId: currentParentId.value,
    })

    newHost.value = {
        name: "",
        host: "",
        port: 22,
        username: "",
        authType: "password",
        password: "",
    }

    await load()
}

const editingHostId = ref<number | null>(null)

const editHost = ref({
    id: 0,
    name: "",
    host: "",
    port: 22,
    username: "",
    authType: "password",
    password: "",
})

function startEditHost(h: Host) {
    editingHostId.value = h.id
    editHost.value = {
        id: h.id,
        name: h.name,
        host: h.host,
        port: h.port,
        username: h.username,
        authType: h.auth_type,
        password: h.password || "",
    }
}

async function saveEditHost() {
    await updateHost({
        id: editHost.value.id,
        name: editHost.value.name,
        host: editHost.value.host,
        port: editHost.value.port,
        username: editHost.value.username,
        auth_type: editHost.value.authType,
        group_id: currentParentId.value,
        password: editHost.value.password,
    })

    editingHostId.value = null
    await load()
}

function cancelEditHost() {
    editingHostId.value = null
}

function startRenameGroup(g: Group) {
    editingGroupId.value = g.id
    editGroupName.value = g.name
}

async function saveRenameGroup() {
    if (!editingGroupId.value) return

    await renameGroup(editingGroupId.value, editGroupName.value)
    editingGroupId.value = null
    editGroupName.value = ""
    await load()
}

function cancelRenameGroup() {
    editingGroupId.value = null
}

async function removeGroup(g: Group) {
    if (!confirm(`Delete group "${g.name}"?`)) return

    try {
        await deleteGroup(g.id)
        await load()
    } catch (e: any) {
        alert(e)
    }
}

async function runCommand() {
    if (!selectedHost.value || !command.value) return

    running.value = true
    output.value = ""

    try {
        output.value = await sshExec(
            selectedHost.value.id,
            command.value
        )
    } catch (e: any) {
        output.value = `ERROR:\n${e}`
    } finally {
        running.value = false
    }
}
const showForm = reactive({
    formHost: false,
    formGroup: false,
})

onMounted(load)
</script>

<template>

    <div class="flex overflow-hidden">
        <!-- LEFT (dynamic) -->
        <div class="flex-1 p-4">
            <div class="w-full bg-gray-800 px-2 py-2 rounded-lg flex gap-2 mb-4">
                <div @click="showForm.formHost = true"
                    class="text-md text-white border px-2 py-1 rounded w-fit flex gap-2 cursor-pointer hover:bg-gray-700 hover:text-gray-200">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5"
                        stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="m6.75 7.5 3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0 0 21 18V6a2.25 2.25 0 0 0-2.25-2.25H5.25A2.25 2.25 0 0 0 3 6v12a2.25 2.25 0 0 0 2.25 2.25Z" />
                    </svg>

                    New Host
                </div>
                <div @click="showForm.formGroup = true"
                    class="text-md text-white border px-2 py-1 flex gap-2 rounded w-fit cursor-pointer hover:bg-gray-700 hover:text-gray-200">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5"
                        stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
                    </svg>
                    New Group
                </div>
            </div>
            <!-- <button class="px-4 py-2 bg-blue-600 text-white rounded" @click="showForm = true">
                Add
            </button> -->

            <Breadcrumb v-if="breadcrumb.length > 0" :items="breadcrumb" @navigate="navigateBreadcrumb" />


            <!-- Back -->
            <button v-if="breadcrumb.length > 0" @click="goBack" class="border px-2 py-1 text-sm">
                ← Back
            </button>

            <!-- Create group -->


            <!-- Groups -->
            <div>
                <h2 class="font-semibold text-md mb-2">Groups</h2>
                <div class="flex flex-wrap gap-2">
                    <div v-for="g in groups" :key="g.id"
                        class="group flex justify-between w-52 items-center bg-gray-800 border text-gray-300 p-4 mb-1 rounded-md border-gray-800 hover:bg-gray-700">
                        <div v-if="editingGroupId !== g.id" class="cursor-pointer flex gap-2 hover:text-green-500"
                            @click="enterGroup(g)">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5"
                                stroke="currentColor" class="size-6">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
                            </svg>
                            {{ g.name }}
                        </div>

                        <div class="opacity-0 group-hover:opacity-100 flex gap-2 text-sm ">
                            <button @click.stop="startRenameGroup(g)">
                                <div class="cursor-pointer hover:text-green-500">
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" stroke-width="1.5"
                                        stroke="currentColor" class="size-6">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L6.832 19.82a4.5 4.5 0 0 1-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 0 1 1.13-1.897L16.863 4.487Zm0 0L19.5 7.125" />
                                    </svg>

                                </div>
                            </button>
                            <button @click.stop="removeGroup(g)">
                                <div class="cursor-pointer hover:text-red-500">
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                                        stroke-width="1.5" stroke="currentColor" class="size-6">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                                    </svg>
                                </div>
                            </button>
                        </div>
                    </div>
                </div>
                <!-- <ul>
                    <li v-for="g in groups" :key="g.id" class="flex justify-between items-center border p-2 mb-1">
                        <div v-if="editingGroupId !== g.id" class="cursor-pointer" @click="enterGroup(g)">
                            📁 {{ g.name }}
                        </div>

                        <div v-else class="flex gap-2 w-full">
                            <input v-model="editGroupName" class="border px-2 py-1 flex-1" />
                            <button @click="saveRenameGroup" class="border px-2">Save</button>
                            <button @click="cancelRenameGroup" class="border px-2">Cancel</button>
                        </div>

                        <div class="flex gap-2 text-sm">
                            <button @click.stop="startRenameGroup(g)">✏️</button>
                            <button @click.stop="removeGroup(g)">🗑</button>
                        </div>
                    </li>
                </ul> -->

            </div>
            <div>
                <h2 class="font-semibold">Hosts</h2>
                <!-- <div class="border p-3 space-y-2">
            <h3 class="font-semibold">Create Host</h3>

            <input v-model="newHost.name" placeholder="Name" class="border px-2 py-1 w-full" />
            <input v-model="newHost.host" placeholder="Host / IP" class="border px-2 py-1 w-full" />
            <input v-model.number="newHost.port" type="number" class="border px-2 py-1 w-full" />
            <input v-model="newHost.username" placeholder="Username" class="border px-2 py-1 w-full" />
            <input v-model="newHost.password" placeholder="Password" class="border px-2 py-1 w-full" />

            <select v-model="newHost.authType" class="border px-2 py-1 w-full">
                <option value="password">Password</option>
                <option value="key">SSH Key</option>
            </select>

            <button @click="addHost" class="border px-3 py-1">
                Add Host
            </button>
        </div> -->

                <ul>
                    <li v-for="h in hosts" :key="h.id" @click="selectedHost = h" class="border p-2 mb-2">
                        <!-- VIEW MODE -->
                        <div v-if="editingHostId !== h.id" class="flex justify-between">
                            <div>
                                🖥 {{ h.name }} ({{ h.username }}@{{ h.host }}:{{ h.port }})
                            </div>
                            <button @click="startEditHost(h)" class="text-sm underline">
                                ✏️ Edit
                            </button>
                        </div>

                        <!-- EDIT MODE -->
                        <div v-else class="space-y-2">
                            <input v-model="editHost.name" class="border px-2 py-1 w-full" />
                            <input v-model="editHost.host" class="border px-2 py-1 w-full" />
                            <input v-model.number="editHost.port" type="number" class="border px-2 py-1 w-full" />
                            <input v-model="editHost.username" class="border px-2 py-1 w-full" />

                            <select v-model="editHost.authType" class="border px-2 py-1 w-full">
                                <option value="password">Password</option>
                                <option value="key">SSH Key</option>
                            </select>

                            <div class="flex gap-2">
                                <button @click="saveEditHost" class="border px-3 py-1">
                                    Save
                                </button>
                                <button @click="cancelEditHost" class="border px-3 py-1">
                                    Cancel
                                </button>
                            </div>
                        </div>
                    </li>
                    <div v-if="selectedHost" class="border p-3">
                        <SshTerminal :host="selectedHost" />
                    </div>

                </ul>

            </div>

            <!-- list / table -->
        </div>

        <!-- RIGHT (fixed, stable) -->
        <Transition name="slide">
            <aside v-if="showForm.formGroup" class="w-[320px] shrink-0 bg-gray-900 border-l p-4">
                <div class="flex bg-gray-800 p-2 rounded-lg items-center mb-8">
                    <button class="text-sm text-gray-500 hover:text-white cursor-pointer"
                        @click="showForm.formGroup = false">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5"
                            stroke="currentColor" class="size-6">
                            <path stroke-linecap="round" stroke-linejoin="round"
                                d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
                        </svg>
                    </button>
                    <div class="flex-1 flex justify-center">
                        <span class="text-lg font-semibold">Create New Group</span>
                    </div>
                </div>

                <!-- form -->
                <div class="">

                    <form class="max-w-sm mx-auto">
                        <label for="input-group-1" class="block mb-2.5 text-md font-medium text-heading">Group
                            Name</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 start-0 flex items-center ps-3 pointer-events-none w-8 ">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                                    stroke-width="1.5" stroke="currentColor" class="size-6">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
                                </svg>


                            </div>
                            <input v-model="newGroupName" type="text" id="input-group-1"
                                class="block w-full ps-9 pe-3 py-2.5 bg-gray-700 border border-gray-600 text-white text-sm rounded-md placeholder-gray-400 focus:ring-2 focus:ring-blue-600 focus:border-blue-600"
                                placeholder="Group name" />
                        </div>
                    </form>

                    <div class="flex justify-center">

                        <button @click="addGroup"
                            class="border border-gray-600 px-2 py-0.5 mt-8 rounded-md bg-green-700 text-white hover:bg-green-800 cursor-pointer">
                            Create Group
                        </button>
                    </div>
                </div>
            </aside>
        </Transition>

    </div>
</template>

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
