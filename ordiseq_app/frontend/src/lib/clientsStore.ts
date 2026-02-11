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

export type SequencerType = "none" | "euclidean" | "chords";

export interface SequenceChord {
  id: string;
  root: number;
  chordType: string;
  bars: number;
  customNotes?: number[];
  customLabel?: string;
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

export const STEP_BEATS = 0.25; // 16th note
const GATE_RATIO = 0.5;

/** Compute the StepState[] for a single EuclidRow (manual or euclidean). */
export function computeStepPattern(row: EuclidRow): StepState[] {
  if (row.manualPattern) return row.manualPattern;

  const hitPat = bjorklund(row.length, row.hits);
  const accentPat = row.accents > 0 ? bjorklund(row.hits, row.accents) : [];
  const combined: StepState[] = [];
  let hitIndex = 0;
  for (let s = 0; s < hitPat.length; s++) {
    if (!hitPat[s]) {
      combined.push("off");
    } else {
      const isAccent = accentPat.length > 0 && accentPat[hitIndex % accentPat.length];
      combined.push(isAccent ? "accent" : "hit");
      hitIndex++;
    }
  }
  if (row.rotation === 0 || combined.length === 0) return combined;
  const r = ((row.rotation % combined.length) + combined.length) % combined.length;
  return [...combined.slice(r), ...combined.slice(0, r)];
}

function gcd(a: number, b: number): number {
  while (b) { [a, b] = [b, a % b]; }
  return a;
}

function lcm(a: number, b: number): number {
  return (a / gcd(a, b)) * b;
}

export function defaultEuclidRow(note: number = 36): EuclidRow {
  return { note, length: 16, hits: 4, rotation: 0, accents: 0, velocity: 100, accentVelocity: 127 };
}

// --- Meta-sequencer ---

export interface EuclidRowMeta {
  slots: Record<string, EuclidRow>;  // "A".."Z"
  metaSequence: string;              // e.g. "AAAA"
  activeSlot: string;                // currently editing letter (UI-only, not persisted)
}

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

export function defaultEuclidRowMeta(note: number = 36): EuclidRowMeta {
  const base = defaultEuclidRow(note);
  const slots: Record<string, EuclidRow> = {};
  for (const ch of ALPHABET) {
    slots[ch] = { ...base };
  }
  return { slots, metaSequence: "AAAA", activeSlot: "A" };
}

/** Convert stored rows (no activeSlot) to runtime EuclidRowMeta[]. */
export function storedToRowMetas(stored: { slots: Record<string, EuclidRow>; metaSequence: string }[]): EuclidRowMeta[] {
  return stored.map(s => ({ ...s, activeSlot: "A" }));
}

const MAX_EXPANDED_LENGTH = 256;

/** Parse a meta-sequence string into an array of slot letters.
 *  Supports: plain letters "ABCD", repeat "A4", group repeat "(AB)2", nested "(A2B)3".
 *  Returns null on parse error or if expansion exceeds limit. */
export function parseMetaSequence(input: string): string[] | null {
  const trimmed = input.trim().toUpperCase();
  if (trimmed.length === 0) return [];

  let pos = 0;

  function parseNumber(): number {
    let numStr = "";
    while (pos < trimmed.length && trimmed[pos] >= "0" && trimmed[pos] <= "9") {
      numStr += trimmed[pos];
      pos++;
    }
    return numStr.length > 0 ? parseInt(numStr, 10) : 1;
  }

  function parseSequence(): string[] | null {
    const result: string[] = [];
    while (pos < trimmed.length) {
      const ch = trimmed[pos];
      if (ch === ")") break;
      if (ch === "(") {
        pos++; // skip '('
        const inner = parseSequence();
        if (inner === null) return null;
        if (pos >= trimmed.length || trimmed[pos] !== ")") return null;
        pos++; // skip ')'
        const repeat = parseNumber();
        for (let i = 0; i < repeat; i++) {
          result.push(...inner);
          if (result.length > MAX_EXPANDED_LENGTH) return null;
        }
      } else if (ch >= "A" && ch <= "Z") {
        const letter = ch;
        pos++;
        const repeat = parseNumber();
        for (let i = 0; i < repeat; i++) {
          result.push(letter);
          if (result.length > MAX_EXPANDED_LENGTH) return null;
        }
      } else {
        return null; // invalid character
      }
    }
    return result;
  }

  const result = parseSequence();
  if (result === null || pos !== trimmed.length) return null;
  if (result.length === 0) return null;
  return result;
}

const MAX_CLIP_NOTES = 10_000;

/** Generate a MidiClip from euclidean row metas with meta-sequences.
 *  Each row's meta-sequence expands to a series of slot patterns concatenated.
 *  Clip length = LCM of all rows' total steps (polyrhythm).
 *  Notes capped at 10,000. */
export function euclideanMetaToClip(metas: EuclidRowMeta[]): MidiClip {
  interface ExpandedRow {
    notes: ClipNote[];
    totalSteps: number;
  }

  const expandedRows: ExpandedRow[] = [];

  for (const meta of metas) {
    const letters = parseMetaSequence(meta.metaSequence) ?? ["A"];
    let stepOffset = 0;
    const rowNotes: ClipNote[] = [];

    for (const letter of letters) {
      const row = meta.slots[letter] ?? meta.slots["A"];
      const pattern = computeStepPattern(row);
      const baseVel = (row.velocity ?? 100) / 127;
      const accVel = (row.accentVelocity ?? 127) / 127;

      for (let step = 0; step < row.length; step++) {
        const state = pattern[step];
        if (state !== "off") {
          const vel = state === "accent" ? accVel : baseVel;
          if (vel > 0) {
            rowNotes.push({
              note: row.note,
              channel: 0,
              velocity: vel,
              start_beats: (stepOffset + step) * STEP_BEATS,
              duration_beats: STEP_BEATS * GATE_RATIO,
            });
          }
        }
      }
      stepOffset += row.length;
    }

    expandedRows.push({ notes: rowNotes, totalSteps: stepOffset });
  }

  // Clip length = LCM of all rows' total expanded steps
  const totalClipSteps = expandedRows.reduce((acc, r) => r.totalSteps > 0 ? lcm(acc, r.totalSteps) : acc, 1);
  const lengthBeats = totalClipSteps * STEP_BEATS;

  // Tile each row's notes to fill the total clip length
  const allNotes: ClipNote[] = [];
  for (const er of expandedRows) {
    if (er.totalSteps === 0) continue;
    const repeatsNeeded = totalClipSteps / er.totalSteps;
    const rowLengthBeats = er.totalSteps * STEP_BEATS;
    for (let rep = 0; rep < repeatsNeeded; rep++) {
      const offset = rep * rowLengthBeats;
      for (const n of er.notes) {
        allNotes.push({ ...n, start_beats: n.start_beats + offset });
      }
      if (allNotes.length > MAX_CLIP_NOTES) break;
    }
    if (allNotes.length > MAX_CLIP_NOTES) break;
  }

  if (allNotes.length > MAX_CLIP_NOTES) {
    allNotes.length = MAX_CLIP_NOTES;
  }

  return { name: "Euclidean", length_beats: lengthBeats, notes: allNotes };
}

/** Generate a MidiClip from a chord sequence. Async because non-custom chords
 *  require a backend call to resolve chord notes. */
export async function chordsToClip(sequence: SequenceChord[], octaveStart: number): Promise<MidiClip> {
  const notes: ClipNote[] = [];
  let beatPos = 0;
  for (const chord of sequence) {
    let chordNotes: number[];
    if (chord.chordType === "Custom" && chord.customNotes) {
      chordNotes = chord.customNotes;
    } else {
      const rootMidi = octaveStart + chord.root;
      chordNotes = await invoke("get_chord_notes", { rootMidi, chordType: chord.chordType });
    }
    const fullBeats = chord.bars * 4;
    const durationBeats = fullBeats - 0.25;
    for (const n of chordNotes) {
      notes.push({ note: n, channel: 0, velocity: 0.8, start_beats: beatPos, duration_beats: durationBeats });
    }
    beatPos += fullBeats;
  }
  return { name: "Chord Sequence", length_beats: beatPos, notes };
}

// --- Persistence by client name ---

const STORAGE_KEY = "sequencer-configs";

interface StoredEuclidRowMeta {
  slots: Record<string, EuclidRow>;
  metaSequence: string;
}

interface StoredConfig {
  type: SequencerType;
  rows: StoredEuclidRowMeta[];
  noteTrigger?: boolean;
  chordSequence?: SequenceChord[];
  octaveStart?: number;
  version?: number;
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

function migrateOldEuclidRow(r: any, index: number): EuclidRow {
  return {
    note: r.note ?? 36 + index,
    length: r.length,
    hits: r.hits,
    rotation: r.rotation,
    accents: r.accents ?? 0,
    velocity: r.velocity ?? 100,
    accentVelocity: r.accentVelocity ?? 127,
    ...(r.manualPattern ? { manualPattern: r.manualPattern } : {}),
  };
}

export function loadConfigForClient(name: string, program: number = 1): StoredConfig | null {
  const all = loadAllConfigs();
  // Try per-program key first, then migrate from legacy (unkeyed) config
  let config = all[configKey(name, program)] as any;
  if (!config && program === 1 && all[name]) {
    config = all[name];
    all[configKey(name, 1)] = config;
    delete all[name];
    saveAllConfigs(all);
  }
  if (!config) return null;

  // Detect old format: rows are flat EuclidRow[] (have "note" at top level, no "slots")
  if (config.rows && config.rows.length > 0 && !config.version) {
    config.rows = (config.rows as any[]).map((oldRow: any, i: number) => {
      if (oldRow.slots) return oldRow; // already new format
      const migrated = migrateOldEuclidRow(oldRow, i);
      const slots: Record<string, EuclidRow> = {};
      for (const ch of ALPHABET) {
        slots[ch] = { ...migrated };
      }
      return { slots, metaSequence: "AAAA" } as StoredEuclidRowMeta;
    });
    config.version = 2;
    all[configKey(name, program)] = config;
    saveAllConfigs(all);
  }

  return config as StoredConfig;
}

export function saveConfigForClient(
  name: string, type: SequencerType, rowMetas: EuclidRowMeta[], program: number = 1, noteTrigger: boolean = false,
  chordSequence?: SequenceChord[], octaveStart?: number,
) {
  const all = loadAllConfigs();
  const storedRows: StoredEuclidRowMeta[] = rowMetas.map(m => ({
    slots: m.slots,
    metaSequence: m.metaSequence,
  }));
  all[configKey(name, program)] = { type, rows: storedRows, noteTrigger, chordSequence, octaveStart, version: 2 };
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
