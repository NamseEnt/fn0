"use strict";
// Copyright 2016-2022, Pulumi Corporation.
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
const localWorkspace_1 = require("./localWorkspace");
const remoteStack_1 = require("./remoteStack");
const stack_1 = require("./stack");
/**
 * {@link RemoteWorkspace} is the execution context containing a single remote
 * Pulumi project.
 */
class RemoteWorkspace {
    /**
     * Creates a stack backed by a {@link RemoteWorkspace} with source code from
     * the specified Git repository. Pulumi operations on the stack (preview,
     * update, refresh, and destroy) are performed remotely.
     *
     * @param args
     *  A set of arguments to initialize a {@link RemoteStack} with a remote
     *  Pulumi program from a Git repository.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static async createStack(args, opts) {
        const ws = await createLocalWorkspace(args, opts);
        const stack = await stack_1.Stack.create(args.stackName, ws);
        return remoteStack_1.RemoteStack.create(stack);
    }
    /**
     * Selects an existing stack backed by a {@link RemoteWorkspace} with source
     * code from the specified Git repository. Pulumi operations on the stack
     * (preview, update, refresh, and destroy) are performed remotely.
     *
     * @param args
     *  A set of arguments to initialize a {@link RemoteStack} with a remote
     *  Pulumi program from a Git repository.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static async selectStack(args, opts) {
        const ws = await createLocalWorkspace(args, opts);
        const stack = await stack_1.Stack.select(args.stackName, ws);
        return remoteStack_1.RemoteStack.create(stack);
    }
    /**
     * Creates or selects an existing stack backed by a {@link RemoteWorkspace}
     * with source code from the specified Git repository. Pulumi operations on
     * the stack (preview, update, refresh, and destroy) are performed remotely.
     *
     * @param args
     *  A set of arguments to initialize a RemoteStack with a remote Pulumi program from a Git repository.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static async createOrSelectStack(args, opts) {
        const ws = await createLocalWorkspace(args, opts);
        const stack = await stack_1.Stack.createOrSelect(args.stackName, ws);
        return remoteStack_1.RemoteStack.create(stack);
    }
    constructor() { } // eslint-disable-line @typescript-eslint/no-empty-function
}
exports.RemoteWorkspace = RemoteWorkspace;
async function createLocalWorkspace(args, opts) {
    if (!isFullyQualifiedStackName(args.stackName)) {
        throw new Error(`stack name "${args.stackName}" must be fully qualified.`);
    }
    if (!args.url && !opts?.inheritSettings) {
        throw new Error("url is required if inheritSettings is not set.");
    }
    if (args.branch && args.commitHash) {
        throw new Error("branch and commitHash cannot both be specified.");
    }
    if (!args.branch && !args.commitHash && !opts?.inheritSettings) {
        throw new Error("either branch or commitHash is required if inheritSettings is not set.");
    }
    if (args.auth) {
        if (args.auth.sshPrivateKey && args.auth.sshPrivateKeyPath) {
            throw new Error("sshPrivateKey and sshPrivateKeyPath cannot both be specified.");
        }
    }
    const localOpts = {
        remote: true,
        remoteGitProgramArgs: args,
        remoteEnvVars: opts?.envVars,
        remotePreRunCommands: opts?.preRunCommands,
        remoteSkipInstallDependencies: opts?.skipInstallDependencies,
        remoteInheritSettings: opts?.inheritSettings,
        remoteExecutorImage: opts?.executorImage,
    };
    return await localWorkspace_1.LocalWorkspace.create(localOpts);
}
/**
 * @internal
 *  Exported only so it can be tested.
 */
function isFullyQualifiedStackName(stackName) {
    if (!stackName) {
        return false;
    }
    const split = stackName.split("/");
    return split.length === 3 && !!split[0] && !!split[1] && !!split[2];
}
exports.isFullyQualifiedStackName = isFullyQualifiedStackName;
//# sourceMappingURL=remoteWorkspace.js.map