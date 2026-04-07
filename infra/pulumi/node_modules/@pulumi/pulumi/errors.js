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
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const grpc = __importStar(require("@grpc/grpc-js"));
const utils = __importStar(require("./utils"));
/**
 * {@link RunError} can be used for terminating a program abruptly, but
 * resulting in a clean exit rather than the usual verbose unhandled error logic
 * which emits the source program text and complete stack trace. This type
 * should be rarely used. Ideally {@link ResourceError} should always be used so
 * that as many errors as possible can be associated with a resource.
 */
class RunError extends Error {
    constructor() {
        super(...arguments);
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention,no-underscore-dangle,id-blacklist,id-match
        this.__pulumiRunError = true;
    }
    /**
     * Returns true if the given object is a {@link RunError}. This is designed
     * to work even when multiple copies of the Pulumi SDK have been loaded into
     * the same process.
     */
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiRunError");
    }
}
exports.RunError = RunError;
/**
 * {@link ResourceError} can be used for terminating a program abruptly,
 * specifically associating the problem with a {@link Resource}. Depending on
 * the nature of the problem, clients can choose whether or not the call stack
 * should be hidden as well. This should be very rare, and would only indicate
 * that presenting the stack to the user would not be useful/be detrimental.
 */
class ResourceError extends Error {
    constructor(message, resource, hideStack) {
        super(message);
        this.resource = resource;
        this.hideStack = hideStack;
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention, no-underscore-dangle, id-blacklist, id-match
        this.__pulumResourceError = true;
    }
    /**
     * Returns true if the given object is a {@link ResourceError}. This is
     * designed to work even when multiple copies of the Pulumi SDK have been
     * loaded into the same process.
     */
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumResourceError");
    }
}
exports.ResourceError = ResourceError;
function isGrpcError(err) {
    const code = err.code;
    return code === grpc.status.UNAVAILABLE || code === grpc.status.CANCELLED;
}
exports.isGrpcError = isGrpcError;
/**
 * {@link InputPropertyError} is an error type that is used to indicate that a
 * an input property has failed validation.
 *
 * The engine will pretty print this error for the user.
 */
class InputPropertyError extends Error {
    constructor(args) {
        super(args.reason);
        /** @internal */
        this.__pulumiInputPropertyError = true;
        this.propertyPath = args.propertyPath;
        this.reason = args.reason;
    }
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiInputPropertyError");
    }
}
exports.InputPropertyError = InputPropertyError;
/**
 * {@link InputPropertiesError} is an error type that is used to indicate that
 * multiple input properties have failed validation.
 *
 * The engine will pretty print this error for the user.
 */
class InputPropertiesError extends Error {
    constructor(args) {
        super(args.message);
        /** @internal */
        this.__pulumiInputPropertiesError = true;
        this.message = args.message;
        this.errors = args.errors;
    }
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiInputPropertiesError");
    }
}
exports.InputPropertiesError = InputPropertiesError;
//# sourceMappingURL=errors.js.map