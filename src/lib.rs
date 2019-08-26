#![cfg_attr(not(test), no_std)]

use core::cell::Cell;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr;

pub trait PawnExt {
    type Inner;
    fn pawn(&self) -> Ticket<'_, Self::Inner> where Self::Inner: Default {
        self.pawn_with(Default::default())
    }
    fn pawn_with(&self, temp: Self::Inner) -> Ticket<'_, Self::Inner>;
}
impl<T> PawnExt for Cell<T> {
    type Inner = T;
    fn pawn_with(&self, temp: Self::Inner) -> Ticket<'_, Self::Inner> {
        Ticket {
            cell: self,
            value: ManuallyDrop::new(self.replace(temp)),
        }
    }
}

#[derive(Clone)]
pub struct Ticket<'a, T> {
    cell: &'a Cell<T>,
    value: ManuallyDrop<T>,
}

impl<T> Deref for Ticket<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Ticket<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for Ticket<'_, T> {
    fn drop(&mut self) {
        // Safe because we're in the destructor and don't use it after this
        let value = unsafe { ManuallyDrop::into_inner(ptr::read(&mut self.value)) };
        self.cell.replace(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let v = Cell::new(vec![1]);
        v.pawn().push(2);
        {
            let mut t = v.pawn();
            t.push(3);
            t.push(4);
        }
        assert_eq!(v.into_inner(), [1, 2, 3, 4]);
    }

    #[test]
    fn pawn_twice() {
        let v = Cell::new(vec![1]);
        {
            let mut a = v.pawn();
            assert_eq!(a[..], [1]);

            // You can pawn something again without failing,
            // you'll just get something useless.
            let mut b = v.pawn();
            assert_eq!(b[..], []);

            a.push(2);

            b.push(3); // This affects something soon ignored
        }
        assert_eq!(v.into_inner(), [1, 2]);
    }
}
