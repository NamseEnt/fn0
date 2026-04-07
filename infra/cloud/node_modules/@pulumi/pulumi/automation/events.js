"use strict";
// Copyright 2016-2020, Pulumi Corporation.
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
/**
 * A {@link DiffKind} describes the kind of difference between two values
 * reported in a diff.
 */
var DiffKind;
(function (DiffKind) {
    /**
     * Indicates that the property was added.
     */
    DiffKind["add"] = "add";
    /**
     * Indicates that the property was added and requires that the resource be replaced.
     */
    DiffKind["addReplace"] = "add-replace";
    /**
     * Indicates that the property was deleted.
     */
    DiffKind["delete"] = "delete";
    /**
     * Indicates that the property was deleted and requires that the resource be replaced.
     */
    DiffKind["deleteReplace"] = "delete-replace";
    /**
     * Indicates that the property was updated.
     */
    DiffKind["update"] = "update";
    /**
     * Indicates that the property was updated and requires that the resource be replaced.
     */
    DiffKind["updateReplace"] = "update-replace";
})(DiffKind = exports.DiffKind || (exports.DiffKind = {}));
//# sourceMappingURL=events.js.map