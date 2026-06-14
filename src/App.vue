<script setup lang="ts">
import Index from "./Index.vue"
import SshTerminal from "./components/SshTerminal.vue"
import { useTabStore } from "./stores/tabStore"
import { Computer, X, House, Orbit } from "lucide-vue-next"
import ConfirmDialog from './components/ConfirmDialog.vue'
import Toast from './components/Toast.vue'
import AboutDialog from './components/AboutDialog.vue'
import UpdateDialog from './components/UpdateDialog.vue'
import { ref } from 'vue'

const tabStore = useTabStore()
const showAbout = ref(false)

</script>

<template>
  <div class="app-shell flex h-screen flex-col overflow-hidden text-white">

    <!-- Tab Bar -->
    <div class="app-tabs flex h-12 shrink-0 items-end gap-1 border-b border-white/6 px-3">
      <button class="brand-mark mr-2 flex h-10 shrink-0 items-center gap-2 rounded-lg px-2 transition hover:bg-white/4"
        title="About NetHopper" @click="showAbout = true">
        <span
          class="grid size-7 place-items-center rounded-lg bg-cyan-400 text-slate-950 shadow-[0_0_24px_rgba(34,211,238,.2)]">
          <Orbit :size="17" :stroke-width="2.4" />
        </span>
        <span class="text-sm font-semibold tracking-tight">NetHopper</span>
        <span class="rounded bg-white/6 px-1.5 py-0.5 text-[9px] font-bold text-slate-500">0.1.2</span>
      </button>
      <!-- Tab Home (tidak bisa di-close) -->
      <div class="terminal-tab flex h-9 items-center gap-2 rounded-t-lg border border-b-0 px-3 text-xs font-medium"
        :class="tabStore.activeTabId === null ? 'terminal-tab-active' : 'text-slate-500 hover:text-slate-200'"
        @click="tabStore.activeTabId = null">
        <House :size="14" />
        Workspace
      </div>

      <!-- Tab SSH -->
      <div v-for="tab in tabStore.tabs" :key="tab.id"
        class="terminal-tab flex h-9 max-w-48 items-center gap-2 rounded-t-lg border border-b-0 px-3 text-xs font-medium"
        :class="tabStore.activeTabId === tab.id
          ? 'terminal-tab-active'
          : 'text-slate-500 hover:text-slate-200'" @click="tabStore.activeTabId = tab.id">
        <span class="size-1.5 shrink-0 rounded-full bg-emerald-400"></span>
        <Computer :size="14" class="shrink-0" />
        <span class="truncate">{{ tab.host.name }}</span>
        <button class="ml-1 rounded p-0.5 text-slate-600 hover:bg-white/8 hover:text-rose-400"
          @click.stop="tabStore.closeTab(tab.id)">
          <X :size="14" />
        </button>
      </div>

    </div>

    <!-- Konten Full -->
    <div class="flex-1 min-h-0 overflow-hidden">

      <!-- Home -->
      <Index v-show="tabStore.activeTabId === null" class="h-full" />

      <!-- Terminal per tab -->
      <template v-for="tab in tabStore.tabs" :key="tab.id">
        <SshTerminal v-show="tabStore.activeTabId === tab.id" :host="tab.host" class="h-full" />
      </template>

    </div>

  </div>

  <ConfirmDialog />
  <Toast />
  <AboutDialog :open="showAbout" @close="showAbout = false" />
  <UpdateDialog />
</template>
