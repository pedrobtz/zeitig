#!/usr/bin/env sh
# Vendor the Rust dependencies for an offline (CRAN) build.
#
# Run from the package root after any change to src/rust/Cargo.toml or
# src/rust/Cargo.lock:
#
#   sh tools/vendor.sh
#
# Produces
#   src/rust/vendor.tar.xz  sources of every crate in Cargo.lock (extracted by
#                           src/Makevars at install time)
#   inst/AUTHORS            authors and licence of each vendored crate
#   LICENSE.note            summary of the licences of the vendored crates
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUST_DIR="${ROOT}/src/rust"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

cd "${RUST_DIR}"
cargo vendor -q --locked --versioned-dirs --manifest-path Cargo.toml "${WORK}/vendor" > /dev/null

# Deterministic archive: sorted entries, fixed owner and timestamps.
tar -C "${WORK}" --sort=name --owner=0 --group=0 --numeric-owner \
  --mtime='2000-01-01 00:00Z' -cf - vendor | xz -9e -T1 > "${RUST_DIR}/vendor.tar.xz"

cargo metadata --locked --format-version 1 --manifest-path Cargo.toml > "${WORK}/metadata.json"

Rscript --vanilla - "${WORK}/metadata.json" "${ROOT}" <<'REOF'
args <- commandArgs(trailingOnly = TRUE)
meta <- jsonlite::fromJSON(args[[1]], simplifyVector = FALSE)
root <- args[[2]]
pkgs <- Filter(function(p) !is.null(p$source), meta$packages)
pkgs <- pkgs[order(vapply(pkgs, function(p) p$name, ""), vapply(pkgs, function(p) p$version, ""))]

authors <- vapply(pkgs, function(p) {
  who <- unlist(p$authors)
  if (length(who) == 0) who <- "(no authors listed; see the crate repository)"
  who <- gsub("\\s*<[^>]*>", "", who)
  paste0(
    p$name, " ", p$version, " (", p$license, ")\n",
    paste0("  - ", who, collapse = "\n")
  )
}, "")
writeLines(
  c(
    "The zeitig package bundles the following Rust crates in",
    "src/rust/vendor.tar.xz. Their authors and licences are:",
    "",
    authors
  ),
  file.path(root, "inst", "AUTHORS"),
  sep = "\n"
)

licenses <- vapply(pkgs, function(p) p$license, "")
by_license <- split(
  vapply(pkgs, function(p) paste(p$name, p$version), ""),
  licenses
)
note <- c(
  "The zeitig package itself is licensed under the MIT licence (see LICENSE).",
  "",
  "It bundles the sources of the Rust crates listed in inst/AUTHORS",
  "(src/rust/vendor.tar.xz). All of them are available under permissive",
  "licences compatible with MIT; where a crate offers a choice of licences it",
  "is used under the MIT terms (unicode-ident additionally ships Unicode data",
  "under the Unicode-3.0 licence). The licence expressions, as declared by",
  "each crate, are:",
  ""
)
for (lic in sort(names(by_license))) {
  note <- c(note, paste0(lic, ":"), paste0("  - ", by_license[[lic]]), "")
}
writeLines(note, file.path(root, "LICENSE.note"))
REOF

echo "vendor.tar.xz: $(du -h "${RUST_DIR}/vendor.tar.xz" | cut -f1)"
