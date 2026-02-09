<script lang="ts">
  interface Props {
    value: number;
    min: number;
    max: number;
    label?: string;
    formatValue?: (v: number) => string;
    onchange: (value: number) => void;
  }

  let { value, min, max, label, formatValue, onchange }: Props = $props();

  const SIZE = 40;
  const CX = SIZE / 2;
  const CY = SIZE / 2;
  const R = 14;
  const STROKE = 3;
  // Arc from 135deg to 405deg (270deg sweep)
  const START_ANGLE = 135;
  const END_ANGLE = 405;
  const SWEEP = END_ANGLE - START_ANGLE;

  function angleForValue(v: number): number {
    if (max <= min) return START_ANGLE;
    return START_ANGLE + ((v - min) / (max - min)) * SWEEP;
  }

  function polarToXY(angleDeg: number, r: number): { x: number; y: number } {
    const rad = (angleDeg * Math.PI) / 180;
    return { x: CX + r * Math.cos(rad), y: CY + r * Math.sin(rad) };
  }

  function arcPath(startDeg: number, endDeg: number, r: number): string {
    const s = polarToXY(startDeg, r);
    const e = polarToXY(endDeg, r);
    const sweep = endDeg - startDeg;
    const large = sweep > 180 ? 1 : 0;
    return `M ${s.x} ${s.y} A ${r} ${r} 0 ${large} 1 ${e.x} ${e.y}`;
  }

  let dragging = false;
  let dragStartY = 0;
  let dragStartValue = 0;
  let containerEl: HTMLDivElement;

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    dragStartY = e.clientY;
    dragStartValue = value;
    containerEl.setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    const dy = dragStartY - e.clientY;
    const range = max - min;
    // 150px drag = full range
    const delta = Math.round((dy / 150) * range);
    const newVal = Math.max(min, Math.min(max, dragStartValue + delta));
    if (newVal !== value) {
      onchange(newVal);
    }
  }

  function onPointerUp() {
    dragging = false;
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const dir = e.deltaY < 0 ? 1 : -1;
    const newVal = Math.max(min, Math.min(max, value + dir));
    if (newVal !== value) {
      onchange(newVal);
    }
  }

  let currentAngle = $derived(angleForValue(value));
  let pointer = $derived(polarToXY(currentAngle, R - 3));
  let valuePath = $derived(value > min ? arcPath(START_ANGLE, currentAngle, R) : "");
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="dial"
  bind:this={containerEl}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
>
  <svg width={SIZE} height={SIZE} viewBox="0 0 {SIZE} {SIZE}">
    <!-- Background track -->
    <path
      d={arcPath(START_ANGLE, END_ANGLE, R)}
      fill="none"
      stroke="rgba(255,255,255,0.12)"
      stroke-width={STROKE}
      stroke-linecap="round"
    />
    <!-- Value arc -->
    {#if valuePath}
      <path
        d={valuePath}
        fill="none"
        stroke="rgb(var(--color-2))"
        stroke-width={STROKE}
        stroke-linecap="round"
      />
    {/if}
    <!-- Pointer dot -->
    <circle cx={pointer.x} cy={pointer.y} r="2.5" fill="#f8f8f2" />
  </svg>
  <span class="dial-value">{formatValue ? formatValue(value) : value}</span>
  {#if label}
    <span class="dial-label">{label}</span>
  {/if}
</div>

<style>
  .dial {
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: ns-resize;
    user-select: none;
    touch-action: none;
  }

  .dial-value {
    font-size: 0.75rem;
    color: #f8f8f2;
    margin-top: -4px;
    pointer-events: none;
  }

  .dial-label {
    font-size: 0.6rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    pointer-events: none;
  }
</style>
