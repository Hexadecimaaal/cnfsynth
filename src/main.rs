#![feature(impl_trait_in_assoc_type)]
#![feature(new_range_api)]
extern crate core;

use core::range::Range;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str::FromStr;
use crate::bit::{Allocate, Allocation, Bit, FullAdder, RippleAdder, RippleAdderInput, Synthesize, View};
use crate::cnf::{Cnf, Sat};

mod bit;
mod cnf;

fn main() {
    let mut result = Cnf::from(vec![]);
    let bits = 64;
    let rippleadder = RippleAdder::new(bits as isize);
    let newtop = result.top + rippleadder.size();
    let rippleadder = rippleadder.allocate(Range::from(result.top..newtop));
    result.top = newtop;
    // println!("{:?}", rippleadder.outputs().c_out);
    // result.clauses.push(vec![-rippleadder.outputs().c_out].into());
    let mut a = vec![];
    for _ in 0..bits {
        let newtop = result.top + Bit.size();
        a.push(Bit.allocate(Range::from(result.top..newtop)));
        result.top = newtop;
    }
    let an : u64 = (-114514i64) as u64;
    for i in 0..bits {
        if an & (1 << i) != 0 {
            result.clauses.push(vec![a[i].outputs()].into());
        } else {
            result.clauses.push(vec![-a[i].outputs()].into());
        }
    }
    let mut b = vec![];
    for _ in 0..bits {
        let newtop = result.top + Bit.size();
        b.push(Bit.allocate(Range::from(result.top..newtop)));
        result.top = newtop;
    }
    let bn : u64 = (-1919810i64) as u64;
    for i in 0..bits {
        if bn & (1 << i) != 0 {
            result.clauses.push(vec![b[i].outputs()].into());
        } else {
            result.clauses.push(vec![-b[i].outputs()].into());
        }
    }
    let c_in = Bit.allocate(Range::from(result.top..result.top+Bit.size()));
    result.top += Bit.size();
    result.clauses.push(vec![-c_in.outputs()].into());
    let rippleadder_input = RippleAdderInput {
        a: a.iter().map(|bit| bit.outputs()).collect(),
        b: b.iter().map(|bit| bit.outputs()).collect(),
        c_in: c_in.outputs(),
    };
    result.clauses.extend(rippleadder.synthesize(rippleadder_input));
    let mut kissat = Command::new("kissat").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = kissat.stdin.take().unwrap();
    // println!("{}", result);
    stdin.write_all(result.to_string().as_bytes()).unwrap();
    drop(stdin);
    let output = kissat.wait_with_output().unwrap();
    println!("{}", String::from_utf8(output.stderr).unwrap());
    let result = String::from_utf8(output.stdout).unwrap();
    let sat = Sat::from_str(&result).unwrap();
    // println!("{:?}", sat);
    let view = rippleadder.view(sat).unwrap();
    println!("{:?}", view);
}
