import * as pulumi from "@pulumi/pulumi";
import { gzipSync } from "node:zlib";

export function buildWorkerUserData(
  cloudInit: pulumi.Input<string>,
): pulumi.Output<string> {
  return pulumi.secret(
    pulumi
      .output(cloudInit)
      .apply((content) =>
        gzipSync(Buffer.from(content, "utf8")).toString("base64")
      ),
  );
}

export function buildWorkerInstanceMetadata(
  userData: pulumi.Input<string>,
  sshPublicKey: pulumi.Input<string | undefined>,
): pulumi.Output<{ [key: string]: string }> {
  return pulumi.secret(
    pulumi.all([userData, sshPublicKey]).apply(([encodedUserData, sshKey]) => {
      const metadata: { [key: string]: string } = {
        user_data: encodedUserData,
      };
      if (sshKey) metadata.ssh_authorized_keys = sshKey;
      return metadata;
    }),
  );
}
