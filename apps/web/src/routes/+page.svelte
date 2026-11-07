<script lang="ts">
  import Machine, {
    type MachineData,
    type MemoryData,
  } from "$lib/components/Machine.svelte";
  import {
    currentTime_ms,
    msToString,
    relativeTimeToString,
  } from "$lib/dateTime";
  import {
    getCpuUsages,
    getMachines,
    getPhysicalMemory,
    getSwapMemory,
  } from "$lib/request";
  import { onDestroy, onMount } from "svelte";

  let machines: string[] | null = $state(null);
  let cpuUsages: (number | null)[] = $state([]);
  let physicalMemory: (MemoryData | null)[] = $state([]);
  let swapMemory: (MemoryData | null)[] = $state([]);
  let machineData: MachineData[] = $derived(
    machines === null
      ? []
      : (machines as string[]).map((name, i) => ({
          name,
          cpuUsage: cpuUsages[i],
          physicalMemory: physicalMemory[i],
          swapMemory: swapMemory[i],
        }))
  );
  let loadedFirstData = $state(false);
  let disconnectedSince: number | null = $state(null);
  let currentTime: number = $state(currentTime_ms());
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
      machines = await getMachines()
        .then((machines) => {
          disconnectedSince = null;

          return machines;
        })
        .catch(() => {
          if (disconnectedSince === null) {
            disconnectedSince = currentTime_ms();
          }

          currentTime = currentTime_ms();

          return null;
        });
      loadedFirstData = true;

      cpuUsages = await getCpuUsages(machines);
      physicalMemory = await getPhysicalMemory(machines);
      swapMemory = await getSwapMemory(machines);
    }, 1000);
  });

  onDestroy(() => {
    if (loop !== null) {
      clearInterval(loop);
      loop = null;
    }
  });
</script>

<title>Watchtower</title>

<h1>Watchtower</h1>

<div class="machines">
  {#if machines === null}
    {#if loadedFirstData}
      {@const downtime_ms = currentTime - (disconnectedSince as number)}
      <div class="offline">
        <h2>Server Offline!</h2>

        <p>
          Offline for at least {relativeTimeToString(downtime_ms / 1000)} (since {msToString(
            currentTime - downtime_ms,
            "{hh24}:{min}:{ss}, {month} {date}, {yyyy}"
          )})
        </p>
      </div>
    {:else}
      <p>Loading data...</p>
    {/if}
  {:else}
    {#if machines.length === 0}
      <p>No machines monitored!</p>
    {:else}
      {#each machines as machine, i (machine)}
        <Machine {...machineData[i]} />
      {/each}
    {/if}
  {/if}
</div>

<style lang="scss">
  h1 {
    text-align: center;
  }

  .machines {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-evenly;

    gap: 10px 20px;
  }

  .offline {
    display: block;
    text-align: center;
  }
</style>
