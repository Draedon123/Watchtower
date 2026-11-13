import { dev } from "$app/env";
import type { MachineData } from "./components/MachineCard.svelte";

const WATCHTOWER_ADDRESS = dev
  ? "http://127.0.0.1:9824"
  : "http://watchtowerapi.draedon.xyz";

async function request(method: string, path: string): Promise<string> {
  const response = await fetch(`${WATCHTOWER_ADDRESS}/${path}`, {
    method,
  });

  if (!response.ok) {
    throw new Error(`${response.status}: ${response.statusText}`);
  }

  return response.text();
}

function post(path: string): Promise<string> {
  return request("POST", path);
}

async function getMachines(): Promise<string[]> {
  const response = await post("machine/list");

  return response === ""
    ? []
    : response
        .split("\n")
        .sort((a, b) => a.localeCompare(b, "en", { numeric: true }));
}

async function getAll(
  machines: string[] | null
): Promise<(MachineData | null)[]> {
  if (machines === null) {
    return [];
  }

  const promises = machines.map((machine) =>
    post(`machine/${machine}/get`).catch(() => null)
  );

  const data = await Promise.all(promises);

  return data.map((data, i) => {
    if (data === null) {
      return null;
    }

    const lines = data.split("\n");

    const entry: MachineData = { name: machines[i] };

    for (let i = 0; i < lines.length / 4; i++) {
      const key = lines[i * 4];
      const type = lines[i * 4 + 1];
      const value = lines[i * 4 + 2];
      // const timestamp = lines[i + 3];

      const parsedValue = parseValue(type, value);

      switch (key) {
        case "cpu_usage": {
          entry.cpuUsage = parsedValue;
          break;
        }

        case "memory_physical": {
          entry.physicalMemory = {
            available_kib: parsedValue[0],
            total_kib: parsedValue[1],
          };
          break;
        }

        case "memory_swap": {
          entry.swapMemory = {
            available_kib: parsedValue[0],
            total_kib: parsedValue[1],
          };
          break;
        }
      }
    }

    return entry;
  });
}

/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
function parseValue(type: string, value: string): any {
  switch (type) {
    case "f32": {
      return parseFloat(value);
    }

    case "vec2u64": {
      return value.split(",").map((value) => parseInt(value));
    }

    default: {
      return null;
    }
  }
}

export { post, getMachines, getAll };
