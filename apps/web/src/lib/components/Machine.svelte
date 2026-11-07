<script lang="ts" module>
  type Nullable<T> = {
    [P in keyof T]: T[P] | null;
  };

  type MemoryData = {
    total_kib: number;
    available_kib: number;
  };

  type MachineData = Partial<
    Nullable<{
      /** 0 - 1*/
      cpuUsage: number;
      physicalMemory: MemoryData;
      swapMemory: MemoryData;
    }>
  > & { name: string };

  export type { MachineData, MemoryData };
</script>

<script lang="ts">
  import { untrack } from "svelte";

  import LineGraph from "./graphs/LineGraph.svelte";

  let data: MachineData = $props();

  let cpuUsages: { x: number[]; y: number[] } = $state({ x: [], y: [] });
  const trackTime_s = 60;

  $effect(() => {
    const cpuUsage = data.cpuUsage;

    if (cpuUsage === undefined || cpuUsage === null) {
      return;
    }

    untrack(() => {
      const x = [...cpuUsages.x];
      const y = [...cpuUsages.y];

      if (x.length <= trackTime_s) {
        x.push(x.length);
        y.push(cpuUsage);
      } else {
        y.shift();
        y.push(cpuUsage);
      }

      cpuUsages = { x, y };
    });
  });
</script>

<div class="container">
  <h2>{data.name}</h2>

  <div class="grid">
    <div>
      <div class="stat">
        {#if !data.cpuUsage}
          <p>Could not fetch CPU usage</p>
        {:else}
          <p>CPU Usage: {(data.cpuUsage * 100).toFixed(2)}%</p>
        {/if}
      </div>
      <div class="stat">
        {#if !data.physicalMemory}
          <p>Could not fetch physical memory</p>
        {:else}
          {@const usedMemory_gib =
            (data.physicalMemory.total_kib -
              data.physicalMemory.available_kib) /
            1024 /
            1024}
          {@const totalMemory_gib = data.physicalMemory.total_kib / 1024 / 1024}
          {@const memoryUsagePercent = (usedMemory_gib / totalMemory_gib) * 100}
          <p>
            Physical Memory: {usedMemory_gib.toFixed(
              2
            )}/{totalMemory_gib.toFixed(2)} GiB ({memoryUsagePercent.toFixed(
              2
            )}%)
          </p>
        {/if}
      </div>
      <div class="stat">
        {#if !data.swapMemory}
          <p>Could not fetch swap memory</p>
        {:else}
          {@const usedMemory_gib =
            (data.swapMemory.total_kib - data.swapMemory.available_kib) /
            1024 /
            1024}
          {@const totalMemory_gib = data.swapMemory.total_kib / 1024 / 1024}
          {@const memoryUsagePercent = (usedMemory_gib / totalMemory_gib) * 100}
          <p>
            Swap Memory: {usedMemory_gib.toFixed(2)}/{totalMemory_gib.toFixed(
              2
            )} GiB ({memoryUsagePercent.toFixed(2)}%)
          </p>
        {/if}
      </div>
    </div>

    <div class="graph">
      <LineGraph
        data={cpuUsages}
        markerColour="#b0b"
        lineColour="#8188"
        lineWidth={3}
        markerRadius={1}
        left={0}
        right={trackTime_s}
        bottom={0}
        top={1}
        borderColour="#fff"
      ></LineGraph>
    </div>
  </div>
</div>

<style lang="scss">
  h2 {
    text-align: center;
  }

  .container {
    flex-grow: 1;
    max-width: 100%;

    background: #0008;

    padding: 5px;

    border: 1px solid #808080;
    border-radius: 10px;
  }

  .grid {
    display: grid;
    grid-template-columns: max-content 1fr;
  }

  .graph {
    max-width: 100%;
    height: 200px;
    margin: 0 5ch 1em 5ch;
  }

  @media (max-width: 400px) {
    .container {
      max-width: 600px;
    }
  }

  @media (min-width: 400px) and (max-width: 600px) {
    .container {
      min-width: 400px;
      max-width: 600px;
    }
  }
</style>
