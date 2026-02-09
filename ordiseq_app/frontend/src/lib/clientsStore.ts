import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface ClientInfo {
  id: number;
  name: string;
  version: string;
  bpm: number;
  playing: boolean;
  connected: boolean;
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

export interface EuclidRow {
  note: number;
  length: number;
  hits: number;
  rotation: number;
  accents: number;   // 0..hits, euclidean accent count
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

export async function sendClipToClient(clientId: number, clip: MidiClip) {
  await invoke("send_clip_to_client", { clientId, clip });
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
const ACCENT_VELOCITY = 1.0;

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
    const pattern = rotatePattern(bjorklund(row.length, row.hits), row.rotation);
    const midiNote = row.note;

    for (let step = 0; step < totalSteps; step++) {
      if (pattern[step % row.length]) {
        notes.push({
          note: midiNote,
          channel: 0,
          velocity: DEFAULT_VELOCITY,
          start_beats: step * STEP_BEATS,
          duration_beats: STEP_BEATS * GATE_RATIO,
        });
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
  return { note, length: 16, hits: 4, rotation: 0 };
}

// --- Persistence by client name ---

const STORAGE_KEY = "sequencer-configs";

interface StoredConfig {
  type: SequencerType;
  rows: EuclidRow[];
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

export function loadConfigForClient(name: string): StoredConfig | null {
  const all = loadAllConfigs();
  const config = all[name];
  if (!config) return null;
  // Migrate old rows missing the `note` field
  config.rows = config.rows.map((r, i) => ({
    note: r.note ?? 36 + i,
    length: r.length,
    hits: r.hits,
    rotation: r.rotation,
  }));
  return config;
}

export function saveConfigForClient(name: string, type: SequencerType, rows: EuclidRow[]) {
  const all = loadAllConfigs();
  all[name] = { type, rows };
  saveAllConfigs(all);
}

export function renameStoredConfig(oldName: string, newName: string) {
  const all = loadAllConfigs();
  if (all[oldName]) {
    all[newName] = all[oldName];
    delete all[oldName];
    saveAllConfigs(all);
  }
}
