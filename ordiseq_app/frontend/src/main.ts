import "./app.css";
import App from "./App.svelte";
import ShaderBackground from "./lib/ShaderBackground.svelte";
import { mount } from "svelte";
import { themeColors, applyTheme } from "./lib/themeStore";
import { initializeSettings, randomizeTheme, randomizeShader } from "./lib/shaderStore";
import { loadAlwaysOnTop, loadIconShape, loadShaderSettings, loadWindowPosition, saveWindowPosition, loadLastPage, saveLastPage } from "./lib/settingsStore";
import type { WindowPosition } from "./lib/settingsStore";
import { applyIcon } from "./lib/iconGenerator";
import { getCurrentWindow, availableMonitors, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import type { Monitor } from "@tauri-apps/api/window";
import { startPolling, clients } from "./lib/clientsStore";
import { currentPage } from "./lib/router";
import { get } from "svelte/store";
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
  let savedPos: WindowPosition | null = null;
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

    // Restore window position before show (setSize before show breaks WebView).
    // Only restore if the full window fits entirely within a single monitor.
    savedPos = await loadWindowPosition();
    if (savedPos && savedPos.width && savedPos.height) {
      const monitors = await availableMonitors();
      // Estimate outer size by adding current decoration (title bar + borders) to saved inner size
      const curOuter = await win.outerSize();
      const curInner = await win.innerSize();
      const outerW = savedPos.width + (curOuter.width - curInner.width);
      const outerH = savedPos.height + (curOuter.height - curInner.height);
      if (isWindowWithinAnyMonitor(savedPos.x, savedPos.y, outerW, outerH, monitors)) {
        await win.setPosition(new PhysicalPosition(savedPos.x, savedPos.y));
      } else {
        savedPos = null;
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

    // F10 = randomize shader, F12 = randomize color theme
    document.addEventListener("keydown", async (e) => {
      if (e.key === "F10") {
        e.preventDefault();
        randomizeShader();
      } else if (e.key === "F12") {
        e.preventDefault();
        const { color1 } = randomizeTheme();
        const shape = await loadIconShape();
        await applyIcon(shape, color1);
      }
    });

    // Double-click on graphics page toggles fullscreen
    document.addEventListener("dblclick", async () => {
      if (get(currentPage) === "graphics") {
        const isFullscreen = await win.isFullscreen();
        await win.setFullscreen(!isFullscreen);
      }
    });

    // Start polling for connected clients
    startPolling();

    // Restore last page and set up auto-switch logic
    const lastPage = await loadLastPage();
    if (lastPage === "clients") {
      // User was explicitly on Clients last time — stay there, no auto-switch
      currentPage.set("clients");
    } else {
      // Default flow: start on clients, auto-switch to graphics on first client connect
      const unsubClients = clients.subscribe((list) => {
        if (list.length > 0 && get(currentPage) === "clients") {
          currentPage.set("graphics");
          unsubClients();
        }
      });
    }

    // Persist page changes
    currentPage.subscribe((page) => {
      saveLastPage(page);
    });

    // Start transport sync (beat position interpolation for shaders)
    startTransportSync();

    // Save window position on close (only if windowed and within screen bounds)
    win.onCloseRequested(async (event) => {
      event.preventDefault();
      try {
        const fullscreen = await win.isFullscreen();
        if (fullscreen) { win.destroy(); return; }
        const pos = await win.outerPosition();
        const outerSize = await win.outerSize();
        const innerSize = await win.innerSize();
        const monitors = await availableMonitors();
        if (isWindowWithinAnyMonitor(pos.x, pos.y, outerSize.width, outerSize.height, monitors)) {
          await saveWindowPosition({
            x: pos.x,
            y: pos.y,
            width: innerSize.width,
            height: innerSize.height,
          });
        }
      } catch (e) {
        console.warn("Failed to save window position on close:", e);
      }
      win.destroy();
    });
  } finally {
    // Always show window, even if init errors
    await win.show();
    // Restore window size after show (setSize before show breaks WebView)
    if (savedPos && savedPos.width && savedPos.height) {
      await win.setSize(new PhysicalSize(savedPos.width, savedPos.height));
    }
    if (alwaysOnTop) {
      await win.setFocus();
    }
  }
}

const appPromise = init();

export default appPromise;
