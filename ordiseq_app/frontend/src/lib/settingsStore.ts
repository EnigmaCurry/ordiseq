import { load } from "@tauri-apps/plugin-store";

export interface ShaderSettings {
  selectedShader: string;
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

const STORE_FILE = "settings.json";
const SHADER_KEY = "shader";
const ALWAYS_ON_TOP_KEY = "alwaysOnTop";

const defaultSettings: ShaderSettings = {
  selectedShader: "retrogrid",
  uniforms: {
    pitch: 0.0,
    speed: 0.02,
    zoom: 12.0,
    fisheye: 0.04,
    overlay: 0.9,
    color1: "#ff1493",
    color2: "#00ffde",
    color3: "#bd93f9",
    color4: "#50fa7b",
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
