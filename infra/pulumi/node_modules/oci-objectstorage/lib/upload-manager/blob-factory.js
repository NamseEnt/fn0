"use strict";
/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getContent = void 0;
const browser_blob_1 = require("./browser-blob");
const node_fs_blob_1 = require("./node-fs-blob");
const stream_blob_1 = require("./stream-blob");
/*
 * @Summary getBlob method to retreive proper type of blob
 *
 * @param Content
 * @param UploadOptions
 * @returns UploadableBlob Object
 */
function getContent(content, options) {
    return __awaiter(this, void 0, void 0, function* () {
        const chunkSize = options.allowedMemoryUsage / options.maxConcurrentUploads;
        if (content.blob)
            return new browser_blob_1.BrowserBlob(content.blob, chunkSize);
        else if (content.filePath)
            return new node_fs_blob_1.NodeFSBlob(content.filePath, chunkSize);
        else if (content.stream)
            return new stream_blob_1.StreamBlob(content.stream, chunkSize);
        else
            throw Error("Unidentified Blob type Found");
    });
}
exports.getContent = getContent;
//# sourceMappingURL=blob-factory.js.map