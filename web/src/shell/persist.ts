// A per-browser copy of what the first screens need (the run list, the computed test points), so a
// returning visit renders at once while a fresh copy loads in the background. IndexedDB, because the
// run list is about a megabyte of JSON. A convenience only: every read and write may fail (private
// window, blocked storage) and the page then simply waits for the network.

const DB = "ignis-cache";
const STORE = "kv";

function open(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const r = indexedDB.open(DB, 1);
    r.onupgradeneeded = () => r.result.createObjectStore(STORE);
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}

export interface Kept<T> {
  at: number;
  value: T;
}

export async function readKept<T>(key: string): Promise<Kept<T> | undefined> {
  try {
    const db = await open();
    return await new Promise((resolve) => {
      const q = db.transaction(STORE, "readonly").objectStore(STORE).get(key);
      q.onsuccess = () => resolve(q.result as Kept<T> | undefined);
      q.onerror = () => resolve(undefined);
    });
  } catch {
    return undefined;
  }
}

export async function keep<T>(key: string, value: T): Promise<void> {
  try {
    const db = await open();
    db.transaction(STORE, "readwrite").objectStore(STORE).put({ at: Date.now(), value } satisfies Kept<T>, key);
  } catch {
    /* storage blocked: the next visit loads from the network */
  }
}
