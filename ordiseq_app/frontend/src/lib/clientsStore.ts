import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface ClientInfo {
  id: number;
  name: string;
  version: string;
  bpm: number;
  playing: boolean;
  connected: boolean;
  program: number;
}

export interface MidiClip {
  name: string;
  length_beats: number;
  notes: ClipNote[];
}

export interface ClipNote {
  note: number;
  channel: number;
  velocity: number;
  start_beats: number;
  duration_beats: number;
}

export type StepState = "off" | "hit" | "accent";

export interface EuclidRow {
  note: number;
  length: number;
  hits: number;
  rotation: number;
  accents: number;          // 0..hits, euclidean accent count
  velocity: number;         // hit velocity 0-127
  accentVelocity: number;   // accent velocity 0-127
  manualPattern?: StepState[];  // present = manual edit mode
}

export type SequencerType = "none" | "euclidean";

export interface SequencerConfig {
  type: SequencerType;
  euclidean: EuclidRow[];
}

const NOTE_NAMES = [
  "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

export function midiNoteName(note: number): string {
  const octave = Math.floor(note / 12) - 1;
  return `${NOTE_NAMES[note % 12]}${octave}`;
}

export const clients = writable<ClientInfo[]>([]);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export function startPolling() {
  if (pollInterval) return;
  fetchClients();
  pollInterval = setInterval(fetchClients, 1000);
}

export function stopPolling() {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
}

async function fetchClients() {
  try {
    const result = await invoke<ClientInfo[]>("get_clients");
    clients.set(result);
  } catch (e) {
    console.error("Failed to fetch clients:", e);
  }
}

export async function sendClipToClient(clientId: number, clip: MidiClip, program: number = 0) {
  await invoke("send_clip_to_client", { clientId, clip, program });
}

export async function sendPlayModeToClient(clientId: number, program: number, noteTrigger: boolean) {
  await invoke("set_play_mode", { clientId, program, noteTrigger });
}

export async function renameClient(clientId: number, name: string) {
  await invoke("rename_client", { clientId, name });
}

/** Bjorklund's algorithm: distribute `hits` evenly across `length` steps. */
export function bjorklund(length: number, hits: number): boolean[] {
  if (length <= 0) return [];
  if (hits <= 0) return new Array(length).fill(false);
  if (hits >= length) return new Array(length).fill(true);

  let groups: boolean[][] = [];
  for (let i = 0; i < hits; i++) groups.push([true]);
  let remainder: boolean[][] = [];
  for (let i = 0; i < length - hits; i++) remainder.push([false]);

  while (remainder.length > 1) {
    const newGroups: boolean[][] = [];
    const minLen = Math.min(groups.length, remainder.length);
    for (let i = 0; i < minLen; i++) {
      newGroups.push([...groups[i], ...remainder[i]]);
    }
    const leftoverGroups = groups.slice(minLen);
    const leftoverRemainder = remainder.slice(minLen);
    groups = newGroups;
    remainder = leftoverGroups.length > 0 ? leftoverGroups : leftoverRemainder;
  }

  // Flatten
  const pattern: boolean[] = [];
  for (const g of groups) pattern.push(...g);
  for (const r of remainder) pattern.push(...r);
  return pattern;
}

/** Rotate a pattern by `rotation` steps. */
function rotatePattern(pattern: boolean[], rotation: number): boolean[] {
  if (pattern.length === 0) return pattern;
  const r = ((rotation % pattern.length) + pattern.length) % pattern.length;
  return [...pattern.slice(r), ...pattern.slice(0, r)];
}

const STEP_BEATS = 0.25; // 16th note
const GATE_RATIO = 0.5;

function gcd(a: number, b: number): number {
  while (b) { [a, b] = [b, a % b]; }
  return a;
}

function lcm(a: number, b: number): number {
  return (a / gcd(a, b)) * b;
}

/** Generate a MidiClip from euclidean sequencer rows.
 *  Clip length = LCM of all row lengths so each row loops independently (polyrhythm). */
export function euclideanToClip(rows: EuclidRow[]): MidiClip {
  const totalSteps = rows.reduce((acc, r) => lcm(acc, r.length), 1);
  const lengthBeats = totalSteps * STEP_BEATS;
  const notes: ClipNote[] = [];

  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    const baseVel = (row.velocity ?? 100) / 127;
    const accVel = (row.accentVelocity ?? 127) / 127;

    // Determine per-step pattern: manual or euclidean
    let stepPattern: StepState[];
    if (row.manualPattern) {
      stepPattern = row.manualPattern;
    } else {
      // Apply accents before rotation: assign accent to each hit, then rotate the combined pattern
      const hitPattern = bjorklund(row.length, row.hits);
      const accentPattern = row.accents > 0 ? bjorklund(row.hits, row.accents) : [];
      const combined: StepState[] = [];
      let hitIdx = 0;
      for (let s = 0; s < row.length; s++) {
        if (!hitPattern[s]) {
          combined.push("off");
        } else {
          const isAccent = accentPattern.length > 0 && accentPattern[hitIdx % accentPattern.length];
          combined.push(isAccent ? "accent" : "hit");
          hitIdx++;
        }
      }
      // Rotate the combined result
      if (row.rotation === 0 || combined.length === 0) {
        stepPattern = combined;
      } else {
        const r = ((row.rotation % combined.length) + combined.length) % combined.length;
        stepPattern = [...combined.slice(r), ...combined.slice(0, r)];
      }
    }

    for (let step = 0; step < totalSteps; step++) {
      const si = step % row.length;
      const state = stepPattern[si];
      if (state !== "off") {
        const vel = state === "accent" ? accVel : baseVel;
        if (vel > 0) {
          notes.push({
            note: row.note,
            channel: 0,
            velocity: vel,
            start_beats: step * STEP_BEATS,
            duration_beats: STEP_BEATS * GATE_RATIO,
          });
        }
      }
    }
  }

  return {
    name: "Euclidean",
    length_beats: lengthBeats,
    notes,
  };
}

export function defaultEuclidRow(note: number = 36): EuclidRow {
  return { note, length: 16, hits: 4, rotation: 0, accents: 0, velocity: 100, accentVelocity: 127 };
}

// --- Persistence by client name ---

const STORAGE_KEY = "sequencer-configs";

interface StoredConfig {
  type: SequencerType;
  rows: EuclidRow[];
  noteTrigger?: boolean;
}

function loadAllConfigs(): Record<string, StoredConfig> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveAllConfigs(configs: Record<string, StoredConfig>) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(configs));
}

function configKey(name: string, program: number): string {
  return `${name}:P${program}`;
}

export function loadConfigForClient(name: string, program: number = 1): StoredConfig | null {
  const all = loadAllConfigs();
  // Try per-program key first, then migrate from legacy (unkeyed) config
  let config = all[configKey(name, program)];
  if (!config && program === 1 && all[name]) {
    // Migrate legacy config to program 1
    config = all[name];
    all[configKey(name, 1)] = config;
    delete all[name];
    saveAllConfigs(all);
  }
  if (!config) return null;
  // Migrate old rows missing fields
  config.rows = config.rows.map((r, i) => ({
    note: r.note ?? 36 + i,
    length: r.length,
    hits: r.hits,
    rotation: r.rotation,
    accents: r.accents ?? 0,
    velocity: r.velocity ?? 100,
    accentVelocity: r.accentVelocity ?? 127,
    ...(r.manualPattern ? { manualPattern: r.manualPattern } : {}),
  }));
  return config;
}

export function saveConfigForClient(name: string, type: SequencerType, rows: EuclidRow[], program: number = 1, noteTrigger: boolean = false) {
  const all = loadAllConfigs();
  all[configKey(name, program)] = { type, rows, noteTrigger };
  saveAllConfigs(all);
}

export function renameStoredConfig(oldName: string, newName: string) {
  const all = loadAllConfigs();
  let changed = false;
  // Rename all program keys for this client
  for (let p = 1; p <= 16; p++) {
    const oldKey = configKey(oldName, p);
    if (all[oldKey]) {
      all[configKey(newName, p)] = all[oldKey];
      delete all[oldKey];
      changed = true;
    }
  }
  // Also migrate legacy key
  if (all[oldName]) {
    all[configKey(newName, 1)] = all[oldName];
    delete all[oldName];
    changed = true;
  }
  if (changed) saveAllConfigs(all);
}
