import { invoke } from "@tauri-apps/api/core"
import type { Group, Host, SshResponse } from "./types"

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
