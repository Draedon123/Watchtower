import { dev } from "$app/env";

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

  return response === "" ? [] : response.split("\n");
}

async function getCpuUsages(machines: string[]): Promise<(number | null)[]> {
  const cpuUsagePromises = machines.map((machine) =>
    post(`get/${machine}/cpu_usage`).catch(() => null)
  );

  const cpuUsagesRaw = await Promise.all(cpuUsagePromises);
  return cpuUsagesRaw.map((usage) => {
    const usageNumeric =
      usage === null ? NaN : parseFloat(usage.split("\n")[0]);

    return isNaN(usageNumeric) ? null : usageNumeric;
  });
}

export { post, getMachines, getCpuUsages };
