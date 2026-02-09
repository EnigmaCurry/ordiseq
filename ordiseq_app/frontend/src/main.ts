import "./app.css";
import App from "./App.svelte";
import ShaderBackground from "./lib/ShaderBackground.svelte";
import { mount } from "svelte";
import { themeColors, applyTheme } from "./lib/themeStore";
import { initializeSettings } from "./lib/shaderStore";
import { loadAlwaysOnTop, loadIconShape, loadShaderSettings } from "./lib/settingsStore";
import { applyIcon } from "./lib/iconGenerator";
import { getCurrentWindow } from "@tauri-apps/api/window";

// Initialize app after loading settings
async function init() {
  // Load persisted settings first
  await initializeSettings();

  // Restore always-on-top setting
  const alwaysOnTop = await loadAlwaysOnTop();
  if (alwaysOnTop) {
    await getCurrentWindow().setAlwaysOnTop(true);
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
  const shaderBackground = mount(ShaderBackground, {
    target: shaderContainer,
  });

  // Mount main app
  const app = mount(App, {
    target: document.getElementById("app")!,
  });

  // Subscribe to theme colors and apply them
  themeColors.subscribe(applyTheme);

  // Bring window to front after everything is mounted
  if (alwaysOnTop) {
    await getCurrentWindow().setFocus();
  }

  return { app, shaderBackground };
}

const appPromise = init();

export default appPromise;
