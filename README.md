# nix/lix nixpkgs eval benchmark

This flake has all(?) available nix versions from nixpkgs benchmarked against
the nixpkgs ci eval singleSystem output/package.

To run it yourself just do `nix run .#nixbenchWrapped` and it will print out the
versions and times to stdout as well as a json file in ./data, if you want to
contribute to the data here open a pr and I'll happily merge it. In the future I
will make actual stats, graphs and all that nerd shit but I'm not yet sure when
that will be...

> [!WARNING]
> this flake uses `chunkSize = 15000;` which for me uses over >16gb of memory.
> if you do not have over 16gb free lower to anything from 500 to 5000

here is a table of the times for my system (r5 7600x, 64gb ddr5@6000mts) on
2026-04-14 at nixpkgs/0da3c44a9460a26d2025ec3ed2ec60a895eb1114 - full data file
is in ./data/1776078227.json

also pls take these numbers with a grain of salt as this was not ran at idle so
the scheduler might have added a few seconds to some results

| version                               | my times (s) |
| ------------------------------------- | ------------ |
| nix-2.35.0pre20260413_bec8cdc         | 43.21        |
| nix-2.34.5                            | 45.021       |
| nix-2.31.4                            | 45.84        |
| nix-2.30.4+1                          | 51.37        |
| nix-2.28.6                            | 132.12       |
| nix-2.27.2                            | 134.57       |
| nix-2.26.5                            | 143.31       |
| nix-2.25.6                            | 128.55       |
| nix-2.24.16                           | 127.24       |
| nix-2.23.5                            | 214.39       |
| nix-2.22.5                            | 227.44       |
| nix-2.22.5                            | 227.44       |
| nix-2.21.6                            | 226.65       |
| nix-2.20.10                           | 57.23        |
| nix-2.19.8                            | 63.11        |
| nix-2.18.10                           | 63.09        |
|                                       |              |
| lix-2.96.0-devpre20260411-dev_9899ed2 | 45.69        |
| lix-2.95.1                            | 43.86        |
| lix-2.94.1                            | 44.52        |
| lix-2.93.3                            | 55.61        |
| lix-2.92.3                            | 56.61        |
| lix-2.91.3                            | 56.79        |

TODOS:

- [ ] write incomplete data to disk?
- [ ] automatic table gen
