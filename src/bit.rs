use core::range::Range;
use std::{cell::OnceCell, rc::Rc};

use crate::cnf::{Clause, Sat};
use crate::cnf::Lit;


pub trait Allocation {
    type Output;
    fn vars(&self) -> impl IntoIterator<Item=Lit>;
    fn outputs(&self) -> Self::Output;
}

pub trait Allocate {
    type Result: Allocation;
    fn size(&self) -> isize;
    fn allocate(&self, range: Range<Lit>) -> Self::Result;
}

pub trait Synthesize<I>: Allocation {
    fn synthesize(&self, input: I) -> impl IntoIterator<Item=Clause>;
}

pub struct BitAllocation(Lit);

impl Allocation for BitAllocation {
    type Output = Lit;
    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0)
    }
    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl Synthesize<()> for BitAllocation {
    fn synthesize(&self, _input: ()) -> impl IntoIterator<Item=Clause> {
        vec![]
    }
}

pub struct Bit;

impl Allocate for Bit {
    type Result = BitAllocation;
    fn size(&self) -> isize {
        1
    }
    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        BitAllocation(range.start)
    }
}

pub struct Assert;

impl Allocation for Assert {
    type Output = ();

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::empty()
    }

    fn outputs(&self) -> Self::Output {
        ()
    }
}

impl<T> Synthesize<T> for Assert
where
    T: IntoIterator<Item=Lit>,
{
    fn synthesize(&self, input: T) -> impl IntoIterator<Item=Clause> {
        input.into_iter().map(|lit| Clause {
            lits: vec![lit],
        })
    }
}

pub struct NotAllocation(Lit);

impl Allocation for NotAllocation
{
    type Output = Lit;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0 as Lit)
    }

    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl Synthesize<Lit> for NotAllocation
{
    fn synthesize(&self, input: Lit) -> impl IntoIterator<Item=Clause> {
        vec![Clause {
            lits: vec![-input, -(self.0 as Lit)],
        }]
    }
}

pub struct Not;

impl Allocate for Not
where
{
    type Result = NotAllocation;

    fn size(&self) -> isize {
        1
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, 1);
        NotAllocation(range.start)
    }
}

// pub struct NotModuleAllocation<T>
// where
//     T: Allocation<Output=Lit>,
// {
//     bit: T,
//     not: NotAllocation,
// }
//
// impl<T> Allocation for NotModuleAllocation<T>
// where
//     T: Allocation<Output=Lit>,
// {
//     type Output = Lit;
//
//     fn vars(&self) -> impl IntoIterator<Item=Lit> {
//         self.bit.vars().into_iter()
//             .chain(self.not.vars())
//     }
//
//     fn outputs(&self) -> Self::Output {
//         self.not.outputs()
//     }
// }
//
// impl<T, U> Synthesize<U> for NotModuleAllocation<T>
// where
//     T: Synthesize<U, Output=Lit>,
// {
//     fn synthesize(&self, input: U) -> impl IntoIterator<Item=Clause> {
//         self.not.synthesize(self.bit.outputs().clone() as Lit)
//             .into_iter()
//             .chain(self.bit.synthesize(input))
//     }
// }

// pub struct NotModule<T>
// where
//     T: Allocate,
//     T::Result: Allocation<Output=Lit>,
// {
//     bit: T,
//     not: Not,
// }
//
// impl<T> Allocate for NotModule<T>
// where
//     T: Allocate,
//     T::Result: Allocation<Output=Lit>,
// {
//     type Result = NotModuleAllocation<T::Result>;
//
//     fn size(&self) -> usize {
//         self.bit.size() + self.not.size()
//     }
//
//     fn allocate(&self, range: Range<Lit>) -> Self::Result {
//         assert_eq!(range.end - range.start, self.size().try_into().unwrap());
//         let bit = self.bit.allocate(Range::from(range.start..range.end - 1));
//         let not = self.not.allocate(Range::from(range.end - 1..range.end));
//         NotModuleAllocation {
//             bit,
//             not,
//         }
//     }
// }
//
// impl core::ops::Not for Bit {
//     type Output = NotModule<Bit>;
//
//     fn not(self) -> Self::Output {
//         NotModule { bit: self, not: Not }
//     }
// }

pub struct AllAllocation(Lit);

impl Allocation for AllAllocation {
    type Output = Lit;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0)
    }

    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl<T> Synthesize<T> for AllAllocation
where
    T: IntoIterator<Item=Lit>,
{
    fn synthesize(&self, inputs: T) -> impl IntoIterator<Item=Clause> {
        let output = self.0;
        let mut result = vec![];
        let mut back_clause = vec![];
        for input in inputs {
            result.push(Clause {
                lits: vec![input, -output],
            });
            back_clause.push(-input);
        }
        back_clause.push(output);
        result.push(back_clause.into());
        result
    }
}

pub struct All;

impl Allocate for All {
    type Result = AllAllocation;

    fn size(&self) -> isize {
        1
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        AllAllocation(range.start)
    }
}

pub struct AnyAllocation(Lit);

impl Allocation for AnyAllocation {
    type Output = Lit;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0)
    }

    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl<T> Synthesize<T> for AnyAllocation
where
    T: IntoIterator<Item=Lit>,
{
    fn synthesize(&self, inputs: T) -> impl IntoIterator<Item=Clause> {
        let output = self.0;
        let mut result = vec![];
        let mut back_clause = vec![];
        for input in inputs {
            result.push(Clause {
                lits: vec![-input, output],
            });
            back_clause.push(input);
        }
        back_clause.push(-output);
        result.push(back_clause.into());
        result
    }
}

pub struct Any;

impl Allocate for Any {
    type Result = AnyAllocation;

    fn size(&self) -> isize {
        1
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        AnyAllocation(range.start)
    }
}

pub struct AndAllocation(Lit);

impl Allocation for AndAllocation {
    type Output = Lit;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0)
    }

    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl Synthesize<(Lit, Lit)> for AndAllocation
{
    fn synthesize(&self, (a, b): (Lit, Lit)) -> impl IntoIterator<Item=Clause> {
        let output = self.0;
        vec![
            Clause {
                lits: vec![-a, -b, output],
            },
            Clause {
                lits: vec![a, -output],
            },
            Clause {
                lits: vec![b, -output],
            },
        ]
    }
}

pub struct And;

impl Allocate for And {
    type Result = AndAllocation;

    fn size(&self) -> isize {
        1
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        AndAllocation(range.start)
    }
}

pub struct OrAllocation(Lit);

impl Allocation for OrAllocation {
    type Output = Lit;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0)
    }

    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl Synthesize<(Lit, Lit)> for OrAllocation
{
    fn synthesize(&self, (a, b): (Lit, Lit)) -> impl IntoIterator<Item=Clause> {
        let output = self.0;
        vec![
            Clause {
                lits: vec![a, b, -output],
            },
            Clause {
                lits: vec![-a, output],
            },
            Clause {
                lits: vec![-b, output],
            },
        ]
    }
}

pub struct Or;

impl Allocate for Or {
    type Result = OrAllocation;

    fn size(&self) -> isize {
        1
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        OrAllocation(range.start)
    }
}


pub struct XorAllocation(Lit);

impl Allocation for XorAllocation {
    type Output = Lit;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        std::iter::once(self.0)
    }

    fn outputs(&self) -> Self::Output {
        self.0
    }
}

impl Synthesize<(Lit, Lit)> for XorAllocation
{
    fn synthesize(&self, (a, b): (Lit, Lit)) -> impl IntoIterator<Item=Clause> {
        let output = self.0;
        vec![
            Clause {
                lits: vec![-a, -b, -output],
            },
            Clause {
                lits: vec![a, b, -output],
            },
            Clause {
                lits: vec![a, -b, output],
            },
            Clause {
                lits: vec![-a, b, output],
            },
        ]
    }
}

impl Synthesize<(Lit, Lit, Lit)> for XorAllocation
{
    fn synthesize(&self, (a, b, c): (Lit, Lit, Lit)) -> impl IntoIterator<Item=Clause> {
        let output = self.0;
        vec![
            Clause {
                lits: vec![-a, -b, -c, output],
            },
            Clause {
                lits: vec![-a, -b, c, -output],
            },
            Clause {
                lits: vec![-a, b, -c, -output],
            },
            Clause {
                lits: vec![-a, b, c, output],
            },
            Clause {
                lits: vec![a, -b, -c, -output],
            },
            Clause {
                lits: vec![a, -b, c, output],
            },
            Clause {
                lits: vec![a, b, -c, output],
            },
            Clause {
                lits: vec![a, b, c, -output],
            },
        ]
    }
}


pub struct Xor;

impl Allocate for Xor {
    type Result = XorAllocation;

    fn size(&self) -> isize {
        1
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        XorAllocation(range.start)
    }
}

pub struct FullAdderAllocation {
    s: XorAllocation,
    c_out: OrAllocation,
    p: XorAllocation,
    p_and_c_in: AndAllocation,
    g: AndAllocation,
}

pub struct FullAdderOutput {
    pub s: Lit,
    pub c_out: Lit,
    pub p: Lit,
    pub g: Lit,
}

impl Allocation for FullAdderAllocation {
    type Output = FullAdderOutput;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        self.s.vars().into_iter()
            .chain(self.c_out.vars())
            .chain(self.p.vars())
            .chain(self.p_and_c_in.vars())
            .chain(self.g.vars())
    }

    fn outputs(&self) -> Self::Output {
        let s = self.s.outputs();
        let c_out = self.c_out.outputs();
        let p = self.p.outputs();
        let g = self.g.outputs();
        FullAdderOutput {
            s,
            c_out,
            p,
            g,
        }
    }
}

impl Synthesize<(Lit, Lit, Lit)> for FullAdderAllocation
{
    fn synthesize(&self, (a, b, c_in): (Lit, Lit, Lit)) -> impl IntoIterator<Item=Clause> {
        self.s.synthesize((a, b, c_in))
            .into_iter()
            .chain(self.c_out.synthesize((self.g.outputs(), self.p_and_c_in.outputs())))
            .chain(self.p.synthesize((a, b)))
            .chain(self.p_and_c_in.synthesize((self.p.outputs(), c_in)))
            .chain(self.g.synthesize((a, b)))
    }
}

pub struct FullAdder;

impl Allocate for FullAdder {
    type Result = FullAdderAllocation;

    fn size(&self) -> isize {
        Xor.size() + Or.size() + Xor.size() + And.size() + And.size()
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        let s = Xor.allocate(Range::from(range.start..range.start + 1));
        let c_out = Or.allocate(Range::from(range.start + 1..range.start + 2));
        let p = Xor.allocate(Range::from(range.start + 2..range.start + 3));
        let p_and_c_in = And.allocate(Range::from(range.start + 3..range.start + 4));
        let g = And.allocate(Range::from(range.start + 4..range.start + 5));
        FullAdderAllocation {
            s,
            c_out,
            p,
            p_and_c_in,
            g,
        }
    }
}

pub struct RippleAdderAllocation {
    adders: Vec<FullAdderAllocation>,
}

pub struct RippleAdderOutput {
    pub s: Vec<Lit>,
    pub c_out: Lit,
}

impl Allocation for RippleAdderAllocation {
    type Output = RippleAdderOutput;

    fn vars(&self) -> impl IntoIterator<Item=Lit> {
        self.adders.iter().flat_map(|a| a.vars())
    }

    fn outputs(&self) -> Self::Output {
        let s = self.adders.iter().map(|a| a.s.outputs()).collect::<Vec<_>>();
        let c_out = self.adders.last().unwrap().c_out.outputs();
        RippleAdderOutput {
            s,
            c_out,
        }
    }
}

pub struct RippleAdderInput {
    pub(crate) a: Vec<Lit>,
    pub(crate) b: Vec<Lit>,
    pub(crate) c_in: Lit,
}

impl Synthesize<RippleAdderInput> for RippleAdderAllocation {
    fn synthesize(&self, input: RippleAdderInput) -> impl IntoIterator<Item=Clause> {
        assert_eq!(input.a.len(), self.adders.len());
        assert_eq!(input.b.len(), self.adders.len());
        let mut result = vec![];
        let mut c_in = input.c_in;
        for (i, a) in input.a.into_iter().enumerate() {
            let b = input.b[i];
            let adder = &self.adders[i];
            let output = adder.outputs();
            result.extend(adder.synthesize((a, b, c_in)));
            c_in = output.c_out;
        }
        result
    }
}

pub struct RippleAdder {
    width: isize,
}

impl RippleAdder {
    pub fn new(width: isize) -> Self {
        RippleAdder { width }
    }
}

impl Allocate for RippleAdder {
    type Result = RippleAdderAllocation;

    fn size(&self) -> isize {
        self.width * FullAdder.size()
    }

    fn allocate(&self, range: Range<Lit>) -> Self::Result {
        assert_eq!(range.end - range.start, self.size().try_into().unwrap());
        let adders = (0..self.width).map(|i| {
            FullAdder.allocate(Range::from(
                range.start + (i * FullAdder.size()) as Lit
                    ..range.start + ((i + 1) * FullAdder.size()) as Lit
            ))
        }).collect();
        RippleAdderAllocation {
            adders,
        }
    }
}

pub trait View<V> : Allocation {
    fn view(&self, s : Sat) -> Option<V>;
}

#[derive(Debug)]
pub struct RippleAdderView {
    s : i64,
    c_out : bool,
}

impl View<RippleAdderView> for RippleAdderAllocation {
    fn view(&self, sat : Sat) -> Option<RippleAdderView> {
        let sat = sat.0?;
        let mut s = 0;
        for bit in self.outputs().s.iter().rev() {
            s <<= 1;
            // dbg!(bit, sat[bit.abs() as usize]);
            s |= (sat[bit.abs() as usize] >= 0) as i64;
        }
        Some(RippleAdderView {
            s,
            c_out : sat[self.outputs().c_out.abs() as usize] > 0,
        })
    }
}
