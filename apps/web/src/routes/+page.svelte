<script lang="ts">
    import { onDestroy, onMount } from "svelte";

  let machines: string[] = $state([]);
  let cpu_usages: number[] = $state([]);
  let loop: NodeJS.Timeout | null = null;

  function setIntervalImmediately(callback: () => void, delay_ms: number): NodeJS.Timeout{
    callback();

    return setInterval(callback, delay_ms);
  }

  onMount(() => {
    loop = setIntervalImmediately(async() => {
      const request = await fetch("http://watchtowerapi.draedon.xyz/get/machines", { method: "POST"});
      const raw_text = await request.text();
      
      machines = raw_text.split("\n");

      const cpu_usage_promises = machines.map(machine => fetch(`http://watchtowerapi.draedon.xyz/get/${machine}/cpu_usage`, {method: "POST"}).then(response => response.text()));
      const cpu_usages_raw = await Promise.all(cpu_usage_promises);

      cpu_usages = cpu_usages_raw.map(usage => parseFloat(usage));
    }, 5000);
  })

  onDestroy(() => {
    if(loop !== null){
      clearInterval(loop);
      loop = null;
    }
  })
</script>

<h1>Watchtower</h1>

<title>Watchtower</title>
  <!-- eslint-disable-next-line svelte/require-each-key -->
  {#each machines as machine, i}
    <h1>{machine}</h1>

    <p>CPU Usage: {(cpu_usages[i] * 100).toFixed(2)}%</p>
  {/each}
