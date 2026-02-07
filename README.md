The documentation for this project is available at [https://samuelzila.github.io/dice_parser](https://samuelzila.github.io/dice_parser).

This crate provides functionality to parse and evaluate dice roll expressions in the rust programming language.
It supports standard arithmetic operations and dice notation (e.g., "2d6" for rolling two
six-sided dice).

# Features

- Parsing of expressions with numbers, operators (+, -, *, /), parentheses, as well as dice opperations.
- Optional logging of individual dice rolls through the `DiceLogger` struct.

# Example
```
let expression : Expression = "(12d8 + 34)/2".try_into().unwrap();
let mut logger = DiceLogger::new();

let result = expression.eval(&mut Some(&mut logger)).unwrap();

// The expression should evaluate to a value between 23 and 65, since the minimum roll for 12d8
//is 12 and the maximum is 96.
assert!(23.0 <= result && result <= 65.0);
// The logger should contain 12 entries, one for each die rolled.
assert_eq!(logger.iter().len(), 12);
```
