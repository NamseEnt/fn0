"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chai_1 = require("chai");
const path_1 = require("path");
const fs_1 = __importDefault(require("fs"));
const crypto_1 = __importDefault(require("crypto"));
const oci_common_1 = require("oci-common");
const MEMIBYTE = 1024 * 1024;
const partSize = 20 * MEMIBYTE;
const sampleString = "This is sample file for upload manager unit tests.";
const sampleBuffer = Buffer.from(sampleString);
describe("Test Chunker", () => {
    var filename = "sample-file.txt";
    var fullpath = path_1.join(__dirname, filename);
    let stream;
    let dataFeeder;
    let dataPart;
    const text = fs_1.default.readFileSync(fullpath, "utf8");
    const hash = crypto_1.default.createHash("md5");
    hash.update(text);
    const md5HashResult = hash.digest("base64");
    it("should return correct MD5 hash for the stream ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            stream = fs_1.default.createReadStream(fullpath);
            dataFeeder = oci_common_1.getChunk(stream, partSize);
            dataPart = (yield dataFeeder.next()).value;
            chai_1.expect(dataPart.md5Hash).equals(md5HashResult);
        });
    });
    it("should return correct Data for the stream ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            chai_1.expect(dataPart.data.toString()).equals("This is sample file for upload manager unit tests.");
        });
    });
    it("should return correct MD5 hash for the string ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            dataFeeder = oci_common_1.getChunk(sampleString, partSize);
            dataPart = (yield dataFeeder.next()).value;
            chai_1.expect(dataPart.md5Hash).equals(md5HashResult);
        });
    });
    it("should return correct Data for the string ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            chai_1.expect(dataPart.data.toString()).equals("This is sample file for upload manager unit tests.");
        });
    });
    it("should return correct MD5 hash for the buffer ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            dataFeeder = oci_common_1.getChunk(sampleBuffer, partSize);
            dataPart = (yield dataFeeder.next()).value;
            chai_1.expect(dataPart.md5Hash).equals(md5HashResult);
        });
    });
    it("should return correct Data for the Buffer ", function () {
        return __awaiter(this, void 0, void 0, function* () {
            chai_1.expect(dataPart.data.toString()).equals("This is sample file for upload manager unit tests.");
        });
    });
});
//# sourceMappingURL=chunker.spec.js.map