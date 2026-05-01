export type ToastVariant = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
    id: number;
    variant: ToastVariant;
    title: string;
    body?: string | null;
    expiresAt: number | null;
}

const DEFAULT_DURATION_MS = 5000;

class ToastStore {
    toasts = $state<Toast[]>([]);
    private nextId = 1;

    push(variant: ToastVariant, title: string, body?: string | null, durationMs?: number | null): number {
        const id = this.nextId++;
        const ttl = durationMs === null ? null : durationMs ?? DEFAULT_DURATION_MS;
        const toast: Toast = {
            id,
            variant,
            title,
            body: body ?? null,
            expiresAt: ttl ? Date.now() + ttl : null,
        };
        this.toasts = [...this.toasts, toast];
        if (ttl) {
            setTimeout(() => this.dismiss(id), ttl);
        }
        return id;
    }

    info(title: string, body?: string | null, durationMs?: number | null) {
        return this.push('info', title, body, durationMs);
    }
    success(title: string, body?: string | null, durationMs?: number | null) {
        return this.push('success', title, body, durationMs);
    }
    warning(title: string, body?: string | null, durationMs?: number | null) {
        return this.push('warning', title, body, durationMs);
    }
    error(title: string, body?: string | null, durationMs?: number | null) {
        return this.push('error', title, body, durationMs ?? 8000);
    }

    dismiss(id: number) {
        this.toasts = this.toasts.filter((t) => t.id !== id);
    }

    clear() {
        this.toasts = [];
    }
}

export const toastStore = new ToastStore();
