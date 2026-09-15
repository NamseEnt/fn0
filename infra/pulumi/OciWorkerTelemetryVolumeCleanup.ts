import * as pulumi from "@pulumi/pulumi";
import * as common from "oci-common";
import * as core from "oci-core";

interface OciWorkerTelemetryVolumeCleanupInputs {
  region: string;
  compartmentId: string;
  instancePoolId: string;
  instanceConfigurationId: string;
  expectedAttachedVolumeCount: number;
  volumeDisplayNames: string[];
  managedByTagKey: string;
  managedByTagValue: string;
  roleTagKey: string;
  roleTagValue: string;
}

type OciWorkerTelemetryVolumeCleanupOutputs =
  OciWorkerTelemetryVolumeCleanupInputs & {
    deletedVolumeIds: string[];
  };

function createClients(region: string): {
  blockstorageClient: core.BlockstorageClient;
  computeClient: core.ComputeClient;
} {
  const authenticationProvider =
    new common.ConfigFileAuthenticationDetailsProvider();
  authenticationProvider.setRegion(region);
  return {
    blockstorageClient: new core.BlockstorageClient({
      authenticationDetailsProvider: authenticationProvider,
    }),
    computeClient: new core.ComputeClient({
      authenticationDetailsProvider: authenticationProvider,
    }),
  };
}

async function listTargetVolumes(
  blockstorageClient: core.BlockstorageClient,
  inputs: OciWorkerTelemetryVolumeCleanupInputs,
): Promise<core.models.Volume[]> {
  const volumes: core.models.Volume[] = [];
  for (const displayName of inputs.volumeDisplayNames) {
    for await (const volume of blockstorageClient.listAllVolumes({
      compartmentId: inputs.compartmentId,
      displayName,
      lifecycleState: "AVAILABLE",
    })) {
      const freeformTags = volume.freeformTags ?? {};
      if (
        freeformTags[inputs.managedByTagKey] !== inputs.managedByTagValue ||
        freeformTags[inputs.roleTagKey] !== inputs.roleTagValue
      ) {
        continue;
      }
      volumes.push(volume);
    }
  }
  return volumes;
}

async function listAttachedVolumeIds(
  computeClient: core.ComputeClient,
  compartmentId: string,
): Promise<Set<string>> {
  const attachedVolumeIds = new Set<string>();
  for await (const attachment of computeClient.listAllVolumeAttachments({
    compartmentId,
  })) {
    if (attachment.lifecycleState !== "DETACHED") {
      attachedVolumeIds.add(attachment.volumeId);
    }
  }
  return attachedVolumeIds;
}

async function waitForExpectedVolumeAttachments(
  blockstorageClient: core.BlockstorageClient,
  computeClient: core.ComputeClient,
  inputs: OciWorkerTelemetryVolumeCleanupInputs,
): Promise<Set<string>> {
  const deadline = Date.now() + 300_000;
  while (Date.now() < deadline) {
    const [volumes, attachedVolumeIds] = await Promise.all([
      listTargetVolumes(blockstorageClient, inputs),
      listAttachedVolumeIds(computeClient, inputs.compartmentId),
    ]);
    const currentConfigurationVolumeCount = volumes.filter(
      (volume) =>
        volume.freeformTags?.["oci:compute:instanceconfiguration"] ===
        inputs.instanceConfigurationId,
    ).length;
    const attachedTargetVolumeCount = volumes.filter((volume) =>
      attachedVolumeIds.has(volume.id),
    ).length;
    if (
      currentConfigurationVolumeCount >= inputs.expectedAttachedVolumeCount ||
      attachedTargetVolumeCount >= inputs.expectedAttachedVolumeCount
    ) {
      return attachedVolumeIds;
    }
    await new Promise((resolve) => setTimeout(resolve, 5_000));
  }
  throw new Error(
    `Timed out waiting for telemetry volume attachment in instance pool ${inputs.instancePoolId}`,
  );
}

async function deleteOrphanedVolumes(
  inputs: OciWorkerTelemetryVolumeCleanupInputs,
  waitForCurrentConfiguration: boolean,
): Promise<string[]> {
  const { blockstorageClient, computeClient } = createClients(inputs.region);
  const attachedVolumeIds = waitForCurrentConfiguration
    ? await waitForExpectedVolumeAttachments(
        blockstorageClient,
        computeClient,
        inputs,
      )
    : await listAttachedVolumeIds(computeClient, inputs.compartmentId);
  const targetVolumes = await listTargetVolumes(blockstorageClient, inputs);
  const currentConfigurationVolumeIds = new Set(
    waitForCurrentConfiguration
      ? targetVolumes
          .filter(
            (volume) =>
              volume.freeformTags?.["oci:compute:instanceconfiguration"] ===
              inputs.instanceConfigurationId,
          )
          .map((volume) => volume.id)
      : [],
  );
  const deletedVolumeIds: string[] = [];

  for (const volume of targetVolumes) {
    if (
      attachedVolumeIds.has(volume.id) ||
      currentConfigurationVolumeIds.has(volume.id)
    ) {
      continue;
    }
    try {
      await blockstorageClient.deleteVolume({ volumeId: volume.id });
      deletedVolumeIds.push(volume.id);
    } catch (error) {
      const refreshedAttachedVolumeIds = await listAttachedVolumeIds(
        computeClient,
        inputs.compartmentId,
      );
      if (
        refreshedAttachedVolumeIds.has(volume.id) ||
        currentConfigurationVolumeIds.has(volume.id)
      ) {
        continue;
      }
      throw error;
    }
  }
  return deletedVolumeIds;
}

const provider: pulumi.dynamic.ResourceProvider<
  OciWorkerTelemetryVolumeCleanupInputs,
  OciWorkerTelemetryVolumeCleanupOutputs
> = {
  async create(inputs) {
    const deletedVolumeIds = await deleteOrphanedVolumes(inputs, true);
    return {
      id: "oci-worker-telemetry-volume-cleanup",
      outs: { ...inputs, deletedVolumeIds },
    };
  },

  async read(id, outputs) {
    return { id, props: outputs };
  },

  async update(id, oldInputs, inputs) {
    void id;
    void oldInputs;
    const deletedVolumeIds = await deleteOrphanedVolumes(inputs, true);
    return { outs: { ...inputs, deletedVolumeIds } };
  },

  async delete(id, outputs) {
    void id;
    await deleteOrphanedVolumes(outputs, false);
  },
};

export class OciWorkerTelemetryVolumeCleanup extends pulumi.dynamic.Resource {
  public readonly deletedVolumeIds!: pulumi.Output<string[]>;

  constructor(
    name: string,
    args: {
      region: pulumi.Input<string>;
      compartmentId: pulumi.Input<string>;
      instancePoolId: pulumi.Input<string>;
      instanceConfigurationId: pulumi.Input<string>;
      expectedAttachedVolumeCount: pulumi.Input<number>;
      volumeDisplayNames: pulumi.Input<pulumi.Input<string>[]>;
      managedByTagKey: pulumi.Input<string>;
      managedByTagValue: pulumi.Input<string>;
      roleTagKey: pulumi.Input<string>;
      roleTagValue: pulumi.Input<string>;
    },
    opts?: pulumi.CustomResourceOptions,
  ) {
    super(provider, name, args, opts);
  }
}
