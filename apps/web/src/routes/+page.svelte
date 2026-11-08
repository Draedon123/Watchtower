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
  import { onDestroy, onMount, untrack } from "svelte";

  let machines: string[] | null | Error = $state(
    null as string[] | null | Error
  );

  let machineStats = $state({
    cpuUsages: [] as (number | null)[],
    physicalMemory: [] as (MemoryData | null)[],
    swapMemory: [] as (MemoryData | null)[],
  });

  let machineData: MachineData[] = $derived.by(() => {
    if (machines === null || machines instanceof Error) {
      return [];
    }

    return untrack(() =>
      (machines as string[]).map((name, i) => ({
        name,
        cpuUsage: machineStats.cpuUsages[i],
        physicalMemory: machineStats.physicalMemory[i],
        swapMemory: machineStats.swapMemory[i],
      }))
    );
  });

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
      const newMachines = await getMachines()
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

      if (newMachines instanceof Error) {
        return;
      }

      const [cpuUsages, physicalMemory, swapMemory] = await Promise.all([
        getCpuUsages(newMachines),
        getPhysicalMemory(newMachines),
        getSwapMemory(newMachines),
      ]);

      machines = newMachines;
      machineStats = {
        cpuUsages,
        physicalMemory,
        swapMemory,
      };
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

<style lang="scss">
  h1 {
    text-align: center;
  }

  .machines {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px 20px;
    margin: 0 auto;
  }

  .offline {
    display: block;
    text-align: center;
  }

  @media (max-width: 700px) {
    .machines {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  @media (min-width: 700px) and (max-width: 1050px) {
    .machines {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
