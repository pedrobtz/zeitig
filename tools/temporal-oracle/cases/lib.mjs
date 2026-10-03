// Shared helpers for the case generators. Everything here is deterministic:
// random sampling uses a fixed-seed PRNG so regenerating gives the same cases.

export function rng(seed) {
  // mulberry32
  let a = seed >>> 0;
  const next = () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  return {
    next,
    int: (lo, hi) => lo + Math.floor(next() * (hi - lo + 1)),
    pick: (xs) => xs[Math.floor(next() * xs.length)],
  };
}

export const ROUNDING_MODES = [
  "ceil", "floor", "expand", "trunc", "halfCeil", "halfFloor", "halfExpand", "halfTrunc",
  "halfEven",
];

export const DATE_UNITS = ["year", "month", "week", "day"];
export const TIME_UNITS = ["hour", "minute", "second", "millisecond", "microsecond", "nanosecond"];
export const ALL_UNITS = [...DATE_UNITS, ...TIME_UNITS];

// Valid rounding increments for a time unit (divisors of the next larger
// unit, excluding the unit itself), plus one invalid value.
export const INCREMENTS = {
  hour: [1, 2, 3, 4, 6, 8, 12],
  minute: [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30],
  second: [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30],
  millisecond: [1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 125, 200, 250, 500],
  microsecond: [1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 125, 200, 250, 500],
  nanosecond: [1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 125, 200, 250, 500],
};

export function unitIndex(u) {
  return ALL_UNITS.indexOf(u);
}

// All (largest, smallest) pairs with largest >= smallest from `units`, plus
// "auto" as largest.
export function unitPairs(units) {
  const out = [];
  for (let i = 0; i < units.length; i++) {
    for (let j = i; j < units.length; j++) out.push([units[i], units[j]]);
  }
  for (const s of units) out.push(["auto", s]);
  return out;
}

// Removes options that are undefined so the fixture stays compact.
export function opts(o) {
  const out = {};
  for (const [k, v] of Object.entries(o)) if (v !== undefined) out[k] = v;
  return Object.keys(out).length ? out : undefined;
}
