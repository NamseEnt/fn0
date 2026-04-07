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
const output_1 = require("./output");
const resource_1 = require("./resource");
/**
 * Manages a reference to a Pulumi stack. The referenced stack's outputs are
 * available via the {@link StackReference.outputs} property or the
 * {@link StackReference.output} method.
 */
class StackReference extends resource_1.CustomResource {
    /**
     * Create a {@link StackReference} resource with the given unique name,
     * arguments, and options.
     *
     * If args is not specified, the name of the referenced stack will be the
     * name of the {@link StackReference} resource.
     *
     * @param name
     *  The _unique_ name of the stack reference.
     * @param args
     *  The arguments to use to populate this resource's properties.
     * @param opts
     *  A bag of options that control this resource's behavior.
     */
    constructor(name, args, opts) {
        args = args || {};
        const stackReferenceName = args.name || name;
        super("pulumi:pulumi:StackReference", name, {
            name: stackReferenceName,
            outputs: undefined,
            secretOutputNames: undefined,
        }, { ...opts, id: stackReferenceName });
    }
    /**
     * Fetches the value of the named stack output, or `undefined` if the stack
     * output was not found.
     *
     * @param name
     *  The name of the stack output to fetch.
     */
    getOutput(name) {
        // Note that this is subtly different from "apply" here. A default "apply" will set the secret bit if any
        // of the inputs are a secret, and this.outputs is always a secret if it contains any secrets. We do this dance
        // so we can ensure that the Output we return is not needlessly tainted as a secret.
        const value = output_1.all([output_1.output(name), this.outputs]).apply(([n, os]) => os[n]);
        // 'value' is an Output produced by our own `.apply` implementation.  So it's safe to
        // `.allResources!` on it.
        return new output_1.Output(value.resources(), value.promise(), value.isKnown, isSecretOutputName(this, output_1.output(name)), value.allResources());
    }
    /**
     * Fetches the value of the named stack output, or throws an error if the
     * output was not found.
     *
     * @param name
     *  The name of the stack output to fetch.
     */
    requireOutput(name) {
        const value = output_1.all([output_1.output(this.name), output_1.output(name), this.outputs]).apply(([stackname, n, os]) => {
            if (!os.hasOwnProperty(n)) {
                throw new Error(`Required output '${n}' does not exist on stack '${stackname}'.`);
            }
            return os[n];
        });
        return new output_1.Output(value.resources(), value.promise(), value.isKnown, isSecretOutputName(this, output_1.output(name)), value.allResources());
    }
    /**
     * Fetches the value of the named stack output and builds a
     * {@link StackReferenceOutputDetails} with it.
     *
     * The returned object has its `value` or `secretValue` fields set depending
     * on whether the output is a secret. Neither field is set if the output was
     * not found.
     *
     * @param name
     *  The name of the stack output to fetch.
     */
    async getOutputDetails(name) {
        const [out, isSecret] = await this.readOutputValue("getOutputValueDetails", name, false /*required*/);
        if (isSecret) {
            return { secretValue: out };
        }
        else {
            return { value: out };
        }
    }
    /**
     * Fetches the value promptly of the named stack output. May return
     * undefined if the value is not known for some reason.
     *
     * This operation is not supported (and will throw) if the named stack
     * output is a secret.
     *
     * @param name
     *  The name of the stack output to fetch.
     */
    async getOutputValue(name) {
        const [out, isSecret] = await this.readOutputValue("getOutputValue", name, false /*required*/);
        if (isSecret) {
            throw new Error("Cannot call 'getOutputValue' if the referenced stack output is a secret. Use 'getOutput' instead.");
        }
        return out;
    }
    /**
     * Fetches the value promptly of the named stack output. Throws an error if
     * the stack output is not found.
     *
     * This operation is not supported (and will throw) if the named stack
     * output is a secret.
     *
     * @param name
     *  The name of the stack output to fetch.
     */
    async requireOutputValue(name) {
        const [out, isSecret] = await this.readOutputValue("requireOutputSync", name, true /*required*/);
        if (isSecret) {
            throw new Error("Cannot call 'requireOutputValue' if the referenced stack output is a secret. Use 'requireOutput' instead.");
        }
        return out;
    }
    async readOutputValue(callerName, outputName, required) {
        const out = required ? this.requireOutput(outputName) : this.getOutput(outputName);
        return Promise.all([out.promise(), out.isSecret]);
    }
}
exports.StackReference = StackReference;
async function isSecretOutputName(sr, name) {
    const nameOutput = output_1.output(name);
    // If either the name or set of secret outputs is unknown, we can't do anything smart, so we just copy the
    // secretness from the entire outputs value.
    if (!((await nameOutput.isKnown) && (await sr.secretOutputNames.isKnown))) {
        return await sr.outputs.isSecret;
    }
    // Otherwise, if we have a list of outputs we know are secret, we can use that list to determine if this
    // output should be secret. Names could be falsy here in cases where we are using an older CLI that did
    // not return this information (in this case we again fallback to the secretness of outputs value).
    const names = await sr.secretOutputNames.promise();
    if (!names) {
        return await sr.outputs.isSecret;
    }
    return names.includes(await nameOutput.promise());
}
//# sourceMappingURL=stackReference.js.map