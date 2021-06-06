use crate::memory::{MemoryWord, MemoryWordSigned};
use crate::memory::memory_map::MemoryMap;

use super::location::Location;
use super::registers::{Register, RegisterManager};

/// Defines the reset vector location
const VECTOR_RESET: MemoryWord = 0x400;

/// Defines the IRQ reset vector location
//const VECTOR_IRQ: MemoryWord = 0x401;

/// Creates the Solarium CPU parameters
pub struct SolariumCPU
{
    memory_map: MemoryMap,
    registers: RegisterManager
}

impl SolariumCPU
{
    /// Creates a new CPU parameter
    pub fn new() -> SolariumCPU
    {
        // Create the CPU
        let mut cpu = SolariumCPU
        {
            memory_map: MemoryMap::new(),
            registers: RegisterManager::new()
        };

        // Initiate the reset
        cpu.reset();

        // Return the CPU
        return cpu;
    }

    /// Resets the CPU to a known state as a hard-reset
    pub fn reset(&mut self)
    {
        self.memory_map.reset();
        self.registers.reset();
        self.registers.set(&Register::ProgramCounter, VECTOR_RESET);
    }

    fn get_location_value(&self, loc: &Location) -> Result<MemoryWord, String>
    {
        return match loc
        {
            Location::Register(ind) => Ok(self.registers.get(&Register::from_index(*ind))),
            Location::AddressOf(ind) => Ok(self.memory_map.get(self.registers.get(&Register::from_index(*ind)))),
            Location::Value(v) => Ok(*v as MemoryWord)
        }
    }

    fn set_location_value(&mut self, loc: &Location, val: MemoryWord) -> Result<bool, String>
    {
        return match loc
        {
            Location::Register(ind) => Ok(self.registers.set(&Register::from_index(*ind), val)),
            Location::AddressOf(ind) => Ok(self.memory_map.set(self.registers.get(&Register::from_index(*ind)), val)),
            Location::Value(_) => Err("cannot set an immediate value".to_string())
        }
    }

    /// Step the CPU
    pub fn step(&mut self) -> bool
    {
        // Define the current memory word
        let pc = self.registers.get(&Register::ProgramCounter);
        let inst = self.memory_map.get(pc);

        // Increment the PC
        self.registers.set(&Register::ProgramCounter, pc);

        // Extract the different argument types
        let opcode = (inst & 0xFF) as u8;
        let arg0 = ((inst & 0xFF00) >> 8) as u8;
        let arg1 = ((inst & 0xFF0000) >> 16) as u8;
        let arg2 = ((inst & 0xFF000000) >> 24) as u8;

        // Match opcode parameters
        if opcode == 0x0 // NOOP
        {
            // NOOP
        }
        else if opcode == 0x30 // COPY
        {
            // Determine the source and controller
            let src_loc = match Location::from_arg(arg0)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };
            let dst_loc = match Location::from_arg(arg1)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            // Copy from one location to the other
            let src_val = match self.get_location_value(&src_loc)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            match self.set_location_value(&dst_loc, src_val)
            {
                Ok(b) => if !b { println!("Unable to set memory location with given value {0:}", src_val); },
                Err(e) => panic!(e)
            }
        }
        else if opcode >= 0x40 && opcode < 0x50 // Arithmetic
        {
            // Obtain the locations and values for each of the three arguments
            let loc_a = match Location::from_arg(arg0)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            let loc_b = match Location::from_arg(arg1)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            let loc_c = match Location::from_arg(arg2)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            let val_a = match self.get_location_value(&loc_a)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            let val_c = match self.get_location_value(&loc_b)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            // Determine the resulting values
            let result = match opcode - 0x40
            {
                0 => val_a + val_b,
                1 => val_a - val_b,
                2 => val_a * val_b,
                3 => val_a / val_b,
                4 => val_a % val_b,
                _ => panic!("unknown opcode provided")
            } as MemoryWord;

            // Store the resulting value
            let result = match self.set_location_value(&loc_c, result)
            {
                Ok(v) => v,
                Err(e) => panic!(e)
            };

            if !result
            {
                panic!("unable to set memory location for instruction {0:}", inst);
            }
        }
        else if opcode >= 0x20 && opcode < 0x30 // Jump
        {
            // Extract arguments
            let is_immediate_relative = arg0 & 0x1 > 0;
            let reg_1_val = self.registers.get(&Register::GP((arg1 & 0xF) as usize)) as MemoryWordSigned;
            let reg_2_val = self.registers.get(&Register::GP(((arg2 & 0xF0) >> 4) as usize)) as MemoryWordSigned;

            // Determine the new program counter
            let new_pc;
            if is_immediate_relative
            {
                new_pc = (pc as MemoryWordSigned + (arg2 as i8) as MemoryWordSigned) as MemoryWord;
            }
            else
            {
                new_pc = self.registers.get(&Register::GP(arg2 as usize));
            }

            // Determine the jump type
            let jmp_type = opcode - 0x20;

            // Iterate based on jump types to determine if we will jump
            let will_jump = match jmp_type
            {
                0 => true,
                1 => reg_1_val != reg_2_val,
                2 => reg_1_val == reg_2_val,
                3 => reg_1_val < 0,
                4 => reg_1_val > 0,
                5 => reg_1_val >= reg_2_val,
                6 => reg_1_val > reg_2_val,
                7 => reg_1_val < reg_2_val,
                8 => reg_1_val <= reg_2_val,
                _ => panic!("unknown jump command provided")
            };

            // Perform the jump if needed
            if will_jump
            {
                self.registers.set(&Register::ProgramCounter, new_pc);
            }
        }
        else
        {
            panic!("unknown instruction provided");
        }

        // Return success
        return true;
    }
}
