<script lang="ts">
  import ConnectionFailure from "$lib/components/ConnectionFailure.svelte";
  import type { MemoryData } from "$lib/components/MachineCard.svelte";
  import { F32Value, Vec2U64Value } from "$lib/DataTypes";
  import {
    connectionStatus,
    getAllStatisticsByKey,
    updateMachines,
  } from "$lib/request.svelte";
  import { onDestroy, onMount } from "svelte";

  let averageCpuUsage: number = $state(0);
  let physicalMemory: MemoryData[] = $state([]);
  let swapMemory: MemoryData[] = $state([]);

  let totalPhysicalUsed_gib: number = $derived(
    physicalMemory.reduce(
      (sum, current) => sum + current.total_kib - current.available_kib,
      0
    ) /
      1024 /
      1024
  );
  let totalPhysical_gib: number = $derived(
    physicalMemory.reduce((sum, current) => sum + current.total_kib, 0) /
      1024 /
      1024
  );
  let physicalUsedPercent: number = $derived(
    totalPhysical_gib === 0
      ? 0
      : 100 * (totalPhysicalUsed_gib / totalPhysical_gib)
  );

  let totalSwapUsed_gib: number = $derived(
    swapMemory.reduce(
      (sum, current) => sum + current.total_kib - current.available_kib,
      0
    ) /
      1024 /
      1024
  );
  let totalSwap_gib: number = $derived(
    swapMemory.reduce((sum, current) => sum + current.total_kib, 0) /
      1024 /
      1024
  );
  let swapUsedPercent: number = $derived(
    totalPhysical_gib === 0 ? 0 : 100 * (totalSwapUsed_gib / totalSwap_gib)
  );

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

      const [cpuUsages, _physicalMemory, _swapMemory] = await Promise.all([
        getAllStatisticsByKey("cpu_usage"),
        getAllStatisticsByKey("memory_physical"),
        getAllStatisticsByKey("memory_swap"),
      ]);

      physicalMemory = _physicalMemory.map((memory) => {
        if (!(memory.data instanceof Vec2U64Value)) {
          throw new Error(
            `Expected memory_physical to have ${Vec2U64Value.stringName}. Instead received ${memory.data.stringName}`
          );
        }

        return {
          available_kib: memory.data.value[0],
          total_kib: memory.data.value[1],
        };
      });

      swapMemory = _swapMemory.map((memory) => {
        if (!(memory.data instanceof Vec2U64Value)) {
          throw new Error(
            `Expected memory_swap to have ${Vec2U64Value.stringName}. Instead received ${memory.data.stringName}`
          );
        }

        return {
          available_kib: memory.data.value[0],
          total_kib: memory.data.value[1],
        };
      });

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
    Average cpu usage across all machines:
    {(averageCpuUsage * 100).toFixed(2)}%
  </p>

  <p>
    Total physical memory usage across all machines: {totalPhysicalUsed_gib.toFixed(
      2
    )}/{totalPhysical_gib.toFixed(2)}GiB ({physicalUsedPercent.toFixed(2)}%)
  </p>

  <p>
    Total swap memory usage across all machines: {totalSwapUsed_gib.toFixed(
      2
    )}/{totalSwap_gib.toFixed(2)}GiB ({swapUsedPercent.toFixed(2)}%)
  </p>

  <small>Pie charts soon:tm:</small>
{/if}

<style lang="scss">
  h1 {
    text-align: center;
    margin-bottom: 0;
  }
</style>
