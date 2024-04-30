pub fn decompress(data: &[u8], original_size: usize) -> Vec<u8> {
    if original_size == data.len() {
        return data.to_vec();
    }

    let mut target = vec![0; original_size];
    let mut src_pos = 0;
    let mut tgt_pos = 0;

    while src_pos + 1 <= data.len() {
        let flag = data[src_pos];

        for i in 0..8 {
            src_pos += 1;

            if (flag & (1 << i)) != 0 {
                target[tgt_pos] = data[src_pos];
                tgt_pos += 1;
            } else {
                let e = (data[src_pos] as usize) * 256 + data[src_pos + 1] as usize;
                let len = ((e >> 8) & 0x000f) + 2;
                let addr = ((e << 4) & 0x0ff0) + ((e >> 12) & 0x00ff);

                for _ in 0..len {
                    target[tgt_pos] = target[tgt_pos - addr - 1];
                    tgt_pos += 1;
                }
                src_pos += 1;
            }

            if src_pos + 1 >= data.len() {
                break;
            }
        }

        src_pos += 1;
    }

    target
}
