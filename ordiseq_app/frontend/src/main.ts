import "./app.css";
import App from "./App.svelte";
import ShaderBackground from "./lib/ShaderBackground.svelte";
import { mount } from "svelte";
import { themeColors, applyTheme } from "./lib/themeStore";
import { initializeSettings, randomizeTheme } from "./lib/shaderStore";
import { loadAlwaysOnTop, loadIconShape, loadShaderSettings, loadWindowPosition, saveWindowPosition } from "./lib/settingsStore";
import type { WindowPosition } from "./lib/settingsStore";
import { applyIcon } from "./lib/iconGenerator";
import { getCurrentWindow, availableMonitors, PhysicalPosition } from "@tauri-apps/api/window";
import type { Monitor } from "@tauri-apps/api/window";
import { startPolling } from "./lib/clientsStore";
import { startTransportSync } from "./lib/transportStore";

function isWindowWithinAnyMonitor(
  x: number, y: number, width: number, height: number,
  monitors: Monitor[],
): boolean {
  return monitors.some((m) => {
    const left = m.position.x;
    const top = m.position.y;
    const right = m.position.x + m.size.width;
    const bottom = m.position.y + m.size.height;
    return x >= left && y >= top && (x + width) <= right && (y + height) <= bottom;
  });
}

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

    // Restore window position if saved and still within a monitor
    const savedPos = await loadWindowPosition();
    if (savedPos) {
      const monitors = await availableMonitors();
      const size = await win.outerSize();
      if (isWindowWithinAnyMonitor(savedPos.x, savedPos.y, size.width, size.height, monitors)) {
        await win.setPosition(new PhysicalPosition(savedPos.x, savedPos.y));
      }
    }

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

    // F12 = randomize color theme
    document.addEventListener("keydown", async (e) => {
      if (e.key === "F12") {
        e.preventDefault();
        const { color1 } = randomizeTheme();
        const shape = await loadIconShape();
        await applyIcon(shape, color1);
      }
    });

    // Start polling for connected clients
    startPolling();

    // Start transport sync (beat position interpolation for shaders)
    startTransportSync();

    // Save window position on close (only if within screen bounds)
    win.onCloseRequested(async (event) => {
      event.preventDefault();
      try {
        const pos = await win.outerPosition();
        const size = await win.outerSize();
        const monitors = await availableMonitors();
        if (isWindowWithinAnyMonitor(pos.x, pos.y, size.width, size.height, monitors)) {
          await saveWindowPosition({ x: pos.x, y: pos.y });
        }
      } catch (e) {
        console.warn("Failed to save window position on close:", e);
      }
      win.destroy();
    });
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
