//! This crate provides functionality to parse and evaluate dice roll expressions.
//! It supports standard arithmetic operations and dice notation (e.g., "2d6" for rolling two
//! six-sided dice).
//!
//! # Features
//!
//! - Parsing of expressions with numbers, operators (+, -, *, /), parentheses, as well as dice
//! expression.
//! - Optional logging of individual dice rolls through the `DiceLogger` struct.
//!
//! # Examples
//! ```
//! # use dice_parser::{Expression, DiceLogger};
//!
//! # fn main() {
//! let expression : Expression = "(12d8 + 34)/2".try_into().unwrap();
//! let mut logger = DiceLogger::new();
//!
//! let result = expression.eval(&mut Some(&mut logger)).unwrap();
//!
//! // The expression should evaluate to a value between 23 and 65, since the minimum roll for 12d8
//! //is 12 and the maximum is 96.
//! assert!(23.0 <= result && result <= 65.0);
//! // The logger should contain 12 entries, one for each die rolled.
//! assert_eq!(logger.iter().len(), 12);
//! # }
//! ```

use std::{fmt::Display, ops::Deref};

use rand::{Rng, rng};

#[derive(Clone, Debug, Default)]
/// A logger for dice rolls, which can be used to keep track of the individual rolls that were made
/// during the evaluation of an expression. The functionality is implemented in the dice rolling
/// functions directly.
///
/// The `DiceLogger` struct is basically a wrapper around a `Vec<u32>`. It can be dereferenced to a
/// `Vec<u32>`.
///
/// # Examples
/// ```
/// # use dice_parser::DiceLogger;
/// let expression : dice_parser::Expression = "12d8+34".try_into().unwrap();
/// let mut logger = DiceLogger::new();
/// let _ = expression.eval(&mut Some(&mut logger)).unwrap();
///
/// assert_eq!(logger.iter().len(), 12);
/// ```
pub struct DiceLogger {
    data: Vec<u32>,
}
impl DiceLogger {
    /// Creates a new, empty `DiceLogger`. Same as `DiceLogger::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    fn append(&mut self, collection: &mut Vec<u32>) {
        self.data.append(collection);
    }
}

impl Display for DiceLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data.is_empty() {
            write!(f, "No dice rolled")
        } else {
            write!(
                f,
                "{}",
                self.data
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

/// Allows `DiceLogger` to be dereferenced to a `Vec<u32>`, so that you can use it as if it were a
/// vector of dice rolls.
impl Deref for DiceLogger {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<DiceLogger> for Vec<u32> {
    /// Converts a `DiceLogger` into a `Vec<u32>`, consuming the logger in the process.
    fn from(logger: DiceLogger) -> Self {
        logger.data
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Token {
    Number(u32),
    Op(char),
    Eof,
}

struct Lexer {
    tokens: Vec<Token>,
}
impl Lexer {
    fn new(input: &str) -> Result<Self, String> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut iterator = input.chars().filter(|it| !it.is_whitespace()).peekable();

        while iterator.peek().is_some() {
            let c = iterator.next().unwrap();
            match c {
                '+' | '-' | '*' | '/' | '(' | ')' => tokens.push(Token::Op(c)),
                'd' | 'D' => tokens.push(Token::Op('d')),
                '0'..='9' => {
                    let mut number = c.to_digit(10).unwrap();
                    while let Some(&next) = iterator.peek() {
                        if next.is_digit(10) {
                            number = number * 10 + iterator.next().unwrap().to_digit(10).unwrap();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(number));
                }
                _ => return Err(format!("Unexpected character: {}", c)),
            }
        }

        tokens.reverse();
        Ok(Lexer { tokens })
    }
    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }
    fn peek(&mut self) -> Token {
        self.tokens.last().copied().unwrap_or(Token::Eof)
    }
}

#[derive(Clone, Debug)]
/// Represents a parsed expression, which can be either a number or an operation with operands. The
/// `eval` method can be used to evaluate the expression, optionally logging any dice rolls that
/// occur during the evaluation. The `TryFrom<&str>` implementation allows you to parse a string
/// into an `Expression`.
///
/// # Examples
/// ```
/// # use dice_parser::Expression;
/// let expression : Expression = "12d8+34".try_into().unwrap();
/// let result = expression.eval(&mut None).unwrap();
///
/// assert!(46.0 <= result && result <= 130.0);
/// ```
pub enum Expression {
    Number(u32),
    Operation(char, Vec<Expression>),
}
impl TryFrom<&str> for Expression {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut lexer = Lexer::new(value)?;
        Ok(parse_expression(&mut lexer, 0.)?)
    }
}
/// The default expression is just the number 0, which evaluates to 0.0.
impl Default for Expression {
    fn default() -> Self {
        Expression::Number(0)
    }
}
impl Expression {
    /// Evaluates the expression, returning the result as a `f32`. If a `DiceLogger` is provided,
    /// any dice rolls that occur during the evaluation will be logged in the logger.
    ///
    /// # Results
    ///
    /// If the expression is valid, the result will be a `f32` representing the evaluated value of
    /// the expression. If the expression is invalid (e.g., contains an unknown operator), an error
    /// message will be returned as a `String`.
    pub fn eval(&self, dice_logger: &mut Option<&mut DiceLogger>) -> Result<f32, String> {
        Ok(match self {
            Expression::Number(n) => *n as f32,

            Expression::Operation(operator, operands) => {
                let lhs = operands.first().unwrap().eval(dice_logger)?;
                let rhs = operands.last().unwrap().eval(dice_logger)?;

                match operator {
                    '+' => lhs + rhs,
                    '-' => lhs - rhs,
                    '*' => lhs * rhs,
                    '/' => lhs / rhs,
                    'd' => {
                        let (sum, mut collection) = roll_dice(lhs as u32, rhs as u32);
                        if let Some(dice_logger) = dice_logger {
                            dice_logger.append(&mut collection);
                        }
                        sum as f32
                    }

                    _ => return Err(format!("Unknown operator: {}", operator)),
                }
            }
        })
    }
}

fn operation_priority(op: char) -> Result<(f32, f32), String> {
    Ok(match op {
        '+' | '-' => (1.0, 1.1),
        '*' | '/' => (2.0, 2.1),
        'd' => (3.0, 3.1),
        _ => return Err(format!("Unknown operator: {:?}", op)),
    })
}

fn parse_expression(lexer: &mut Lexer, min_bp: f32) -> Result<Expression, String> {
    let mut lhs = match lexer.next() {
        Token::Number(n) => Expression::Number(n),
        Token::Op('(') => {
            let lhs = parse_expression(lexer, 0.0)?;
            assert_eq!(lexer.next(), Token::Op(')'));
            lhs
        }
        t => return Err(format!("Expected a number, found: {:?}", t)),
    };

    loop {
        let op = match lexer.peek() {
            Token::Eof => break,
            Token::Op(')') => break,
            Token::Op(op) => op,
            t => return Err(format!("Expected an operator, found: {:?}", t)),
        };
        let (l_bp, r_bp) = operation_priority(op)?;
        if l_bp < min_bp {
            break;
        }
        lexer.next();
        let rhs = parse_expression(lexer, r_bp)?;
        lhs = Expression::Operation(op, vec![lhs, rhs]);
    }
    Ok(lhs)
}

fn roll_dice(amount: u32, sides: u32) -> (u32, Vec<u32>) {
    let results: Vec<u32> = (0..amount).map(|_| rng().random_range(1..=sides)).collect();
    let sum = results.iter().sum();
    (sum, results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        let mut logger = DiceLogger::new();
        let expression: Expression = ("12d8+34").try_into().unwrap();
        expression.eval(&mut Some(&mut logger)).unwrap();

        assert_eq!(logger.iter().len(), 12);
    }
    #[test]
    fn test_evaluation() {
        let expression: Expression = ("15+30000/(2*10)").try_into().unwrap();
        assert_eq!(expression.eval(&mut None).unwrap(), 1515.0);
    }
}
