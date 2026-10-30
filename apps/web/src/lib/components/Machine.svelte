<script lang="ts" module>
  type Nullable<T> = {
    [P in keyof T]: T[P] | null;
  };

  type MemoryData = {
    total_kib: number;
    available_kib: number;
  };

  type MachineData = Partial<
    Nullable<{
      /** 0 - 1*/
      cpuUsage: number;
      physicalMemory: MemoryData;
      swapMemory: MemoryData;
    }>
  > & { name: string };

  export type { MachineData, MemoryData };
</script>

<script lang="ts">
  let data: MachineData = $props();
</script>

<h1>{data.name}</h1>

{#if !data.cpuUsage}
  <p>Could not fetch CPU usage</p>
{:else}
  <p>CPU Usage: {(data.cpuUsage * 100).toFixed(2)}%</p>
{/if}

{#if !data.physicalMemory}
  <p>Could not fetch physical memory</p>
{:else}
  {@const usedMemory_gib =
    (data.physicalMemory.total_kib - data.physicalMemory.available_kib) /
    1024 /
    1024}
  {@const totalMemory_gib = data.physicalMemory.total_kib / 1024 / 1024}
  {@const memoryUsagePercent = (usedMemory_gib / totalMemory_gib) * 100}
  <p>
    Physical Memory: {usedMemory_gib.toFixed(2)}/{totalMemory_gib.toFixed(2)} GiB
    ({memoryUsagePercent.toFixed(2)}%)
  </p>
{/if}

{#if !data.swapMemory}
  <p>Could not fetch swap memory</p>
{:else}
  {@const usedMemory_gib =
    (data.swapMemory.total_kib - data.swapMemory.available_kib) / 1024 / 1024}
  {@const totalMemory_gib = data.swapMemory.total_kib / 1024 / 1024}
  {@const memoryUsagePercent = (usedMemory_gib / totalMemory_gib) * 100}
  <p>
    Swap Memory: {usedMemory_gib.toFixed(2)}/{totalMemory_gib.toFixed(2)} GiB ({memoryUsagePercent.toFixed(
      2
    )}%)
  </p>
{/if}
