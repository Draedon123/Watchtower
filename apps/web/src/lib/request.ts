import { dev } from "$app/env";
import type { MachineData } from "./components/MachineCard.svelte";
import { F32Value, stringToValue, Vec2U64Value } from "./DataTypes";

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

async function getAllStatisticsByMachine(
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

      const parsedValue = stringToValue(type, value);

      switch (key) {
        case "cpu_usage": {
          if (!(parsedValue instanceof F32Value)) {
            throw new Error(
              `Expected f32 value. Instead received ${parsedValue.stringName}`
            );
          }

          entry.cpuUsage = parsedValue.value;
          break;
        }

        case "memory_physical":
        case "memory_swap": {
          if (!(parsedValue instanceof Vec2U64Value)) {
            throw new Error(
              `Expected vec2u64 value. Instead received ${parsedValue.stringName}`
            );
          }

          entry[key === "memory_swap" ? "swapMemory" : "physicalMemory"] = {
            available_kib: parsedValue.value[0],
            total_kib: parsedValue.value[1],
          };
          break;
        }
      }
    }

    return entry;
  });
}

export { post, getMachines, getAllStatisticsByMachine };
