import * as pulumi from "@pulumi/pulumi";
export declare class ForteR2 extends pulumi.ComponentResource {
    static isInstance(obj: any): obj is ForteR2;
    readonly bucketName: pulumi.Output<string>;
    readonly endpoint: pulumi.Output<string>;
    readonly publicBaseUrl: pulumi.Output<string>;
    readonly accessKeyId: pulumi.Output<string>;
    readonly secretAccessKey: pulumi.Output<string>;
    readonly accountId: pulumi.Output<string>;
    constructor(name: string, args: ForteR2Args, opts?: pulumi.ComponentResourceOptions);
}
export interface ForteR2Args {
    accountId: pulumi.Input<string>;
    zoneId: pulumi.Input<string>;
    domain: pulumi.Input<string>;
    staticHostname: pulumi.Input<string>;
    bucketName: pulumi.Input<string>;
    location?: pulumi.Input<string>;
}
