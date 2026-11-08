<script lang="ts">
  type Props = {
    data: { x: number[]; y: number[] };
    left?: number;
    bottom?: number;
    right?: number;
    top?: number;
    markerColour?: string;
    lineColour?: string;
    markerRadius?: number;
    lineWidth?: number;
    borderColour?: string;
  };

  let {
    data,
    left: _left,
    bottom: _bottom,
    right: _right,
    top: _top,
    markerColour = "#fff",
    lineColour = "#fff",
    markerRadius = 5,
    lineWidth = 2,
    borderColour = "none",
  }: Props = $props();

  let x = $derived(data.x);
  let y = $derived(data.y);

  let minX = $derived(min(x));
  let maxX = $derived(max(x));
  let minY = $derived(min(y));
  let maxY = $derived(max(y));

  let rangeX = $derived(maxX - minX);
  let rangeY = $derived(maxY - minY);

  const PADDING = 0.1;

  let left = $derived(_left ?? minX - PADDING * rangeX);
  let right = $derived(_right ?? maxX + PADDING * rangeX);
  let bottom = $derived(_bottom ?? minY - PADDING * rangeY);
  let top = $derived(_top ?? maxY + PADDING * rangeY);

  let width: number = $state(1);
  let height: number = $state(1);

  let yAxisGrid = $derived.by(() => {
    const NUM_LINES = 4;
    const gridSpacing = (top - bottom) / (NUM_LINES + 1);
    const grid: number[] = [];

    for (let i = 1; i <= NUM_LINES; i++) {
      grid.push(bottom + i * gridSpacing);
    }

    return grid;
  });

  function xPos(x: number) {
    return width * ((x - left) / (right - left));
  }

  function yPos(y: number) {
    return height - height * ((y - bottom) / (top - bottom));
  }

  function min(array: number[]): number {
    return array.reduce(
      (min, current) => (current < min ? current : min),
      Infinity
    );
  }

  function max(array: number[]): number {
    return array.reduce(
      (max, current) => (current > max ? current : max),
      -Infinity
    );
  }
</script>

<svg
  {width}
  {height}
  bind:clientWidth={width}
  bind:clientHeight={height}
  style="border: 1px solid {borderColour}"
>
  {#each yAxisGrid as y (y)}
    <line
      class="grid"
      x1={xPos(left)}
      y1={yPos(y)}
      x2={xPos(right)}
      y2={yPos(y)}
    />
  {/each}

  {#each { length: x.length }, i}
    {@const xpos = xPos(x[i])}
    {@const ypos = yPos(y[i])}

    {#if i < x.length - 1}
      <line
        class="connector"
        x1={xpos}
        y1={ypos}
        x2={xPos(x[i + 1])}
        y2={yPos(y[i + 1])}
        stroke={lineColour}
        stroke-width={lineWidth}
      />
    {/if}

    <circle
      class="point"
      cx={xpos}
      cy={ypos}
      r={markerRadius}
      fill={markerColour}
    />
  {/each}
</svg>

<style lang="scss">
  svg {
    width: 100%;
    height: 100%;
  }

  .grid {
    stroke: #8888;
  }
</style>
