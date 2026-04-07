"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OciHeadQuarterVcn = void 0;
const pulumi = require("@pulumi/pulumi");
const oci = require("@pulumi/oci");
class OciHeadQuarterVcn extends pulumi.ComponentResource {
    constructor(name, args, opts) {
        super("pkg:index:oci-head-quarter", name, args, opts);
        const { suffix, region } = args;
        const provider = new oci.Provider("provider", {
            region,
        });
        const compartment = new oci.identity.Compartment("compartment", {
            description: "Compartment for fn0 OCI Head Quarter",
            name: pulumi.interpolate `fn0-hq-${suffix}`,
        }, { parent: this, provider });
        this.compartmentId = compartment.id;
        const vcn = new oci.core.Vcn("vcn", {
            compartmentId: compartment.id,
            isIpv6enabled: true,
            isOracleGuaAllocationEnabled: true,
            cidrBlocks: ["10.0.0.0/16"],
        }, { parent: this, provider });
        this.vcnId = vcn.id;
        this.ipv6cidrBlocks = vcn.ipv6cidrBlocks;
    }
}
exports.OciHeadQuarterVcn = OciHeadQuarterVcn;
// import * as tls from "@pulumi/tls";
// function createHostPem(
//   parent: pulumi.Resource,
//   provider: oci.Provider,
//   {
//     compartmentId,
//     suffix,
//   }: {
//     compartmentId: pulumi.Input<string>;
//     suffix: pulumi.Input<string>;
//   }
// ) {
//   const vault = new oci.kms.Vault(
//     "vault",
//     {
//       compartmentId,
//       displayName: pulumi.interpolate`fn0-hq-${suffix}-vault`,
//       vaultType: "DEFAULT",
//     },
//     { parent, provider }
//   );
//   const key = new oci.kms.Key(
//     "key",
//     {
//       compartmentId,
//       managementEndpoint: vault.managementEndpoint,
//       displayName: "fn0-host-key",
//       keyShape: {
//         algorithm: "AES",
//         length: 32,
//       },
//     },
//     { parent, provider }
//   );
//   const secret = new oci.vault.Secret(
//     "pem",
//     {
//       compartmentId,
//       keyId: key.id,
//       secretName: pulumi.interpolate`host-pem-${suffix}`,
//       vaultId: vault.id,
//       secretContent: {
//         contentType: "BASE64",
//       },
//     },
//     { parent, provider }
//   );
//   return {};
// }
//# sourceMappingURL=OciHeadQuarterVcn.js.map