/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
/// <reference types="node" />
import { Readable } from "stream";
import { UploadableBlob } from "./uploadable-blob";
export declare class NodeFSBlob implements UploadableBlob {
    private readonly filePath;
    private readonly highWaterMark;
    size: number;
    type: string;
    private readonly start;
    private readonly end;
    private readonly fileSize;
    constructor(filePath: string, highWaterMark: number, start?: number, end?: number, fileSize?: number);
    getData(): Promise<Readable>;
    slice(start?: number | undefined, end?: number | undefined, contentType?: string | undefined): UploadableBlob;
    getMD5Hash(): Promise<string>;
    arrayBuffer(): Promise<ArrayBuffer>;
    text(): Promise<string>;
    streamToString(stream: Readable): Promise<string>;
    stream(): ReadableStream;
}
