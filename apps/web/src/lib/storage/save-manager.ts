import type { SimulationState } from '@era/era-core';

const DB_NAME = 'era_sim_db';
const STORE_NAME = 'save_states';
const DB_VERSION = 1;
const AUTO_SAVE_KEY = 'latest_auto_save';

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    if (typeof indexedDB === 'undefined') {
      return reject(new Error('IndexedDB not supported in this environment'));
    }

    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * 自动保存当前游玩状态到 IndexedDB (带 localStorage 保底)
 */
export async function saveGameState(state: SimulationState): Promise<void> {
  const serialized = JSON.stringify(state, (_key, value) =>
    typeof value === 'bigint' ? value.toString() : value,
  );

  try {
    const db = await openDatabase();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      const req = store.put(serialized, AUTO_SAVE_KEY);
      req.onsuccess = () => resolve();
      req.onerror = () => reject(req.error);
    });
  } catch {
    // IndexedDB 失败时降级到 localStorage
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(AUTO_SAVE_KEY, serialized);
    }
  }
}

/**
 * 读取最近保存的存档
 */
export async function loadLatestSave(): Promise<SimulationState | null> {
  try {
    const db = await openDatabase();
    const raw = await new Promise<string | null>((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const req = store.get(AUTO_SAVE_KEY);
      req.onsuccess = () => resolve((req.result as string) ?? null);
      req.onerror = () => reject(req.error);
    });

    if (raw) {
      return JSON.parse(raw) as SimulationState;
    }
  } catch {
    // 降级检查 localStorage
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(AUTO_SAVE_KEY);
      if (raw) return JSON.parse(raw) as SimulationState;
    }
  }
  return null;
}

/**
 * 导出存档为 JSON 文件下载到本地
 */
export function exportSaveAsJSON(state: SimulationState): void {
  const jsonStr = JSON.stringify(
    state,
    (_key, value) => (typeof value === 'bigint' ? value.toString() : value),
    2,
  );
  const blob = new Blob([jsonStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `era_save_${state.protagonist.birth_year}_${state.current_year}_seed${state.seed}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * 从本地选择的 JSON 文件反序列化导入存档
 */
export async function importSaveFromJSON(file: File): Promise<SimulationState> {
  const text = await file.text();
  const parsed = JSON.parse(text) as SimulationState;
  if (!parsed?.protagonist || typeof parsed.current_year !== 'number') {
    throw new Error('无效的 Era 存档文件格式');
  }
  await saveGameState(parsed);
  return parsed;
}
