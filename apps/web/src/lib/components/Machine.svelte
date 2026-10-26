<script lang="ts" module>
  type Nullable<T> = {
    [P in keyof T]: T[P] | null;
  };

  type MachineData = Nullable<{
    /** 0 - 1*/
    cpuUsage: number;
  }> & { name: string };

  export type { MachineData };
</script>

<script lang="ts">
  let data: MachineData = $props();
</script>

<h1>{data.name}</h1>
{#if data.cpuUsage === null}
  <p>Could not fetch CPU usage</p>
{:else}
  <p>CPU Usage: {(data.cpuUsage * 100).toFixed(2)}%</p>
{/if}
