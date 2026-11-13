<script lang="ts">
  import { F32Value } from "$lib/DataTypes";
  import { getAllStatisticsByKey, getMachines } from "$lib/request";
  import { onDestroy, onMount } from "svelte";

  let averageCpuUsage: number = $state(0);
  let machines: string[] = $state([]);
  let loop: NodeJS.Timeout | null = null;

  function setIntervalImmediately(
    callback: () => void,
    delay_ms?: number
  ): NodeJS.Timeout {
    callback();

    return setInterval(callback, delay_ms);
  }

  onMount(() => {
    loop = setIntervalImmediately(async () => {
      machines = await getMachines();

      const cpuUsages = await getAllStatisticsByKey("cpu_usage");

      const sum = cpuUsages.reduce((sum, current) => {
        if (!(current.data instanceof F32Value)) {
          throw new Error(
            `Expected cpu_usage to have ${F32Value.stringName}. Instead received ${current.data.stringName}`
          );
        }

        return sum + current.data.value;
      }, 0);

      averageCpuUsage = cpuUsages.length === 0 ? 0 : sum / cpuUsages.length;
    }, 1000);
  });

  onDestroy(() => {
    if (loop !== null) {
      clearInterval(loop);
      loop = null;
    }
  });
</script>

<h1>Watchtower</h1>

<p>
  Average cpu usage across {machines.length} machines: {(
    averageCpuUsage * 100
  ).toFixed(2)}%
</p>

<style lang="scss">
  h1 {
    text-align: center;
  }
</style>
