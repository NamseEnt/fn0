import * as pulumi from "@pulumi/pulumi";
export declare class ForteR2 extends pulumi.ComponentResource {
    /**
     * Returns true if the given object is an instance of ForteR2.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ForteR2;
    readonly accessKeyId: pulumi.Output<string>;
    readonly accountId: pulumi.Output<string>;
    readonly bucketName: pulumi.Output<string>;
    readonly endpoint: pulumi.Output<string>;
    readonly publicBaseUrl: pulumi.Output<string>;
    readonly secretAccessKey: pulumi.Output<string>;
    /**
     * Create a ForteR2 resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ForteR2Args, opts?: pulumi.ComponentResourceOptions);
}
/**
 * The set of arguments for constructing a ForteR2 resource.
 */
export interface ForteR2Args {
    accountId: pulumi.Input<string>;
    bucketName: pulumi.Input<string>;
    domain: pulumi.Input<string>;
    location?: pulumi.Input<string>;
    staticHostname: pulumi.Input<string>;
    zoneId: pulumi.Input<string>;
}
