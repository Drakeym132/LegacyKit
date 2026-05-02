export interface LogEntry {
    text: string;
    type: 'stdout' | 'stderr' | 'info';
    timestamp: number;
}

export interface LogEventPayload {
    text: string;
    type: LogEntry['type'];
}

const MAX_ENTRIES = 2000;

class LogStore {
    logs = $state<LogEntry[]>([]);

    append(text: string, type: 'stdout' | 'stderr' | 'info' = 'stdout') {
        this.logs.push({ text, type, timestamp: Date.now() });
        if (this.logs.length > MAX_ENTRIES) {
            this.logs.splice(0, this.logs.length - MAX_ENTRIES);
        }
    }

    clear() {
        this.logs = [];
    }
}

export const logStore = new LogStore();
