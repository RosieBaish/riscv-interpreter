pub const MEMORY_SIZE: usize = 4096;
use crate::instruction::Register;

pub fn sext<const ARRLEN: usize>(input: [bool; ARRLEN]) -> Register {
  let mut total: u64 = 0;
  for (index, value) in input.iter().enumerate() {
    total |= (*value as u64) << index;
  }
  for i in ARRLEN..64 {
    total |= (input[ARRLEN - 1] as u64) << i;
  }
  log!("sext({:?}) = {}", input, total);
  Register { value: total }
}

pub fn sext_n(input: Register, current_len: u32) -> Register {
  let mut total: u64 = 0;
  for i in 0..current_len {
    total |= input.value & (1 << i);
  }
  let highbit = (input.value & (1 << (current_len - 1))) >> (current_len - 1);
  for i in current_len..64 {
    total |= highbit << i;
  }
  Register { value: total }
}

pub fn signed_lt(left: Register, right: Register) -> bool {
  let s_left = i64::from_ne_bytes(left.value.to_ne_bytes());
  let s_right = i64::from_ne_bytes(right.value.to_ne_bytes());
  s_left < s_right
}

pub fn arith_r_shift_i(val: Register, offset: u64) -> Register {
  let s_val = i64::from_ne_bytes(val.value.to_ne_bytes());
  // The >> operator is an arithmetic shift if the LHS is signed
  // And a logical shift if not, which is what we want.
  let shifted_s_val = s_val >> offset;
  Register {
    value: u64::from_ne_bytes(shifted_s_val.to_ne_bytes()),
  }
}

pub fn arith_r_shift(val: Register, offset: Register) -> Register {
  arith_r_shift_i(val, offset.value)
}

pub fn read(
  mem: &[u8; MEMORY_SIZE],
  address: Register,
  length: u32,
) -> Register {
  assert!(length == 8 || length == 16 || length == 32 || length == 64);
  assert!(address.value as usize + ((length / 8) as usize) <= MEMORY_SIZE);

  let mut val: u64 = 0;

  for i in 0..(length / 8) {
    val += (mem[address.value as usize + i as usize] as u64) << (i * 8);
  }
  Register { value: val }
}

pub fn read_sext(
  mem: &[u8; MEMORY_SIZE],
  address: Register,
  length: u32,
) -> Register {
  sext_n(read(mem, address, length), length)
}

pub fn write(
  mem: &mut [u8; MEMORY_SIZE],
  address: Register,
  length: u32,
  val: Register,
) {
  log!("write(mem, {}, {}, {})", address, length, val);
  assert!(length == 8 || length == 16 || length == 32 || length == 64);
  assert!(address.value as usize + ((length / 8) as usize) <= MEMORY_SIZE);
  for i in 0..(length / 8) {
    mem[address.value as usize + i as usize] =
      ((val.value >> (i * 8)) & 0xFF) as u8;
  }
}
