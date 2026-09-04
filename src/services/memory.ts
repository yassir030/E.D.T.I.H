import type { MemoryRecord } from "../types";
import { createId } from "../utils";

export type MemoryService = {
  saveMemory: (key: string, value: string) => MemoryRecord;
  getMemories: () => MemoryRecord[];
  searchMemories: (query: string) => MemoryRecord[];
  deleteMemory: (id: string) => boolean;
};

class InMemoryMemoryService implements MemoryService {
  private records: MemoryRecord[] = [];

  saveMemory(key: string, value: string): MemoryRecord {
    const record: MemoryRecord = {
      id: createId(),
      key: key.trim(),
      value: value.trim(),
      createdAt: Date.now(),
    };
    this.records = [record, ...this.records];
    return record;
  }

  getMemories(): MemoryRecord[] {
    return [...this.records];
  }

  searchMemories(query: string): MemoryRecord[] {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return this.getMemories();
    }
    return this.records.filter(
      (record) =>
        record.key.toLowerCase().includes(needle) ||
        record.value.toLowerCase().includes(needle),
    );
  }

  deleteMemory(id: string): boolean {
    const next = this.records.filter((record) => record.id !== id);
    const changed = next.length !== this.records.length;
    this.records = next;
    return changed;
  }
}

export const memoryService: MemoryService = new InMemoryMemoryService();
