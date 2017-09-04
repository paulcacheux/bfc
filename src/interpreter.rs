use std::io::{self, Read, Write, Bytes};

use utils;
use ir::Atom;

pub fn interpret<R: Read, W: Write>(ir: &Vec<Atom>, reader: R, writer: W) -> Result<(), InterpreterError> {
    let mut int = Interpreter::new(reader, writer);
    int.interpret(ir)
}

#[derive(Debug)]
pub enum InterpreterError {
    IndexOutOfBounds(usize),
    EmptyInput,
    IOError(io::Error),
}

const MEM_SIZE: usize = 30_000;

#[derive(Debug)]
struct Interpreter<R: Read, W: Write> {
    memory: Vec<u8>,
    ptr: usize,
    reader: Bytes<R>,
    writer: W,
}


impl<R: Read, W: Write> Interpreter<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Interpreter {
            memory: vec![0; MEM_SIZE],
            ptr: 0,
            reader: reader.bytes(),
            writer,
        }
    }

    fn set_memory_offset(&mut self, offset: isize, value: u8) -> Result<(), InterpreterError> {
        let ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
        if let Some(cell) = self.memory.get_mut(ptr) {
            *cell = value;
            Ok(())
        } else {
            Err(InterpreterError::IndexOutOfBounds(ptr))
        }
    }

    fn get_memory_offset(&self, offset: isize) -> Result<u8, InterpreterError> {
        let ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
        if let Some(cell) = self.memory.get(ptr) {
            Ok(*cell)
        } else {
            Err(InterpreterError::IndexOutOfBounds(ptr))
        }
    }

    fn interpret(&mut self, ir: &Vec<Atom>) -> Result<(), InterpreterError> {
        for atom in ir {
            match *atom {
                Atom::MovePtr(offset) => {
                    self.ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
                },
                Atom::SetValue(value, offset) => {
                    self.set_memory_offset(offset, value)?;
                },
                Atom::IncValue(inc, offset) => {
                    let old_value = self.get_memory_offset(offset)?;
                    let new_value = utils::offset_u8(old_value, inc);
                    self.set_memory_offset(offset, new_value)?;
                },
                Atom::Print(offset) => {
                    let to_write = self.get_memory_offset(offset)?;
                    self.writer
                        .write(&[to_write])
                        .unwrap();
                },
                Atom::Read(offset) => {
                    if let Some(next) = self.reader.next() {
                        match next {
                            Ok(c) => {
                                self.set_memory_offset(offset, c)?;
                            },
                            Err(err) => {
                                return Err(InterpreterError::IOError(err));
                            }, 
                        }
                    } else {
                        return Err(InterpreterError::EmptyInput);
                    }
                },
                Atom::Loop(ref sub) => {
                    while self.get_memory_offset(0)? != 0 {
                        self.interpret(sub)?;
                    }
                }
            }
        }
        Ok(())
    }
}