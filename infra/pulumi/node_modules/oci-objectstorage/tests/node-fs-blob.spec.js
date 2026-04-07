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
const chai_1 = require("chai");
const node_fs_blob_1 = require("../lib/upload-manager/node-fs-blob");
const path_1 = require("path");
const HIGH_WATER_MARK = 20 * 1024 * 1024;
describe("Test Node FS Blob ", () => {
    var filename = "sample-file.txt";
    var fullpath = path_1.join(__dirname, filename);
    const blob = new node_fs_blob_1.NodeFSBlob(fullpath, HIGH_WATER_MARK);
    it("should return correct MD5 hash for the Blob ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            chai_1.expect(yield blob.getMD5Hash()).equals("s1ysCdHSTNe25nVk1MTpDQ==");
        });
    });
    it("should return correct Data for the Blob ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            const data = yield blob.getData();
            chai_1.expect(yield streamToString(data)).equals("This is sample file for upload manager unit tests.");
        });
    });
    function streamToString(stream) {
        const chunks = [];
        return new Promise((resolve, reject) => {
            stream.on("data", chunk => chunks.push(chunk));
            stream.on("error", reject);
            stream.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
        });
    }
});
//# sourceMappingURL=node-fs-blob.spec.js.map