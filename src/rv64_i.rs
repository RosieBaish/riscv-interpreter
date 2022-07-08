use std::cell::RefCell;

pub fn sext<const ARRLEN: usize>(input: [bool; ARRLEN]) -> u64 {
  let mut total: u64 = 0;
  for i in 0..ARRLEN {
    total |= (input[i] as u64) << i;
  }
  for i in ARRLEN..64 {
    total |= (input[ARRLEN - 1] as u64) << i;
  }
  total
}

pub fn sext_n(input: u64, current_len: u32) -> u64 {
  let mut total: u64 = 0;
  for i in 0..current_len {
    total |= input & (1 << i);
  }
  let highbit = input & (1 << current_len) >> current_len;
  for i in current_len..64 {
    total |= highbit << i;
  }
  total
}

pub fn signed_lt(left: u64, right: u64) -> bool {
  let s_left = i64::from_ne_bytes(left.to_ne_bytes());
  let s_right = i64::from_ne_bytes(right.to_ne_bytes());
  return s_left < s_right;
}

pub fn arith_r_shift(val: u64, offset: u64) -> u64 {
  let s_val = i64::from_ne_bytes(val.to_ne_bytes());
  // The >> operator is an arithmetic shift if the LHS is signed
  // And a logical shift if not, which is what we want.
  let shifted_s_val = s_val >> offset;
  u64::from_ne_bytes(shifted_s_val.to_ne_bytes())
}

const MEMORY_SIZE: usize = 1024;
thread_local! {static MEMORY: RefCell<[u8; MEMORY_SIZE]> = RefCell::new([0; MEMORY_SIZE]);}

pub fn mem_read(address: u64, length: u32) -> u64 {
  assert!(length == 8 || length == 16 || length == 32 || length == 64);
  assert!(address as usize + ((length / 8) as usize) < MEMORY_SIZE);

  let mut val: u64 = 0;

  MEMORY.with(|m| {
    let mem: [u8; MEMORY_SIZE] = *m.borrow();
    for i in 0..(length / 8) {
      val += (mem[address as usize] as u64) << (i * 8);
    }
  });
  val
}

pub fn mem_read_sext(address: u64, length: u32) -> u64 {
  sext_n(mem_read(address, length), length)
}

pub fn mem_write(address: u64, length: u32, val: u64) {
  assert!(length == 8 || length == 16 || length == 32 || length == 64);
  assert!(address as usize + ((length / 8) as usize) < MEMORY_SIZE);

  MEMORY.with(|m| {
    let mut mem: [u8; MEMORY_SIZE] = *m.borrow_mut();
    for i in 0..(length / 8) {
      mem[address as usize] = ((val >> (i * 8)) & 0xFF) as u8;
    }
  });
}
