<script lang="ts">
  import { page } from "$app/state";
  import MachineCard from "$lib/components/MachineCard.svelte";
  import type { MachineData } from "$lib/components/MachineCard.svelte";
  import { getAllStatisticsByMachine } from "$lib/request.svelte";
  import { onDestroy, onMount, untrack } from "svelte";

  let machineName = $derived(page.params.slug as string);
  let data: MachineData | null = $state(null);
  let loop: NodeJS.Timeout | null = null;
  let hasLoadedData = $state(false);

  // hack to force a full and instant refresh of `data` and the `MachineCard`
  $effect(() => {
    /* eslint-disable-next-line @typescript-eslint/no-unused-expressions */
    machineName;

    untrack(() => {
      data = null;
      hasLoadedData = false;
    });
  });

  function setIntervalImmediately(
    callback: () => void,
    delay_ms?: number
  ): NodeJS.Timeout {
    callback();

    return setInterval(callback, delay_ms);
  }

  onMount(async () => {
    loop = setIntervalImmediately(async () => {
      data = (await getAllStatisticsByMachine([machineName]))[0];
      hasLoadedData = true;
    }, 1000);
  });

  onDestroy(() => {
    if (loop !== null) {
      clearInterval(loop);
      loop = null;
    }
  });
</script>

<h1>{machineName}</h1>
<p style="text-align: center">
  This is just placeholder data! In reality, there should be more detailed data
  here
</p>

<div class="card">
  {#if data === null}
    {#if !hasLoadedData}
      <p>Loading data...</p>
    {:else}
      <p>Failed to load data!</p>
    {/if}
  {:else}
    <MachineCard {...data} />
  {/if}
</div>

<style lang="scss">
  h1 {
    text-align: center;
    margin-bottom: 0;
  }

  .card {
    width: 100%;
    height: 70%;

    display: flex;
    justify-content: center;
    align-items: center;
  }
</style>
