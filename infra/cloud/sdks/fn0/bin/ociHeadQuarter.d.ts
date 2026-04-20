import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
export declare class OciHeadQuarter extends pulumi.ComponentResource {
    /**
     * Returns true if the given object is an instance of OciHeadQuarter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OciHeadQuarter;
    readonly kubeconfig: pulumi.Output<string>;
    /**
     * Create a OciHeadQuarter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OciHeadQuarterArgs, opts?: pulumi.ComponentResourceOptions);
}
/**
 * The set of arguments for constructing a OciHeadQuarter resource.
 */
export interface OciHeadQuarterArgs {
    awsAccessKeyId: pulumi.Input<string>;
    awsRegion: pulumi.Input<string>;
    awsSecretAccessKey: pulumi.Input<string>;
    compartmentId: pulumi.Input<string>;
    cwasmBucket: pulumi.Input<inputs.CwasmBucketArgsArgs>;
    dnsProvider: pulumi.Input<inputs.DnsProviderArgArgs>;
    docDbToken: pulumi.Input<string>;
    docDbUrl: pulumi.Input<string>;
    envEncryptionKeyBase64: pulumi.Input<string>;
    forteDb: pulumi.Input<inputs.ForteDbArgsArgs>;
    forteR2: pulumi.Input<inputs.ForteR2ArgsArgs>;
    grafanaRegion: pulumi.Input<string>;
    grafanaSlug: pulumi.Input<string>;
    ipv6cidrBlocks: pulumi.Input<string[]>;
    ociRegion: pulumi.Input<string>;
    sccacheAccessKeyId: pulumi.Input<string>;
    sccacheBucket: pulumi.Input<string>;
    sccacheEndpoint: pulumi.Input<string>;
    sccacheRegion: pulumi.Input<string>;
    sccacheSecretAccessKey: pulumi.Input<string>;
    selfDnsCloudflareApiToken: pulumi.Input<string>;
    selfDnsCloudflareZoneId: pulumi.Input<string>;
    selfDnsHostname: pulumi.Input<string>;
    sites: pulumi.Input<inputs.SiteArgsArgs[]>;
    suffix: pulumi.Input<string>;
    vcnId: pulumi.Input<string>;
    wasmBucket: pulumi.Input<string>;
}
