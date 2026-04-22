#!/usr/bin/env ts-node
/**
 * Service Discovery Tool
 * Scans oci-typescript-sdk/lib/ and generates service metadata
 */

import * as fs from "fs";
import * as path from "path";

interface ServiceMetadata {
  name: string;
  path: string;
  clients: string[];
  modelCount: number;
  requestCount: number;
  responseCount: number;
  priority: number; // 1=high, 2=medium, 3=low
}

// Priority services to implement first
const PRIORITY_SERVICES = new Map<string, number>([
  ["core", 1], // Compute, VirtualNetwork, Blockstorage
  ["identity", 1], // IAM
  ["objectstorage", 1], // Object storage
  ["database", 1], // Database
  ["containerengine", 1], // Kubernetes
  ["audit", 2],
  ["monitoring", 2],
  ["logging", 2],
]);

async function discoverServices(): Promise<ServiceMetadata[]> {
  const sdkPath = path.join(__dirname, "../../oci-typescript-sdk/lib");

  if (!fs.existsSync(sdkPath)) {
    console.error(`Error: TypeScript SDK not found at ${sdkPath}`);
    console.error(
      "Please ensure oci-typescript-sdk is cloned in the parent directory."
    );
    process.exit(1);
  }

  const services: ServiceMetadata[] = [];
  const serviceDirs = fs
    .readdirSync(sdkPath, { withFileTypes: true })
    .filter((dirent) => dirent.isDirectory())
    .filter(
      (dirent) => dirent.name !== "common" && dirent.name !== "node_modules"
    )
    .map((dirent) => dirent.name);

  console.error(`Found ${serviceDirs.length} services`);

  for (const serviceName of serviceDirs) {
    const servicePath = path.join(sdkPath, serviceName);
    const indexPath = path.join(servicePath, "index.ts");

    if (!fs.existsSync(indexPath)) {
      console.error(`  Skipping ${serviceName} - no index.ts`);
      continue;
    }

    // Extract client names from index.ts
    const indexContent = fs.readFileSync(indexPath, "utf-8");
    const clientMatches = indexContent.matchAll(/export import (\w+Client) =/g);
    const clients = Array.from(clientMatches).map((match) => match[1]);

    // Count models, requests, responses
    const modelPath = path.join(servicePath, "lib/model");
    const requestPath = path.join(servicePath, "lib/request");
    const responsePath = path.join(servicePath, "lib/response");

    const modelCount = fs.existsSync(modelPath)
      ? fs
          .readdirSync(modelPath)
          .filter((f) => f.endsWith(".ts") && f !== "index.ts").length
      : 0;

    const requestCount = fs.existsSync(requestPath)
      ? fs
          .readdirSync(requestPath)
          .filter((f) => f.endsWith(".ts") && f !== "index.ts").length
      : 0;

    const responseCount = fs.existsSync(responsePath)
      ? fs
          .readdirSync(responsePath)
          .filter((f) => f.endsWith(".ts") && f !== "index.ts").length
      : 0;

    const priority = PRIORITY_SERVICES.get(serviceName) || 3;

    services.push({
      name: serviceName,
      path: `lib/${serviceName}`,
      clients,
      modelCount,
      requestCount,
      responseCount,
      priority,
    });

    console.error(
      `  ${serviceName}: ${clients.length} clients, ${modelCount} models (priority ${priority})`
    );
  }

  // Sort by priority, then by name
  services.sort((a, b) => {
    if (a.priority !== b.priority) {
      return a.priority - b.priority;
    }
    return a.name.localeCompare(b.name);
  });

  return services;
}

async function main() {
  console.error("OCI SDK Service Discovery");
  console.error("==========================\n");

  const services = await discoverServices();

  console.error(`\nDiscovered ${services.length} services`);
  console.error(
    `Priority 1 (high): ${services.filter((s) => s.priority === 1).length}`
  );
  console.error(
    `Priority 2 (medium): ${services.filter((s) => s.priority === 2).length}`
  );
  console.error(
    `Priority 3 (low): ${services.filter((s) => s.priority === 3).length}`
  );

  // Output JSON to stdout
  console.log(JSON.stringify(services, null, 2));
}

main().catch((err) => {
  console.error("Error:", err);
  process.exit(1);
});
