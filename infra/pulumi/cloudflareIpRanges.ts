/**
 * Cloudflare's published edge ranges, as of 2026-07-30.
 *
 * Once users point their own DNS at the worker fleet the origin IP stops being
 * unpublished, so ingress has to be an actual control rather than obscurity.
 * These are the only addresses that ever legitimately open a TLS connection to
 * the fleet: every request arrives through some Cloudflare zone's edge, whether
 * the platform's or a user's.
 *
 * Cloudflare changes these rarely and announces changes; refresh with
 * `curl https://api.cloudflare.com/client/v4/ips` and re-run `pulumi up`.
 * A stale list fails closed — new Cloudflare edges cannot reach the origin —
 * so treat a sudden rise in edge-side 521s as the signal to refresh.
 */
export const CLOUDFLARE_IPV4_RANGES = [
  "173.245.48.0/20",
  "103.21.244.0/22",
  "103.22.200.0/22",
  "103.31.4.0/22",
  "141.101.64.0/18",
  "108.162.192.0/18",
  "190.93.240.0/20",
  "188.114.96.0/20",
  "197.234.240.0/22",
  "198.41.128.0/17",
  "162.158.0.0/15",
  "104.16.0.0/13",
  "104.24.0.0/14",
  "172.64.0.0/13",
  "131.0.72.0/22",
];

export const CLOUDFLARE_IPV6_RANGES = [
  "2400:cb00::/32",
  "2606:4700::/32",
  "2803:f800::/32",
  "2405:b500::/32",
  "2405:8100::/32",
  "2a06:98c0::/29",
  "2c0f:f248::/32",
];
