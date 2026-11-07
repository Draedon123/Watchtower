<script lang="ts">
  type Props = {
    x: number[];
    y: number[];
    left?: number;
    bottom?: number;
    right?: number;
    top?: number;
  };

  let {
    x,
    y,
    left: _left,
    bottom: _bottom,
    right: _right,
    top: _top,
  }: Props = $props();

  let minX = $derived(
    x.reduce((min, current) => (current < min ? current : min))
  );
  let maxX = $derived(
    x.reduce((max, current) => (current > max ? current : max))
  );
  let minY = $derived(
    y.reduce((min, current) => (current < min ? current : min))
  );
  let maxY = $derived(
    y.reduce((max, current) => (current > max ? current : max))
  );

  let rangeX = $derived(maxX - minX);
  let rangeY = $derived(maxY - minY);

  const PADDING = 0.1;

  let left = $derived(_left ?? minX - PADDING * rangeX);
  let right = $derived(_right ?? maxX + PADDING * rangeX);
  let bottom = $derived(_bottom ?? minY - PADDING * rangeY);
  let top = $derived(_top ?? maxY + PADDING * rangeY);

  let width: number = $state(1);
  let height: number = $state(1);

  function xPos(x: number) {
    return width * ((x - left) / (right - left));
  }

  function yPos(y: number) {
    return height - height * ((y - bottom) / (top - bottom));
  }
</script>

<svg {width} {height} bind:clientWidth={width} bind:clientHeight={height}>
  {#each { length: x.length }, i}
    {@const xpos = xPos(x[i])}
    {@const ypos = yPos(y[i])}
    <circle class="point" cx={xpos} cy={ypos} r="5" />
    {#if i > 0}
      <line
        class="connector"
        x1={xPos(x[i - 1])}
        y1={yPos(y[i - 1])}
        x2={xpos}
        y2={ypos}
      />
    {/if}
  {/each}
</svg>

<style lang="scss">
  svg {
    width: 100%;
    height: 100%;
  }

  .point {
    fill: white;
  }

  .connector {
    stroke: white;
  }
</style>
