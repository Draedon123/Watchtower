<script lang="ts">
  import LineGraph from "$lib/components/graphs/LineGraph.svelte";
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

  let machines: string[] | null | Error = $state(null);
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
        .catch((error: Error) => {
          if (disconnectedSince === null) {
            disconnectedSince = currentTime_ms();
          }

          currentTime = currentTime_ms();

          return error;
        });

      if (machines instanceof Error) {
        return;
      }

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
  {#if machines instanceof Error}
    {@const downtime_ms = currentTime - (disconnectedSince as number)}
    <div class="offline">
      <h2>Server Offline!</h2>

      <p>Error: {machines.message}</p>

      <p>
        Offline for at least {relativeTimeToString(downtime_ms / 1000)} (since {msToString(
          currentTime - downtime_ms,
          "{hh24}:{min}:{ss}, {month} {date}, {yyyy}"
        )})
      </p>
    </div>
  {:else if machines === null}
    <p>Loading data...</p>
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

<div class="graph">
  <LineGraph
    x={[0, 1, 2, 3]}
    y={[0, 1, 4, 9]}
    markerColour="#b0b"
    lineColour="#8188"
    lineWidth={3}
    borderColour="#fff"
  ></LineGraph>
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

  .graph {
    width: 50vw;
    height: 50vh;
    padding: 25px;
    margin: 10px;
  }
</style>
