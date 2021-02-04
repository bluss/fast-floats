//! Experimental (unstable) “fast-math” wrappers for f32, f64
//!
//! These wrappers enable the [“fast-math”][1] flags for the operations
//! where there are intrinsics for this (add, sub, mul, div, rem).
//! The wrappers exist so that we have a quick & easy way **to experiment**
//! with fast math flags and further that feature in Rust.
//!
//! Note that as of this writing, the Rust instrinsics use the “fast” flag
//! documented in the langref; this enables all the float flags.
//!
//! [1]: http://llvm.org/docs/LangRef.html#fast-math-flags
//!
//! # Rust Version
//!
//! This crate is nightly only and experimental. Breaking changes can occur at
//! any time, if changes in Rust require it.
#![no_std]
#![feature(core_intrinsics)]

#[cfg(feature = "num-traits")]
extern crate num_traits;

#[cfg(feature = "num-traits")]
use num_traits::Zero;

extern crate core as std;

use std::intrinsics::{fadd_fast, fsub_fast, fmul_fast, fdiv_fast, frem_fast};
use std::ops::{
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
};

/// “fast-math” wrapper for f32 and f64.
///
/// The `Fast` type enforces no invariant and can hold any f32, f64 values.
/// See crate docs for more details.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
#[repr(transparent)]
pub struct Fast<F>(pub F);

/// “fast-math” wrapper for `f64`
pub type FF64 = Fast<f64>;
/// “fast-math” wrapper for `f32`
pub type FF32 = Fast<f32>;

impl<F> Fast<F> {
    /// Get the inner value
    pub fn get(self) -> F { self.0 }
}

impl<F> From<F> for Fast<F> {
    fn from(x: F) -> Self { Fast(x) }
}

impl Into<f32> for Fast<f32> {
    fn into(self: Self) -> f32 { self.get() }
}

impl Into<f64> for Fast<f64> {
    fn into(self: Self) -> f64 { self.get() }
}

// for demonstration purposes
#[cfg(test)]
pub fn fast_sum(xs: &[f64]) -> f64 {
    xs.iter().map(|&x| Fast(x)).fold(Fast(0.), |acc, x| acc + x).get()
}

// for demonstration purposes
#[cfg(test)]
pub fn fast_dot(xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter().zip(ys).fold(Fast(0.), |acc, (&x, &y)| acc + Fast(x) * Fast(y)).get()
}

#[cfg(test)]
pub fn regular_sum(xs: &[f64]) -> f64 {
    xs.iter().map(|&x| x).fold(0., |acc, x| acc + x)
}

macro_rules! impl_op {
    ($($name:ident, $method:ident, $intrins:ident;)*) => {
        $(
        // Fast<F> + F
        impl $name<f64> for Fast<f64> {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: f64) -> Self::Output {
                unsafe {
                    Fast($intrins(self.0, rhs))
                }
            }
        }

        impl $name<f32> for Fast<f32> {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: f32) -> Self::Output {
                unsafe {
                    Fast($intrins(self.0, rhs))
                }
            }
        }

        // F + Fast<F>
        impl $name<Fast<f64>> for f64 {
            type Output = Fast<f64>;
            #[inline(always)]
            fn $method(self, rhs: Fast<f64>) -> Self::Output {
                Fast(self).$method(rhs.0)
            }
        }

        impl $name<Fast<f32>> for f32 {
            type Output = Fast<f32>;
            #[inline(always)]
            fn $method(self, rhs: Fast<f32>) -> Self::Output {
                Fast(self).$method(rhs.0)
            }
        }

        // Fast<F> + Fast<F>
        impl $name for Fast<f64> {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self::Output {
                self.$method(rhs.0)
            }
        }

        impl $name for Fast<f32> {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self::Output {
                self.$method(rhs.0)
            }
        }
        )*

    }
}

macro_rules! impl_assignop {
    ($($name:ident, $method:ident, $intrins:ident;)*) => {
        $(
        impl<F, Rhs> $name<Rhs> for Fast<F>
            where Self: Add<Rhs, Output=Self> + Copy,
        {
            #[inline(always)]
            fn $method(&mut self, rhs: Rhs) {
                *self = *self + rhs
            }
        }
        )*

    }
}

impl_op! {
    Add, add, fadd_fast;
    Sub, sub, fsub_fast;
    Mul, mul, fmul_fast;
    Div, div, fdiv_fast;
    Rem, rem, frem_fast;
}

impl_assignop! {
    AddAssign, add_assign, fadd_fast;
    SubAssign, sub_assign, fsub_fast;
    MulAssign, mul_assign, fmul_fast;
    DivAssign, div_assign, fdiv_fast;
    RemAssign, rem_assign, frem_fast;
}

/*
impl<Z> Zero for Fast<Z> where Z: Zero {
    fn zero() -> Self { Fast(Z::zero()) }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
*/
#[cfg(feature = "num-traits")]
impl Zero for Fast<f64> {
    fn zero() -> Self { Fast(<_>::zero()) }
    fn is_zero(&self) -> bool { self.get().is_zero() }
}
#[cfg(feature = "num-traits")]
impl Zero for Fast<f32> {
    fn zero() -> Self { Fast(<_>::zero()) }
    fn is_zero(&self) -> bool { self.get().is_zero() }
}

use std::fmt;
macro_rules! impl_format {
    ($($name:ident)+) => {
        $(
        impl<F: fmt::$name> fmt::$name for Fast<F> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }
        )+
    }
}

impl_format!(Debug Display LowerExp UpperExp);

/// Extension trait providing fast arithmetic as *unsafe* methods on `f32` and `f64`.
///
/// Compared to the [`Fast`] newtype, this approach sacrifices the ability to use symbolic
/// operators (`+`, `-`, `*`, `/`, `%`), but does not require an explicit wrapper and preserves the
/// `unsafe` marker from the underlying intrinsics.
pub trait FastFloat {
    unsafe fn fast_add(self, other: Self) -> Self;
    unsafe fn fast_sub(self, other: Self) -> Self;
    unsafe fn fast_mul(self, other: Self) -> Self;
    unsafe fn fast_div(self, other: Self) -> Self;
    unsafe fn fast_rem(self, other: Self) -> Self;
}

macro_rules! impl_fast_float {
    ($t:ty) => {
        impl FastFloat for $t {
            unsafe fn fast_add(self, other: Self) -> Self {
                fadd_fast(self, other)
            }

            unsafe fn fast_sub(self, other: Self) -> Self {
                fsub_fast(self, other)
            }

            unsafe fn fast_mul(self, other: Self) -> Self {
                fmul_fast(self, other)
            }

            unsafe fn fast_div(self, other: Self) -> Self {
                fdiv_fast(self, other)
            }

            unsafe fn fast_rem(self, other: Self) -> Self {
                frem_fast(self, other)
            }
        }
    };
}

impl_fast_float! {f32}
impl_fast_float! {f64}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_op {
        ($($op:tt)+) => {
            $(
                assert_eq!(Fast(2.) $op Fast(1.), Fast(2. $op 1.));
            )+
        }
    }

    #[test]
    fn each_op() {
        test_op!(+ - * / %);
    }
}
