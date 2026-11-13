<script lang="ts">
  type Text = Partial<{
    size_px: number;
    colour: string;
  }> & { message: string };

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
    title?: Text;
    xLabel?: Text;
    yLabel?: Text;
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
    title,
    xLabel,
    yLabel,
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

  let plotLeft = $derived(_left ?? minX - PADDING * rangeX);
  let plotRight = $derived(_right ?? maxX + PADDING * rangeX);
  let plotBottom = $derived(_bottom ?? minY - PADDING * rangeY);
  let plotTop = $derived(_top ?? maxY + PADDING * rangeY);

  let svgWidth_px: number = $state(1);
  let svgHeight_px: number = $state(1);

  let titleHeight_px: number = $derived(title?.size_px ?? 20);
  let titlePadding_px: number = $derived(titleHeight_px * 1);
  let titleHeightEffective_px: number = $derived(
    title === undefined ? 0 : titleHeight_px + titlePadding_px
  );

  let xLabelHeight_px: number = $derived(xLabel?.size_px ?? 16);
  let xLabelPadding_px: number = $derived(xLabelHeight_px * 1);
  let xLabelHeightEffective_px: number = $derived(
    xLabel === undefined ? 0 : xLabelHeight_px + xLabelPadding_px
  );

  let yLabelHeight_px: number = $derived(yLabel?.size_px ?? 16);
  let yLabelPadding_px: number = $derived(yLabelHeight_px * 1);
  let yLabelHeightEffective_px: number = $derived(
    yLabel === undefined ? 0 : yLabelHeight_px + yLabelPadding_px
  );

  let plotWidth_px: number = $derived(svgWidth_px - yLabelHeightEffective_px);
  let plotHeight_px: number = $derived(
    svgHeight_px - titleHeightEffective_px - xLabelHeightEffective_px
  );

  let yAxisGrid = $derived.by(() => {
    const NUM_LINES = 5;
    const gridSpacing = (plotTop - plotBottom) / NUM_LINES;
    const grid: number[] = [];

    for (let i = 0; i <= NUM_LINES; i++) {
      grid.push(plotBottom + i * gridSpacing);
    }

    return grid;
  });

  function xPlotToPx(x: number) {
    return (
      yLabelHeightEffective_px +
      plotWidth_px * ((x - plotLeft) / (plotRight - plotLeft))
    );
  }

  function yPlotToPx(y: number) {
    return (
      svgHeight_px -
      xLabelHeightEffective_px -
      plotHeight_px * ((y - plotBottom) / (plotTop - plotBottom))
    );
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
  width={svgWidth_px}
  height={svgHeight_px}
  bind:clientWidth={svgWidth_px}
  bind:clientHeight={svgHeight_px}
>
  {#if title !== undefined}
    <text
      class="title"
      style="font-size: {titleHeight_px}; stroke: {title.colour ??
        '#fff'}; fill: {title.colour ?? '#fff'}"
      x={svgWidth_px / 2}
      y={titleHeightEffective_px / 2}>{title.message}</text
    >
  {/if}

  {#if xLabel !== undefined}
    <text
      class="x-label"
      style="font-size: {xLabelHeight_px}; stroke: {xLabel.colour ??
        '#fff'}; fill: {xLabel.colour ?? '#fff'}"
      x={svgWidth_px / 2}
      y={svgHeight_px - xLabelHeightEffective_px / 2}>{xLabel.message}</text
    >
  {/if}

  {#if yLabel !== undefined}
    <text
      class="y-label"
      style="font-size: {yLabelHeight_px}; stroke: {yLabel.colour ??
        '#fff'}; fill: {yLabel.colour ?? '#fff'}"
      x={yLabelHeightEffective_px / 2}
      y={svgHeight_px / 2}>{yLabel.message}</text
    >
  {/if}

  {#each yAxisGrid as y (y)}
    <line
      class="grid"
      x1={xPlotToPx(plotLeft)}
      y1={yPlotToPx(y)}
      x2={xPlotToPx(plotRight)}
      y2={yPlotToPx(y)}
    />
  {/each}

  {#each { length: x.length }, i}
    {@const xpos = xPlotToPx(x[i])}
    {@const ypos = yPlotToPx(y[i])}

    {#if i < x.length - 1}
      <line
        class="connector"
        x1={xpos}
        y1={ypos}
        x2={xPlotToPx(x[i + 1])}
        y2={yPlotToPx(y[i + 1])}
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

  .title,
  .x-label,
  .y-label {
    text-anchor: middle;
    alignment-baseline: middle;
  }

  .y-label {
    transform-origin: center;
    transform-box: fill-box;
    transform: rotate(-90deg);
  }
</style>
