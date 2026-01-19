import { listen } from "@tauri-apps/api/event";

export type SshProgressEvent = {
    task_id: string;
    status: "running" | "error" | "cancelled";
};

export type SshStdoutEvent = {
    task_id: string;
    chunk: string;
};

export type SshDoneEvent = {
    task_id: string;
    exit_code: number;
};

export async function registerSshListeners(
    taskId: string,
    handlers: {
        onProgress?: (e: SshProgressEvent) => void;
        onStdout?: (e: SshStdoutEvent) => void;
        onDone?: (e: SshDoneEvent) => void;
    }
) {
    const unlistenProgress = await listen<SshProgressEvent>(
        "ssh:progress",
        (e) => e.payload.task_id === taskId && handlers.onProgress?.(e.payload)
    );

    const unlistenStdout = await listen<SshStdoutEvent>(
        "ssh:stdout",
        (e) => e.payload.task_id === taskId && handlers.onStdout?.(e.payload)
    );

    const unlistenDone = await listen<SshDoneEvent>(
        "ssh:done",
        (e) => e.payload.task_id === taskId && handlers.onDone?.(e.payload)
    );

    return () => {
        unlistenProgress();
        unlistenStdout();
        unlistenDone();
    };
}
