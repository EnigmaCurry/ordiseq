import "./app.css";
import App from "./App.svelte";
import ShaderBackground from "./lib/ShaderBackground.svelte";
import { mount } from "svelte";
import { themeColors, applyTheme } from "./lib/themeStore";
import { initializeSettings } from "./lib/shaderStore";
import { loadAlwaysOnTop, loadIconShape, loadShaderSettings } from "./lib/settingsStore";
import { applyIcon } from "./lib/iconGenerator";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { startPolling } from "./lib/clientsStore";
import { startTransportSync } from "./lib/transportStore";

// Initialize app after loading settings
async function init() {
  const win = getCurrentWindow();
  let alwaysOnTop = false;
  try {
    // Load persisted settings first
    await initializeSettings();

    // Restore always-on-top setting
    alwaysOnTop = await loadAlwaysOnTop();
    if (alwaysOnTop) {
      await win.setAlwaysOnTop(true);
    }

    // Restore window icon shape with primary color
    const iconShape = await loadIconShape();
    const shaderSettings = await loadShaderSettings();
    await applyIcon(iconShape, shaderSettings.uniforms.color1);

    // Create shader background container
    const shaderContainer = document.createElement("div");
    shaderContainer.id = "shader-container";
    document.body.prepend(shaderContainer);

    // Create overlay element
    const overlay = document.createElement("div");
    overlay.id = "shader-overlay";
    document.body.insertBefore(overlay, document.getElementById("app"));

    // Mount shader background
    mount(ShaderBackground, {
      target: shaderContainer,
    });

    // Mount main app
    mount(App, {
      target: document.getElementById("app")!,
    });

    // Subscribe to theme colors and apply them
    themeColors.subscribe(applyTheme);

    // Start polling for connected clients
    startPolling();

    // Start transport sync (beat position interpolation for shaders)
    startTransportSync();
  } finally {
    // Always show window, even if init errors
    await win.show();
    if (alwaysOnTop) {
      await win.setFocus();
    }
  }
}

const appPromise = init();

export default appPromise;
