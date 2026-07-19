# Handoff C7-C — Fix is_managed_install grok-oss identity (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |

## Goal

Fix pre-existing failure `is_managed_install_matches_only_the_bin_grok_target` that hardcodes `bin/grok` while product binary is `grok-oss` (identity cutover). Align test and any production check with `PRODUCT_BIN_NAME`.

## Owned

Minimal files for managed-install path + test only.

## Acceptance

Test green; no behavior regression for grok-oss install detection.

## Report

Files, RED/GREEN.
