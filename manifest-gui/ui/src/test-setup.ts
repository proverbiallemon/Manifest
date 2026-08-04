// Node 22+ ships a global `localStorage` stub that is only functional under
// `--experimental-webstorage` with a valid `--localstorage-file`. Vitest's
// jsdom environment (vitest@3.2.7) does not override globals that already
// exist on the Node global object, so that non-functional stub shadows
// jsdom's real localStorage in every test file. Replace it with a minimal
// in-memory implementation before any test module runs.
let store: Record<string, string> = {};

const memoryStorage: Storage = {
  getItem: (key: string) => (key in store ? store[key] : null),
  setItem: (key: string, value: string) => {
    store[key] = String(value);
  },
  removeItem: (key: string) => {
    delete store[key];
  },
  clear: () => {
    store = {};
  },
  key: (index: number) => Object.keys(store)[index] ?? null,
  get length() {
    return Object.keys(store).length;
  },
};

Object.defineProperty(globalThis, "localStorage", {
  value: memoryStorage,
  configurable: true,
  writable: true,
});
