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

  return response.split("\n");
}

async function getCpuUsages(machines: string[]): Promise<number[]> {
  const cpuUsagePromises = machines.map((machine) =>
    post(`get/${machine}/cpu_usage`)
  );

  const cpuUsagesRaw = await Promise.all(cpuUsagePromises);
  return cpuUsagesRaw.map((usage) => parseFloat(usage.split("\n")[0]));
}

export { post, getMachines, getCpuUsages };
