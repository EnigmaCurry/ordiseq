import { derived } from "svelte/store";
import { shaderConfig } from "./shaderStore";

function rgbToCss(rgb: number[]): string {
  const r = Math.round(rgb[0] * 255);
  const g = Math.round(rgb[1] * 255);
  const b = Math.round(rgb[2] * 255);
  return `${r}, ${g}, ${b}`;
}

function rgbToHex(rgb: number[]): string {
  const r = Math.round(rgb[0] * 255).toString(16).padStart(2, "0");
  const g = Math.round(rgb[1] * 255).toString(16).padStart(2, "0");
  const b = Math.round(rgb[2] * 255).toString(16).padStart(2, "0");
  return `#${r}${g}${b}`;
}

/** Compute relative luminance from linear-space 0-1 RGB (sRGB gamma). */
function relativeLuminance(rgb: number[]): number {
  const linearize = (c: number) =>
    c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  return (
    0.2126 * linearize(rgb[0]) +
    0.7152 * linearize(rgb[1]) +
    0.0722 * linearize(rgb[2])
  );
}

/** Return dark or light text color for best contrast on the given background. */
function textOnColor(rgb: number[]): string {
  return relativeLuminance(rgb) > 0.179 ? "0, 0, 0" : "255, 255, 255";
}

export const themeColors = derived(shaderConfig, ($config) => {
  const color1 = ($config.uniforms.u_color1 as number[]) || [1, 0.08, 0.58];
  const color2 = ($config.uniforms.u_color2 as number[]) || [0, 1, 0.87];
  const color3 = ($config.uniforms.u_color3 as number[]) || [0.74, 0.58, 0.98];
  const color4 = ($config.uniforms.u_color4 as number[]) || [0.31, 0.98, 0.48];
  const overlay = ($config.uniforms.u_overlay as number) ?? 0.4;

  return {
    color1: rgbToCss(color1),
    color2: rgbToCss(color2),
    color3: rgbToCss(color3),
    color4: rgbToCss(color4),
    textOn1: textOnColor(color1),
    textOn2: textOnColor(color2),
    color1Hex: rgbToHex(color1),
    color2Hex: rgbToHex(color2),
    color3Hex: rgbToHex(color3),
    color4Hex: rgbToHex(color4),
    overlay,
  };
});

export function applyTheme(colors: {
  color1: string;
  color2: string;
  color3: string;
  color4: string;
  textOn1: string;
  textOn2: string;
  overlay: number;
}) {
  const root = document.documentElement;
  root.style.setProperty("--color-1", colors.color1);
  root.style.setProperty("--color-2", colors.color2);
  root.style.setProperty("--color-3", colors.color3);
  root.style.setProperty("--color-4", colors.color4);
  root.style.setProperty("--text-on-1", colors.textOn1);
  root.style.setProperty("--text-on-2", colors.textOn2);
  root.style.setProperty("--overlay-opacity", colors.overlay.toString());
}
