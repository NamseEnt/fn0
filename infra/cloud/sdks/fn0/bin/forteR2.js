"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ForteR2 = void 0;
const pulumi = require("@pulumi/pulumi");
const utilities = require("./utilities");
class ForteR2 extends pulumi.ComponentResource {
    static isInstance(obj) {
        if (obj === undefined || obj === null) {
            return false;
        }
        return obj['__pulumiType'] === ForteR2.__pulumiType;
    }
    constructor(name, args, opts) {
        let resourceInputs = {};
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
        }
        else {
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
exports.ForteR2 = ForteR2;
/** @internal */
ForteR2.__pulumiType = 'fn0:index:ForteR2';
//# sourceMappingURL=forteR2.js.map