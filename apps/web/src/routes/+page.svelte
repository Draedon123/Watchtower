<script lang="ts">
  import ConnectionFailure from "$lib/components/ConnectionFailure.svelte";
  import { F32Value } from "$lib/DataTypes";
  import {
    connectionStatus,
    getAllStatisticsByKey,
    updateMachines,
  } from "$lib/request.svelte";
  import { onDestroy, onMount } from "svelte";

  let averageCpuUsage: number = $state(0);
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
      await updateMachines();

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

{#if connectionStatus.connectedMachines instanceof Error}
  <ConnectionFailure />
{:else if connectionStatus.connectedMachines === null}
  <p class="center">Loading data...</p>
{:else}
  <p>
    Average cpu usage across {connectionStatus.connectedMachines.length} machines:
    {(averageCpuUsage * 100).toFixed(2)}%
  </p>
{/if}

<style lang="scss">
  h1 {
    text-align: center;
    margin-bottom: 0;
  }
</style>
