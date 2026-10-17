//!
//! Add your code for Problems 1, 3, and 5 in this file.

use std::collections::VecDeque;
use std::fmt::{self, Display};

/// The abstract syntax of a well-formed Brainfuck program.
///
/// A loop contains another complete `BFProg`, so an unmatched `[` or `]`
/// cannot be represented by this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BFProg {
    IncData,
    DecData,
    IncByte,
    DecByte,
    Output,
    Input,
    Loop(Box<BFProg>),
    Sequence(Vec<BFProg>),
}

impl Display for BFProg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BFProg::IncData => write!(f, ">"),
            BFProg::DecData => write!(f, "<"),
            BFProg::IncByte => write!(f, "+"),
            BFProg::DecByte => write!(f, "-"),
            BFProg::Output => write!(f, "."),
            BFProg::Input => write!(f, ","),
            BFProg::Loop(body) => write!(f, "[{body}]"),
            BFProg::Sequence(programs) => {
                for program in programs {
                    write!(f, "{program}")?;
                }
                Ok(())
            }
        }
    }
}

/// The state needed to execute a Brainfuck program.
pub struct BFInterpreter {
    tape: Vec<u8>,
    data_pointer: usize,
    input: VecDeque<u8>,
    output: Vec<u8>,
}

impl BFInterpreter {
    pub fn new() -> Self {
        Self {
            tape: vec![0],
            data_pointer: 0,
            input: VecDeque::new(),
            output: Vec::new(),
        }
    }

    /// Replaces the interpreter's unread input bytes.
    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input.into();
    }

    /// Returns the bytes emitted by `.` commands so far.
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    pub fn interpret(&mut self, prog: &BFProg) {
        match prog {
            BFProg::IncData => {
                self.data_pointer += 1;
                if self.data_pointer == self.tape.len() {
                    self.tape.push(0);
                }
            }
            BFProg::DecData => {
                assert!(
                    self.data_pointer > 0,
                    "the BF data pointer cannot move left from position 0"
                );
                self.data_pointer -= 1;
            }
            BFProg::IncByte => {
                self.tape[self.data_pointer] = self.tape[self.data_pointer].wrapping_add(1);
            }
            BFProg::DecByte => {
                self.tape[self.data_pointer] = self.tape[self.data_pointer].wrapping_sub(1);
            }
            BFProg::Output => self.output.push(self.tape[self.data_pointer]),
            BFProg::Input => {
                self.tape[self.data_pointer] = self
                    .input
                    .pop_front()
                    .expect("the BF program requested a byte after its input was exhausted");
            }
            BFProg::Loop(body) => {
                while self.tape[self.data_pointer] != 0 {
                    self.interpret(body);
                }
            }
            BFProg::Sequence(programs) => {
                for program in programs {
                    self.interpret(program);
                }
            }
        }
    }
}

impl Default for BFInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a string containing exactly the eight BF command characters.
///
/// The stack stores the sequence currently being built at each bracket
/// nesting depth. A closing bracket completes the innermost loop.
pub fn parse_bf(bf_str: &str) -> Option<BFProg> {
    let mut sequence_stack: Vec<Vec<BFProg>> = vec![Vec::new()];

    for character in bf_str.chars() {
        let command = match character {
            '>' => BFProg::IncData,
            '<' => BFProg::DecData,
            '+' => BFProg::IncByte,
            '-' => BFProg::DecByte,
            '.' => BFProg::Output,
            ',' => BFProg::Input,
            '[' => {
                sequence_stack.push(Vec::new());
                continue;
            }
            ']' => {
                // The bottom stack entry is the top-level program, so it
                // cannot be closed by a `]`.
                if sequence_stack.len() == 1 {
                    return None;
                }

                let loop_body = sequence_stack.pop()?;
                let enclosing_sequence = sequence_stack.last_mut()?;
                enclosing_sequence.push(BFProg::Loop(Box::new(BFProg::Sequence(loop_body))));
                continue;
            }
            _ => return None,
        };

        sequence_stack.last_mut()?.push(command);
    }

    // More than one entry means that at least one `[` was never closed.
    if sequence_stack.len() != 1 {
        return None;
    }

    sequence_stack.pop().map(BFProg::Sequence)
}

type TmTransition = Option<(u8, u8, usize)>;

// Tom Smeding's BF Turing-machine simulator is available at
// https://git.tomsmeding.com/bfturing/about/. The input-loading phase and the
// simulation loop are separated here so the fixed Goldbach machine can be
// partially evaluated into one closed BF program with no external input.
const BF_TURING_INPUT_LOADER: &str = "+>>>>>>>>>>>+[->>>>,[-<+<+>>]<<[->>+<<]+>+[[-]<->>>>>>>>+<<<<<,[->+>+<<]>>[-<<+>>]+<--------------------------------[>-<[-]]>[<<[-]>>[-]]<,>,[->+>+<<]>>[-<<+>>]+<--------------------------------[>-<[-]]>[<<[-]>>[-]]<,>++++++[-<---------->]<-<<<<<]<[>>[-]<<[-]]>>>>>>>>]<<<<<<<<<<>+++++++[-<++++++++++++>]<-<+>>>>>>+<<<<<<";

const BF_TURING_MAIN_LOOP: &str = ">>++++++++[-<--------->]<[>++++++++[-<+++++++++>]<<>>+[->>>>>+<[->->+<<]>>[-<<+>>]<]<<>>>>[-<+<+>>]<[->+<]+[-<<<<<+<<<[->>+>-<<<]>>[-<<+>>]>>>>>[-<<<<<+>>>>>]<<<<]<<[-<<<<<<<<<<+>>>>>>>>>>>>+<<]>>[-<<+>>]<[-<<<<<<<<<<+>>>>>>>>>>]>+[[-]<<<<<<<<<<[-]>>[-<+<+>>]<<[->>+<<]<<[-<+>>>+<<]<[->+<]>>>[->-<]>[<+>[-]]<[-<<<<<<<<<<+>>>>>>>>>>]>>>[-<<+<+>>>]<<<[->>>+<<<]<[-<<+>>>+<]<<[->>+<<]>>>[->-<]>[<+>[-]]<[-<<<<<<<<<+>>>>>>>>>]<<<<<<<<<<[->>>>>>>>>>>+<<<<<<<<<<<]>[->>>>>>>>>>+<<<<<<<<<<]>>>>>>>>>>[[-]<+>]<[-<<<+<<<<<<<+>>>>>>>>>>]<<<[->>>+<<<]<<<<<<<<<<[>>>>>>>>>>>>>[-]>+<<<<<<<<<<<<<<-]>>>>>>>>>>>>>>[-<<<<<<<<<<<<<<+>>>>>>>>>>>>>>]<<<[-<<<<<<<<<<+>>>>>>>>>>]>[-<<<<<<<<<<+>>>>>>>>>>]>]<<<<<<<<<<[]<<[-]>[-]>>>>>>>>>>>>>>>[-<<<<<+<+>>>>>>]<<<<<<[->>>>>>+<<<<<<]>>>>>>>[-<<<<<+<<+>>>>>>>]<<<<<<<[->>>>>>>+<<<<<<<]>>>>>>>>[-<<<<<+<<<+>>>>>>>>]<<<<<<<<[->>>>>>>>+<<<<<<<<]>>>>>>>>>>>+[<<[->>->+<<<]>>>[-<<<+>>>]<[->>>>>>>>>>+<<<<<<<<<<]<<<<<<<<<<[->>>>>>>>>>+<<<<<<<<<<]>[->>>>>>>>>>+<<<<<<<<<<]>[->>>>>>>>>>+<<<<<<<<<<]>>>>>>>>>>>>>>>>>>]<<<<<<<<<<<[-]>[-<+>]>[-<+>]>[-<+>]>>>+[<[->->+<<]>>[-<<+>>]<[->>>>>+<<<<<]<<<<<[->>>>>+<<<<<]>[->>>>>+<<<<<]>>>>>>>>>]<<<<<>>[-]<<[->>+<<]>[-<+<<+>>>]<<<[->>>+<<<]>>[-[<<+>>[-]]]>[+[<+>[-]]]<<<[<<<<<[]>>>>>>[-<<<<<+>>>>>]<[-]]>>[<[->>>>>+<<<<<]>[-]]+[-<<<<<+<<[->>->+<<<]>>>[-<<<+>>>]<]<<>>++++++++[-<--------->]<]";

/// Returns the encoded transition table for a 25-state Turing machine that
/// halts exactly when it finds a counterexample to Goldbach's conjecture.
///
/// The transition table is the `edited` machine from Google DeepMind's
/// formally verified `Cryptids/Goldbach21.lean`. State 2 is renamed `S`
/// because the BF simulator uses `S` as its initial state. Zero is encoded as
/// the blank-space symbol and one as the character `1`.
fn goldbach_machine_description() -> Vec<u8> {
    const ROWS: [[TmTransition; 2]; 25] = [
        [Some((0, b'>', 11)), Some((1, b'>', 3))],
        [Some((1, b'>', 18)), Some((1, b'>', 1))],
        [Some((1, b'<', 11)), Some((0, b'>', 5))],
        [None, Some((1, b'>', 4))],
        [Some((0, b'<', 16)), Some((1, b'>', 6))],
        [Some((1, b'<', 18)), Some((1, b'>', 5))],
        [Some((0, b'>', 7)), Some((0, b'<', 7))],
        [Some((0, b'<', 14)), Some((1, b'<', 22))],
        [Some((0, b'>', 9)), Some((0, b'<', 8))],
        [Some((1, b'>', 0)), Some((0, b'<', 9))],
        [Some((0, b'>', 11)), Some((0, b'>', 18))],
        [Some((1, b'>', 11)), Some((0, b'>', 12))],
        [Some((1, b'>', 13)), Some((1, b'>', 12))],
        [Some((0, b'<', 14)), Some((0, b'<', 20))],
        [Some((1, b'<', 15)), Some((1, b'<', 14))],
        [Some((1, b'<', 16)), Some((0, b'<', 23))],
        [Some((1, b'<', 18)), Some((1, b'<', 16))],
        [Some((0, b'>', 7)), Some((1, b'<', 18))],
        [Some((0, b'>', 2)), Some((0, b'<', 19))],
        [Some((0, b'>', 0)), Some((1, b'<', 19))],
        [Some((1, b'<', 21)), Some((1, b'<', 20))],
        [Some((0, b'<', 22)), Some((0, b'<', 9))],
        [Some((0, b'>', 0)), Some((0, b'<', 22))],
        [Some((1, b'>', 24)), Some((1, b'<', 23))],
        [Some((1, b'>', 3)), Some((1, b'>', 24))],
    ];
    const STATE_NAMES: &[u8; 25] = b"ABSCDEFGIJKLMNOPQRTUVWXYZ";

    let mut input = Vec::with_capacity(251);
    for (state, transitions) in ROWS.iter().enumerate() {
        for (symbol, transition) in transitions.iter().enumerate() {
            let read_symbol = if symbol == 0 { b' ' } else { b'1' };
            input.extend_from_slice(&[STATE_NAMES[state], read_symbol]);

            match transition {
                Some((write, direction, next_state)) => input.extend_from_slice(&[
                    STATE_NAMES[*next_state],
                    if *write == 0 { b' ' } else { b'1' },
                    *direction,
                ]),
                None => input.extend_from_slice(&[b'H', read_symbol, b'=']),
            }
        }
    }

    // The simulator represents end-of-input by the wrapping byte value -1.
    input.push(u8::MAX);
    input
}

/// Constructs a closed BF program that halts iff Goldbach's conjecture is
/// false. Constructing this value does not run the search.
pub fn goldbach_program() -> BFProg {
    // First, use the BF interpreter to partially evaluate the simulator's
    // finite input-loading phase on the fixed transition table.
    let loader = parse_bf(BF_TURING_INPUT_LOADER).expect("the BF input loader should parse");
    let mut loader_interpreter = BFInterpreter::new();
    loader_interpreter.set_input(goldbach_machine_description());
    loader_interpreter.interpret(&loader);

    // Turn the resulting finite tape into BF initialization commands. Values
    // above 128 are cheaper to create by decrementing from zero.
    let used_tape_length = loader_interpreter
        .tape
        .iter()
        .rposition(|value| *value != 0)
        .map_or(1, |index| index + 1);
    let mut commands = Vec::new();
    for (index, value) in loader_interpreter.tape[..used_tape_length]
        .iter()
        .copied()
        .enumerate()
    {
        if value <= 128 {
            commands.extend(std::iter::repeat_n(BFProg::IncByte, usize::from(value)));
        } else {
            commands.extend(std::iter::repeat_n(
                BFProg::DecByte,
                256 - usize::from(value),
            ));
        }
        if index + 1 < used_tape_length {
            commands.push(BFProg::IncData);
        }
    }

    for _ in loader_interpreter.data_pointer..used_tape_length - 1 {
        commands.push(BFProg::DecData);
    }

    let simulation_loop =
        parse_bf(BF_TURING_MAIN_LOOP).expect("the BF Turing-machine loop should parse");
    commands.push(simulation_loop);
    BFProg::Sequence(commands)
}

fn main() {
    println!();
    println!("ECS 240 HW1");
    println!("-----------");

    let example_1 = BFProg::Sequence(vec![
        BFProg::Loop(Box::new(BFProg::Sequence(vec![
            BFProg::IncData,
            BFProg::DecData,
            BFProg::IncByte,
            BFProg::DecByte,
            BFProg::Loop(Box::new(BFProg::Sequence(vec![BFProg::Output]))),
        ]))),
        BFProg::Loop(Box::new(BFProg::Sequence(vec![BFProg::IncByte]))),
    ]);
    let example_2 = BFProg::Sequence(vec![BFProg::Input, BFProg::IncByte, BFProg::Output]);

    println!("Example BF program 1: {example_1}");
    println!("Example BF program 2: {example_2}");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bf_syntax_valid_program() {
        let prog = BFProg::Sequence(vec![
            BFProg::Loop(Box::new(BFProg::Sequence(vec![
                BFProg::IncData,
                BFProg::DecData,
                BFProg::IncByte,
                BFProg::DecByte,
                BFProg::Loop(Box::new(BFProg::Sequence(vec![BFProg::Output]))),
            ]))),
            BFProg::Loop(Box::new(BFProg::Sequence(vec![BFProg::IncByte]))),
        ]);

        assert_eq!(prog.to_string(), "[><+-[.]][+]".to_string());
    }

    #[test]
    fn test_bf_syntax_invalid_program() {
        // There is no BFProg variant for a standalone `]`, so the exact
        // invalid string `]><+[]` cannot be constructed.
        let prog = BFProg::Sequence(vec![
            BFProg::IncData,
            BFProg::DecData,
            BFProg::IncByte,
            BFProg::Loop(Box::new(BFProg::Sequence(vec![]))),
        ]);

        assert!(prog.to_string() != "]><+[]".to_string());
    }

    #[test]
    fn test_bf_interpreter_1() {
        // Read 5 and 6, add the first cell into the second, and output 11.
        let add_two_inputs = BFProg::Sequence(vec![
            BFProg::Input,
            BFProg::IncData,
            BFProg::Input,
            BFProg::DecData,
            BFProg::Loop(Box::new(BFProg::Sequence(vec![
                BFProg::DecByte,
                BFProg::IncData,
                BFProg::IncByte,
                BFProg::DecData,
            ]))),
            BFProg::IncData,
            BFProg::Output,
        ]);
        let mut interpreter = BFInterpreter::new();
        interpreter.set_input(vec![5, 6]);

        interpreter.interpret(&add_two_inputs);

        assert_eq!(interpreter.output(), &[11]);
    }

    #[test]
    fn test_bf_interpreter_2() {
        // BF bytes use arithmetic modulo 256: 0 - 1 = 255 and 255 + 1 = 0.
        let wrapping_arithmetic = BFProg::Sequence(vec![
            BFProg::DecByte,
            BFProg::Output,
            BFProg::IncByte,
            BFProg::Output,
        ]);
        let mut interpreter = BFInterpreter::new();

        interpreter.interpret(&wrapping_arithmetic);

        assert_eq!(interpreter.output(), &[255, 0]);
    }

    #[test]
    fn test_parser_1() {
        let parsed = parse_bf("[><+-[.]][+]").expect("the valid BF program should parse");

        assert_eq!(parsed.to_string(), "[><+-[.]][+]");
    }

    #[test]
    fn test_parser_2() {
        assert_eq!(parse_bf("]><+[]"), None);
        assert_eq!(parse_bf("[+"), None);
        assert_eq!(parse_bf("+x-"), None);
    }

    #[test]
    fn test_parser_3() {
        // Display omits Sequence nodes. Therefore, parsing the display of a
        // single command produces an equivalent top-level Sequence rather
        // than the exact enum value that was displayed.
        let original = BFProg::IncByte;
        let displayed = original.to_string();
        let reparsed = parse_bf(&displayed).expect("the displayed program should parse");

        assert_eq!(displayed, "+");
        assert_ne!(reparsed, original);
        assert_eq!(reparsed.to_string(), displayed);
    }

    #[test]
    fn test_goldbach_program_is_closed_bf() {
        let source = goldbach_program().to_string();

        assert!(!source.contains(','));
        assert!(parse_bf(&source).is_some());
    }
}
