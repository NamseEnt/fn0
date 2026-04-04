import * as pulumi from "@pulumi/pulumi";
import * as common from "oci-common";
import * as core from "oci-core";

interface CustomWorkerImageInputs {
  compartmentId: string;
  availabilityDomain: string;
  subnetId: string;
  baseImageId: string;
  displayName: string;
}

function getAuthProvider(): common.ConfigFileAuthenticationDetailsProvider {
  return new common.ConfigFileAuthenticationDetailsProvider();
}

async function waitForInstanceState(
  computeClient: core.ComputeClient,
  instanceId: string,
  targetState: string,
  timeoutMs: number = 600_000
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const { instance } = await computeClient.getInstance({
      instanceId,
    });
    if (instance.lifecycleState === targetState) return;
    if (instance.lifecycleState === "TERMINATED") {
      throw new Error("Instance terminated unexpectedly");
    }
    await new Promise((r) => setTimeout(r, 10_000));
  }
  throw new Error(`Timeout waiting for instance ${targetState}`);
}

async function waitForImageState(
  computeClient: core.ComputeClient,
  imageId: string,
  targetState: string,
  timeoutMs: number = 1_200_000
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const { image } = await computeClient.getImage({ imageId });
    if (image.lifecycleState === targetState) return;
    await new Promise((r) => setTimeout(r, 10_000));
  }
  throw new Error(`Timeout waiting for image ${targetState}`);
}

const provider: pulumi.dynamic.ResourceProvider = {
  async create(
    inputs: CustomWorkerImageInputs
  ): Promise<pulumi.dynamic.CreateResult> {
    const auth = getAuthProvider();
    const computeClient = new core.ComputeClient({
      authenticationDetailsProvider: auth,
    });

    const cloudInit = `#!/bin/bash
dnf install -y podman
poweroff`;

    const { instance } = await computeClient.launchInstance({
      launchInstanceDetails: {
        compartmentId: inputs.compartmentId,
        availabilityDomain: inputs.availabilityDomain,
        shape: "VM.Standard.A1.Flex",
        shapeConfig: { ocpus: 1, memoryInGBs: 6 },
        sourceDetails: {
          sourceType: "image",
          imageId: inputs.baseImageId,
        } as core.models.InstanceSourceViaImageDetails,
        createVnicDetails: {
          subnetId: inputs.subnetId,
          assignPublicIp: false,
        },
        metadata: {
          user_data: Buffer.from(cloudInit).toString("base64"),
        },
        displayName: "fn0-image-builder-temp",
      },
    });

    const instanceId = instance.id;

    try {
      await waitForInstanceState(computeClient, instanceId, "STOPPED");

      const { image } = await computeClient.createImage({
        createImageDetails: {
          compartmentId: inputs.compartmentId,
          instanceId,
          displayName: inputs.displayName,
        },
      });

      await waitForImageState(
        computeClient,
        image.id,
        "AVAILABLE"
      );

      return {
        id: image.id,
        outs: { imageId: image.id, ...inputs },
      };
    } finally {
      await computeClient
        .terminateInstance({
          instanceId,
          preserveBootVolume: false,
        })
        .catch(() => {});
    }
  },

  async read(
    id: string,
    props: CustomWorkerImageInputs & { imageId: string }
  ): Promise<pulumi.dynamic.ReadResult> {
    const auth = getAuthProvider();
    const computeClient = new core.ComputeClient({
      authenticationDetailsProvider: auth,
    });

    try {
      const { image } = await computeClient.getImage({ imageId: id });
      if (image.lifecycleState === "AVAILABLE") {
        return { id, props };
      }
    } catch {}

    return { id: "", props: undefined as any };
  },

  async delete(
    id: string,
    _props: CustomWorkerImageInputs & { imageId: string }
  ): Promise<void> {
    const auth = getAuthProvider();
    const computeClient = new core.ComputeClient({
      authenticationDetailsProvider: auth,
    });

    await computeClient
      .deleteImage({ imageId: id })
      .catch(() => {});
  },
};

export class CustomWorkerImage extends pulumi.dynamic.Resource {
  public readonly imageId!: pulumi.Output<string>;

  constructor(
    name: string,
    args: {
      compartmentId: pulumi.Input<string>;
      availabilityDomain: pulumi.Input<string>;
      subnetId: pulumi.Input<string>;
      baseImageId: pulumi.Input<string>;
      displayName: pulumi.Input<string>;
    },
    opts?: pulumi.CustomResourceOptions
  ) {
    super(
      provider,
      name,
      { imageId: undefined, ...args },
      opts
    );
  }
}
