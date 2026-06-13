<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { usePasswordManagerStore } from '../../stores/passwordManager';
import { SquarePen, Trash, Copy, CheckCircle, KeyRound } from 'lucide-vue-next';
import { useToastStore } from '../../stores/toastStore';
import { copyPassword } from '../../api';

const toast = useToastStore()

const passwordManagerStore = usePasswordManagerStore()
onMounted(() => {
    passwordManagerStore.load()
})


const idxCopied = ref(-1)

const copyToClipboard = async (id: number, idx: number) => {
    try {
        await copyPassword(id)

        idxCopied.value = idx
        toast.success('copied to clipboard')

        setTimeout(() => {
            idxCopied.value = -1
        }, 1500)

    } catch (err) {
        console.error(err)
    }
}



</script>
<template>
    <section>
        <div v-if="passwordManagerStore.passwords.length" class="surface overflow-hidden rounded-xl">
            <div class="grid grid-cols-[1.2fr_1fr_100px] border-b border-white/6 px-5 py-3 text-[10px] font-bold uppercase tracking-[.14em] text-slate-600">
                <span>Credential</span><span>Secret</span><span class="text-right">Actions</span>
            </div>
            <div v-for="(p, i) in passwordManagerStore.passwords" :key="p.id"
                class="grid grid-cols-[1.2fr_1fr_100px] items-center border-b border-white/5 px-5 py-4 last:border-0 hover:bg-white/[.025]">
                <div class="flex items-center gap-3"><div class="grid size-9 place-items-center rounded-lg bg-violet-400/10 text-violet-300"><KeyRound :size="16" /></div><div><p class="text-sm font-semibold text-slate-200">{{ p.name }}</p><p class="mt-0.5 text-[11px] text-slate-600">Encrypted credential</p></div></div>
                <button class="flex w-fit items-center gap-2 rounded-lg border border-white/6 bg-black/15 px-3 py-2 font-mono text-xs text-slate-500 hover:border-cyan-400/20 hover:text-cyan-300" @click="copyToClipboard(p.id, i)">
                    <span>••••••••••••</span><CheckCircle v-if="idxCopied === i" :size="14" class="text-emerald-400" /><Copy v-else :size="14" />
                </button>
                <div class="flex justify-end gap-1"><button class="icon-button size-8" title="Edit" @click="passwordManagerStore.openEditFormManagePassword(p)"><SquarePen :size="15" /></button><button class="icon-button size-8 hover:!text-rose-400" title="Delete" @click="passwordManagerStore.handleDeletePassword(p)"><Trash :size="15" /></button></div>
            </div>
        </div>
        <div v-else class="surface grid min-h-56 place-items-center rounded-2xl border-dashed text-center"><div><div class="mx-auto mb-3 grid size-11 place-items-center rounded-xl bg-violet-400/10 text-violet-300"><KeyRound :size="20" /></div><p class="text-sm font-medium text-slate-300">Your vault is empty</p><p class="mt-1 text-xs text-slate-600">Create a credential to reuse it across hosts.</p></div></div>
    </section>


</template>
