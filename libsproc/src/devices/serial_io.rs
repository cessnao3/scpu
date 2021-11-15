use std::cell::RefCell;
use std::collections::VecDeque;

use crate::memory::MemorySegment;
use crate::common::{MemoryWord, SolariumError};

pub struct SerialInputOutputDevice
{
    base_address: usize,
    input_queue: RefCell<VecDeque<char>>,
    output_queue: VecDeque<char>
}

impl SerialInputOutputDevice
{
    const DEVICE_MEM_SIZE: usize = 4;
    const OFFSET_INPUT_SIZE: usize = 0;
    const OFFSET_INPUT_GET: usize = 1;
    const OFFSET_OUTPUT_SIZE: usize = 2;
    const OFFSET_OUTPUT_SET: usize = 3;
}

impl MemorySegment for SerialInputOutputDevice
{
    /// Provides the word at the requested memory location
    fn get(&self, ind: usize) -> Result<MemoryWord, SolariumError>
    {
        if !self.within(ind)
        {
            return Err(SolariumError::InvalidMemoryAccess(ind));
        }

        let offset = ind - self.base_address;

        return match offset
        {
            Self::OFFSET_INPUT_SIZE => Ok(MemoryWord::new(self.input_queue.borrow().len() as u16)),
            Self::OFFSET_INPUT_GET =>
            {
                match self.input_queue.borrow_mut().pop_front()
                {
                    Some(v) => Ok(MemoryWord::new(v as u16)),
                    None => Ok(MemoryWord::new(0))
                }
            },
            Self::OFFSET_OUTPUT_SIZE => Ok(MemoryWord::new(self.output_queue.len() as u16)),
            Self::OFFSET_OUTPUT_SET => Ok(MemoryWord::new(0)),
            _ => Err(SolariumError::InvalidMemoryAccess(ind))
        };
    }

    /// Sets the word at the requested memory location with the given data
    /// Returns true if the value could be set; otherwise returns false
    fn set(&mut self, ind: usize, data: MemoryWord) -> Result<(), SolariumError>
    {
        if !self.within(ind)
        {
            return Err(SolariumError::InvalidMemoryAccess(ind))
        }

        let offset = ind - self.base_address;

        return match offset
        {
            Self::OFFSET_OUTPUT_SET =>
            {
                self.output_queue.push_back((data.get() as u8) as char);
                Ok(())
            },
            _ => Err(SolariumError::InvalidMemoryWrite(ind))
        }
    }

    /// Resets the memory segment
    fn reset(&mut self)
    {
        self.input_queue.borrow_mut().clear();
        self.output_queue.clear();
    }

    /// Provides the starting address of the memory segment
    fn start_address(&self) -> usize
    {
        return self.base_address;
    }

    /// Provides the length of the memory segment
    fn address_len(&self) -> usize
    {
        return Self::DEVICE_MEM_SIZE;
    }

    /// Determines if the given memory index is within the memory segment
    fn within(&self, ind: usize) -> bool
    {
        return ind >= self.base_address && ind < self.base_address + self.address_len();
    }
}
