import * as pulumi from "@pulumi/pulumi";
import * as utilities from "./utilities";

export class ForteR2 extends pulumi.ComponentResource {
    /** @internal */
    public static readonly __pulumiType = 'fn0:index:ForteR2';

    public static isInstance(obj: any): obj is ForteR2 {
        if (obj === undefined || obj === null) {
            return false;
        }
        return obj['__pulumiType'] === ForteR2.__pulumiType;
    }

    declare public /*out*/ readonly bucketName: pulumi.Output<string>;
    declare public /*out*/ readonly endpoint: pulumi.Output<string>;
    declare public /*out*/ readonly publicBaseUrl: pulumi.Output<string>;
    declare public /*out*/ readonly accessKeyId: pulumi.Output<string>;
    declare public /*out*/ readonly secretAccessKey: pulumi.Output<string>;
    declare public /*out*/ readonly accountId: pulumi.Output<string>;

    constructor(name: string, args: ForteR2Args, opts?: pulumi.ComponentResourceOptions) {
        let resourceInputs: pulumi.Inputs = {};
        opts = opts || {};
        if (!opts.id) {
            if (args?.accountId === undefined && !opts.urn) {
                throw new Error("Missing required property 'accountId'");
            }
            if (args?.zoneId === undefined && !opts.urn) {
                throw new Error("Missing required property 'zoneId'");
            }
            if (args?.staticHostname === undefined && !opts.urn) {
                throw new Error("Missing required property 'staticHostname'");
            }
            if (args?.bucketName === undefined && !opts.urn) {
                throw new Error("Missing required property 'bucketName'");
            }
            resourceInputs["accountId"] = args?.accountId;
            resourceInputs["zoneId"] = args?.zoneId;
            resourceInputs["staticHostname"] = args?.staticHostname;
            resourceInputs["bucketName"] = args?.bucketName;
            resourceInputs["location"] = args?.location;
            resourceInputs["endpoint"] = undefined /*out*/;
            resourceInputs["publicBaseUrl"] = undefined /*out*/;
            resourceInputs["accessKeyId"] = undefined /*out*/;
            resourceInputs["secretAccessKey"] = undefined /*out*/;
        } else {
            resourceInputs["bucketName"] = undefined /*out*/;
            resourceInputs["endpoint"] = undefined /*out*/;
            resourceInputs["publicBaseUrl"] = undefined /*out*/;
            resourceInputs["accessKeyId"] = undefined /*out*/;
            resourceInputs["secretAccessKey"] = undefined /*out*/;
            resourceInputs["accountId"] = undefined /*out*/;
        }
        const secretOpts = { additionalSecretOutputs: ["secretAccessKey"] };
        opts = pulumi.mergeOptions(utilities.resourceOptsDefaults(), opts);
        opts = pulumi.mergeOptions(opts, secretOpts);
        super(ForteR2.__pulumiType, name, resourceInputs, opts, true /*remote*/);
    }
}

export interface ForteR2Args {
    accountId: pulumi.Input<string>;
    zoneId: pulumi.Input<string>;
    staticHostname: pulumi.Input<string>;
    bucketName: pulumi.Input<string>;
    location?: pulumi.Input<string>;
}
