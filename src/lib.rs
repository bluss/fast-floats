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
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(core_intrinsics)]

#[cfg(feature = "num-traits")]
extern crate num_traits;

#[cfg(feature = "num-traits")]
use num_traits::Zero;

#[cfg(not(feature = "std"))]
extern crate core as std;

#[cfg(feature = "std")]
use std::num::FpCategory;

use std::intrinsics::{fadd_fast, fdiv_fast, fmul_fast, frem_fast, fsub_fast};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

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
    #[inline(always)]
    pub fn get(self) -> F {
        self.0
    }
}

impl<F> From<F> for Fast<F> {
    #[inline(always)]
    fn from(x: F) -> Self {
        Fast(x)
    }
}

impl Into<f32> for Fast<f32> {
    #[inline(always)]
    fn into(self: Self) -> f32 {
        self.get()
    }
}

impl Into<f64> for Fast<f64> {
    #[inline(always)]
    fn into(self: Self) -> f64 {
        self.get()
    }
}

// for demonstration purposes
#[cfg(test)]
pub fn fast_sum(xs: &[f64]) -> f64 {
    xs.iter()
        .map(|&x| Fast(x))
        .fold(Fast(0.), |acc, x| acc + x)
        .get()
}

// for demonstration purposes
#[cfg(test)]
pub fn fast_dot(xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter()
        .zip(ys)
        .fold(Fast(0.), |acc, (&x, &y)| acc + Fast(x) * Fast(y))
        .get()
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

#[cfg(feature = "num-traits")]
impl Zero for Fast<f64> {
    fn zero() -> Self {
        Fast(<_>::zero())
    }
    fn is_zero(&self) -> bool {
        self.get().is_zero()
    }
}
#[cfg(feature = "num-traits")]
impl Zero for Fast<f32> {
    fn zero() -> Self {
        Fast(<_>::zero())
    }
    fn is_zero(&self) -> bool {
        self.get().is_zero()
    }
}

#[cfg(feature = "std")]
macro_rules! delegate_no_args {
    ($($method: ident -> $rtype: ty),*$(,)?) => {
        $(
            #[inline(always)]
            pub fn $method(self) -> $rtype {
                self.0.$method().into()
            }
        )*
    };
}

#[cfg(feature = "std")]
macro_rules! delegate_one_arg {
    ($($method: ident($arg: ident : $atype: ty)),*$(,)?) => {
        $(
            #[inline(always)]
            pub fn $method(self, $arg: $atype) -> Self {
                self.0.$method($arg).into()
            }
        )*
    };
}

#[cfg(feature = "std")]
macro_rules! delegate_two_args {
    ($($method: ident($arg1: ident : $atype1: ty, $arg2: ident : $atype2: ty)),*$(,)?) => {
        $(
            #[inline(always)]
            pub fn $method(self, $arg1: $atype1, $arg2: $atype2) -> Self {
                self.0.$method($arg1, $arg2).into()
            }
        )*
    };
}

#[cfg(feature = "std")]
macro_rules! delegate_conversion_from {
    ($primitive: ty { $($method: ident($from: ident : $from_type: ty)),*$(,)? }) => {
        $(
            #[inline(always)]
            pub fn $method($from: $from_type) -> Self {
                <$primitive>::$method($from).into()
            }
        )*
    };
}

#[cfg(feature = "std")]
macro_rules! impl_delegations {
    ($wrapper: ident, $float: ident, $btype: ty, $bytes: literal) => {
        impl $wrapper {
            delegate_no_args! {
                floor -> Self,
                ceil -> Self,
                round -> Self,
                trunc -> Self,
                fract -> Self,
                abs -> Self,
                signum -> Self,
                sqrt -> Self,
                exp -> Self,
                exp2 -> Self,
                ln -> Self,
                log2 -> Self,
                log10 -> Self,
                cbrt -> Self,
                sin -> Self,
                cos -> Self,
                tan -> Self,
                asin -> Self,
                acos -> Self,
                atan -> Self,
                exp_m1 -> Self,
                ln_1p -> Self,
                sinh -> Self,
                cosh -> Self,
                tanh -> Self,
                asinh -> Self,
                acosh -> Self,
                atanh -> Self,
                is_nan -> bool,
                is_infinite -> bool,
                is_finite -> bool,
                is_normal -> bool,
                classify -> FpCategory,
                is_sign_positive -> bool,
                is_sign_negative -> bool,
                recip -> Self,
                to_degrees -> Self,
                to_radians -> Self,
                to_bits -> $btype,
                to_be_bytes -> [u8; $bytes],
                to_le_bytes -> [u8; $bytes],
                to_ne_bytes -> [u8; $bytes],
            }

            delegate_one_arg! {
                copysign(sign: $float),
                div_euclid(rhs: $float),
                rem_euclid(rhs: $float),
                powi(rhs: i32),
                powf(rhs: $float),
                log(base: $float),
                hypot(other: $float),
                atan2(other: $float),
                max(other: $float),
                min(other: $float),
            }

            delegate_two_args! {
                mul_add(a: $float, b: $float),
            }

            delegate_conversion_from! { $float {
                from_bits(v: $btype),
                from_be_bytes(bytes: [u8; $bytes]),
                from_le_bytes(bytes: [u8; $bytes]),
                from_ne_bytes(bytes: [u8; $bytes]),
            }}

            #[inline(always)]
            pub fn sin_cos(self) -> ($wrapper, $wrapper) {
                let (sin, cos) = self.0.sin_cos();
                (sin.into(), cos.into())
            }
        }
    };
}

#[cfg(feature = "std")]
impl_delegations! { FF32, f32, u32, 4 }

#[cfg(feature = "std")]
impl_delegations! { FF64, f64, u64, 8 }

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
