// src/api/ssh.ts
import { invoke } from "@tauri-apps/api/core";
import { registerSshListeners } from "../events/sshEvents";
import { nanoid } from "nanoid";

export async function runSshCommand(hostId: number, command: string) {
    const taskId = nanoid();

    const cleanup = await registerSshListeners(taskId, {
        onStdout: (e) => console.log(e.chunk),
        onDone: () => cleanup(),
    });

    await invoke("ssh_exec_start", {
        taskId,
        hostId,
        command,
    });

    return {
        cancel: () => invoke("ssh_exec_cancel", { taskId }),
    };
}
