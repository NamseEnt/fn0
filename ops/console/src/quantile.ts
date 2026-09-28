export interface CumulativeBucket {
  /** Upper bound in the histogram's unit; `Infinity` for `+Inf`. */
  upperBound: number;
  /** Observations at or below `upperBound`. */
  count: number;
}

/**
 * The `histogram_quantile` convention over cumulative buckets: find the
 * bucket holding rank `q * total` and interpolate linearly inside it, the
 * first bucket starting at 0 and the `+Inf` bucket answering its lower bound.
 * Counts are made monotone first, because per-bucket increases taken over the
 * same window can disagree by a sample at its edges.
 *
 * Returns `null` when there is nothing to rank.
 */
export function histogramQuantile(
  q: number,
  buckets: readonly CumulativeBucket[],
): number | null {
  if (!(q >= 0 && q <= 1)) {
    return null;
  }
  const sorted = buckets
    .filter(
      (bucket) => !Number.isNaN(bucket.upperBound) && Number.isFinite(bucket.count),
    )
    .map((bucket) => ({ ...bucket }))
    .sort((left, right) => left.upperBound - right.upperBound);
  if (sorted.length === 0) {
    return null;
  }
  let runningMax = 0;
  for (const bucket of sorted) {
    runningMax = Math.max(runningMax, bucket.count);
    bucket.count = runningMax;
  }
  const total = sorted[sorted.length - 1]!.count;
  if (total <= 0) {
    return null;
  }
  const rank = q * total;
  let lowerBound = 0;
  let lowerCount = 0;
  for (const bucket of sorted) {
    if (bucket.count >= rank && bucket.count > lowerCount) {
      if (bucket.upperBound === Infinity) {
        return lowerBound;
      }
      const lower = Math.min(lowerBound, bucket.upperBound);
      return (
        lower +
        ((bucket.upperBound - lower) * (rank - lowerCount)) /
          (bucket.count - lowerCount)
      );
    }
    if (bucket.upperBound !== Infinity) {
      lowerBound = bucket.upperBound;
    }
    lowerCount = bucket.count;
  }
  return lowerBound;
}

/** Parses Signy's `le` label: a decimal bound, or `+Inf`. */
export function parseUpperBound(le: string | undefined): number | null {
  if (le === undefined) {
    return null;
  }
  if (le === "+Inf") {
    return Infinity;
  }
  const bound = Number(le);
  return Number.isFinite(bound) ? bound : null;
}
