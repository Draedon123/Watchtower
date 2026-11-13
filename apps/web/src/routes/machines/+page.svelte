<script lang="ts">
  import MachineCard, {
    type MachineData,
  } from "$lib/components/MachineCard.svelte";
  import {
    currentTime_ms,
    msToString,
    relativeTimeToString,
  } from "$lib/dateTime";
  import { getAllStatisticsByMachine, getMachines } from "$lib/request";
  import { onDestroy, onMount } from "svelte";

  let machines: string[] | null | Error = $state(
    null as string[] | null | Error
  );

  let machineData: (MachineData | null)[] = $state([]);

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
        machines = newMachines;
        return;
      }

      machines = newMachines;
      machineData = await getAllStatisticsByMachine(machines);
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

{#if machines instanceof Error}
  {@const downtime_ms = currentTime - (disconnectedSince as number)}
  <div class="center">
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
  <p class="center">Loading data...</p>
{:else}
  <small class="subtitle">
    Monitoring {machines.length} machine{machines.length !== 1 ? "s" : ""}
  </small>

  <div class="machines">
    {#each machines as machine, i (machine)}
      {@const data =
        machineData[i] === null ? { name: machine } : machineData[i]}
      <MachineCard {...data} />
    {/each}
  </div>
{/if}

<style lang="scss">
  h1 {
    text-align: center;
    margin-bottom: 0;
  }

  .subtitle {
    text-align: center;
    display: block;
    margin: 0.5em 0 1em 0;
  }

  .machines {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px 20px;
    margin: 0 auto;
  }

  .center {
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
