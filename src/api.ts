import { invoke } from "@tauri-apps/api/core"
import type { Group, Host, Password, SshResponse, Tunnel } from "./types"

export function listGroups(parentId: number | null) {
    return invoke<Group[]>("list_groups_by_parent", {
        parentId,
    })
}

export function listHosts(groupId: number | null) {
    return invoke<Host[]>("list_hosts_by_group", {
        groupId,
    })
}

export function createGroup(name: string, parentId: number | null) {
    return invoke("create_group", {
        name,
        parentId,
    })
}

export async function createHost(payload: {
    name: string
    host: string
    port: number
    username: string
    authType: string
    groupId: number | null
    passwordId: number | null
}) {
    return invoke("create_host", payload)
}

export async function deleteHost(id: number) {
    return invoke("delete_host", { id })
}

export async function updateHost(host: Host) {
    return invoke("update_host", {
        id: host.id,
        name: host.name,
        host: host.host,
        port: host.port,
        username: host.username,
        authType: host.auth_type,
        groupId: host.group_id,
        passwordId: host.password_id,
    })
}

export function renameGroup(id: number, name: string) {
    return invoke("rename_group", { id, name })
}

export function deleteGroup(id: number) {
    return invoke("delete_group", { id })
}

export async function sshExec(hostId: number, command: string): Promise<string> {
    const res = await invoke<SshResponse>("ssh_exec", {
        hostId,
        command,
    })

    if (!res.ok) {
        // toast.error(res.error.message)
        console.log(res.error.message);
        return res.error.message
    }
    return res.data.stdout
}

// ── Password ────────────────────────────────────────────────────────────────

export async function createPassword(name: string, password: string) {
    return invoke("create_password", {
        name,
        password,
    })
}

export async function updatePassword(password: Password) {
    return invoke("update_password", {
        id: password.id,
        name: password.name,
        password: password.password,
    })
}

export async function listPasswords(): Promise<Password[]> {
    return invoke("list_passwords")
}

export async function deletePassword(id: number) {
    return invoke("delete_password", { id })
}

export async function copyPassword(id: number) {
    return invoke("copy_password", { id })
}


// ssh tunnel

export async function listTunnels() {
    return invoke<Tunnel[]>("list_tunnels")
}

export async function createTunnel(payload: Tunnel) {
    return invoke("create_tunnel", { payload })
}

export async function deleteTunnel(id: number) {
    return invoke("delete_tunnel", { id })
}

export async function updateTunnel(tunnel: Tunnel) {
    return invoke("update_tunnel", {
        id: tunnel.id,
        payload: tunnel,
    })
}

export async function getTunnel(id: number) {
    return invoke<Tunnel>("get_tunnel", { id })
}

export async function startTunnel(id: number) {
    return invoke("start_tunnel", { id })
}

export async function stopTunnel(id: number) {
    return invoke("stop_tunnel", { id })
}

export async function listActiveTunnels(): Promise<number[]> {
    return invoke("list_active_tunnels")
}