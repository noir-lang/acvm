# Benchmark results

## Math operations

| Operation | ark_bn254 time (ns) | uint time (ns) | from uint time (ns) |
| :--- | :---: | :---: | :---: |
| multiplication    | 22.796  | 0.99923      | 22.775      |
| subtraction    | 10.052  | 1.0178       |  10.033       |
| addition    | 1.0185  | 1.0383      |  1.0532      |
| division    | 3168.1  | 11.791      | 3165.8      |

## Other functions

| Operation | Execution time (ns) |
| :--- | :---: |
| pow    | 463.76  |
| try_from_str    | 182.71  |
| num_bits    | 1063.4  |
| to_u128    | 91.771  |
| inverse    | 3120.2  |
| to_hex    | 292.32  |
| from_hex    | 533.96  |
| to_be_bytes    | 85.729  |
| bits    | 905.03  |
| fetch_nearest_bytes    | 113.40  |
| and    | 452.39  |
| xor    | 439.77  |
