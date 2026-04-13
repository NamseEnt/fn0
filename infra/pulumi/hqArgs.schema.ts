import * as pulumi from '@pulumi/pulumi';
export interface HqArgs {
  aws: pulumi.Input<AwsArgs>;
  caCertPem: pulumi.Input<string>;
  dnsProvider: pulumi.Input<DnsProviderArg>;
  docDb: pulumi.Input<DocDbArgs>;
  selfDns: pulumi.Input<SelfDnsArgs>;
  sites: pulumi.Input<Array<SiteArgs>>;
  wsSecret: pulumi.Input<string>;
}
export interface AwsArgs {
  accessKeyId: pulumi.Input<string>;
  cwasmBucket: pulumi.Input<string>;
  region: pulumi.Input<string>;
  secretAccessKey: pulumi.Input<string>;
  wasmBucket: pulumi.Input<string>;
}
export interface CloudflareDnsProviderArgs {
  apiToken: pulumi.Input<string>;
  asteriskDomain: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
}
export interface DnsProviderArg {
  cloudflare?: pulumi.Input<CloudflareDnsProviderArgs>;
}
export interface DocDbArgs {
  token: pulumi.Input<string>;
  url: pulumi.Input<string>;
}
export interface HostProviderArg {
  ociComputeVm?: pulumi.Input<OciComputeVmHostProviderArgs>;
}
export interface OciComputeVmHostProviderArgs {
  availabilityDomain: pulumi.Input<string>;
  compartmentId: pulumi.Input<string>;
  envs: pulumi.Input<Record<string, string>>;
  fingerprint: pulumi.Input<string>;
  imageId: pulumi.Input<string>;
  memoryInGbs: pulumi.Input<number>;
  ocpus: pulumi.Input<number>;
  physicsCpuCores: pulumi.Input<number>;
  privateKeyBase64: pulumi.Input<string>;
  region: pulumi.Input<string>;
  shape: pulumi.Input<string>;
  sshAuthorizedKeys: pulumi.Input<string>;
  sshPrivateKeyBase64: pulumi.Input<string>;
  subnetId: pulumi.Input<string>;
  tenancyId: pulumi.Input<string>;
  userId: pulumi.Input<string>;
  workerImageUrl: pulumi.Input<string>;
}
export interface SelfDnsArgs {
  cloudflareApiToken: pulumi.Input<string>;
  cloudflareZoneId: pulumi.Input<string>;
  hostname: pulumi.Input<string>;
}
export interface SiteArgs {
  hostProvider: pulumi.Input<HostProviderArg>;
  name: pulumi.Input<string>;
}
