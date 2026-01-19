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
    password: string | null
}
type SshExecResult = {
    success: boolean
    exit_code: number
    stdout: string
}

export type SshResponse =
    | { ok: true; data: SshExecResult }
    | { ok: false; error: { kind: string; message: string } }
