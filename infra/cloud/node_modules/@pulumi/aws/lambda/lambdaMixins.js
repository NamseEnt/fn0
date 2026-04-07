"use strict";
// Copyright 2016-2018, Pulumi Corporation.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
Object.defineProperty(exports, "__esModule", { value: true });
exports.CallbackFunction = exports.createFunctionFromEventHandler = exports.isEventHandler = exports.EventSubscription = void 0;
const pulumi = require("@pulumi/pulumi");
const iam = require("../iam");
const utils = require("../utils");
const function_1 = require("./function");
const _1 = require(".");
;
/**
 * Base type for all subscription types.  An event subscription represents a connection between some
 * AWS resource and an AWS lambda that will be triggered when something happens to that resource.
 */
class EventSubscription extends pulumi.ComponentResource {
    constructor(type, name, opts) {
        super(type, name, {}, opts);
    }
}
exports.EventSubscription = EventSubscription;
function isEventHandler(obj) {
    return function_1.Function.isInstance(obj) || obj instanceof Function;
}
exports.isEventHandler = isEventHandler;
function createFunctionFromEventHandler(name, handler, opts) {
    if (handler instanceof Function) {
        return new CallbackFunction(name, { callback: handler }, opts);
    }
    else {
        return handler;
    }
}
exports.createFunctionFromEventHandler = createFunctionFromEventHandler;
/**
 * A CallbackFunction is a special type of aws.lambda.Function that can be created out of an actual
 * JavaScript function instance.  The function instance will be analyzed and packaged up (including
 * dependencies) into a form that can be used by AWS Lambda.  See
 * https://www.pulumi.com/docs/concepts/inputs-outputs/function-serialization/ for additional
 * details on this process.
 * If no IAM Role is specified, CallbackFunction will automatically use the following managed policies:
 * `AWSLambda_FullAccess`
 * `CloudWatchFullAccessV2`
 * `CloudWatchEventsFullAccess`
 * `AmazonS3FullAccess`
 * `AmazonDynamoDBFullAccess`
 * `AmazonSQSFullAccess`
 * `AmazonKinesisFullAccess`
 * `AWSCloudFormationReadOnlyAccess`
 * `AmazonCognitoPowerUser`
 * `AWSXrayWriteOnlyAccess`
 */
class CallbackFunction extends function_1.Function {
    constructor(name, args, opts = {}) {
        if (!name) {
            throw new Error("Missing required resource name");
        }
        if (args.callback && args.callbackFactory) {
            throw new pulumi.RunError("Cannot provide both [callback] and [callbackFactory]");
        }
        const func = args.callback || args.callbackFactory;
        if (!func) {
            throw new Error("One of [callback] or [callbackFactory] must be provided.");
        }
        let role;
        if (args.role) {
            role = args.role;
        }
        else {
            // Attach a role and then, if there are policies, attach those too.
            role = new iam.Role(name, {
                assumeRolePolicy: JSON.stringify(lambdaRolePolicy),
            }, opts);
            if (!args.policies) {
                const policies = [iam.ManagedPolicy.LambdaFullAccess, iam.ManagedPolicy.CloudWatchFullAccessV2,
                    iam.ManagedPolicy.CloudWatchEventsFullAccess, iam.ManagedPolicy.AmazonS3FullAccess,
                    iam.ManagedPolicy.AmazonDynamoDBFullAccess, iam.ManagedPolicy.AmazonSQSFullAccess,
                    iam.ManagedPolicy.AmazonKinesisFullAccess, iam.ManagedPolicy.AmazonCognitoPowerUser,
                    iam.ManagedPolicy.AWSXrayWriteOnlyAccess,
                ];
                for (const policy of policies) {
                    const attachment = new iam.RolePolicyAttachment(`${name}-${utils.sha1hash(policy)}`, {
                        role: role.name,
                        policyArn: policy,
                    }, opts);
                }
            }
            if (args.policies) {
                const policies = Array.isArray(args.policies)
                    ? args.policies.map(arn => [utils.sha1hash(arn), arn])
                    : Object.entries(args.policies);
                for (const [key, policyArn] of policies) {
                    // RolePolicyAttachment objects don't have a physical identity, and create/deletes are processed
                    // structurally based on the `role` and `policyArn`.  So we need to make sure our Pulumi name matches the
                    // structural identity by using a name that includes the role name and policyArn.
                    const attachment = new iam.RolePolicyAttachment(`${name}-${key}`, {
                        role: role.name,
                        policyArn,
                    }, opts);
                }
            }
        }
        // Now compile the function text into an asset we can use to create the lambda. Note: to
        // prevent a circularity/deadlock, we list this Function object as something that the
        // serialized closure cannot reference.
        const handlerName = "handler";
        const serializedFileNameNoExtension = "__index";
        const closure = pulumi.runtime.serializeFunction(func, {
            serialize: _ => true,
            exportName: handlerName,
            isFactoryFunction: !!args.callbackFactory,
            allowSecrets: true,
        });
        const codePaths = computeCodePaths(closure, serializedFileNameNoExtension, args.codePathOptions);
        const code = pulumi.output(new pulumi.asset.AssetArchive(codePaths));
        code.isSecret = closure.then(c => {
            if (c.containsSecrets) {
                pulumi.log.warn(`A secret value was captured and serialized into the body of the Lambda Function '${name}'. ` +
                    `This value will be stored as an encrypted Pulumi secret, but may be available in plain text ` +
                    `inside the AWS deployment package. You can use 'pulumi.unsecret' to convert the value to a non-secret ` +
                    `value if this is not a sensitive value, or else use Secrets Manager or environment variables to pass the ` +
                    `sensitive data to your function.`, this);
                return true;
            }
            return false;
        });
        // Copy over all option values into the function args.  Then overwrite anything we care
        // about with our own values.  This ensures that clients can pass future supported
        // lambda options without us having to know about it.
        const functionArgs = {
            ...args,
            code: code,
            handler: serializedFileNameNoExtension + "." + handlerName,
            runtime: args.runtime || _1.Runtime.NodeJS20dX,
            role: iam.Role.isInstance(role) ? role.arn : role,
            timeout: args.timeout === undefined ? 180 : args.timeout,
        };
        // If there is no custom Runtime argument being passed to the user
        // then we should add "runtime" to the ignoreChanges of the CustomResourceOptions
        // This is because as of 12/16/19, we upgraded the default NodeJS version from 8.x to 12.x as 12.x is latest LTS
        // We don't want to force recreation of user defined Lambdas because of this change
        if (!args.runtime) {
            pulumi.mergeOptions(opts, { ignoreChanges: ["runtime"] });
        }
        super(name, functionArgs, opts);
        if (iam.Role.isInstance(role)) {
            this.roleInstance = role;
        }
    }
}
exports.CallbackFunction = CallbackFunction;
// computeCodePaths calculates an AssetMap of files to include in the Lambda package.
async function computeCodePaths(closure, serializedFileNameNoExtension, codePathOptions) {
    const serializedFunction = await closure;
    // Construct the set of paths to include in the archive for upload.
    let codePaths = {
        // Always include the serialized function.
        [serializedFileNameNoExtension + ".js"]: new pulumi.asset.StringAsset(serializedFunction.text),
    };
    // AWS Lambda always provides `aws-sdk`, so skip this.  Do this before processing user-provided
    // extraIncludePackages so that users can force aws-sdk to be included (if they need a specific
    // version).
    codePathOptions = codePathOptions || {};
    codePathOptions.extraExcludePackages = codePathOptions.extraExcludePackages || [];
    codePathOptions.extraExcludePackages.push("aws-sdk");
    const modulePaths = await pulumi.runtime.computeCodePaths(codePathOptions);
    for (const [path, asset] of modulePaths) {
        codePaths[path] = asset;
    }
    return codePaths;
}
const lambdaRolePolicy = {
    "Version": "2012-10-17",
    "Statement": [
        {
            "Action": "sts:AssumeRole",
            "Principal": {
                "Service": "lambda.amazonaws.com",
            },
            "Effect": "Allow",
            "Sid": "",
        },
    ],
};
//# sourceMappingURL=lambdaMixins.js.map