#[macro_use]
extern crate lazy_static;
use std::ops::{Mul, Add, Div, BitXor, BitXorAssign};
use std::fmt;
use std::io::{stdin, stdout, Write};

#[derive(Clone, Copy, Debug)]
pub struct GF256 {
    val: u8
}

impl GF256 {
    pub fn new(val: u8) -> GF256 {
        GF256 { val }
    }

    pub fn from_u8array(array: &[u8]) -> Result<[GF256;16], ()> {
        if array.len() != 16 {
            return Err(());
        }
        let mut res = [GF256::new(0);16];
        for i in 0..16 {
            res[i] = GF256::new(array[i]);
        }
        Ok(res)
    }

    // 加算は何があろうとすべて排他的論理和
    fn add_by_xor(a: u8, b: u8) -> GF256 {
        GF256::new(a ^ b)
    }
}

impl fmt::Display for GF256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>02x}", self.val)
    }
}

fn gmul(x: u8, y: u8) -> u8 {
    let mut r: usize = 0;
    let mut a: usize = x as usize;
    let mut b: usize = y as usize;

    // よく見るとaの繰り上がり以外は普通の掛け算
    for _ in 0..8 {

        if (b & 1) == 1 {
            r ^= a;
        }

        let hi_bit_set = a & 0x80;
        a <<= 1;
        // 排他的論理和をうまく機能させるため、足す数となるaを調整する
        // 原始多項式 P(x) = x^8 + x^4+ x^3 + x + 1
        // を利用するので0x1bを使用。
        if hi_bit_set == 0x80 {
            a -= 0xff + 1;
            a ^= 0x1b;
        }

        b >>= 1;
    }

    // 範囲内に収まっているか検査
    if 0xff < r {
        panic!("Something Wrong in Gmul. r = {}", r);
    }

    r as u8
}

impl Mul for GF256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        GF256::new(gmul(self.val, rhs.val))
    }
}

impl Add for GF256 {
    type Output = Self;

    fn add(self, rhs: Self) ->Self {
        GF256::add_by_xor(self.val, rhs.val)
    }
}

impl BitXor for GF256 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        GF256::add_by_xor(self.val, rhs.val)
    }
}

impl BitXorAssign for GF256 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = GF256::add_by_xor(self.val, rhs.val);
    }
}

fn ginv() -> [u8; 256] {
    let mut res = [0u8; 256];
    for i in 1..=255 {
        if res[i as usize] != 0 {
            continue;
        }
        for j in 1..=255 {
            if gmul(i, j) == 1 {
                res[i as usize] = j;
                res[j as usize] = i;
            }
        }
    }

    res
}

lazy_static! {
    static ref GINV_TABLE: [u8; 256] = ginv();
}

impl Div for GF256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        self * GF256::new(GINV_TABLE[rhs.val as usize])
    }
}

fn main() {
    let mut stack: Vec<GF256> = Vec::new();
    'main:loop {
        print!("{}\n> ", stack
               .iter()
               .map(|v| v.to_string())
               .collect::<Vec<_>>()
               .join(" ")
        );
        stdout().flush().unwrap();
        let mut exp = String::new();
        stdin().read_line(&mut exp).unwrap();
        let vals = exp.trim().split(" ");

        for v in vals {
            match v {
                "+" | "^" | "-" => {
                    let a = stack.pop().unwrap_or(GF256::new(0));
                    let b = stack.pop().unwrap_or(GF256::new(0));
                    stack.push(a ^ b);
                },
                "*" => {
                    let a = stack.pop().unwrap_or(GF256::new(1));
                    let b = stack.pop().unwrap_or(GF256::new(1));
                    stack.push(a * b);
                },
                "/" => {
                    let a = stack.pop().unwrap_or(GF256::new(0));
                    let b = stack.pop().unwrap_or(GF256::new(0));
                    stack.push(b / a);
                },
                n if n.parse::<u8>().is_ok() => {
                    stack.push(GF256::new(n.parse::<u8>().unwrap()));
                },
                n if u8::from_str_radix(n.trim_start_matches("0x"), 16).is_ok() => {
                    stack.push(GF256::new(u8::from_str_radix(n.trim_start_matches("0x"), 16).unwrap()));
                },
                "_d" => {
                    let p = stack.iter()
                        .map(|v| {
                            v.val.to_string()
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("DEC: {}", p);
                },
                "_c" => {
                    stack = vec![];
                },
                _ => {
                    break 'main;
                }
            }
        }
    }
}
