<script lang="ts">
  import ConnectionFailure from "$lib/components/ConnectionFailure.svelte";
  import MachineCard, {
    type MachineData,
  } from "$lib/components/MachineCard.svelte";
  import {
    connectionStatus,
    getAllStatisticsByMachine,
    updateMachines,
  } from "$lib/request.svelte";
  import { onDestroy, onMount } from "svelte";

  let machineData: (MachineData | null)[] = $state([]);

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

      if (Array.isArray(connectionStatus.connectedMachines)) {
        machineData = await getAllStatisticsByMachine(
          connectionStatus.connectedMachines
        );
      }
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
  <small class="subtitle">
    Monitoring {connectionStatus.connectedMachines.length} machine{connectionStatus
      .connectedMachines.length !== 1
      ? "s"
      : ""}
  </small>

  <div class="machines">
    {#each connectionStatus.connectedMachines as machine, i (machine)}
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
