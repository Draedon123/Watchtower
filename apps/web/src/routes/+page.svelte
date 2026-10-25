<script lang="ts">
  import { getCpuUsages, getMachines } from "$lib/request";
  import { onDestroy, onMount } from "svelte";

  let machines: string[] | null = $state(null);
  let cpuUsages: (number | null)[] = $state([]);
  let loop: NodeJS.Timeout | null = null;
  let loadedFirstData = $state(false);

  function setIntervalImmediately(
    callback: () => void,
    delay_ms?: number
  ): NodeJS.Timeout {
    callback();

    return setInterval(callback, delay_ms);
  }

  onMount(() => {
    loop = setIntervalImmediately(async () => {
      machines = await getMachines().catch(() => null);
      cpuUsages =
        machines === null
          ? []
          : await getCpuUsages(machines).catch(() =>
              new Array(machines?.length).fill(null)
            );

      loadedFirstData = true;
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
{#if machines === null}
  {#if loadedFirstData}
    <p>Could not fetch machines. Perhaps the server is down?</p>
  {:else}
    <p>Loading data...</p>
  {/if}
{:else}
  {#if machines.length === 0}
    <p>No machines monitored!</p>
  {:else}
    {#each machines as machine, i (machine)}
      <h1>{machine}</h1>

      {#if cpuUsages[i] === null}
        <p>Could not fetch CPU usage</p>
      {:else}
        <p>CPU Usage: {(cpuUsages[i] * 100).toFixed(2)}%</p>
      {/if}
    {/each}
  {/if}
{/if}
