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
const utils = __importStar(require("../utils"));
/**
 * An {@link Archive} represents a collection of named assets.
 */
class Archive {
    constructor() {
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention,no-underscore-dangle,id-blacklist,id-match
        this.__pulumiArchive = true;
    }
    /**
     * Returns true if the given object is an {@link Archive}. This is designed
     * to work even when multiple copies of the Pulumi SDK have been loaded into
     * the same process.
     */
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiArchive");
    }
}
exports.Archive = Archive;
/**
 * An {@link AssetArchive} is an archive created from an in-memory collection of
 * named assets or other archives.
 */
class AssetArchive extends Archive {
    constructor(assets) {
        super();
        this.assets = Promise.resolve(assets);
    }
}
exports.AssetArchive = AssetArchive;
/**
 * A {@link FileArchive} is a file-based archive, or a collection of file-based
 * assets. This can be a raw directory or a single archive file in one of the
 * supported formats (`.tar`, `.tar.gz`, or `.zip`).
 */
class FileArchive extends Archive {
    constructor(path) {
        super();
        this.path = Promise.resolve(path);
    }
}
exports.FileArchive = FileArchive;
/**
 * A {@link RemoteArchive} is a file-based archive fetched from a remote
 * location. The URI's scheme dictates the protocol for fetching the archive's
 * contents: `file://` is a local file (just like a {@link FileArchive}),
 * `http://` and `https://` specify HTTP and HTTPS, respectively, and specific
 * providers may recognize custom schemes.
 */
class RemoteArchive extends Archive {
    constructor(uri) {
        super();
        this.uri = Promise.resolve(uri);
    }
}
exports.RemoteArchive = RemoteArchive;
//# sourceMappingURL=archive.js.map