# memory-alignment-tests

I just learned about memory alignment from [Production Twitter on One Machine? 100Gbps NICs and NVMe are fast - Tristan Hume](https://thume.ca/2023/01/02/one-machine-twitter/) and [this @mitchellh thread](https://twitter.com/mitchellh/status/1769143787862049013) and I wanted to experiment to see how much of a difference it makes.

I tested:

- Iterating over an array containing struct elements spanning 3 cache lines, explicitly tagged with `repr(align(64))`
- Iterating over an array containing struct elements spanning 3 cache lines, not tagged with `repr(align(64))`
- Iterating over an array containing struct elements spanning 7 bytes

Results, using `hyperfine`

| Command        |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
| :------------- | ---------: | -------: | -------: | ----------: |
| `explicit`     |  3.2 ± 0.3 |      2.9 |      4.5 | 1.02 ± 0.10 |
| `implicit`     |  3.1 ± 0.1 |      2.9 |      4.5 |        1.00 |
| `no-alignment` | 30.2 ± 0.3 |     29.8 |     32.8 | 9.68 ± 0.43 |

I found out that there's no real point of explicitly tagging the structs which are known to have a size which is a multiple of a cache line size. I'll have to learn more about when alignment is actually useful then.

But it's interesting/gratifying to see that the `no-alignment` case is significantly slower.
