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
/**
 * {@link RemoteStack} is an isolated, independently configurable instance of a
 * Pulumi program that is operated on remotely.
 */
class RemoteStack {
    constructor(stack) {
        this.stack = stack;
        const ws = stack.workspace;
        if (!(ws instanceof localWorkspace_1.LocalWorkspace)) {
            throw new Error("expected workspace to be an instance of LocalWorkspace");
        }
    }
    /**
     * @internal
     */
    static create(stack) {
        return new RemoteStack(stack);
    }
    /**
     * The name identifying the Stack.
     */
    get name() {
        return this.stack.name;
    }
    /**
     * Creates or updates the resources in a stack by executing the program in
     * the Workspace. This operation runs remotely.
     *
     * @param opts
     *  Options to customize the behavior of the update.
     *
     * @see https://www.pulumi.com/docs/cli/commands/pulumi_up/
     */
    up(opts) {
        return this.stack.up(opts);
    }
    /**
     * Performs a dry-run update to a stack, returning pending changes. This
     * operation runs remotely.
     *
     * @param opts
     *  Options to customize the behavior of the preview.
     *
     * @see https://www.pulumi.com/docs/cli/commands/pulumi_preview/
     */
    preview(opts) {
        return this.stack.preview(opts);
    }
    /**
     * Compares the current stack’s resource state with the state known to exist
     * in the actual cloud provider. Any such changes are adopted into the
     * current stack. This operation runs remotely.
     *
     * @param opts
     *  Options to customize the behavior of the refresh.
     */
    refresh(opts) {
        return this.stack.refresh(opts);
    }
    /**
     * Deletes all resources in a stack, leaving all history and configuration
     * intact. This operation runs remotely.
     *
     * @param opts
     *  Options to customize the behavior of the destroy.
     */
    destroy(opts) {
        return this.stack.destroy(opts);
    }
    /**
     * Gets the current set of stack outputs from the last {@link Stack.up}.
     */
    outputs() {
        return this.stack.outputs();
    }
    /**
     * Returns a list summarizing all previous and current results from Stack
     * lifecycle operations (up/preview/refresh/destroy).
     */
    history(pageSize, page) {
        // TODO: Find a way to allow showSecrets as an option that doesn't require loading the project.
        return this.stack.history(pageSize, page, false);
    }
    /**
     * Stops a stack's currently running update. It returns an error if no
     * update is currently running. Note that this operation is _very
     * dangerous_, and may leave the stack in an inconsistent state if a
     * resource operation was pending when the update was canceled.
     */
    cancel() {
        return this.stack.cancel();
    }
    /**
     * Exports the deployment state of the stack. This can be combined with
     * {@link Stack.importStack} to edit a stack's state (such as recovery from
     * failed deployments).
     */
    exportStack() {
        return this.stack.exportStack();
    }
    /**
     * Imports the specified deployment state into a pre-existing stack. This
     * can be combined with {@link Stack.exportStack} to edit a stack's state
     * (such as recovery from failed deployments).
     *
     * @param state
     *  The stack state to import.
     */
    importStack(state) {
        return this.stack.importStack(state);
    }
}
exports.RemoteStack = RemoteStack;
//# sourceMappingURL=remoteStack.js.map