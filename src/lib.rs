mod utils;

use wasm_bindgen::prelude::*;
// use rlp::RlpStream;
// use primitive_types::U256;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn read_buffer(idx: i32) -> i32;
    fn setlen(idx: i32);
    fn getlen() -> i32;
    fn write_buffer(idx: i32, c: i32);
    fn usegas(gas: i32);
    fn rvec(ptr: *mut u8, idx: i32, len: i32);
    fn wvec(ptr: *mut u8, idx: i32, len: i32);
}

fn handle_uint(output: &mut Vec<u8>, num: &[u8]) {
    let mut first_zero = 32;
    for i in 0..32 {
        if num[32-i-1] == 0 {
            first_zero = i;
            break
        }
    }
    if first_zero == 0 {
        // the int is 0
        output.push(0x80)
    } else if first_zero == 1 && num[31] <= 0x7f {
        output.push(num[31])
    } else {
        // needed bytes
        output.push((first_zero + 0x80) as u8);
        for i in 0..first_zero {
            output.push(num[32-i-1])
        }
    }
}

fn needed_bytes(num: usize) -> u8 {
    // needed bytes
    let mut acc = num;
    for i in 0..8 {
        if acc == 0 {
            return i
        }
        acc = acc/256
    }
    return 8    
}

fn handle_bytes(output: &mut Vec<u8>, num: &[u8]) {
    let len = num.len();
    if num.len() == 0 {
        output.push(0x80)
    } else if num.len() == 1 && num[0] <= 0x7f {
        output.push(num[0])
    } else if num.len() <= 55 {
        output.push((num.len() + 0x80) as u8);
        for b in num {
            output.push(*b)
        }
    } else {
        let needed = needed_bytes(len);
        output.push(needed + 0xb7);
        for i in 0..needed {
            output.push(((len >> (8*i)) & 0xff) as u8)
        }
        for b in num {
            output.push(*b)
        }
    }
}

fn handle_address(output: &mut Vec<u8>, num: &[u8]) {
    let mut first_zero = 32;
    for i in 0..32 {
        if num[32-i-1] == 0 {
            first_zero = i;
            break
        }
    }
    if first_zero == 0 {
        output.push(0x80)
    } else {
        handle_bytes(output, &num[12..32])
    }
}

fn read_int(num: &[u8]) -> usize {
    let mut res : usize = 0;
    for i in 28..32 {
        res = res * 256;
        res += num[i] as usize;
    }
    res
}

pub fn process(v: Vec<u8>) -> Vec<u8> {
    let mut output = vec![];
    output.reserve(1024);
    let len = read_int(&v[160..192]); // data len
    /*
    let s = RlpStream::new_list(9);
    // s.append(v[]);
    let res = rlp::encode_list(&lst);
    v
    */
    output.push(0xc0 + 9);
    handle_uint(&mut output, &v[0..32]); // seqnum
    handle_uint(&mut output, &v[32..64]); // gas price
    handle_uint(&mut output, &v[64..96]); // gas limit
    handle_address(&mut output, &v[96..128]); // address
    handle_uint(&mut output, &v[128..160]); // value
    handle_bytes(&mut output, &v[288..288+len]); // data
    handle_uint(&mut output, &v[192..224]); // v
    handle_uint(&mut output, &v[224..256]); // r
    handle_uint(&mut output, &v[256..288]); // s
    output
}

#[wasm_bindgen]
pub fn test() -> u32 {
    let input_len = getlen();
    let mut input = vec![0; input_len as usize];

    rvec(input.as_mut_ptr(), 0, input_len);

    usegas(input_len / 10 + 1);

    let mut output = process(input);

    wvec(output.as_mut_ptr(), 0, output.len() as i32);
    setlen(output.len() as i32);

    0

}
