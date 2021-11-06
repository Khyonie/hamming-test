#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#[cfg(test)]

use std::u16;

fn main()
{

}

struct HammingDataBlock
{
    hammingdata: u16,
    priorhammingdata: u16,
    original: u16
}

impl HammingDataBlock
{
    fn new(data: u16) -> Self
    {
        // If the data is greater than 11 bit, panic
        if data >> 11 > 0
        {
            panic!("Expected data to be an 11 bit integer.");
        }

        let mut buffer: u16 = data.clone() << 5;
        // Work a little magic to get the data in the correct spots
        // n P P #
        // P # # #
        // P # # #
        // # # # #
        //
        // ----_-###_####_#### - Given
        // ####_####_###-_---- - << 5
        // ####_###-_####_---- - (data & 0xFE00) | ((data & !0xFE00) >> 1)
        // ####_###-_###-_#--- - (data % FFE0) | ((data) % !0xFFE0) >> 1)

        // ####_###P_###P_#PPn

        buffer = (buffer & 0xFE00) | ((buffer & !0xFE00) >> 1);
        buffer = (buffer & 0xFFE0) | ((buffer & !0xFFE0) >> 1);

        // Perform earmarking

        // Bit 1 (Controls all odd bits)
        let mut count: u8 = 0;
        
        for i in 0..=15
        {
            if i % 2 == 0 // Only parse odd bits
            {
                continue;
            }

            if get_bit_at_position(&buffer, i)
            {
                count += 1;
            }
        }

        if count % 2 != 0 // Flip the first parity bit to ensure parity
        {
            buffer ^= 0b0000_0000_0000_0010;
        }

        // Bit 2 (Controls all "right-handed" bits)
        {
            count = 0;
            let right_handed: [u8; 8] = [2, 3, 6, 7, 10, 11, 14, 15];
            for i in right_handed.iter()
            {
                if get_bit_at_position(&buffer, *i)
                {
                    count += 1;
                }
            }
            if count % 2 != 0 // Flip the second parity bit to ensure parity
            {
                buffer ^= 0b0000_0000_0000_0100;
            }
        }

        // Bit 4 (Controls all "second and fourth row" bits)
        {
            let second_and_fourth_row: [u8; 8] = [4, 5, 6, 7, 12, 13, 14, 15];
            count = 0;
            for i in second_and_fourth_row.iter()
            {
                if get_bit_at_position(&buffer, *i)
                {
                    count += 1;
                }
            }

            if count % 2 != 0 // Flip the third parity bit to ensure parity
            {
                buffer ^= 0b0000_0000_0001_0000;
            }
        }

        // Bit 8 (Controls all upper bits)
        count = 0;
        for i in 8..=15
        {
            if get_bit_at_position(&buffer, i)
            {
                count += 1;
            }
        }

        if count % 2 != 0 // Flip the final parity bit to ensure parity
        {
            buffer ^= 0b0000_0001_0000_0000;
        }

        // Finally, check if the number is odd, if so, flip the 0th bit
        // This allows us to detect if two or more errors have occurred
        if buffer % 2 != 0 
        {
            buffer ^= 0b0000_0000_0000_0001;
        }

        HammingDataBlock
        {
            hammingdata: buffer,
            priorhammingdata: buffer,
            original: data
        }
    }

    fn get_value(&self) -> u16
    {
        self.hammingdata
    }

    fn get_orignal_value(&self) -> u16
    {
        self.original
    }

    fn get_prior_value(&self) -> u16
    {
        self.priorhammingdata
    }

    fn convert_to_original(&self) -> u16
    {
        let mut copy = self.hammingdata;

                                                  // Given: ####_###P_###P_#PPn
        copy >>= 3;                                      // ---#_####_##P#_##P#
        copy = (copy & 0x0001) | ((copy & 0x1FFC) >> 1); // ----_####_####_P###
        copy = (copy & 0x000F) | ((copy & 0x0FF0) >> 1); // ----_-###_####_####

        copy
    }

    fn print(&self)
    {
        let line1: u8 =  (self.hammingdata & 0b0000_0000_0000_1111) as u8;
        let line2: u8 = ((self.hammingdata & 0b0000_0000_1111_0000) >> 4) as u8;
        let line3: u8 = ((self.hammingdata & 0b0000_1111_0000_0000) >> 8) as u8;
        let line4: u8 = ((self.hammingdata & 0b1111_0000_0000_0000) >> 12) as u8;

        println!("{:0>4b}\n{:0>4b}\n{:0>4b}\n{:0>4b}", line1, line2, line3, line4);
    }

    fn fix(&mut self)
    {
        const test_2_values: [u8; 8] = [2, 3, 6, 7, 10, 11, 14, 15];
        const test_2_negatives: [u8; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
        const test_3_values: [u8; 8] = [4, 5, 6, 7, 12, 13, 14, 15];
        const test_3_negatives: [u8; 8] = [0, 1, 2, 3, 8, 9, 10, 11];

        let mut count: u8 = 0;

        for i in 0..=15
        {
            if get_bit_at_position(&self.hammingdata, i)
            {
                count += 1;
            }
        }

        if count % 2 == 0 
        {
            //return;
        }

        count = 0;

        let mut target: [u8; 16] = [1; 16];

        // Test 1
        for i in 0..=15
        {
            if i % 2 == 0 
            {
                continue;
            }

            if get_bit_at_position(&self.hammingdata, i)
            {
                count += 1;
            }
        }

        if count % 2 != 0 // Target is odd
        {
            // println!("Target is odd,");

            for i in 0..=15
            {
                if i % 2 != 0 
                {
                    continue;
                }

                target[i as usize] = 0;
            }
        } else { // Target is even
            // println!("Target is even,");

            for i in 0..=15
            {
                if i % 2 == 0 
                {
                    continue;
                }

                target[i as usize] = 0;
            }
        }

        // print_len16_u8_array(&target);

        // Test 2
        count = 0;
        for i in test_2_values.iter()
        {
            if get_bit_at_position(&self.hammingdata, *i)
            {
                count += 1;
            }
        }

        if count % 2 != 0
        {
            // println!("Target fell under second test,");

            for i in test_2_negatives.iter()
            {
                target[*i as usize] = 0;
            }
        } else {
            // println!("Target did not fall under second test,");

            for i in test_2_values.iter()
            {
                target[*i as usize] = 0;
            }
        }

        // print_len16_u8_array(&target);

        // Test 3
        count = 0;
        for i in test_3_values.iter()
        {
            if get_bit_at_position(&self.hammingdata, *i)
            {
                count += 1;
            }
        }

        if count % 2 != 0
        {
            // println!("Target fell under third test,");

            for i in test_3_negatives.iter()
            {
                target[*i as usize] = 0;
            }
        } else {
            // println!("Target did not fall under third test,");

            for i in test_3_values.iter()
            {
                target[*i as usize] = 0;
            }
        }

        // print_len16_u8_array(&target);

        count = 0;
        // Test 4
        for i in 8..=15
        {
            if get_bit_at_position(&self.hammingdata, i)
            {
                count += 1;
            }
        }

        if count % 2 != 0 // Target is an upper bit
        {
            // println!("Target is an upper bit,");

            for i in 0..=7
            {
                target[i as usize] = 0;
            }
        } else {
            // println!("Target is a lower bit,");

            for i in 8..=15
            {
                target[i as usize] = 0;
            }
        } // Target is a lower bit

        // print_len16_u8_array(&target);

        // Perform fix
        for i in 0..=15
        {
            if target[i as usize] == 0
            {
                continue;
            }

            self.hammingdata ^= 0b0000_0000_0000_0001 << i;
            // println!("A bitflip at position {} was found and fixed.", i);
        }
    }

    fn zap_bit(&mut self, position: u8)
    {
        self.hammingdata ^= 0b0000_0000_0000_0001 << position
    }
}

pub fn get_bit_at_position(data: &u16, position: u8) -> bool
{
    data & (0b0000_0000_0000_0001 << position) > 0
}

/// Tests all values between 0 to 2^11 exclusive. Fails are tracked.
/// All bit positions will be zapped, one at a time. If the fixed value is not equal to the original, the bit position and original value are logged.
#[test]
pub fn run_test()
{
    let mut hammingvalue: HammingDataBlock;
    let mut fails: Vec<[u16; 2]> = Vec::new();

    for value in 0..2u16.pow(11)
    {
        for position in 0..=15
        {
            hammingvalue = HammingDataBlock::new(value);
            hammingvalue.zap_bit(position);

            hammingvalue.fix();

            if hammingvalue.get_value() != hammingvalue.get_prior_value()
            {
                fails.push([value, position as u16]);
                println!("Fail with value: {} @ position {}", value, position);
            }
        }
    }

    println!("Total fails: {}/32768 ({}%)", fails.len(), 100 * (fails.len()/32768));

    let mut min: u16 = 0;
    let mut positional_stats: [u32; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    if fails.len() > 0
    {
        println!("Test failed. Fail statistics:");

        for stat in fails.iter()
        {
            if stat[0] > min
            {
                min = stat[0]
            }

            positional_stats[stat[1] as usize] += 1;
        }

        let mut max_pos: u8 = 0;

        for i in 0..=15
        {
            if positional_stats[i] > positional_stats[max_pos as usize]
            {
                max_pos = i as u8;
            }
        }

        println!("Highest failed number: {}", min);
        println!("Most failed position: {}", max_pos);
    }

    assert!(fails.len() == 0);
}

fn print_len16_u8_array(value: &[u8; 16])
{
    println!("[{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}]", value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7], value[8], value[9], value[10], value[11], value[12], value[13], value[14], value[15]);
}