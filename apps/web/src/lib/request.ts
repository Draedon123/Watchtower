import { dev } from "$app/env";
import type { MemoryData } from "./components/Machine.svelte";

const WATCHTOWER_ADDRESS = dev
  ? "http://127.0.0.1:9824"
  : "http://watchtowerapi.draedon.xyz";

async function request(method: string, path: string): Promise<string> {
  return (await fetch(`${WATCHTOWER_ADDRESS}/${path}`, { method })).text();
}

function post(path: string): Promise<string> {
  return request("POST", path);
}

const GET_MACHINES_PATH = "get/machines";
async function getMachines(): Promise<string[]> {
  const response = await post(GET_MACHINES_PATH);

  return response === "" ? [] : response.split("\n").sort();
}

async function getCpuUsages(machines: string[]): Promise<(number | null)[]> {
  const cpuUsagePromises = machines.map((machine) =>
    post(`get/${machine}/cpu_usage`).catch(() => null)
  );

  const cpuUsagesRaw = await Promise.all(cpuUsagePromises);
  return cpuUsagesRaw.map((usage) => {
    const usageNumeric =
      usage === null ? NaN : parseFloat(usage.split("\n")[1]);

    return isNaN(usageNumeric) ? null : usageNumeric;
  });
}

async function getMemory(
  machines: string[],
  type: "physical" | "swap"
): Promise<(MemoryData | null)[]> {
  const key = type === "physical" ? "memory_physical" : "memory_swap";
  const promises = machines.map((machine) =>
    post(`get/${machine}/${key}`).catch(() => null)
  );
  const rawData = await Promise.all(promises);

  return rawData.map((data) => {
    if (data === "" || data === null) {
      return null;
    }

    const lines = data.split("\n");
    const split = lines[1].split(",");
    const available_kib = parseInt(split[0]);
    const total_kib = parseInt(split[1]);

    if (isNaN(available_kib) || isNaN(total_kib)) {
      return null;
    }

    const memoryData: MemoryData = {
      available_kib,
      total_kib,
    };

    return memoryData;
  });
}

async function getPhysicalMemory(
  machines: string[]
): Promise<(MemoryData | null)[]> {
  return getMemory(machines, "physical");
}

async function getSwapMemory(
  machines: string[]
): Promise<(MemoryData | null)[]> {
  return getMemory(machines, "swap");
}

export { post, getMachines, getCpuUsages, getPhysicalMemory, getSwapMemory };
