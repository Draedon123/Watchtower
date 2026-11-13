<script lang="ts">
  import {
    currentTime_ms,
    msToString,
    relativeTimeToString,
  } from "$lib/dateTime";
  import { connectionStatus } from "$lib/request.svelte";
  import { onDestroy, onMount } from "svelte";

  let currentTime = $state(currentTime_ms());
  let downtime_ms = $derived(
    connectionStatus.disconnectedSince === null
      ? null
      : currentTime - connectionStatus.disconnectedSince
  );

  let loop: NodeJS.Timeout | null;

  onMount(() => {
    loop = setInterval(() => {
      currentTime = currentTime_ms();
    }, 50);
  });

  onDestroy(() => {
    if (loop !== null) {
      clearInterval(loop);
      loop = null;
    }
  });
</script>

{#if downtime_ms !== null}
  <div class="center">
    <h2>Server Offline!</h2>

    <p>
      Error: {connectionStatus.error !== null
        ? connectionStatus.error.message
        : "Unknown error"}
    </p>

    <p>
      Offline for at least {relativeTimeToString(downtime_ms / 1000)} (since {msToString(
        currentTime - downtime_ms,
        "{hh24}:{min}:{ss}, {month} {date}, {yyyy}"
      )})
    </p>
  </div>
{/if}

<style lang="scss">
  .center {
    text-align: center;
  }
</style>
