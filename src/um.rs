use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::iter;
use std::io::{self, Read, Write};

const OP_FUNCTION_TABLE: [&'static Fn(&mut Um, usize, usize, usize, u32) -> UmResult<bool>; 14] =
[
    &Um::cmov,
    &Um::idx,
    &Um::amd,
    &Um::add,
    &Um::mul,
    &Um::div,
    &Um::nand,
    &Um::halt,
    &Um::alloc,
    &Um::abnd,
    &Um::outp,
    &Um::inp,
    &Um::ld,
    &Um::orth,
];

const OP_STRING_TABLE: [&'static str; 14] = 
[
    "CMOV",
    "IDX",
    "AMD",
    "ADD",
    "MUL",
    "DIV",
    "NAND",
    "HALT",
    "ALLOC",
    "ABND",
    "OUTP",
    "INP",
    "LD",
    "ORTH",
];

#[derive(Debug)]
pub enum UmError
{
    InvalidInstruction,
    InvalidArrayAccess,
    InvalidArrayAbandonment,
    DivideByZero,
    InvalidLoad,
    InvalidOutput,
    ExecutionFingerOutOfBounds,
}

pub type UmResult<T> = Result<T, UmError>;

impl fmt::Display for UmError
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
    {
        match *self
        {
            UmError::InvalidInstruction =>
                write!(fmt, "InvalidInstruction"),
            UmError::InvalidArrayAccess =>
                write!(fmt, "InvalidArrayAccess"),
            UmError::InvalidArrayAbandonment =>
                write!(fmt, "InvalidArrayAbandonment"),
            UmError::DivideByZero =>
                write!(fmt, "DivideByZero"),
            UmError::InvalidLoad =>
                write!(fmt, "InvalidLoad"),
            UmError::InvalidOutput =>
                write!(fmt, "InvalidOutput"),
            UmError::ExecutionFingerOutOfBounds =>
                write!(fmt, "ExecutionFingerOutOfBounds"),
        }
    }
}

impl Error for UmError
{
    fn description(&self) -> &str
    {
        match *self
        {
            UmError::InvalidInstruction =>
                "Execution Finger does not indicate a platter that describes a valid instruction",
            UmError::InvalidArrayAccess =>
                "Indexed or amended array that is not active",
            UmError::InvalidArrayAbandonment =>
                "Abandoned '0' array or an array that was not active",
            UmError::DivideByZero =>
                "Divided by zero",
            UmError::InvalidLoad =>
                "Loaded program from array that is not active",
            UmError::InvalidOutput =>
                "Output a value greater than 255",
            UmError::ExecutionFingerOutOfBounds =>
                "Execution Finger aims outside the bounds of the '0' array",
        }
    }

    fn cause(&self) -> Option<&Error>
    {
        match *self
        {
            UmError::InvalidInstruction =>
                None,
            UmError::InvalidArrayAccess =>
                None,
            UmError::InvalidArrayAbandonment =>
                None,
            UmError::DivideByZero =>
                None,
            UmError::InvalidLoad =>
                None,
            UmError::InvalidOutput =>
                None,
            UmError::ExecutionFingerOutOfBounds =>
                None,
        }
    }
}

#[derive(Debug)]
pub struct Um
{
    regs: [u32; 8],
    zero: Vec<u32>,
    arrays: HashMap<u32, Vec<u32>>,
    next_array: u32,
    execution_finger: usize,
}

impl Um
{
    pub fn new(program: Vec<u32>) -> Um
    {
        Um
        {
            regs: [0u32; 8],
            zero: program,
            arrays: HashMap::new(),
            next_array: 1,
            execution_finger: 0,
        }
    }

    pub fn next_op(&mut self) -> UmResult<bool>
    {
        let operator = try!(self.zero
                                .get(self.execution_finger)
                                .ok_or(UmError::ExecutionFingerOutOfBounds))
                                .clone();
        let opcode = ((operator & 0xF0000000) >> 28) as usize;
        let a = ((operator & 0b111_000_000) >> 6) as usize;
        let b = ((operator & 0b000_111_000) >> 3) as usize;
        let c = (operator & 0b000_000_111) as usize;

        /* For debug purposes */
/*        self.print_state();
 *        println!("");
 */

        self.execution_finger += 1;

        if opcode < OP_FUNCTION_TABLE.len()
        {
            OP_FUNCTION_TABLE[opcode](self, a, b, c, operator)
        }
        else
        {
            Err(UmError::InvalidInstruction)
        }

        /* I like this version of the last if statement, too */
/*        try!(OP_FUNCTION_TABLE.get(opcode)
 *                              .ok_or(UmError::InvalidInstruction))
 *                              (self, a, b, c, operator)
 */
    }

    pub fn print_state(&self)
    {
        println!("ef: 0x{:08X}", self.execution_finger);
        match self.zero.get(self.execution_finger)
        {
            Some(operator) =>
            {
                let opcode = ((operator & 0xF0000000) >> 28) as usize;
                let a = ((operator & 0b111_000_000) >> 6) as usize;
                let b = ((operator & 0b000_111_000) >> 3) as usize;
                let c = (operator & 0b000_000_111) as usize;

                if opcode < OP_STRING_TABLE.len()
                {
                    println!("OPCODE: {} 0x{:08X}", OP_STRING_TABLE[opcode], operator);
                }
                else
                {
                    println!("OPCODE: {} (INVALID)", OP_STRING_TABLE[opcode]);
                }
                println!("a: {}    b: {}    c: {}", a, b, c);
            },
            None =>
            {
                println!("Execution finger out of bounds");
            },
        }
        println!("0: 0x{:08X}", self.regs[0]);
        println!("1: 0x{:08X}", self.regs[1]);
        println!("2: 0x{:08X}", self.regs[2]);
        println!("3: 0x{:08X}", self.regs[3]);
        println!("4: 0x{:08X}", self.regs[4]);
        println!("5: 0x{:08X}", self.regs[5]);
        println!("6: 0x{:08X}", self.regs[6]);
        println!("7: 0x{:08X}", self.regs[7]);
    }

    fn cmov(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.regs[c] != 0
        {
            self.regs[a] = self.regs[b]
        }

        Ok(true)
    }

    fn idx(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.regs[b] == 0
        {
            self.regs[a] = try!(self.zero.get(self.regs[c] as usize).ok_or(UmError::InvalidArrayAccess)).clone();
        }
        else
        {
            self.regs[a] = try!(self.arrays
                                    .get(&self.regs[b])
                                    .and_then(|x| x.get(self.regs[c] as usize))
                                    .ok_or(UmError::InvalidArrayAccess)).clone();
        }

        Ok(true)
    }

    fn amd(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.regs[a] == 0
        {
            self.zero[self.regs[b] as usize] = self.regs[c];
        }
        else
        {
            let rega = self.regs[a];
            let regb = self.regs[b];
            *try!(self.arrays
                      .get_mut(&rega)
                      .and_then(|x| x.get_mut(regb as usize))
                      .ok_or(UmError::InvalidArrayAccess)) = self.regs[c];
        }

        Ok(true)
    }

    fn add(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        self.regs[a] = self.regs[b].wrapping_add(self.regs[c]);

        Ok(true)
    }

    fn mul(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        self.regs[a] = self.regs[b].wrapping_mul(self.regs[c]);

        Ok(true)
    }

    fn div(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.regs[c] != 0
        {
            /* Wrapping division is impossible, but we put it here for consistancy */
            self.regs[a] = self.regs[b].wrapping_div(self.regs[c]);

            Ok(true)
        }
        else
        {
            Err(UmError::DivideByZero)
        }
    }

    fn nand(&mut self, a: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        self.regs[a] = (self.regs[b] & self.regs[c]) ^ 0xFFFFFFFF;

        Ok(true)
    }

    fn halt(&mut self, _: usize, _: usize, _: usize, _: u32) -> UmResult<bool>
    {
        Ok(false)
    }

    fn alloc(&mut self, _: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        self.arrays.insert(self.next_array, iter::repeat(0u32).take(self.regs[c] as usize).collect());
        self.regs[b] = self.next_array;
        self.next_array += 1;

        Ok(true)
    }

    fn abnd(&mut self, _: usize, _: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.arrays.contains_key(&self.regs[c])
        {
            self.arrays.remove(&self.regs[c]);
            Ok(true)
        }
        else
        {
            Err(UmError::InvalidArrayAbandonment)
        }
    }

    fn outp(&mut self, _: usize, _: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.regs[c] < 0x100
        {
            print!("{}", self.regs[c] as u8 as char);
            /* TODO: Error correctly */
            io::stdout().flush().unwrap();

            Ok(true)
        }
        else
        {
            Err(UmError::InvalidOutput)
        }
    }

    fn inp(&mut self, _: usize, _: usize, c: usize, _: u32) -> UmResult<bool>
    {
        let mut buf = [0u8];

        /* If an error happens, don't do anything. */
        if let Ok(_) = io::stdin().read_exact(&mut buf)
        {
            self.regs[c] = buf[0] as u32;
        }

        Ok(true)
    }

    fn ld(&mut self, _: usize, b: usize, c: usize, _: u32) -> UmResult<bool>
    {
        if self.regs[b] != 0
        {
            self.zero = try!(self.arrays
                                 .get(&self.regs[b])
                                 .ok_or(UmError::InvalidLoad))
                                 .clone();
        }
        self.execution_finger = self.regs[c] as usize;

        Ok(true)
    }

    fn orth(&mut self, _: usize, _: usize, _: usize, instr: u32) -> UmResult<bool>
    {
        self.regs[((instr & 0x0E000000) >> 25) as usize] = instr & 0x01FFFFFF;

        Ok(true)
    }
}
