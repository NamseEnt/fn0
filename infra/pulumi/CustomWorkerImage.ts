import * as pulumi from "@pulumi/pulumi";
import * as common from "oci-common";
import * as core from "oci-core";
import { Client as SshClient, utils as sshUtils } from "ssh2";

interface CustomWorkerImageInputs {
  compartmentId: string;
  vcnId: string;
  availabilityDomain: string;
  baseImageId: string;
  displayName: string;
  internetGatewayId: string;
}

function getAuthProvider(): common.ConfigFileAuthenticationDetailsProvider {
  return new common.ConfigFileAuthenticationDetailsProvider();
}

function generateSshKeyPair() {
  const keys = sshUtils.generateKeyPairSync("rsa", { bits: 2048 });
  return { openSshPub: keys.public, privateKey: keys.private };
}

async function waitForState<T>(
  poll: () => Promise<T>,
  getState: (result: T) => string,
  targetState: string,
  failStates: string[] = [],
  timeoutMs: number = 300_000
): Promise<T> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const result = await poll();
    const state = getState(result);
    if (state === targetState) return result;
    if (failStates.includes(state))
      throw new Error(`Reached fail state: ${state}`);
    await new Promise((r) => setTimeout(r, 10_000));
  }
  throw new Error(`Timeout waiting for ${targetState}`);
}

async function sshExec(
  host: string,
  privateKey: string,
  command: string,
  timeoutMs: number = 300_000
): Promise<string> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error("SSH command timeout")),
      timeoutMs
    );
    const conn = new SshClient();
    conn
      .on("ready", () => {
        conn.exec(command, (err, stream) => {
          if (err) {
            clearTimeout(timer);
            conn.end();
            return reject(err);
          }
          let stdout = "";
          let stderr = "";
          stream.on("data", (data: Buffer) => (stdout += data.toString()));
          stream.stderr.on("data", (data: Buffer) => (stderr += data.toString()));
          stream.on("close", (code: number) => {
            clearTimeout(timer);
            conn.end();
            if (code !== 0) reject(new Error(`Exit ${code}: ${stderr}`));
            else resolve(stdout);
          });
        });
      })
      .on("error", (err) => {
        clearTimeout(timer);
        reject(err);
      })
      .connect({ host, port: 22, username: "opc", privateKey });
  });
}

async function waitForSsh(
  host: string,
  privateKey: string,
  timeoutMs: number = 300_000
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      await sshExec(host, privateKey, "echo ok", 10_000);
      return;
    } catch {}
    await new Promise((r) => setTimeout(r, 10_000));
  }
  throw new Error("Timeout waiting for SSH");
}

const provider: pulumi.dynamic.ResourceProvider = {
  async create(
    inputs: CustomWorkerImageInputs
  ): Promise<pulumi.dynamic.CreateResult> {
    const auth = getAuthProvider();
    const computeClient = new core.ComputeClient({
      authenticationDetailsProvider: auth,
    });
    const vnClient = new core.VirtualNetworkClient({
      authenticationDetailsProvider: auth,
    });

    const { openSshPub, privateKey: sshPrivateKey } = generateSshKeyPair();

    const routeTable = await vnClient.createRouteTable({
      createRouteTableDetails: {
        compartmentId: inputs.compartmentId,
        vcnId: inputs.vcnId,
        displayName: "builder-rt",
        routeRules: [
          {
            destination: "0.0.0.0/0",
            destinationType: core.models.RouteRule.DestinationType.CidrBlock,
            networkEntityId: inputs.internetGatewayId,
          },
        ],
      },
    });
    const routeTableId = routeTable.routeTable.id;

    const securityList = await vnClient.createSecurityList({
      createSecurityListDetails: {
        compartmentId: inputs.compartmentId,
        vcnId: inputs.vcnId,
        displayName: "builder-sl",
        ingressSecurityRules: [
          {
            protocol: "6",
            source: "0.0.0.0/0",
            tcpOptions: {
              destinationPortRange: { min: 22, max: 22 },
            },
          },
        ],
        egressSecurityRules: [
          { protocol: "all", destination: "0.0.0.0/0" },
        ],
      },
    });
    const securityListId = securityList.securityList.id;

    const subnet = await vnClient.createSubnet({
      createSubnetDetails: {
        compartmentId: inputs.compartmentId,
        vcnId: inputs.vcnId,
        cidrBlock: "10.0.99.0/24",
        displayName: "builder-subnet",
        securityListIds: [securityListId],
        routeTableId,
        prohibitPublicIpOnVnic: false,
      },
    });
    const subnetId = subnet.subnet.id;

    await waitForState(
      () => vnClient.getSubnet({ subnetId }),
      (r) => r.subnet.lifecycleState!,
      "AVAILABLE"
    );

    const cleanup = async () => {
      await vnClient
        .deleteSubnet({ subnetId })
        .catch(() => {});
      await new Promise((r) => setTimeout(r, 30_000));
      await vnClient
        .deleteRouteTable({ rtId: routeTableId })
        .catch(() => {});
      await vnClient
        .deleteSecurityList({ securityListId })
        .catch(() => {});
    };

    let instanceId: string | undefined;

    try {
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
            subnetId,
            assignPublicIp: true,
          },
          metadata: { ssh_authorized_keys: openSshPub },
          displayName: "fn0-image-builder-temp",
        },
      });
      instanceId = instance.id;

      await waitForState(
        () => computeClient.getInstance({ instanceId: instanceId! }),
        (r) => r.instance.lifecycleState!,
        "RUNNING",
        ["TERMINATED"]
      );

      const { items } = await computeClient.listVnicAttachments({
        compartmentId: inputs.compartmentId,
        instanceId,
      });
      let publicIp = "";
      for (const att of items) {
        if (!att.vnicId) continue;
        const { vnic } = await vnClient.getVnic({ vnicId: att.vnicId });
        if (vnic.publicIp) {
          publicIp = vnic.publicIp;
          break;
        }
      }
      if (!publicIp) throw new Error("No public IP found");

      await waitForSsh(publicIp, sshPrivateKey);
      await sshExec(publicIp, sshPrivateKey, "sudo dnf install -y podman && sudo systemctl disable --now firewalld", 300_000);

      await computeClient.instanceAction({ instanceId, action: "STOP" });
      await waitForState(
        () => computeClient.getInstance({ instanceId: instanceId! }),
        (r) => r.instance.lifecycleState!,
        "STOPPED",
        ["TERMINATED"]
      );

      const { image } = await computeClient.createImage({
        createImageDetails: {
          compartmentId: inputs.compartmentId,
          instanceId,
          displayName: inputs.displayName,
        },
      });

      await waitForState(
        () => computeClient.getImage({ imageId: image.id }),
        (r) => r.image.lifecycleState!,
        "AVAILABLE",
        [],
        1_200_000
      );

      return {
        id: image.id,
        outs: { imageId: image.id, ...inputs },
      };
    } finally {
      if (instanceId) {
        await computeClient
          .terminateInstance({ instanceId, preserveBootVolume: false })
          .catch(() => {});
        await waitForState(
          () => computeClient.getInstance({ instanceId: instanceId! }),
          (r) => r.instance.lifecycleState!,
          "TERMINATED",
          [],
          300_000
        ).catch(() => {});
      }
      await cleanup();
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
      if (image.lifecycleState === "AVAILABLE") return { id, props: { ...props, imageId: id } };
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
    await computeClient.deleteImage({ imageId: id }).catch(() => {});
  },
};

export class CustomWorkerImage extends pulumi.dynamic.Resource {
  public readonly imageId!: pulumi.Output<string>;

  constructor(
    name: string,
    args: {
      compartmentId: pulumi.Input<string>;
      vcnId: pulumi.Input<string>;
      availabilityDomain: pulumi.Input<string>;
      baseImageId: pulumi.Input<string>;
      displayName: pulumi.Input<string>;
      internetGatewayId: pulumi.Input<string>;
    },
    opts?: pulumi.CustomResourceOptions
  ) {
    super(provider, name, { imageId: undefined, ...args }, opts);
  }
}
