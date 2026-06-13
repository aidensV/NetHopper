import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Password } from '../types'
import { listPasswords, deletePassword } from '../api'
import { reactive } from 'vue'
import { useConfirmStore } from './confirm'


export const usePasswordManagerStore = defineStore('passwordManager', () => {

    const passwords = ref<Password[]>([])
    const confirm = useConfirmStore()


    const formPassword = reactive({
        id: 0,
        name: '',
        password: '',
    })

    async function load() {
        passwords.value = await listPasswords()

    }

    const showFormManagePassword = ref(false)

    function openFormManagePassword() {
        clearFormPassword()
        showFormManagePassword.value = true
    }

    function closeFormManagePassword() {
        clearFormPassword()
        showFormManagePassword.value = false
    }

    function toggleFormManagePassword() {
        showFormManagePassword.value = !showFormManagePassword.value
    }

    const openEditFormManagePassword = (password: Password) => {
        console.log("YY");

        formPassword.id = password.id
        formPassword.name = password.name
        formPassword.password = password.password
        showFormManagePassword.value = true
    }

    const handleDeletePassword = async (password: Password) => {
        const ok = await confirm.confirm({
            title: "Delete Password",
            message: `Are you sure you want to delete password "${password.name}"?`,
        })

        if (!ok) return

        try {
            await deletePassword(password.id)
            await load()
        } catch (e: any) {
            alert(e)
        }
    }

    const clearFormPassword = () => {
        formPassword.id = 0
        formPassword.name = ''
        formPassword.password = ''
    }

    return {
        passwords,
        load,
        showFormManagePassword,
        openFormManagePassword,
        closeFormManagePassword,
        toggleFormManagePassword,
        openEditFormManagePassword,
        formPassword,
        clearFormPassword,
        handleDeletePassword,
    }
})