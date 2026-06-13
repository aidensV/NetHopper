<script setup lang="ts">
import { usePasswordManagerStore } from '../../stores/passwordManager';
import { ref } from 'vue';
import { Eye, EyeOff, PanelRightClose } from 'lucide-vue-next'

import {
    createPassword,
    updatePassword
} from "../../api"

const passwordManagerStore = usePasswordManagerStore()

const showPassword = ref(false)


async function handleCreatePassword() {
    await createPassword(passwordManagerStore.formPassword.name, passwordManagerStore.formPassword.password)
    passwordManagerStore.closeFormManagePassword()
    passwordManagerStore.load()
}

const handleEditPassword = async () => {
    await updatePassword({
        id: passwordManagerStore.formPassword.id,
        name: passwordManagerStore.formPassword.name,
        password: passwordManagerStore.formPassword.password
    })
    passwordManagerStore.closeFormManagePassword()
    passwordManagerStore.load()
}
</script>

<template>
    <Transition name="slide">
        <aside v-if="passwordManagerStore.showFormManagePassword"
            class="w-[350px] shrink-0 max-h-full overflow-y-auto border-l border-white/8 bg-[#0b111a] p-6">

            <!-- Header -->
            <div class="mb-7 flex items-start justify-between">
                <div><p class="text-base font-semibold text-slate-100">{{ passwordManagerStore.formPassword.id === 0 ? 'New credential' : 'Edit credential' }}</p><p class="mt-1 text-xs text-slate-600">Stored securely on this device</p></div>
                <button class="icon-button size-8" @click="passwordManagerStore.closeFormManagePassword()"><PanelRightClose :size="18" /></button>
            </div>

            <!-- Form -->
            <div class="max-w-sm mx-auto space-y-4">
                <div>
                    <label class="panel-label">Credential name</label>
                    <input v-model="passwordManagerStore.formPassword.name" type="text" placeholder="Name"
                        class="field" />
                </div>

                <div>
                    <label class="panel-label">Password</label>
                    <div class="relative">
                        <input v-model="passwordManagerStore.formPassword.password"
                            :type="showPassword ? 'text' : 'password'" placeholder="Password"
                            class="field pr-10" />

                        <button type="button" @click="showPassword = !showPassword"
                            class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-200">
                            <Eye v-if="showPassword" class="w-5 h-5" />
                            <EyeOff v-else class="w-5 h-5" />
                        </button>
                    </div>
                </div>


                <div class="pt-3">
                    <button
                        @click="passwordManagerStore.formPassword.id == 0 ? handleCreatePassword() : handleEditPassword()"
                        class="primary-button w-full rounded-lg py-2.5 text-sm">
                        {{ passwordManagerStore.formPassword.id == 0 ? 'Create credential' : 'Save changes' }}
                    </button>
                </div>
            </div>
        </aside>
    </Transition>
</template>

<style scoped>
input::-ms-reveal,
input::-ms-clear {
    display: none;
}
</style>
