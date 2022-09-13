use crate::common::{MemoryWord, SolariumError};

use super::MemorySegment;

/// Provides a read-write memory segment type
pub struct ReadOnlySegment
{
    data: Vec<MemoryWord>,
}

impl ReadOnlySegment
{
    /// Defines a new memory segment with empty data, zero, in each memory location
    pub fn new(data: Vec<MemoryWord>) -> Self
    {
        // Create the memory segment
        return Self
        {
            data
        };
    }
}

impl MemorySegment for ReadOnlySegment
{
    /// Provides the word at the requested memory location
    fn get(&self, offset: usize) -> Result<MemoryWord, SolariumError>
    {
        return if self.within(offset)
        {
            Ok(self.data[offset])
        }
        else
        {
            Err(SolariumError::InvalidMemoryAccess(offset))
        };
    }

    /// Provides the word at the requested memory location without affecting the device state
    fn inspect(&self, offset: usize) -> Result<MemoryWord, SolariumError>
    {
        return self.get(offset);
    }

    /// Sets the word at the requested memory location with the given data
    /// Returns true if the value could be set; otherwise returns false
    fn set(&mut self, offset: usize, _: MemoryWord) -> Result<(), SolariumError>
    {
        return Err(SolariumError::InvalidMemoryWrite(offset));
    }

    /// Resets the memory segment
    fn reset(&mut self)
    {
        // Do Nothing
    }

    /// Provides the length of the memory segment
    fn len(&self) -> usize
    {
        return self.data.len();
    }

    /// Determines if the given memory index is within the memory segment
    fn within(&self, offset: usize) -> bool
    {
        return offset < self.len();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use super::super::MEM_MAX_SIZE;

    /// Test the initialization of the memory segment
    #[test]
    fn test_init()
    {
        // Define the base and the size
        let size = 1024;

        // Create the segment
        let mem = ReadOnlySegment::new(
            (0..size).map(|_| MemoryWord::new(0)).collect());

        // Ensure that the expected values match
        assert_eq!(mem.len(), size);

        // Iterate over memory items to check that the correct values are set
        for i in 0..MEM_MAX_SIZE
        {
            let is_within = i < size;
            assert_eq!(mem.within(i), is_within);

            match mem.get(i)
            {
                Ok(_) => assert!(is_within),
                Err(_) => assert!(!is_within)
            };
        }
    }

    /// Provide a default memory segment for testing
    fn get_default_test_segment() -> ReadOnlySegment
    {
        let size = 1024;

        return ReadOnlySegment::new(
            (0..size).map(|_| MemoryWord::new(0)).collect());
    }

    /// Test setting a memory location above the top address
    #[test]
    fn test_panic_set_above()
    {
        let mut mem = get_default_test_segment();
        let result = mem.set(
            mem.len(),
            MemoryWord::new(32));
        assert!(result.is_err());
    }

    /// Test getting a memory location above the top address
    #[test]
    fn test_panic_get_above()
    {
        let mem = get_default_test_segment();
        let result = mem.get(mem.len());
        assert!(result.is_err());
    }

    /// Test the initial base offset value
    #[test]
    fn test_offset_base()
    {
        let size = 1024;

        let mut mem = ReadOnlySegment::new(
            (0..size).map(|i| MemoryWord::new((i + 1) as u16)).collect());

        for i in 0..size
        {
            let success = mem.set(i, MemoryWord::new((i + 1) as u16));
            assert_eq!(success.is_err(), true);
        }

        for i in 0..MEM_MAX_SIZE
        {
            let should_be_within = i < size;
            assert_eq!(mem.within(i), should_be_within);

            let val = mem.get(i);
            assert_eq!(val.is_ok(), should_be_within);
            assert_eq!(val.is_err(), !should_be_within);

            if val.is_ok()
            {
                let mem_val = val.unwrap();
                assert_eq!(mem_val.get() as usize, i + 1);
            }
        }
    }
}
