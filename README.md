# nix/lix nixpkgs eval benchmark

This flake has all(?) available nix versions from nixpkgs benchmarked against
the nixpkgs ci eval singleSystem output/package.

to run it yourself just do `nix run .#build-all` and it will print out the
versions and times.

> [!WARNING]
> this flake uses `chunkSize = 15000;` which for me uses over >16gb of memory.
> if you do not have over 16gb free lower to anything from 500 to 5000

here is a table of the times for my system (r5 7600x, 64gb ddr5@6000mts) and [@isabelroses](http://github.com/isabelroses)'s system (i7-12700KF, 32gb ddr4@3200mts)

| version                                    | my time's (s) | isabel's time (s) |
| ------------------------------------------ | ------------- | ----------------- |
| nix-stable (2.31.3)                        | 50.67         | 81.21             |
| nix-latest (2.34.2)                        | 48.99         | 85.60             |
| nix-git (2.35pre20260305_124b2777)         | 47.85         | 87.26             |
| nix-234 (2.34.2)                           | 48.99         | 85.60             |
| nix-233 (2.33.3)                           | 50.21         | 87.87             |
| nix-232 (2.32.6)                           | 49.43         | 86.76             |
| nix-231 (2.31.3)                           | 50.67         | 81.21             |
| nix-230 (2.30.3+1)                         | 57.63         | 104.60            |
| nix-228 (2.28.5)                           | 131.90        | 237.14            |
|                                            |               |                   |
| lix-stable (2.94.0)                        | 50.90         | 89.50             |
| lix-latest (2.94.0)                        | 50.90         | 89.50             |
| lix-git (2.96.0-pre-20260317_96db7c79cf2a) | 51.47         | 88.08             |
| lix-294 (2.94.0)                           | 50.90         | 89.50             |
| lix-293 (2.93.3)                           | 63.31         | 120.05            |


- [ ] CI to run monthly or smth
- [ ] automatic table gen
- [ ] add dix and snix?
