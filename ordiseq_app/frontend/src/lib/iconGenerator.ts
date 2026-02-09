import { getCurrentWindow } from "@tauri-apps/api/window";
import { Image } from "@tauri-apps/api/image";

export type IconShape =
  | "triangle"
  | "circle"
  | "square"
  | "pentagon"
  | "hexagon"
  | "star"
  | "diamond"
  | "cross"
  | "heart"
  | "lightning"
  | "spiral"
  | "eye"
  | "moon"
  | "infinity"
  | "wave"
  | "arrow"
  | "grid"
  | "ufo"
  | "vulcan"
  | "hacker";

export const iconShapes: { value: IconShape; label: string }[] = [
  { value: "triangle", label: "Triangle" },
  { value: "circle", label: "Circle" },
  { value: "square", label: "Square" },
  { value: "pentagon", label: "Pentagon" },
  { value: "hexagon", label: "Hexagon" },
  { value: "star", label: "Star" },
  { value: "diamond", label: "Diamond" },
  { value: "cross", label: "Cross" },
  { value: "heart", label: "Heart" },
  { value: "lightning", label: "Lightning" },
  { value: "spiral", label: "Spiral" },
  { value: "eye", label: "Eye" },
  { value: "moon", label: "Moon" },
  { value: "infinity", label: "Infinity" },
  { value: "wave", label: "Wave" },
  { value: "arrow", label: "Arrow" },
  { value: "grid", label: "Grid" },
  { value: "ufo", label: "UFO" },
  { value: "vulcan", label: "Vulcan" },
  { value: "hacker", label: "Hacker" },
];

// --- helpers ---

function setupCtx(ctx: CanvasRenderingContext2D, size: number) {
  ctx.strokeStyle = "white";
  ctx.lineWidth = Math.max(1, size * 0.032);
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
}

function polygon(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
  n: number,
  offsetAngle = 0,
) {
  ctx.beginPath();
  for (let i = 0; i < n; i++) {
    const angle = offsetAngle + (i * 2 * Math.PI) / n;
    const x = cx + r * Math.cos(angle);
    const y = cy + r * Math.sin(angle);
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.closePath();
}

// --- shape renderers ---

const sidesMap: Record<string, number> = {
  triangle: 3,
  square: 4,
  pentagon: 5,
  hexagon: 6,
};

function renderPolygon(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
  shape: string,
) {
  const n = sidesMap[shape];
  const offset = n % 2 === 0 ? -Math.PI / 2 + Math.PI / n : -Math.PI / 2;
  polygon(ctx, cx, cy, r, n, offset);
  ctx.stroke();
}

function renderStar(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  const inner = r * 0.4;
  ctx.beginPath();
  for (let i = 0; i < 10; i++) {
    const angle = -Math.PI / 2 + (i * Math.PI) / 5;
    const rad = i % 2 === 0 ? r : inner;
    const x = cx + rad * Math.cos(angle);
    const y = cy + rad * Math.sin(angle);
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.closePath();
  ctx.stroke();
}

function renderDiamond(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  polygon(ctx, cx, cy, r, 4, -Math.PI / 2);
  ctx.stroke();
}

function renderCross(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  const w = r * 0.35;
  ctx.beginPath();
  ctx.moveTo(cx - w, cy - r);
  ctx.lineTo(cx + w, cy - r);
  ctx.lineTo(cx + w, cy - w);
  ctx.lineTo(cx + r, cy - w);
  ctx.lineTo(cx + r, cy + w);
  ctx.lineTo(cx + w, cy + w);
  ctx.lineTo(cx + w, cy + r);
  ctx.lineTo(cx - w, cy + r);
  ctx.lineTo(cx - w, cy + w);
  ctx.lineTo(cx - r, cy + w);
  ctx.lineTo(cx - r, cy - w);
  ctx.lineTo(cx - w, cy - w);
  ctx.closePath();
  ctx.stroke();
}

function renderHeart(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  const s = r * 1.05;
  ctx.beginPath();
  ctx.moveTo(cx, cy + s * 0.7);
  ctx.bezierCurveTo(cx - s * 1.2, cy - s * 0.1, cx - s * 0.5, cy - s * 1.0, cx, cy - s * 0.4);
  ctx.bezierCurveTo(cx + s * 0.5, cy - s * 1.0, cx + s * 1.2, cy - s * 0.1, cx, cy + s * 0.7);
  ctx.stroke();
}

function renderLightning(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  ctx.beginPath();
  ctx.moveTo(cx + r * 0.15, cy - r);
  ctx.lineTo(cx - r * 0.4, cy + r * 0.05);
  ctx.lineTo(cx + r * 0.05, cy + r * 0.05);
  ctx.lineTo(cx - r * 0.15, cy + r);
  ctx.lineTo(cx + r * 0.4, cy - r * 0.05);
  ctx.lineTo(cx - r * 0.05, cy - r * 0.05);
  ctx.closePath();
  ctx.stroke();
}

function renderSpiral(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  ctx.beginPath();
  const turns = 2.5;
  const steps = 120;
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const angle = turns * 2 * Math.PI * t;
    const rad = r * t;
    const x = cx + rad * Math.cos(angle - Math.PI / 2);
    const y = cy + rad * Math.sin(angle - Math.PI / 2);
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.stroke();
}

function renderEye(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  // Almond shape: two arcs
  ctx.beginPath();
  ctx.moveTo(cx - r, cy);
  ctx.quadraticCurveTo(cx, cy - r * 0.9, cx + r, cy);
  ctx.quadraticCurveTo(cx, cy + r * 0.9, cx - r, cy);
  ctx.stroke();
  // Pupil
  ctx.beginPath();
  ctx.arc(cx, cy, r * 0.3, 0, Math.PI * 2);
  ctx.stroke();
}

function renderMoon(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  ctx.beginPath();
  ctx.arc(cx, cy, r, -Math.PI * 0.4, Math.PI * 0.4, false);
  // Inner cutout arc (larger radius, reversed)
  const offset = r * 0.55;
  ctx.arc(cx + offset, cy, r * 0.75, Math.PI * 0.36, -Math.PI * 0.36, true);
  ctx.closePath();
  ctx.stroke();
}

function renderInfinity(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  ctx.beginPath();
  const steps = 120;
  for (let i = 0; i <= steps; i++) {
    const t = (i / steps) * 2 * Math.PI;
    // Lemniscate of Bernoulli parametric form
    const denom = 1 + Math.sin(t) * Math.sin(t);
    const x = cx + (r * Math.cos(t)) / denom;
    const y = cy + (r * Math.sin(t) * Math.cos(t)) / denom;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.closePath();
  ctx.stroke();
}

function renderWave(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  const cycles = 2;
  const steps = 80;
  ctx.beginPath();
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const x = cx - r + t * 2 * r;
    const y = cy + Math.sin(t * cycles * 2 * Math.PI) * r * 0.5;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.stroke();
}

function renderArrow(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  // Upward chevron arrow
  ctx.beginPath();
  ctx.moveTo(cx - r * 0.7, cy + r * 0.3);
  ctx.lineTo(cx, cy - r * 0.5);
  ctx.lineTo(cx + r * 0.7, cy + r * 0.3);
  ctx.stroke();
  // Shaft
  ctx.beginPath();
  ctx.moveTo(cx, cy - r * 0.5);
  ctx.lineTo(cx, cy + r);
  ctx.stroke();
}

function renderGrid(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  const g = r * 0.55;
  // Vertical lines
  ctx.beginPath();
  ctx.moveTo(cx - g, cy - r); ctx.lineTo(cx - g, cy + r);
  ctx.moveTo(cx + g, cy - r); ctx.lineTo(cx + g, cy + r);
  // Horizontal lines
  ctx.moveTo(cx - r, cy - g); ctx.lineTo(cx + r, cy - g);
  ctx.moveTo(cx - r, cy + g); ctx.lineTo(cx + r, cy + g);
  ctx.stroke();
}

function renderUfo(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  // Dome (top half ellipse)
  ctx.beginPath();
  ctx.ellipse(cx, cy - r * 0.05, r * 0.45, r * 0.55, 0, Math.PI, 0);
  ctx.stroke();
  // Saucer body (wide ellipse)
  ctx.beginPath();
  ctx.ellipse(cx, cy - r * 0.05, r, r * 0.3, 0, 0, Math.PI * 2);
  ctx.stroke();
  // Beam (two diverging lines from bottom)
  ctx.beginPath();
  ctx.moveTo(cx - r * 0.25, cy + r * 0.25);
  ctx.lineTo(cx - r * 0.55, cy + r);
  ctx.moveTo(cx + r * 0.25, cy + r * 0.25);
  ctx.lineTo(cx + r * 0.55, cy + r);
  // Beam bottom edge
  ctx.moveTo(cx - r * 0.55, cy + r);
  ctx.lineTo(cx + r * 0.55, cy + r);
  ctx.stroke();
}

function renderVulcan(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  r: number,
) {
  // Palm base
  const bx = cx, by = cy + r * 0.95;
  const wristW = r * 0.45;

  // Wrist
  ctx.beginPath();
  ctx.moveTo(bx - wristW, by);
  ctx.lineTo(bx + wristW, by);
  ctx.stroke();

  // Palm narrows upward
  const palmTop = cy + r * 0.05;
  const palmW = r * 0.55;

  ctx.beginPath();
  ctx.moveTo(bx - wristW, by);
  ctx.lineTo(bx - palmW, palmTop);
  ctx.moveTo(bx + wristW, by);
  ctx.lineTo(bx + palmW, palmTop);
  ctx.stroke();

  // Fingers - V split: index+middle together, ring+pinky together
  const fingerBase = palmTop;
  const fingerLen = r * 0.9;
  const splayInner = r * 0.08; // gap between paired fingers
  const splayOuter = r * 0.2;  // spread of each finger from pair center
  const splitGap = r * 0.15;   // the Vulcan V gap

  // Left pair (index + middle)
  const lPairCx = cx - splitGap;
  ctx.beginPath();
  ctx.moveTo(lPairCx - splayInner, fingerBase);
  ctx.lineTo(lPairCx - splayOuter - splayInner, fingerBase - fingerLen);
  ctx.moveTo(lPairCx + splayInner, fingerBase);
  ctx.lineTo(lPairCx - splayInner, fingerBase - fingerLen);
  ctx.stroke();

  // Right pair (ring + pinky)
  const rPairCx = cx + splitGap;
  ctx.beginPath();
  ctx.moveTo(rPairCx - splayInner, fingerBase);
  ctx.lineTo(rPairCx + splayInner, fingerBase - fingerLen);
  ctx.moveTo(rPairCx + splayInner, fingerBase);
  ctx.lineTo(rPairCx + splayOuter + splayInner, fingerBase - fingerLen);
  ctx.stroke();

  // Thumb (angled out to the left)
  ctx.beginPath();
  ctx.moveTo(bx - palmW, palmTop + r * 0.2);
  ctx.lineTo(bx - palmW - r * 0.3, palmTop - r * 0.3);
  ctx.stroke();
}

// Deterministic pseudo-random from seed
function mulberry32(seed: number) {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function renderHacker(ctx: CanvasRenderingContext2D, size: number) {
  const rand = mulberry32(42);
  const cols = Math.floor(size / 3);
  const dotSize = size / cols;

  for (let col = 0; col < cols; col++) {
    const x = (col + 0.5) * dotSize;
    const streamHead = Math.floor(rand() * (cols + 4));
    const streamLen = 3 + Math.floor(rand() * 5);

    for (let row = 0; row < cols; row++) {
      const dist = streamHead - row;
      if (dist < 0 || dist >= streamLen) continue;

      const y = (row + 0.5) * dotSize;
      const alpha = dist === 0 ? 1.0 : Math.max(0.15, 1.0 - dist / streamLen);
      ctx.fillStyle = `rgba(255,255,255,${alpha})`;
      const r = dotSize * 0.35;
      ctx.fillRect(x - r, y - r, r * 2, r * 2);
    }
  }
}

// --- main render ---

function renderIcon(shape: IconShape, size: number): ImageData {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = size;
  const ctx = canvas.getContext("2d")!;

  if (shape === "hacker") {
    renderHacker(ctx, size);
    return ctx.getImageData(0, 0, size, size);
  }

  const padding = Math.max(3, size * 0.12);
  const cx = size / 2;
  const cy = size / 2;
  const r = size / 2 - padding;

  setupCtx(ctx, size);

  if (shape === "circle") {
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.stroke();
  } else if (shape in sidesMap) {
    renderPolygon(ctx, cx, cy, r, shape);
  } else if (shape === "star") {
    renderStar(ctx, cx, cy, r);
  } else if (shape === "diamond") {
    renderDiamond(ctx, cx, cy, r);
  } else if (shape === "cross") {
    renderCross(ctx, cx, cy, r);
  } else if (shape === "heart") {
    renderHeart(ctx, cx, cy, r);
  } else if (shape === "lightning") {
    renderLightning(ctx, cx, cy, r);
  } else if (shape === "spiral") {
    renderSpiral(ctx, cx, cy, r);
  } else if (shape === "eye") {
    renderEye(ctx, cx, cy, r);
  } else if (shape === "moon") {
    renderMoon(ctx, cx, cy, r);
  } else if (shape === "infinity") {
    renderInfinity(ctx, cx, cy, r);
  } else if (shape === "wave") {
    renderWave(ctx, cx, cy, r);
  } else if (shape === "arrow") {
    renderArrow(ctx, cx, cy, r);
  } else if (shape === "grid") {
    renderGrid(ctx, cx, cy, r);
  } else if (shape === "ufo") {
    renderUfo(ctx, cx, cy, r);
  } else if (shape === "vulcan") {
    renderVulcan(ctx, cx, cy, r);
  }

  return ctx.getImageData(0, 0, size, size);
}

export async function applyIcon(shape: IconShape): Promise<void> {
  const size = 32;
  const imageData = renderIcon(shape, size);
  const rgba = new Uint8Array(imageData.data.buffer);
  const icon = await Image.new(rgba, size, size);
  await getCurrentWindow().setIcon(icon);
}
