// Copyright (C) 2019 Alibaba Cloud Computing. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! Traits to represent an address within an address space.
//!
//! Two traits are defined to present an address within an address space:
//! - [AddressValue](trait.AddressValue.html): stores the raw value of an address. Typically u32,
//! u64 or usize is used to store the raw value. But pointers, such as *u8, can't be used because
//! it doesn't implement the Add and Sub traits.
//! - [Address](trait.Address.html): encapsulates an AddressValue object and defines methods to
//! access and manipulate it.

use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::ops::{Add, BitAnd, BitOr, Sub};

/// Simple helper trait used to store a raw address value.
pub trait AddressValue {
    /// Type of the address raw value.
    type V: Copy
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Add<Output = Self::V>
        + Sub<Output = Self::V>
        + BitAnd<Output = Self::V>
        + BitOr<Output = Self::V>;
}

/// Trait to represent an address within an address space.
///
/// To simplify the design and implementation, assume the same raw data type (AddressValue::V)
/// could be used to store address, size and offset for the address space. Thus the Address trait
/// could be used to manage address, size and offset. On the other hand, type aliases may be
/// defined to improve code readability.
///
/// One design rule is applied to the Address trait that operators (+, -, &, | etc) are not
/// supported and it forces clients to explicitly invoke corresponding methods. But there are
/// always exceptions:
///     Address (BitAnd|BitOr) AddressValue are supported.
pub trait Address:
    AddressValue
    + Sized
    + Default
    + Copy
    + Eq
    + PartialEq
    + Ord
    + PartialOrd
    + BitAnd<<Self as AddressValue>::V, Output = Self>
    + BitOr<<Self as AddressValue>::V, Output = Self>
{
    /// Create an address from a raw address value.
    fn new(addr: Self::V) -> Self;

    /// Get the raw value of the address.
    fn raw_value(&self) -> Self::V;

    /// Returns the bitwise and of the address with the given mask.
    fn mask(&self, mask: Self::V) -> Self::V {
        self.raw_value() & mask
    }

    /// Returns the offset from this address to the given base address and None if there is
    /// underflow.
    fn checked_offset_from(&self, base: Self) -> Option<Self::V>;

    /// Returns the offset from this address to the given base address.
    /// Only use this when `base` is guaranteed not to overflow.
    /// # Examples
    ///
    /// ```
    /// # use vm_memory::{Address, GuestAddress};
    ///   let base = GuestAddress(0x100);
    ///   let addr = GuestAddress(0x150);
    ///   assert_eq!(addr.unchecked_offset_from(base), 0x50);
    /// ```
    fn unchecked_offset_from(&self, base: Self) -> Self::V {
        self.raw_value() - base.raw_value()
    }

    /// Returns the result of the add or None if there is overflow.
    fn checked_add(&self, other: Self::V) -> Option<Self>;

    /// Returns the result of the add and a flag identifying whether there was overflow
    fn overflowing_add(&self, other: Self::V) -> (Self, bool);

    /// Returns the result of the base address + the size.
    /// Only use this when `offset` is guaranteed not to overflow.
    fn unchecked_add(&self, offset: Self::V) -> Self;

    /// Returns the result of the subtraction or None if there is underflow.
    fn checked_sub(&self, other: Self::V) -> Option<Self>;

    /// Returns the result of the subtraction and a flag identifying whether there was overflow
    fn overflowing_sub(&self, other: Self::V) -> (Self, bool);

    /// Returns the result of the subtraction.
    /// Only use this when `other` is guaranteed not to underflow.
    fn unchecked_sub(&self, other: Self::V) -> Self;
}

macro_rules! impl_address_ops {
    ($T:ident, $V:ty) => {
        impl AddressValue for $T {
            type V = $V;
        }

        impl Address for $T {
            fn new(value: $V) -> $T {
                $T(value)
            }

            fn raw_value(&self) -> $V {
                self.0
            }

            fn checked_offset_from(&self, base: $T) -> Option<$V> {
                self.0.checked_sub(base.0)
            }

            fn checked_add(&self, other: $V) -> Option<$T> {
                self.0.checked_add(other).map($T)
            }

            fn overflowing_add(&self, other: $V) -> ($T, bool) {
                let (t, ovf) = self.0.overflowing_add(other);
                ($T(t), ovf)
            }

            fn unchecked_add(&self, offset: $V) -> $T {
                $T(self.0 + offset)
            }

            fn checked_sub(&self, other: $V) -> Option<$T> {
                self.0.checked_sub(other).map($T)
            }

            fn overflowing_sub(&self, other: $V) -> ($T, bool) {
                let (t, ovf) = self.0.overflowing_sub(other);
                ($T(t), ovf)
            }

            fn unchecked_sub(&self, other: $V) -> $T {
                $T(self.0 - other)
            }
        }

        impl Default for $T {
            fn default() -> $T {
                Self::new(0 as $V)
            }
        }

        impl BitAnd<$V> for $T {
            type Output = $T;

            fn bitand(self, other: $V) -> $T {
                $T(self.0 & other)
            }
        }

        impl BitOr<$V> for $T {
            type Output = $T;

            fn bitor(self, other: $V) -> $T {
                $T(self.0 | other)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct MockAddress(pub u64);
    impl_address_ops!(MockAddress, u64);

    #[test]
    fn test_new() {
        assert_eq!(MockAddress::new(0), MockAddress(0));
        assert_eq!(MockAddress::new(std::u64::MAX), MockAddress(std::u64::MAX));
    }

    #[test]
    fn test_offset_from() {
        let base = MockAddress(0x100);
        let addr = MockAddress(0x150);
        assert_eq!(addr.unchecked_offset_from(base), 0x50u64);
        assert_eq!(addr.checked_offset_from(base), Some(0x50u64));
        assert_eq!(base.checked_offset_from(addr), None);
    }

    #[test]
    fn test_equals() {
        let a = MockAddress(0x300);
        let b = MockAddress(0x300);
        let c = MockAddress(0x301);
        assert_eq!(a, MockAddress(a.raw_value()));
        assert_eq!(a, b);
        assert_eq!(b, a);
        assert_ne!(a, c);
        assert_ne!(c, a);
    }

    #[test]
    fn test_cmp() {
        let a = MockAddress(0x300);
        let b = MockAddress(0x301);
        assert!(a < b);
    }

    #[test]
    fn test_mask() {
        let a = MockAddress(0x5050);
        assert_eq!(MockAddress(0x5000), a & 0xff00u64);
        assert_eq!(0x5000, a.mask(0xff00u64));
        assert_eq!(MockAddress(0x5055), a | 0x0005u64);
    }

    fn check_add(a: u64, b: u64, expected_overflow: bool, expected_result: u64) {
        assert_eq!(
            (MockAddress(expected_result), expected_overflow),
            MockAddress(a).overflowing_add(b)
        );
        if expected_overflow {
            assert!(MockAddress(a).checked_add(b).is_none());
            assert!(std::panic::catch_unwind(|| MockAddress(a).unchecked_add(b)).is_err());
        } else {
            assert_eq!(
                Some(MockAddress(expected_result)),
                MockAddress(a).checked_add(b)
            );
            assert_eq!(
                MockAddress(expected_result),
                MockAddress(a).unchecked_add(b)
            );
        }
    }

    #[test]
    fn test_add() {
        // without overflow
        // normal case
        check_add(10, 10, false, 20);
        // edge case
        check_add(std::u64::MAX - 1, 1, false, std::u64::MAX);

        // with overflow
        check_add(std::u64::MAX, 1, true, 0);
    }

    fn check_sub(a: u64, b: u64, expected_overflow: bool, expected_result: u64) {
        assert_eq!(
            (MockAddress(expected_result), expected_overflow),
            MockAddress(a).overflowing_sub(b)
        );
        if expected_overflow {
            assert!(MockAddress(a).checked_sub(b).is_none());
            assert!(MockAddress(a).checked_offset_from(MockAddress(b)).is_none());
            assert!(std::panic::catch_unwind(|| MockAddress(a).unchecked_sub(b)).is_err());
        } else {
            assert_eq!(
                Some(MockAddress(expected_result)),
                MockAddress(a).checked_sub(b)
            );
            assert_eq!(
                Some(expected_result),
                MockAddress(a).checked_offset_from(MockAddress(b))
            );
            assert_eq!(
                MockAddress(expected_result),
                MockAddress(a).unchecked_sub(b)
            );
        }
    }

    #[test]
    fn test_sub() {
        // without overflow
        // normal case
        check_sub(20, 10, false, 10);
        // edge case
        check_sub(1, 1, false, 0);

        // with underflow
        check_sub(0, 1, true, std::u64::MAX);
    }

    #[test]
    fn test_default() {
        assert_eq!(MockAddress::default(), MockAddress(0));
    }

    #[test]
    fn test_bit_and() {
        let a = MockAddress(0x0ff0);
        assert_eq!(a & 0xf00f, MockAddress(0));
    }

    #[test]
    fn test_bit_or() {
        let a = MockAddress(0x0ff0);
        assert_eq!(a | 0xf00f, MockAddress(0xffff));
    }
}
