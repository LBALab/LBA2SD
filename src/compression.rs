use std::collections::VecDeque;

const INDEX_BIT_COUNT: usize = 12;
const LENGTH_BIT_COUNT: usize = 4;
const WINDOW_SIZE: usize = 1 << INDEX_BIT_COUNT;
const RAW_LOOK_AHEAD_SIZE: usize = 1 << LENGTH_BIT_COUNT;
const BREAK_EVEN: usize = ((1 + INDEX_BIT_COUNT + LENGTH_BIT_COUNT) / 9) as usize;
const LOOK_AHEAD_SIZE: usize = RAW_LOOK_AHEAD_SIZE + BREAK_EVEN;

pub(crate) fn compress(input_data: &[u8]) -> Vec<u8> {
    let mut output_data = Vec::new();
    let mut window: VecDeque<u8> = VecDeque::with_capacity(WINDOW_SIZE);
    let mut flag_byte: u8 = 0;
    let mut bit_count: u8 = 0;
    let mut input_pos: usize = 0;

    while input_pos < input_data.len() {
        // Ensure the window does not exceed WINDOW_SIZE
        if window.len() >= WINDOW_SIZE {
            window.pop_front();
        }

        // Find the longest match in the window
        let mut best_match_len = 0;
        let mut best_match_pos = 0;
        let max_search_len = std::cmp::min(LOOK_AHEAD_SIZE, input_data.len() - input_pos);

        for pos in 0..window.len() {
            let mut match_len = 0;
            while match_len < max_search_len && window.get(pos + match_len) == Some(&input_data[input_pos + match_len]) {
                match_len += 1;
            }
            if match_len > best_match_len {
                best_match_len = match_len;
                best_match_pos = pos;
            }
            if best_match_len >= LOOK_AHEAD_SIZE {
                break;
            }
        }

        if best_match_len > BREAK_EVEN {
            // Compressed reference
            let offset = best_match_pos;
            let length = best_match_len - BREAK_EVEN - 1;

            // Pack offset and length into 2 bytes
            let value = ((offset << LENGTH_BIT_COUNT) | length) as u16;
            let high = (value >> 8) as u8;
            let low = (value & 0xFF) as u8;

            // Set flag bit to 0 for compressed
            flag_byte &= !(1 << (7 - bit_count));
            bit_count += 1;
            output_data.push(high);
            output_data.push(low);

            // Add matched bytes to the window
            for i in 0..best_match_len {
                window.push_back(input_data[input_pos + i]);
            }
            input_pos += best_match_len;
        } else {
            // Raw byte
            let byte = input_data[input_pos];

            // Set flag bit to 1 for raw
            flag_byte |= 1 << (7 - bit_count);
            bit_count += 1;
            output_data.push(byte);

            // Add the byte to the window
            window.push_back(byte);
            input_pos += 1;
        }

        // Output flag byte every 8 data elements
        if bit_count == 8 {
            output_data.push(flag_byte);
            flag_byte = 0;
            bit_count = 0;
        }
    }

    // Output remaining flag byte if any
    if bit_count > 0 {
        // Zero-fill the remaining bits
        flag_byte <<= (8 - bit_count);
        output_data.push(flag_byte);
    }

    output_data
}
