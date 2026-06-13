<script setup lang="ts">
import { ref, onMounted, computed } from "vue"
import Breadcrumb from "./components/Breadcrumb.vue"
import { useTabStore } from "./stores/tabStore"
import { useConfirmStore } from "./stores/confirm"
import {
    listGroups,
    listHosts,
    createGroup,
    updateHost,
    renameGroup,
    deleteGroup,
    createHost,
    deleteHost,
} from "./api"
import type { Group, Host } from "./types"
import { Computer, Folder, KeyRound, Plus, SquarePen, X, Network, LayoutGrid, Server, FolderPlus, ArrowRight, PanelRightClose, ShieldCheck } from "lucide-vue-next"
import PasswordManager from "./views/password-manager/PasswordManager.vue"
import { usePasswordManagerStore } from "./stores/passwordManager"
import FormPasswordManager from "./views/password-manager/FormPasswordManager.vue"
import { useToastStore } from "./stores/toastStore"
import TunnelManager from "./views/ssh-tunnel/TunnelManager.vue"

const confirm = useConfirmStore()
const tabStore = useTabStore()
const toast = useToastStore()

const currentParentId = ref<number | null>(null)
const breadcrumb = ref<Group[]>([])
const groups = ref<Group[]>([])
const hosts = ref<Host[]>([])

async function load() {
    groups.value = await listGroups(currentParentId.value)
    hosts.value = await listHosts(currentParentId.value)
}

async function enterGroup(group: Group) {
    breadcrumb.value.push(group)
    currentParentId.value = group.id
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

const showGroupForm = ref(false)
const isEditingGroup = ref(false)
const editingGroupId = ref<number | null>(null)
const groupFormName = ref("")

function openCreateGroupForm() {
    groupFormName.value = ""
    isEditingGroup.value = false
    editingGroupId.value = null
    showGroupForm.value = true
    formHost.value = { id: 0, name: "", host: "", port: 22, username: "", authType: "password", passwordId: 0 }
    showHostForm.value = false
}

function openEditGroupForm(group: Group) {
    groupFormName.value = group.name
    isEditingGroup.value = true
    editingGroupId.value = group.id
    showGroupForm.value = true
    showHostForm.value = false
}

function closeGroupForm() {
    showGroupForm.value = false
    isEditingGroup.value = false
    editingGroupId.value = null
    groupFormName.value = ""
}

async function submitGroupForm() {
    if (!groupFormName.value.trim()) return
    try {
        if (isEditingGroup.value && editingGroupId.value !== null) {
            await renameGroup(editingGroupId.value, groupFormName.value)
            toast.success("Folder renamed")
        } else {
            await createGroup(groupFormName.value, currentParentId.value)
            toast.success("Folder created")
        }
        closeGroupForm()
        await load()
    } catch (e: any) {
        toast.error(`Unable to save folder: ${e}`)
    }
}

async function removeGroup(group: Group) {
    const ok = await confirm.confirm({
        title: "Delete Folder",
        message: `Are you sure you want to delete folder "${group.name}"?`,
    })
    if (!ok) return
    try {
        await deleteGroup(group.id)
        toast.success("Folder deleted")
        await load()
    } catch (e: any) {
        toast.error(`Unable to delete folder: ${e}`)
    }
}

const showHostForm = ref(false)
const emptyHostForm = () => ({
    id: 0,
    name: "",
    host: "",
    port: 22,
    username: "",
    authType: "password",
    passwordId: 0,
})
const formHost = ref(emptyHostForm())

function openCreateHostForm() {
    formHost.value = emptyHostForm()
    showHostForm.value = true
    closeGroupForm()
}

async function addHost() {
    try {
        await createHost({ ...formHost.value, groupId: currentParentId.value })
        toast.success("Host added")
        showHostForm.value = false
        formHost.value = emptyHostForm()
        await load()
    } catch (e: any) {
        toast.error(`Unable to add host: ${e}`)
    }
}

function startEditHost(host: Host) {
    formHost.value = {
        id: host.id,
        name: host.name,
        host: host.host,
        port: host.port,
        username: host.username,
        authType: host.auth_type,
        passwordId: host.password_id || 0,
    }
    showHostForm.value = true
    closeGroupForm()
}

async function saveEditHost() {
    try {
        await updateHost({
            id: formHost.value.id,
            name: formHost.value.name,
            host: formHost.value.host,
            port: formHost.value.port,
            username: formHost.value.username,
            auth_type: formHost.value.authType,
            group_id: currentParentId.value,
            password_id: formHost.value.passwordId,
        })
        showHostForm.value = false
        formHost.value = emptyHostForm()
        toast.success("Host updated")
        await load()
    } catch (e: any) {
        toast.error(`Unable to update host: ${e}`)
    }
}

async function handleDeleteHost(host: Host) {
    const ok = await confirm.confirm({
        title: "Delete Host",
        message: `Are you sure you want to delete host "${host.name}"?`,
    })
    if (!ok) return
    try {
        await deleteHost(host.id)
        toast.success("Host deleted")
        await load()
    } catch (e: any) {
        toast.error(`Unable to delete host: ${e}`)
    }
}

const passwordManagerStore = usePasswordManagerStore()
const activeSection = ref<"hosts" | "passwords" | "tunnels">("hosts")
const pageTitle = computed(() => ({
    hosts: breadcrumb.value.length ? breadcrumb.value[breadcrumb.value.length - 1].name : "All connections",
    passwords: "Password vault",
    tunnels: "SSH tunnels",
})[activeSection.value])
const pageDescription = computed(() => ({
    hosts: "Organize servers and start a secure SSH session.",
    passwords: "Keep reusable credentials encrypted and close at hand.",
    tunnels: "Manage local forwarding and SOCKS5 proxy connections.",
})[activeSection.value])

function switchSection(section: "hosts" | "passwords" | "tunnels") {
    activeSection.value = section
    showHostForm.value = false
    closeGroupForm()
    passwordManagerStore.closeFormManagePassword()
}

onMounted(() => {
    load()
    passwordManagerStore.load()
})
</script>

<template>
    <div class="flex h-full overflow-hidden bg-[#080c13]">
        <aside class="flex w-56 shrink-0 flex-col border-r border-white/6 bg-[#090e16] p-3">
            <div class="px-3 pb-3 pt-2 text-[10px] font-bold uppercase tracking-[.18em] text-slate-600">Workspace</div>
            <nav class="space-y-1">
                <button v-for="item in [
                    { id: 'hosts', label: 'Connections', icon: LayoutGrid },
                    { id: 'passwords', label: 'Password vault', icon: KeyRound },
                    { id: 'tunnels', label: 'SSH tunnels', icon: Network },
                ]" :key="item.id" @click="switchSection(item.id as 'hosts' | 'passwords' | 'tunnels')"
                    class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition"
                    :class="activeSection === item.id ? 'bg-cyan-400/10 text-cyan-300' : 'text-slate-500 hover:bg-white/4 hover:text-slate-200'">
                    <component :is="item.icon" :size="17" />{{ item.label }}
                </button>
            </nav>
            <div class="mt-auto rounded-xl border border-white/6 bg-white/[.025] p-3">
                <div class="mb-2 flex items-center gap-2 text-xs font-semibold text-slate-300"><ShieldCheck :size="15" class="text-emerald-400" /> Local-first security</div>
                <p class="text-[11px] leading-relaxed text-slate-600">Credentials and connection data stay on this device.</p>
            </div>
        </aside>

        <main class="min-w-0 flex-1 overflow-y-auto">
            <header class="sticky top-0 z-20 flex min-h-24 items-center justify-between border-b border-white/6 bg-[#080c13]/90 px-7 py-5 backdrop-blur-xl">
                <div><h1 class="text-xl font-semibold tracking-tight text-slate-100">{{ pageTitle }}</h1><p class="mt-1 text-xs text-slate-500">{{ pageDescription }}</p></div>
                <div class="flex items-center gap-2">
                    <template v-if="activeSection === 'hosts'">
                        <button @click="openCreateGroupForm" class="secondary-button flex items-center gap-2 rounded-lg px-3 py-2 text-xs"><FolderPlus :size="15" /> New folder</button>
                        <button @click="openCreateHostForm" class="primary-button flex items-center gap-2 rounded-lg px-3 py-2 text-xs"><Plus :size="15" /> New host</button>
                    </template>
                    <button v-else-if="activeSection === 'passwords'" @click="passwordManagerStore.openFormManagePassword()" class="primary-button flex items-center gap-2 rounded-lg px-3 py-2 text-xs"><Plus :size="15" /> New credential</button>
                </div>
            </header>

            <div class="p-7">
                <template v-if="activeSection === 'hosts'">
                    <Breadcrumb v-if="breadcrumb.length > 0" :items="breadcrumb" @navigate="navigateBreadcrumb" />
                    <section v-if="groups.length" class="mb-8">
                        <div class="mb-3 flex items-center justify-between"><h2 class="text-xs font-bold uppercase tracking-[.14em] text-slate-500">Folders</h2><span class="text-[11px] text-slate-600">{{ groups.length }} total</span></div>
                        <div class="grid grid-cols-[repeat(auto-fill,minmax(210px,1fr))] gap-3">
                            <article v-for="g in groups" :key="g.id" @click="enterGroup(g)" class="surface surface-hover group flex cursor-pointer items-center gap-3 rounded-xl p-4">
                                <div class="grid size-10 shrink-0 place-items-center rounded-lg bg-amber-400/10 text-amber-300"><Folder :size="19" /></div>
                                <div class="min-w-0 flex-1"><div class="truncate text-sm font-semibold text-slate-200">{{ g.name }}</div><div class="mt-1 text-[11px] text-slate-600">Connection folder</div></div>
                                <div class="flex opacity-0 transition group-hover:opacity-100">
                                    <button class="icon-button size-8" title="Rename folder" @click.stop="openEditGroupForm(g)"><SquarePen :size="15" /></button>
                                    <button class="icon-button size-8 hover:!text-rose-400" title="Delete folder" @click.stop="removeGroup(g)"><X :size="16" /></button>
                                </div>
                            </article>
                        </div>
                    </section>
                    <section>
                        <div class="mb-3 flex items-center justify-between"><h2 class="text-xs font-bold uppercase tracking-[.14em] text-slate-500">Hosts</h2><span class="text-[11px] text-slate-600">{{ hosts.length }} available</span></div>
                        <div v-if="hosts.length" class="grid grid-cols-[repeat(auto-fill,minmax(245px,1fr))] gap-3">
                            <article v-for="h in hosts" :key="h.id" @click="tabStore.openTab(h)" class="surface surface-hover group cursor-pointer rounded-xl p-4">
                                <div class="mb-5 flex items-start justify-between"><div class="grid size-10 place-items-center rounded-lg bg-cyan-400/10 text-cyan-300"><Server :size="19" /></div><div class="flex opacity-0 transition group-hover:opacity-100"><button class="icon-button size-8" title="Edit host" @click.stop="startEditHost(h)"><SquarePen :size="15" /></button><button class="icon-button size-8 hover:!text-rose-400" title="Delete host" @click.stop="handleDeleteHost(h)"><X :size="16" /></button></div></div>
                                <div class="truncate text-sm font-semibold text-slate-100">{{ h.name }}</div>
                                <div class="mt-1 truncate font-mono text-[11px] text-slate-500">{{ h.username }}@{{ h.host }}:{{ h.port }}</div>
                                <div class="mt-4 flex items-center justify-between border-t border-white/6 pt-3"><span class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider text-emerald-400"><span class="size-1.5 rounded-full bg-emerald-400"></span> Ready</span><span class="flex items-center gap-1 text-[11px] font-medium text-slate-500 group-hover:text-cyan-300">Connect <ArrowRight :size="13" /></span></div>
                            </article>
                        </div>
                        <div v-else class="surface grid min-h-56 place-items-center rounded-2xl border-dashed text-center"><div><div class="mx-auto mb-3 grid size-11 place-items-center rounded-xl bg-white/5 text-slate-500"><Computer :size="20" /></div><p class="text-sm font-medium text-slate-300">No hosts in this folder</p><p class="mt-1 text-xs text-slate-600">Add a host to start an SSH session.</p></div></div>
                    </section>
                </template>
                <PasswordManager v-else-if="activeSection === 'passwords'" />
                <TunnelManager v-else />
            </div>
        </main>

        <FormPasswordManager />
        <Transition name="slide">
            <aside v-if="showHostForm" class="form-panel w-[350px] shrink-0 overflow-y-auto border-l border-white/8 bg-[#0b111a] p-6">
                <div class="mb-7 flex items-start justify-between"><div><p class="text-base font-semibold text-slate-100">{{ formHost.id === 0 ? 'New host' : 'Edit host' }}</p><p class="mt-1 text-xs text-slate-600">SSH connection details</p></div><button class="icon-button size-8" @click="showHostForm = false"><PanelRightClose :size="18" /></button></div>
                <div class="space-y-4">
                    <div><label class="panel-label">Display name</label><input v-model="formHost.name" class="field" placeholder="Production server" /></div>
                    <div><label class="panel-label">Host or IP address</label><input v-model="formHost.host" class="field font-mono" placeholder="192.168.1.10" /></div>
                    <div class="grid grid-cols-[1fr_95px] gap-3"><div><label class="panel-label">Username</label><input v-model="formHost.username" class="field" placeholder="root" /></div><div><label class="panel-label">Port</label><input v-model.number="formHost.port" type="number" class="field" /></div></div>
                    <div><label class="panel-label">Authentication</label><select v-model="formHost.authType" class="field"><option value="none">No authentication</option><option value="password">Password</option><option value="key">SSH key</option></select></div>
                    <div v-if="formHost.authType === 'password'"><label class="panel-label">Saved credential</label><select v-model="formHost.passwordId" class="field"><option :value="0">Select credential</option><option v-for="password in passwordManagerStore.passwords" :key="password.id" :value="password.id">{{ password.name }}</option></select></div>
                    <button @click="formHost.id === 0 ? addHost() : saveEditHost()" class="primary-button mt-3 w-full rounded-lg py-2.5 text-sm">{{ formHost.id === 0 ? 'Create host' : 'Save changes' }}</button>
                </div>
            </aside>
        </Transition>
        <Transition name="slide">
            <aside v-if="showGroupForm" class="form-panel w-[350px] shrink-0 border-l border-white/8 bg-[#0b111a] p-6">
                <div class="mb-7 flex items-start justify-between"><div><p class="text-base font-semibold text-slate-100">{{ isEditingGroup ? 'Rename folder' : 'New folder' }}</p><p class="mt-1 text-xs text-slate-600">Keep related connections together</p></div><button class="icon-button size-8" @click="closeGroupForm"><PanelRightClose :size="18" /></button></div>
                <label class="panel-label">Folder name</label><input id="group-name-input" v-model="groupFormName" class="field" placeholder="Production" @keyup.enter="submitGroupForm" />
                <button @click="submitGroupForm" class="primary-button mt-5 w-full rounded-lg py-2.5 text-sm">{{ isEditingGroup ? 'Save changes' : 'Create folder' }}</button>
            </aside>
        </Transition>
    </div>
</template>

<style scoped>
.slide-enter-active, .slide-leave-active { transition: all .22s ease; }
.slide-enter-from, .slide-leave-to { transform: translateX(100%); opacity: 0; }
</style>
