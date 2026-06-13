export interface Group {
    id: number
    name: string
    parent_id: number | null
}

export interface Host {
    id: number
    name: string
    host: string
    port: number
    username: string
    auth_type: string
    group_id: number | null
    password_id: number | null
}


export interface Password {
    id: number
    name: string
    password: string
}
type SshExecResult = {
    success: boolean
    exit_code: number
    stdout: string
}

export type SshResponse =
    | { ok: true; data: SshExecResult }
    | { ok: false; error: { kind: string; message: string } }


// ── SSH Tunnel ──────────────────────────────────────────────────────────────

export interface Tunnel {
    id: number
    name: string
    type: 'local' | 'socks5'
    host_id: number
    local_port: number
    remote_host: string | null
    remote_port: number | null
    created_at: string
}