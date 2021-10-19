use crate::common::{MemoryWord, MemoryWordSigned, SolariumError};
use crate::memory::MemoryMap;

use super::registers::{Register, RegisterManager};

/// Defines the reset vector location
const VECTOR_RESET: MemoryWord = 0x0;

/// Defines the IRQ reset vector location
//const VECTOR_IRQ_SW: MemoryWord = 0x1;
//const VECTOR_HW_HJW: MemoryWord = 0x2;

// Define the stack pointer offset and allowed size
const STACK_POINTER_OFFSET: usize = 0x800;
const STACK_POINTER_MAX_SIZE: usize = 0x800;


/// Creates the Solarium CPU parameters
pub struct SolariumCPU
{
    pub memory_map: MemoryMap,
    pub registers: RegisterManager,
    allow_interrupts: bool
}

impl SolariumCPU
{
    /// Provide the number of registers
    pub const NUM_REGISTERS: usize = Register::NUM_REGISTERS;

    /// Creates a new CPU parameter
    pub fn new() -> SolariumCPU
    {
        // Create the CPU
        let mut cpu = SolariumCPU
        {
            memory_map: MemoryMap::new(),
            registers: RegisterManager::new(),
            allow_interrupts: true
        };

        // Initiate the reset
        cpu.reset();

        // Return the CPU
        return cpu;
    }

    /// Resets the CPU to a known state as a hard-reset
    pub fn reset(&mut self)
    {
        self.soft_reset();
        self.memory_map.reset();
        self.allow_interrupts = true;
    }

    /// Sofr-resets only the registers, but leaves the memory values intact
    pub fn soft_reset(&mut self)
    {
        self.registers.reset();
        self.registers.set(
            Register::ProgramCounter,
            VECTOR_RESET);
    }

    /// Obtains the current value of a register
    pub fn get_register_value(&self, index: usize) -> MemoryWord
    {
        return self.registers.get(Register::from_index(index));
    }

    /// Obtains the current program counter offset
    fn get_pc_offset(&self, reg: Register) -> MemoryWord
    {
        let pc = self.registers.get(Register::ProgramCounter);
        return (pc as i32 + self.registers.get(reg) as i32) as MemoryWord;
    }

    /// Increments the program counter by the specified amount
    fn increment_pc(&mut self, pc_incr: MemoryWordSigned)
    {
        let pc = self.registers.get(Register::ProgramCounter);
        let new_pc = (pc as i32 + pc_incr as i32) as MemoryWord;
        self.registers.set(
            Register::ProgramCounter,
            new_pc);
    }

    /// Obtains the current value of the stack pointer offset from the initial stack location
    fn get_sp_offset(&self) -> MemoryWord
    {
        return self.registers.get(Register::StackPointer);
    }

    /// Pushes a value onto the stack
    fn push_sp(&mut self, value: MemoryWord) -> Result<(), SolariumError>
    {
        let new_sp = self.get_sp_offset() + 1;
        if new_sp as usize > STACK_POINTER_OFFSET + STACK_POINTER_MAX_SIZE
        {
            return Err(SolariumError::StackOverflow);
        }
        else
        {
            self.registers.set(
                Register::StackPointer,
                new_sp);

            return self.memory_map.set(
                self.get_sp_address() - 1,
                value);
        }
    }

    /// Gets the current address just off the end of the stack
    fn get_sp_address(&self) -> usize
    {
        return STACK_POINTER_OFFSET + self.get_sp_offset() as usize;
    }

    /// Pops a value off of the stack and returns the result
    fn pop_sp(&mut self) -> Result<MemoryWord, SolariumError>
    {
        // Attempt to get the current location
        let ret_val = match self.peek_sp()
        {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        // Subtract one from the stack pointer
        self.registers.set(
            Register::StackPointer,
            self.get_sp_offset() - 1);

        // Return the result
        return Ok(ret_val);
    }

    /// Peeks at the value currently on the top of the stack
    fn peek_sp(&self) -> Result<MemoryWord, SolariumError>
    {
        if self.get_sp_offset() == 0
        {
            return Err(SolariumError::StackOverflow);
        }
        else
        {
            return self.memory_map.get(self.get_sp_address() - 1);
        }
    }

    /// Step the CPU
    pub fn step(&mut self) -> Result<(), SolariumError>
    {
        // Define the current memory word
        let pc = self.registers.get(Register::ProgramCounter);
        let inst = match self.memory_map.get(pc as usize)
        {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        // Define the PC increment
        let mut pc_incr = 1 as MemoryWordSigned;

        // Increment the PC
        self.registers.set(Register::ProgramCounter, pc);

        // Extract the different argument types
        let opcode = ((inst & 0xF000) >> 12) as u8;
        let arg0 = ((inst & 0x0F00) >> 8) as u8;
        let arg1 = ((inst & 0x00F0) >> 4) as u8;
        let arg2 = ((inst & 0x000F) >> 0) as u8;

        assert!(opcode & 0xF == opcode);
        assert!(arg0 & 0xF == arg0);
        assert!(arg1 & 0xF == arg1);
        assert!(arg2 & 0xF == arg2);

        // Define a function to combine two arguments into an item
        fn get_immediate_value_signed(
            arg_high: u8,
            arg_low: u8) -> MemoryWordSigned
        {
            assert!(arg_low & 0xF == arg_low);
            assert!(arg_high & 0xF == arg_high);
            return (((arg_high << 4) | arg_low) as i8) as MemoryWordSigned;
        }

        fn get_immediate_value_unsigned(
            arg_high: u8,
            arg_low: u8) -> MemoryWord
        {
            assert!(arg_low & 0xF == arg_low);
            assert!(arg_high & 0xF == arg_high);
            return (((arg_high << 4) | arg_low)) as MemoryWord;
        }

        // Switch based on opcode
        if opcode == 0x0
        {
            // Determine the number of arguments for the given opcode
            if arg0 != 0
            {
                assert!(opcode == 0);

                let reg_a = Register::from_index(arg2 as usize);
                let reg_b = Register::from_index(arg1 as usize);

                match arg0
                {
                    1 => // jmpri
                    {
                        pc_incr = get_immediate_value_signed(
                            arg1,
                            arg2);
                    },
                    2 => // ld
                    {
                        let reg_val = self.registers.get(reg_b);
                        let mem_val = match self.memory_map.get(reg_val as usize)
                        {
                            Ok(v) => v,
                            Err(e) => return Err(e)
                        };

                        self.registers.set(
                            reg_a,
                            mem_val);
                    },
                    3 => // sav
                    {
                        match self.memory_map.set(
                            self.registers.get(reg_a) as usize,
                            self.registers.get(reg_b))
                        {
                            Ok(()) => (),
                            Err(e) => return Err(e)
                        };
                    },
                    4 => // ldr
                    {
                        match self.memory_map.set(
                            self.registers.get(reg_a) as usize,
                            self.get_pc_offset(reg_b))
                        {
                            Ok(()) => (),
                            Err(e) => return Err(e)
                        };
                    },
                    5 => // savr
                    {
                        match self.memory_map.set(
                            self.get_pc_offset(reg_a) as usize,
                            self.registers.get(reg_b))
                        {
                            Ok(()) => (),
                            Err(e) => return Err(e)
                        };
                    },
                    6..=9 => // jz, jzr, jgz, jgzr
                    {
                        let cmp = self.registers.get(reg_b) as MemoryWordSigned;

                        let should_jump = ((arg0 == 6 || arg0 == 7) && cmp == 0) || ((arg0 == 8 || arg0 == 9) && cmp > 0);
                        let jump_relative = arg0 == 7 || arg0 == 9;

                        if should_jump
                        {
                            if jump_relative
                            {
                                pc_incr = self.registers.get(reg_a) as MemoryWordSigned;
                            }
                            else
                            {
                                self.registers.set(
                                    Register::ProgramCounter,
                                    self.registers.get(reg_a));
                            }
                        }
                    },
                    _ => // ERROR
                    {
                        return Err(SolariumError::InvalidInstruction(inst));
                    }
                }
            }
            else if arg1 != 0
            {
                assert!(opcode == 0);
                assert!(arg0 == 0);

                let dest_register = Register::from_index(arg2 as usize);

                match arg1
                {
                    1 => // jmp
                    {
                        self.registers.set(
                            Register::ProgramCounter,
                            self.registers.get(dest_register));
                        pc_incr = 0;
                    },
                    2 => // jmpr
                    {
                        pc_incr = self.registers.get(dest_register) as MemoryWordSigned;
                    },
                    3 => // push
                    {
                        match self.push_sp(self.registers.get(dest_register))
                        {
                            Ok(()) => (),
                            Err(e) => return Err(e)
                        };
                    },
                    4 => // popr
                    {
                        match self.pop_sp()
                        {
                            Ok(val) =>
                            {
                                self.registers.set(
                                    dest_register,
                                    val);
                            },
                            Err(e) => return Err(e)
                        };
                    },
                    5 => // call
                    {
                        // Push all the existing register values
                        for i in 0..Self::NUM_REGISTERS
                        {
                            match self.push_sp(self.registers.get(Register::GP(i)))
                            {
                                Ok(()) => (),
                                Err(e) => return Err(e)
                            };
                        }

                        // Move to the new location
                        let new_loc = self.registers.get(dest_register);
                        self.registers.set(
                            Register::ProgramCounter,
                            new_loc);

                        // Ensure that we run the first instruction at the new location
                        pc_incr = 0;
                    },
                    6 => // int
                    {
                        return Err(SolariumError::InterruptsNotSupported);
                    },
                    _ => // ERROR
                    {
                        return Err(SolariumError::InvalidInstruction(inst));
                    }
                };
            }
            else
            {
                assert!(opcode == 0);
                assert!(arg0 == 0);
                assert!(arg1 == 0);

                match arg2
                {
                    0 => // noop
                    {
                        ()
                    },
                    1 => // inton
                    {
                        self.allow_interrupts = true;
                    },
                    2 => // intoff
                    {
                        self.allow_interrupts = false;
                    },
                    3 => // reset
                    {
                        self.soft_reset();
                        pc_incr = 0;
                    },
                    4 => // pop
                    {
                        match self.pop_sp()
                        {
                            Ok(_) => (),
                            Err(e) => return Err(e)
                        };
                    },
                    5 => // ret
                    {
                        // Pop all register values
                        for i in 0..Self::NUM_REGISTERS
                        {
                            let mem_val = match self.pop_sp()
                            {
                                Ok(v) => v,
                                Err(e) => return Err(e)
                            };

                            self.registers.set(
                                Register::GP(Self::NUM_REGISTERS - 1 - i),
                                mem_val);
                        }
                    }
                    _ => // ERROR
                    {
                        return Err(SolariumError::InvalidInstruction(inst));
                    }
                };
            }
        }
        else if opcode == 1 || opcode == 2 // ldi, ldui
        {
            let immediate = match opcode
            {
                1 =>  get_immediate_value_signed(arg0, arg1) as MemoryWord,
                2 => get_immediate_value_unsigned(arg0, arg1),
                _ => return Err(SolariumError::InvalidInstruction(inst))
            };

            self.registers.set(
                Register::from_index(arg2 as usize),
                immediate);
        }
        else if opcode == 3 // ldir
        {
            let immediate = get_immediate_value_signed(arg0, arg1);

            assert!(pc as i32 + immediate as i32 >= 0);

            let mem_val = match self.memory_map.get((pc as i32 + immediate as i32) as usize)
            {
                Ok(v) => v,
                Err(e) => return Err(e)
            };

            self.registers.set(
                Register::from_index(arg2 as usize),
                mem_val);
        }
        else if opcode <= 13 // arithmetic
        {
            let val_a = self.registers.get(Register::from_index(arg1 as usize));
            let val_b = self.registers.get(Register::from_index(arg0 as usize));

            let result: MemoryWord;

            match opcode
            {
                4 => // add
                {
                    result = val_a + val_b;
                },
                5 => //sub
                {
                    result = val_a - val_b;
                },
                6 => // mul
                {
                    result = (val_a as MemoryWordSigned * val_b as MemoryWordSigned) as MemoryWord;
                },
                7 => // div
                {
                    if val_b == 0
                    {
                        return Err(SolariumError::DivideByZero);
                    }
                    result = (val_a as MemoryWordSigned / val_b as MemoryWordSigned) as MemoryWord;
                },
                8 => // mod
                {
                    if val_b == 0
                    {
                        return Err(SolariumError::ModByZero);
                    }
                    result = (val_a as MemoryWordSigned % val_b as MemoryWordSigned) as MemoryWord;
                },
                9 => // band
                {
                    result = val_a & val_b;
                }
                10 => // bor
                {
                    result = val_a | val_b;
                }
                11 => //bxor
                {
                    result = val_a ^ val_b;
                }
                12 => // bsftl
                {
                    result = val_a << val_b;
                },
                13 => // bsftr
                {
                    result = val_a >> val_b;
                },
                _ => // ERROR
                {
                    return Err(SolariumError::InvalidInstruction(inst));
                }
            }

            let reg_dest = Register::from_index(arg2 as usize);
            self.registers.set(
                reg_dest,
                result);
        }

        // Increment the program counter
        self.increment_pc(pc_incr);

        // Return success
        return Ok(());
    }
}
