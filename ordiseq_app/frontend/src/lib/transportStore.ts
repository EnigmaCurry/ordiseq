import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface SyncStateInfo {
  source_client_id: number | null;
  beat_position: number;
  bpm: number;
  playing: boolean;
  received_at_ms: number;
}

// Raw sync state from backend
export const syncState = writable<SyncStateInfo>({
  source_client_id: null,
  beat_position: 0,
  bpm: 120,
  playing: false,
  received_at_ms: 0,
});

// Interpolated beat position (updated at 60fps)
export const beatPosition = writable<number>(0);
export const transportBpm = writable<number>(120);
export const transportPlaying = writable<boolean>(false);

// Anchor point for interpolation
let anchor = {
  beat: 0,
  localTimeMs: 0,
  bpm: 120,
};

let pollInterval: ReturnType<typeof setInterval> | null = null;
let animFrame: number | null = null;

async function fetchSyncState() {
  try {
    const result = await invoke<SyncStateInfo>("get_sync_state");
    const prev = get(syncState);
    syncState.set(result);

    // Update anchor when we get new data (different from last)
    if (result.received_at_ms > prev.received_at_ms) {
      anchor = {
        beat: result.beat_position,
        localTimeMs: performance.now(),
        bpm: result.bpm,
      };
      transportBpm.set(result.bpm);
      transportPlaying.set(result.playing);
    } else if (!result.playing && prev.playing) {
      // Transport just stopped
      transportPlaying.set(false);
    }
  } catch {
    // Silently ignore -- backend may not have the command yet
  }
}

function interpolationLoop() {
  const state = get(syncState);
  if (state.playing && state.received_at_ms > 0) {
    const now = performance.now();
    const elapsedMs = now - anchor.localTimeMs;
    const elapsedBeats = (elapsedMs / 60000) * anchor.bpm;
    beatPosition.set(anchor.beat + elapsedBeats);
  }
  animFrame = requestAnimationFrame(interpolationLoop);
}

export function startTransportSync() {
  if (pollInterval) return;
  fetchSyncState();
  pollInterval = setInterval(fetchSyncState, 100); // 10Hz
  if (animFrame === null) {
    animFrame = requestAnimationFrame(interpolationLoop);
  }
}

export function stopTransportSync() {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
  if (animFrame !== null) {
    cancelAnimationFrame(animFrame);
    animFrame = null;
  }
}

// Tauri command wrappers
export async function setSyncSource(clientId: number): Promise<void> {
  await invoke("set_sync_source", { clientId });
}

export async function clearSyncSource(): Promise<void> {
  await invoke("clear_sync_source");
}
