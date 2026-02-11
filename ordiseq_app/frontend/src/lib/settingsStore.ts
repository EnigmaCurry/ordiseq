import { load } from "@tauri-apps/plugin-store";

export interface ShaderSettings {
  selectedShader: string;
  colorTheme: string;
  uniforms: {
    pitch: number;
    speed: number;
    zoom: number;
    fisheye: number;
    overlay: number;
    color1: string;
    color2: string;
    color3: string;
    color4: string;
  };
  animationEnabled: boolean;
}

import type { IconShape } from "./iconGenerator";

export interface WindowPosition {
  x: number;
  y: number;
  width: number;
  height: number;
}

const STORE_FILE = "settings.json";
const SHADER_KEY = "shader";
const ALWAYS_ON_TOP_KEY = "alwaysOnTop";
const ICON_SHAPE_KEY = "iconShape";
const LISTEN_PORT_KEY = "listenPort";
const WINDOW_POSITION_KEY = "windowPosition";
const LAST_PAGE_KEY = "lastPage";

const defaultSettings: ShaderSettings = {
  selectedShader: "platonic",
  colorTheme: "sunset",
  uniforms: {
    pitch: 0.0,
    speed: 0.02,
    zoom: 12.0,
    fisheye: 0.04,
    overlay: 0.69,
    color1: "#ffb347",
    color2: "#ff6b6b",
    color3: "#c77dba",
    color4: "#2d1b3d",
  },
  animationEnabled: true,
};

let store: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!store) {
    store = await load(STORE_FILE, { autoSave: true });
  }
  return store;
}

export async function loadShaderSettings(): Promise<ShaderSettings> {
  try {
    const s = await getStore();
    const saved = await s.get<ShaderSettings>(SHADER_KEY);
    if (saved) {
      // Merge with defaults to handle new settings added in updates
      return { ...defaultSettings, ...saved, uniforms: { ...defaultSettings.uniforms, ...saved.uniforms } };
    }
  } catch (e) {
    console.warn("Failed to load settings, using defaults:", e);
  }
  return defaultSettings;
}

export async function saveShaderSettings(settings: ShaderSettings): Promise<void> {
  try {
    const s = await getStore();
    await s.set(SHADER_KEY, settings);
    await s.save();
  } catch (e) {
    console.warn("Failed to save settings:", e);
  }
}

export function getDefaultSettings(): ShaderSettings {
  return { ...defaultSettings, uniforms: { ...defaultSettings.uniforms } };
}

export async function loadAlwaysOnTop(): Promise<boolean> {
  try {
    const s = await getStore();
    const saved = await s.get<boolean>(ALWAYS_ON_TOP_KEY);
    return saved ?? false;
  } catch (e) {
    console.warn("Failed to load always-on-top setting:", e);
    return false;
  }
}

export async function saveAlwaysOnTop(value: boolean): Promise<void> {
  try {
    const s = await getStore();
    await s.set(ALWAYS_ON_TOP_KEY, value);
    await s.save();
  } catch (e) {
    console.warn("Failed to save always-on-top setting:", e);
  }
}

export async function loadIconShape(): Promise<IconShape> {
  try {
    const s = await getStore();
    const saved = await s.get<IconShape>(ICON_SHAPE_KEY);
    return saved ?? "ufo";
  } catch (e) {
    console.warn("Failed to load icon shape setting:", e);
    return "triangle";
  }
}

export async function saveIconShape(value: IconShape): Promise<void> {
  try {
    const s = await getStore();
    await s.set(ICON_SHAPE_KEY, value);
    await s.save();
  } catch (e) {
    console.warn("Failed to save icon shape setting:", e);
  }
}

export async function loadListenPort(): Promise<number> {
  try {
    const s = await getStore();
    const saved = await s.get<number>(LISTEN_PORT_KEY);
    return saved ?? 9850;
  } catch (e) {
    console.warn("Failed to load listen port setting:", e);
    return 9850;
  }
}

export async function saveListenPort(value: number): Promise<void> {
  try {
    const s = await getStore();
    await s.set(LISTEN_PORT_KEY, value);
    await s.save();
  } catch (e) {
    console.warn("Failed to save listen port setting:", e);
  }
}

export async function loadWindowPosition(): Promise<WindowPosition | null> {
  try {
    const s = await getStore();
    const saved = await s.get<WindowPosition>(WINDOW_POSITION_KEY);
    return saved ?? null;
  } catch (e) {
    console.warn("Failed to load window position:", e);
    return null;
  }
}

export async function saveWindowPosition(pos: WindowPosition): Promise<void> {
  try {
    const s = await getStore();
    await s.set(WINDOW_POSITION_KEY, pos);
    await s.save();
  } catch (e) {
    console.warn("Failed to save window position:", e);
  }
}

export async function loadLastPage(): Promise<string | null> {
  try {
    const s = await getStore();
    return (await s.get<string>(LAST_PAGE_KEY)) ?? null;
  } catch {
    return null;
  }
}

export async function saveLastPage(page: string): Promise<void> {
  try {
    const s = await getStore();
    await s.set(LAST_PAGE_KEY, page);
    await s.save();
  } catch (e) {
    console.warn("Failed to save last page:", e);
  }
}

export async function clearAllSettings(): Promise<void> {
  const s = await getStore();
  await s.clear();
  await s.save();
}
