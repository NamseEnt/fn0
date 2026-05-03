import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as tls from "@pulumi/tls";
import * as random from "@pulumi/random";

export interface OciGlobalVaultArgs {
  region: pulumi.Input<string>;
  allowedSubdomain: pulumi.Input<string>;
}

export interface OciGlobalVaultWorkerCredentials {
  ociUserId: pulumi.Output<string>;
  ociTenancyId: pulumi.Output<string>;
  ociFingerprint: pulumi.Output<string>;
  ociPrivateKeyBase64: pulumi.Output<string>;
}

export class OciGlobalVault extends pulumi.ComponentResource {
  public readonly compartmentId: pulumi.Output<string>;
  public readonly vaultId: pulumi.Output<string>;
  public readonly cryptoEndpoint: pulumi.Output<string>;
  public readonly managementEndpoint: pulumi.Output<string>;
  public readonly keyOcid: pulumi.Output<string>;
  public readonly region: pulumi.Output<string>;
  public readonly allowedSubdomain: pulumi.Output<string>;
  public readonly workerCredentials: OciGlobalVaultWorkerCredentials;

  constructor(
    name: string,
    args: OciGlobalVaultArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:oci-global-vault", name, args, opts);

    const compartmentSuffix = new random.RandomString(
      "compartment-suffix",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    ).result;

    const compartment = new oci.identity.Compartment(
      "compartment",
      {
        description: "Compartment for fn0 global vault (envelope encryption)",
        name: pulumi.interpolate`fn0-vault-${compartmentSuffix}`,
        enableDelete: true,
      },
      { parent: this }
    );

    const vault = new oci.kms.Vault(
      "vault",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-vault-${compartmentSuffix}`,
        vaultType: "DEFAULT",
      },
      { parent: this }
    );

    const kek = new oci.kms.Key(
      "kek",
      {
        compartmentId: compartment.id,
        displayName: pulumi.interpolate`fn0-vault-kek-${compartmentSuffix}`,
        managementEndpoint: vault.managementEndpoint,
        keyShape: {
          algorithm: "AES",
          length: 32,
        },
        protectionMode: "SOFTWARE",
      },
      { parent: this }
    );

    const workerUser = new oci.identity.User(
      "worker-user",
      {
        name: pulumi.interpolate`fn0-vault-worker-${compartmentSuffix}`,
        description: "User for fn0 worker vault crypto access (envelope encryption)",
      },
      { parent: this }
    );

    const workerGroup = new oci.identity.Group(
      "worker-group",
      {
        name: pulumi.interpolate`fn0-vault-workers-${compartmentSuffix}`,
        description: "Group for fn0 worker vault crypto access",
      },
      { parent: this }
    );

    new oci.identity.UserGroupMembership(
      "worker-membership",
      {
        userId: workerUser.id,
        groupId: workerGroup.id,
      },
      { parent: this }
    );

    new oci.identity.Policy(
      "worker-policy",
      {
        compartmentId: compartment.id,
        name: pulumi.interpolate`allow-vault-worker-${compartmentSuffix}`,
        description: "Allow fn0 workers to use the global KEK for envelope encryption",
        statements: [
          pulumi.interpolate`Allow group ${workerGroup.name} to use keys in compartment id ${compartment.id} where target.key.id = '${kek.id}'`,
        ],
      },
      { dependsOn: [workerGroup, kek], parent: this }
    );

    const workerApiPrivateKey = new tls.PrivateKey(
      "worker-api-key-pair",
      {
        algorithm: "RSA",
        rsaBits: 2048,
      },
      { parent: this }
    );

    const workerApiKey = new oci.identity.ApiKey(
      "worker-api-key",
      {
        userId: workerUser.id,
        keyValue: workerApiPrivateKey.publicKeyPem,
      },
      { parent: this }
    );

    this.compartmentId = compartment.id;
    this.vaultId = vault.id;
    this.cryptoEndpoint = vault.cryptoEndpoint;
    this.managementEndpoint = vault.managementEndpoint;
    this.keyOcid = kek.id;
    this.region = pulumi.output(args.region);
    this.allowedSubdomain = pulumi.output(args.allowedSubdomain);
    this.workerCredentials = {
      ociUserId: workerUser.id,
      ociTenancyId: workerUser.compartmentId,
      ociFingerprint: workerApiKey.fingerprint,
      ociPrivateKeyBase64: workerApiPrivateKey.privateKeyPemPkcs8.apply((pem) =>
        Buffer.from(pem).toString("base64")
      ),
    };
  }
}
