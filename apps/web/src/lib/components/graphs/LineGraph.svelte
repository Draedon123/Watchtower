<script lang="ts">
  type Props = {
    x: number[];
    y: number[];
  };

  let { x, y }: Props = $props();

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

  let width: number = $state(1);
  let height: number = $state(1);

  function xPos(x: number) {
    return width * ((x - minX) / rangeX);
  }

  function yPos(y: number) {
    return height - height * ((y - minY) / rangeY);
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

  .point,
  .connector {
    fill: white;
    stroke: white;
  }
</style>
