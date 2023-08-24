# topsis

A [TOPSIS](https://en.wikipedia.org/wiki/TOPSIS) implementation written in Rust.

Technique for Order of Preference by Similarity to Ideal Solution ([TOPSIS](https://en.wikipedia.org/wiki/TOPSIS)) is a method for multi-criteria decision making. It compares a set of alternatives based on a pre-specified criterion. It allows trade-offs between criteria, where a poor result in one criterion can be negated by a good result in another criterion.

# Example

```rust
use topsis::calculate;

fn main() {
    let ranking = calculate(
        &[0.64339, 0.28284, 0.07377],
        &[true, true, true],
        &[
          80.0, 70.0, 91.0, 90.0, 80.0, 71.0, 90.0, 78.0, 0.0, 1.0, 0.0, 4.0,
        ],
    );

    println!("{:#?}", ranking);
}
```
