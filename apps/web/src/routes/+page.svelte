<script lang="ts">
  import { getCpuUsages, getMachines } from "$lib/request";
  import { onDestroy, onMount } from "svelte";

  let machines: string[] = $state([]);
  let cpuUsages: number[] = $state([]);
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
      cpuUsages = await getCpuUsages(machines);
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

<title>Watchtower</title>
<!-- eslint-disable-next-line svelte/require-each-key -->
{#each machines as machine, i}
  <h1>{machine}</h1>

  <p>CPU Usage: {(cpuUsages[i] * 100).toFixed(2)}%</p>
{/each}
