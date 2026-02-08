export interface MidiInfo {
  path: string;
  title: string;
  note_count: number;
}

export interface GenerateMidiParams {
  sequence_type: string;
}

// Re-export shader types
export type { ShaderConfig } from "./shaderStore";
export { setShader, setUniforms, resetShader, exampleShaders } from "./shaderStore";
