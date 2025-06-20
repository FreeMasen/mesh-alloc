use std::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

const fn ffsll(v: usize) -> u32 {
    v.trailing_zeros()
}

const fn word_bytes() -> usize {
    std::mem::size_of::<usize>()
}

const fn word_bits() -> usize {
    word_bytes() * 8
}

const fn static_log(v: usize) -> usize {
    if v == 1 {
        0
    } else if v == 2 {
        1
    } else if v > 1 {
        static_log(v / 2) + 1
    } else {
        0
    }
}

const fn word_bit_shift() -> usize {
    static_log(word_bits())
}

const fn rep_size(bit_ct: usize) -> usize {
    word_bits() * ((bit_ct + word_bits() - 1) / word_bits()) / 8
}

const fn word_count(byte_ct: usize) -> usize {
    byte_ct / word_bytes()
}

const fn get_mask(pos: usize) -> usize {
    1 << pos
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ItemPosition {
    item: usize,
    pos: usize,
}
/// Compute the position of an item in a bitmap The returned values
/// are the index of the collection of `usize` entries and then the
/// mask for the bit within that `usize`
fn compute_item_position(index: usize) -> ItemPosition {
    let item = index.wrapping_shr(word_bit_shift() as u32);
    println!("{}>>{}->{item}", index, word_bit_shift());
    let pos = index & (word_bits() - 1);
    println!("{}&{}-1->{pos}", index, word_bits());

    debug_assert!(pos == index - (item << word_bit_shift()));
    ItemPosition { item, pos }
}

pub trait BitmapBase {
    fn invert(&mut self);
    fn set_all(&mut self, bit_ct: usize);
    fn set_at(&mut self, item: usize, pos: usize) -> bool;
    fn unset_at(&mut self, item: usize, pos: usize) -> bool;
    fn in_use_ct(&self) -> u32;
}

pub trait MapBits: BitmapBase {
    fn byte_ct(&self) -> usize;
    fn bit_ct(&self) -> usize;
    fn set_first_empty_with(&mut self, starting_at: usize) -> usize;
    fn set_first_empty(&mut self) -> usize {
        self.set_first_empty_with(0)
    }
}

pub struct AtomicBitmap<const SIZE: usize>([AtomicUsize; SIZE]);

impl<const SIZE: usize> AtomicBitmap<SIZE> {
    pub const fn new() -> Self {
        let inner = [const { AtomicUsize::new(0) }; SIZE];
        Self(inner)
    }
}

impl<const SIZE: usize> BaseBitmapper for AtomicBitmap<SIZE> {
    type Chunk = AtomicUsize;

    fn set_and_exchange_all(&mut self, old_bits: &mut [usize], new_bits: &[usize]) {
        for i in 0..SIZE {
            old_bits[i] = self.0[i].swap(new_bits[i], std::sync::atomic::Ordering::AcqRel);
        }
    }

    fn set_at(&mut self, item: usize, pos: usize) -> Result<(), Error> {
        let mask = get_mask(pos);
        let old_value = self.0[item].load(std::sync::atomic::Ordering::Relaxed);
        while self.0[item]
            .compare_exchange_weak(
                old_value,
                old_value | mask,
                std::sync::atomic::Ordering::Release,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {}
        // return !(oldValue & mask);
        if (old_value & mask) != 0 {
            return Err(Error::SetFailed(item, pos));
        }
        Ok(())
    }

    fn set_all(&mut self) -> Result<(), Error> {
        for chunk in self.0.iter_mut() {
            chunk.store(usize::MAX, Ordering::Relaxed);
        }
        Ok(())
    }

    fn unset_at(&mut self, item: usize, pos: usize) -> Result<(), Error> {
        let mask = get_mask(pos);

        let old_value = self.0[item].load(std::sync::atomic::Ordering::Relaxed);
        while self.0[item]
            .compare_exchange_weak(
                old_value,
                old_value & !mask,
                std::sync::atomic::Ordering::Release,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {}

        if (old_value & mask) == 0 {
            return Err(Error::UnsetFailed(item, pos));
        }
        Ok(())
    }

    fn clear(&mut self) {
        for chunk in self.0.iter_mut() {
            chunk.store(0, Ordering::Relaxed);
        }
    }

    fn bit_ct(&self) -> usize {
        SIZE * word_bits()
    }

    fn bits(&self) -> &[Self::Chunk] {
        &self.0
    }

    fn bits_at_index(&self, idx: usize) -> usize {
        self.0[idx].load(Ordering::Relaxed)
    }

    fn bits_mut(&mut self) -> &mut [Self::Chunk] {
        &mut self.0
    }

    fn in_use_ct(&self) -> usize {
        self.0
            .iter()
            .map(|v| v.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }
}

impl<const SIZE: usize> Debug for AtomicBitmap<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_list();
        for i in &self.0 {
            let v = i.load(Ordering::Relaxed);
            d.entry(&format_args!("{v:0>width$b}", v = v, width = word_bits()));
        }
        d.finish()
    }
}

pub struct RelaxedFixedBitmapBase<const SIZE: usize>([usize; SIZE]);
impl<const SIZE: usize> RelaxedFixedBitmapBase<SIZE> {
    fn new() -> Self {
        Self([0; SIZE])
    }
    fn invert(&mut self) {
        for byte in self.0.iter_mut() {
            *byte = !*byte;
        }
    }
    fn set_all(&mut self, mut bit_ct: usize) {
        for i in 0..bit_ct {
            if bit_ct > 64 {
                self.0[i] = usize::MAX;
                bit_ct -= 64;
            } else {
                self.0[i] = (1 << bit_ct) - 1;
                bit_ct = 0;
            }
        }
    }

    fn in_use_ct(&self) -> u32 {
        self.0.iter().map(|&v| v.count_ones()).sum()
    }
}

impl<const SIZE: usize> BaseBitmapper for RelaxedFixedBitmapBase<SIZE> {
    type Chunk = usize;

    fn set_at(&mut self, item: usize, pos: usize) -> Result<(), Error> {
        let mask = get_mask(pos);

        let old_value = self.0[item];
        self.0[item] = old_value | mask;
        if (old_value & mask) != 0 {
            return Err(Error::SetFailed(item, pos));
        }
        Ok(())
    }

    fn set_all(&mut self) -> Result<(), Error> {
        for chunk in self.0.iter_mut() {
            *chunk = usize::MAX;
        }
        Ok(())
    }

    fn unset_at(&mut self, item: usize, pos: usize) -> Result<(), Error> {
        let mask = get_mask(pos);

        let old_value = self.0[item];
        self.0[item] = old_value & !mask;

        if (old_value & mask) == 0 {
            return Err(Error::UnsetFailed(item, pos));
        }
        Ok(())
    }

    fn clear(&mut self) {
        for chunk in self.0.iter_mut() {
            *chunk = 0;
        }
    }

    fn bit_ct(&self) -> usize {
        SIZE * word_bits()
    }

    fn bits(&self) -> &[Self::Chunk] {
        &self.0
    }

    fn set_and_exchange_all(&mut self, _: &mut [usize], _: &[usize]) {
        // not sure why this is only implemented for teh Atomic base...
    }

    fn bits_at_index(&self, idx: usize) -> usize {
        println!("bits_at_index {:?}[{idx}]->{}", self.0, self.0[idx]);
        self.0[idx]
    }

    fn bits_mut(&mut self) -> &mut [Self::Chunk] {
        &mut self.0
    }

    fn in_use_ct(&self) -> usize {
        self.0.iter().map(|&v| v.count_ones() as usize).sum()
    }
}

impl<const SIZE: usize> std::fmt::Debug for RelaxedFixedBitmapBase<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_list();
        for v in self.0.iter() {
            d.entry(&format_args!("{v:0>width$b}", v = v, width = word_bits()));
        }
        d.finish()
    }
}

pub struct RelaxedBitmapBase(Vec<usize>);

impl std::fmt::Debug for RelaxedBitmapBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_list();
        for &i in self.0.iter() {
            d.entry(&format_args!("{v:0>width$b}", v = i, width = word_bits()));
        }
        d.finish()
    }
}

impl RelaxedBitmapBase {
    pub fn new(size: usize) -> Self {
        Self(vec![0; rep_size(size) / word_bytes()])
    }

    pub fn invert(&mut self) {
        for byte in self.0.iter_mut() {
            *byte = !*byte;
        }
    }

    pub fn set_all(&mut self, mut bit_ct: usize) {
        for i in 0..bit_ct {
            if bit_ct > 64 {
                self.0[i] = usize::MAX;
                bit_ct -= 64;
            } else {
                self.0[i] = (1 << bit_ct) - 1;
                bit_ct = 0;
            }
        }
    }
}

impl BaseBitmapper for RelaxedBitmapBase {
    type Chunk = usize;
    fn set_at(&mut self, item: usize, pos: usize) -> Result<(), Error> {
        let mask = get_mask(pos);

        let old_value = self.0[item];
        self.0[item] = old_value | mask;

        if (old_value & mask) != 0 {
            return Err(Error::SetFailed(item, pos));
        }
        Ok(())
    }

    fn set_all(&mut self) -> Result<(), Error> {
        for chunk in self.0.iter_mut() {
            *chunk = usize::MAX;
        }
        Ok(())
    }

    fn unset_at(&mut self, item: usize, pos: usize) -> Result<(), Error> {
        let mask = get_mask(pos);

        let old_value = self.0[item];
        self.0[item] = old_value & !mask;

        if (old_value & mask) == 0 {
            return Err(Error::SetFailed(item, pos));
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn bit_ct(&self) -> usize {
        word_bits() * self.0.len()
    }

    fn bits(&self) -> &[Self::Chunk] {
        &self.0
    }

    fn bits_mut(&mut self) -> &mut [Self::Chunk] {
        &mut self.0
    }

    fn set_and_exchange_all(&mut self, _: &mut [usize], _: &[usize]) {
        // not sure why this is only implemented for teh Atomic base...
    }

    fn bits_at_index(&self, idx: usize) -> usize {
        println!("bits_at_index {:?}[{idx}]->{:b}", self.0, self.0[idx]);
        self.0[idx]
    }

    fn in_use_ct(&self) -> usize {
        self.0.iter().map(|&v| v.count_ones() as usize).sum()
    }
}

pub type RelaxedFixedBitmap = Bitmap<RelaxedFixedBitmapBase<4>>;
pub type RelaxedBitmap = Bitmap<RelaxedBitmapBase>;
pub struct Bitmap<T = AtomicBitmap<4>>(T);

impl<T> std::fmt::Debug for Bitmap<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Bitmap").field(&self.0).finish()
    }
}

impl Bitmap<()> {
    pub fn new_relaxed(bit_ct: usize) -> RelaxedBitmap {
        Bitmap(RelaxedBitmapBase(vec![
            0;
            dbg!(rep_size(bit_ct) / word_bytes())
        ]))
    }

    pub fn new_relaxed_fixed() -> RelaxedFixedBitmap {
        const DEFAULT_SIZE: usize = chunks_for(256);
        Self::new_relaxed_fixed_sized::<DEFAULT_SIZE>()
    }

    pub fn new_relaxed_fixed_sized<const SIZE: usize>() -> Bitmap<RelaxedFixedBitmapBase<SIZE>> {
        Bitmap(RelaxedFixedBitmapBase([0; SIZE]))
    }

    pub fn new_atomic() -> Bitmap<AtomicBitmap<4>> {
        const DEFAULT_SIZE: usize = chunks_for(256);
        Self::new_atomic_sized::<DEFAULT_SIZE>()
    }

    pub fn new_atomic_sized<const SIZE: usize>() -> Bitmap<AtomicBitmap<SIZE>> {
        Bitmap(AtomicBitmap::new())
    }
}

pub const fn chunks_for(bits: usize) -> usize {
    rep_size(bits) / word_bytes()
}

pub trait BaseBitmapper {
    type Chunk;
    fn clear(&mut self);
    fn bit_ct(&self) -> usize;
    fn set_at(&mut self, item: usize, offset: usize) -> Result<(), Error>;
    fn set_all(&mut self) -> Result<(), Error>;
    fn unset_at(&mut self, item: usize, offset: usize) -> Result<(), Error>;
    fn bits(&self) -> &[Self::Chunk];
    fn bits_mut(&mut self) -> &mut [Self::Chunk];
    fn bits_at_index(&self, idx: usize) -> usize;
    fn set_and_exchange_all(&mut self, old: &mut [usize], new: &[usize]);
    fn in_use_ct(&self) -> usize;
}

impl<T> Bitmapper for Bitmap<T>
where
    T: BaseBitmapper,
{
    type Inner = T;
    fn bit_ct(&self) -> usize {
        self.0.bit_ct()
    }

    fn set_first_empty_with(&mut self, idx: usize) -> Result<usize, Error> {
        debug_assert!(idx < self.bit_ct());
        let ItemPosition { item, mut pos } = compute_item_position(idx);
        let words = self.byte_ct();
        for i in item..words {
            let bits = self.0.bits_at_index(i);

            if bits == usize::MAX {
                pos = 0;
                continue;
            }
            debug_assert!(
                pos <= (word_bits() - 1),
                "expected {pos} to be < {}",
                word_bits()
            );
            let mut unset_bits = !bits;
            debug_assert!(unset_bits != 0);

            // if the offset is 3, we want to mark the first 3 bits as 'set'
            // or 'unavailable'.
            unset_bits &= !((1 << pos) - 1);

            // if, after we've masked off everything below our offset there
            // are no free bits, continue
            if unset_bits == 0 {
                pos = 0;
                continue;
            }
            let off2 = ffsll(unset_bits) as usize;

            if self.0.set_at(i, off2).is_err() {
                pos += 1;
                continue;
            }
            return Ok(word_bits() * i + off2);
        }
        panic!("mesh: bitmap completely full, aborting.")
    }

    fn set_all(&mut self) -> Result<(), Error> {
        self.0.set_all()
    }

    fn try_set(&mut self, idx: usize) -> Result<(), Error> {
        // debug_assert!(idx < self.bit_ct());
        let ItemPosition { item, pos } = compute_item_position(idx);
        self.0.set_at(item, pos)
    }

    fn try_unset(&mut self, idx: usize) -> Result<(), Error> {
        debug_assert!(idx < self.bit_ct());
        let ItemPosition { item, pos } = compute_item_position(idx);
        self.0.unset_at(item, pos)
    }

    fn is_set(&self, idx: usize) -> bool {
        debug_assert!(idx < self.bit_ct());
        let ItemPosition { item, pos } = compute_item_position(idx);
        let bits = self.0.bits_at_index(item);
        let mask = get_mask(pos);
        bits & mask > 0
    }

    fn bits(&self) -> &[T::Chunk] {
        self.0.bits()
    }

    fn bits_mut(&mut self) -> &mut [T::Chunk] {
        self.0.bits_mut()
    }

    fn lowest_bit_set_with(&self, start: usize) -> usize {
        debug_assert!(start < self.bit_ct());
        let ItemPosition {
            item: start_word,
            pos: mut start_off,
        } = compute_item_position(start);
        let word_ct = self.byte_ct() / size_of::<usize>();
        for i in start_word..word_ct {
            let mask = !((1 << start_off) - 1);
            let bits = self.0.bits_at_index(i) & mask;
            start_off = 0;
            if bits == 0 {
                continue;
            }
            let off = ffsll(bits) as usize;
            let bit = word_bits() * i + off;
            return bit.min(self.bit_ct());
        }
        self.bit_ct()
    }

    fn highest_bit_set_with(&self, start: usize) -> usize {
        println!("start: {start} {}", self.byte_ct());
        // debug_assert!(start < self.bit_ct());
        let ItemPosition {
            item: start_word,
            pos: mut start_off,
        } = compute_item_position(start);
        println!("position: {start_word} {start_off}");
        let mut i = start_word;
        loop {
            //uint64_t mask = (1UL << (startOff + 1)) - 1;
            let mut mask = (1usize.wrapping_shl(start_off as u32 + 1)) - 1;
            println!("loop-top: {i} mask: {mask} start_off: {start_off}");
            if start_off == (word_bits() - 1) {
                mask = usize::MAX;
            }
            let bits = self.0.bits_at_index(i); // & mask;
            println!("{bits:0>64b}\n{mask:0>64b}");
            let bits = bits & mask;
            start_off = size_of::<usize>() - 1;
            eprintln!("bits: {bits}, {start}, {start_word}, {start_off}");
            if bits == 0 {
                if i == 0 {
                    break;
                }
                i -= 1;
                continue;
            }

            let off = (word_bits() - bits.leading_zeros() as usize) - 1;
            let bit = word_bits() * i + off;
            eprintln!("bit: {bit}, {off}");
            return bit.min(self.bit_ct());
        }
        0
    }

    fn set_and_exchange_all(&mut self, old: &mut [usize], new: &[usize]) {
        self.0.set_and_exchange_all(old, new);
    }

    fn in_use_ct(&self) -> usize {
        self.0.in_use_ct()
    }
}

pub trait Bitmapper {
    type Inner: BaseBitmapper;
    fn byte_ct(&self) -> usize {
        rep_size(self.bit_ct())
    }

    fn bit_ct(&self) -> usize;

    fn set_first_empty(&mut self) -> Result<usize, Error> {
        self.set_first_empty_with(0)
    }
    fn set_first_empty_with(&mut self, idx: usize) -> Result<usize, Error>;

    fn try_set(&mut self, idx: usize) -> Result<(), Error>;

    fn try_unset(&mut self, idx: usize) -> Result<(), Error>;

    fn set_all(&mut self) -> Result<(), Error>;

    fn is_set(&self, idx: usize) -> bool;

    fn bits(&self) -> &[<Self::Inner as BaseBitmapper>::Chunk];

    fn bits_mut(&mut self) -> &mut [<Self::Inner as BaseBitmapper>::Chunk];

    fn lowest_bit_set(&self) -> usize {
        self.lowest_bit_set_with(0)
    }

    fn lowest_bit_set_with(&self, start: usize) -> usize;

    fn highest_bit_set(&self) -> usize {
        self.highest_bit_set_with(0)
    }

    fn highest_bit_set_with(&self, start: usize) -> usize;

    fn set_and_exchange_all(&mut self, old: &mut [usize], new: &[usize]);

    fn in_use_ct(&self) -> usize;
}

impl<'a, T> IntoIterator for &'a Bitmap<T>
where
    T: BaseBitmapper,
{
    type Item = usize;
    type IntoIter = BitmapIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        BitmapIter {
            idx: 0,
            inner: self,
        }
    }
}

pub struct BitmapIter<'a, T> {
    idx: usize,
    inner: &'a Bitmap<T>,
}

impl<'a, T> Iterator for BitmapIter<'a, T>
where
    T: BaseBitmapper,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.bit_ct() {
            return None;
        }
        let item = self.inner.lowest_bit_set_with(self.idx);
        // protection against the lowest bit set will returning
        // the max index when empty
        if item >= self.inner.bit_ct() || !self.inner.is_set(item) {
            self.idx = self.inner.bit_ct();
            return None;
        }
        self.idx = item.saturating_add(1);
        Some(item)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to unset at index: {0} offset {1}")]
    UnsetFailed(usize, usize),
    #[error("Failed to set at index: {0} offset {1}")]
    SetFailed(usize, usize),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::{rngs::ThreadRng, Rng};

    use crate::bitmap;

    use super::*;

    #[test]
    fn rep_sizes() {
        assert_eq!(0, rep_size(0));
        assert_eq!(word_bytes(), rep_size(1));
        assert_eq!(word_bytes(), rep_size(word_bits()));
        assert_eq!(word_bytes() * 2, rep_size(word_bits() + 1));
        assert_eq!(word_bytes() * 2, rep_size(word_bits() * 2));
        assert_eq!(word_bytes() * 3, rep_size(word_bits() * 2 + 1));
        assert_eq!(word_bytes() * 3, rep_size(word_bits() * 3));
        assert_eq!(word_bytes() * 4, rep_size(word_bits() * 3 + 1));
        assert_eq!(word_bytes() * 4, rep_size(word_bits() * 4));
        assert_eq!(word_bytes() * 5, rep_size(word_bits() * 4 + 1));
        assert_eq!(word_bytes() * 5, rep_size(word_bits() * 5));
        assert_eq!(word_bytes() * 6, rep_size(word_bits() * 5 + 1));
        assert_eq!(word_bytes() * 6, rep_size(word_bits() * 6));
        assert_eq!(word_bytes() * 7, rep_size(word_bits() * 6 + 1));
        assert_eq!(word_bytes() * 7, rep_size(word_bits() * 7));
        assert_eq!(word_bytes() * 8, rep_size(word_bits() * 7 + 1));
        assert_eq!(word_bytes() * 8, rep_size(word_bits() * 8));
    }

    #[test]
    fn compute_item_position_tests() {
        for i in 0..10 {
            let min = i * word_bits();
            for j in 0..word_bits() {
                assert_eq!(
                    ItemPosition { item: i, pos: j },
                    compute_item_position(min + j)
                );
            }
        }
    }

    #[test]
    fn lowest_bit_set_at() {
        let mut bits = Bitmap::new_relaxed(128);
        bits.try_set(6).unwrap();
        assert_eq!(6, bits.lowest_bit_set());
        assert_eq!(6, bits.lowest_bit_set_with(5));
        assert_eq!(6, bits.lowest_bit_set_with(6));
        assert_eq!(128, bits.lowest_bit_set_with(7));
        bits.try_set(123).unwrap();
        assert_eq!(123, bits.lowest_bit_set_with(7));
    }

    #[test]
    fn highest_bit_set_at() {
        // mesh::internal::RelaxedBitmap bits{128};
        let mut bits = Bitmap::new_relaxed(128);
        // bits.tryToSet(6);
        bits.try_set(6).unwrap();
        // ASSERT_EQ(0UL, bits.highestSetBitBeforeOrAt(0));
        assert_eq!(0, bits.highest_bit_set_with(0));
        // ASSERT_EQ(0UL, bits.highestSetBitBeforeOrAt(5));
        assert_eq!(0, bits.highest_bit_set_with(5));

        // ASSERT_EQ(6UL, bits.highestSetBitBeforeOrAt(6));
        assert_eq!(6, bits.highest_bit_set_with(6));

        // ASSERT_EQ(6UL, bits.highestSetBitBeforeOrAt(7));
        assert_eq!(6, bits.highest_bit_set_with(7));
        // ASSERT_EQ(6UL, bits.highestSetBitBeforeOrAt(127));
        assert_eq!(6, bits.highest_bit_set_with(127));
        // bits.tryToSet(123);
        bits.try_set(123).unwrap();
        println!("{bits:#?}");
        // ASSERT_EQ(123UL, bits.highestSetBitBeforeOrAt(127));
        assert_eq!(123, bits.highest_bit_set_with(127));
    }

    #[test]
    fn set_and_exchange_all() {
        const SIZE: usize = 128 / word_bits();
        //   const auto maxCount = 128;
        //   mesh::internal::Bitmap bitmap{maxCount};
        let mut bitmap = Bitmap::new_atomic_sized::<SIZE>();

        //   bitmap.tryToSet(3);
        //   bitmap.tryToSet(4);
        //   bitmap.tryToSet(127);
        bitmap.try_set(3).unwrap();
        bitmap.try_set(4).unwrap();
        bitmap.try_set(127).unwrap();

        //   mesh::internal::RelaxedFixedBitmap newBitmap{maxCount};
        let mut new_bitmap = Bitmap::new_relaxed_fixed_sized::<SIZE>();
        //   newBitmap.setAll(maxCount);
        new_bitmap.set_all();
        //   mesh::internal::RelaxedFixedBitmap localBits{maxCount};
        let mut local_bitmap = Bitmap::new_relaxed_fixed_sized::<SIZE>();
        //   bitmap.setAndExchangeAll(localBits.mut_bits(), newBitmap.bits());
        bitmap.set_and_exchange_all(local_bitmap.bits_mut(), new_bitmap.bits());
        //   localBits.invert();
        local_bitmap.0.invert();
        //   for (auto const &i : localBits) {
        for i in 0..bitmap.bit_ct() {
            //     if (i >= maxCount) {
            //       break;
            //     }
            //     ASSERT_TRUE(bitmap.isSet(i));
            assert!(bitmap.is_set(i), "bitmap index {i} was unset: {bitmap:#?}");
            //     ASSERT_TRUE(newBitmap.isSet(i));
            assert!(
                new_bitmap.is_set(i),
                "new_bitmap index {i} was unset: {new_bitmap:#?}"
            );
            //     switch (i) {
            match i {
                //     case 3:
                //     case 4:
                //     case 127:
                3 | 4 | 127 => {
                    //       ASSERT_FALSE(localBits.isSet(i));
                    //       break;
                    assert!(
                        !local_bitmap.is_set(i),
                        "expected local_bitmap index {i} to be unset: {local_bitmap:#?}"
                    );
                }
                //     default:
                _ => {
                    //       ASSERT_TRUE(localBits.isSet(i));
                    assert!(
                        local_bitmap.is_set(i),
                        "expected local_bitmap index {i} to be set: {local_bitmap:#?}"
                    );
                    //       break;
                    //     }
                    //   }
                }
            }
        }
        // } */
    }

    // TEST(BitmapTest, SetAll) {
    #[test]
    fn set_all() {
        //     const auto maxCount = 88;
        const SIZE: usize = 88 / word_bits();
        //     uint64_t bits1[4] = {0, 0, 0, 0};
        //     mesh::internal::RelaxedBitmap bitmap1{maxCount, reinterpret_cast<char *>(bits1), false};
        let mut bitmap1 = Bitmap::new_relaxed(SIZE);
        //     for (size_t i = 0; i < maxCount; i++) {
        for i in 0..SIZE {
            //       bitmap1.tryToSet(i);
            bitmap1.try_set(i).unwrap();
            //     }
        }

        //     uint64_t bits2[4] = {0, 0, 0, 0};
        //     mesh::internal::RelaxedBitmap bitmap2{maxCount, reinterpret_cast<char *>(bits2), false};
        let mut bitmap2 = Bitmap::new_relaxed(SIZE);
        //     bitmap2.setAll(maxCount);
        bitmap2.set_all();

        //     for (size_t i = 0; i < maxCount; i++) {
        for i in 0..SIZE {
            //       ASSERT_TRUE(bitmap1.isSet(i));
            assert!(
                bitmap1.is_set(i),
                "Expected bitmap1 index {i} to be set: {bitmap1:#?}"
            );
            //       ASSERT_TRUE(bitmap2.isSet(i));
            assert!(
                bitmap2.is_set(i),
                "Expected bitmap2 index {i} to be set: {bitmap2:#?}"
            );
            //     }
        }
        //   }
    }

    // TEST(BitmapTest, SetGet) {

    fn run_get_set_test<T>(mut bm: Bitmap<T>, rng: &mut ThreadRng)
    where
        T: BaseBitmapper + Debug,
    {
        const NTRIALS: usize = 1000;
        let mut rng: Vec<_> = (0..bm.bit_ct())
            .into_iter()
            .map(|_| rng.random::<bool>())
            .collect();
        for i in 0..NTRIALS {
            println!("trial: {i}");
            for (i, b) in rng.iter().copied().enumerate() {
                if b {
                    bm.try_set(i).unwrap_or_else(|e| {
                        panic!("failed to set index {i} {e}: {bm:#?}");
                    });
                } else {
                    assert!(!bm.is_set(i), "expected index {i} to be unset for {bm:#?}");
                }
            }
            for (i, b) in rng.iter().copied().enumerate() {
                if b {
                    assert!(bm.is_set(i));
                    bm.try_unset(i).unwrap();
                } else {
                    assert!(!bm.is_set(i));
                }
            }
        }
    }

    #[test]
    fn set_get() {
        let mut rng = rand::rng();
        const TWO: usize = chunks_for(2);
        const FOUR: usize = chunks_for(4);
        const EIGHT: usize = chunks_for(8);
        const SIXTEEN: usize = chunks_for(16);
        const THIRTY_TWO: usize = chunks_for(32);
        const SIXTY_FOUR: usize = chunks_for(64);
        const ONE_TWENTY_EIGHT: usize = chunks_for(128);
        const TWO_FIFTY_SIX: usize = chunks_for(256);
        run_get_set_test(Bitmap::new_atomic_sized::<TWO>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<FOUR>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<EIGHT>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<SIXTEEN>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<THIRTY_TWO>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<SIXTY_FOUR>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<ONE_TWENTY_EIGHT>(), &mut rng);
        run_get_set_test(Bitmap::new_atomic_sized::<TWO_FIFTY_SIX>(), &mut rng);
    }

    #[test]
    fn set_get_relaxed() {
        let mut rng = rand::rng();
        run_get_set_test(Bitmap::new_relaxed(2), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(4), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(8), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(16), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(32), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(64), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(128), &mut rng);
        run_get_set_test(Bitmap::new_relaxed(256), &mut rng);
    }

    #[test]
    fn set_get_relaxed_fixed() {
        const TWO: usize = chunks_for(2);
        const FOUR: usize = chunks_for(4);
        const EIGHT: usize = chunks_for(8);
        const SIXTEEN: usize = chunks_for(16);
        const THIRTY_TWO: usize = chunks_for(32);
        const SIXTY_FOUR: usize = chunks_for(64);
        const ONE_TWENTY_EIGHT: usize = chunks_for(128);
        const TWO_FIFTY_SIX: usize = chunks_for(256);
        let mut rng = rand::rng();
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<TWO>(), &mut rng);
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<FOUR>(), &mut rng);
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<EIGHT>(), &mut rng);
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<SIXTEEN>(), &mut rng);
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<THIRTY_TWO>(), &mut rng);
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<SIXTY_FOUR>(), &mut rng);
        run_get_set_test(
            Bitmap::new_relaxed_fixed_sized::<ONE_TWENTY_EIGHT>(),
            &mut rng,
        );
        run_get_set_test(Bitmap::new_relaxed_fixed_sized::<TWO_FIFTY_SIX>(), &mut rng);
    }

    // TEST(BitmapTest, Builtins) {
    #[test]
    fn test_builtins() {
        //     mesh::internal::Bitmap b{256};
        let mut b = Bitmap::new_atomic();

        //     uint64_t i = b.setFirstEmpty();
        let i = b.set_first_empty().unwrap();
        //     ASSERT_EQ(i, 0ULL);
        assert_eq!(i, 0);

        //     b.unset(i);
        b.try_unset(i).unwrap();
        //     static constexpr uint64_t curr = 66;
        //     for (size_t i = 0; i < curr; i++) {
        const LOOP_TOP: usize = 66;
        for i in 0..LOOP_TOP {
            //     b.tryToSet(i);
            b.try_set(i);
            //     }
        }

        //     i = b.setFirstEmpty();
        let i2 = b.set_first_empty().unwrap();
        //     ASSERT_EQ(i, curr);
        assert_eq!(i2, LOOP_TOP);
        //     for (size_t i = 0; i < curr; i++) {
        for i in 0..LOOP_TOP {
            //     b.unset(i);
            b.try_unset(i).unwrap();
            //     }
        }

        //     i = b.setFirstEmpty();
        let i3 = b.set_first_empty().unwrap();
        //     ASSERT_EQ(i, 0ULL);
        assert_eq!(i3, 0);
        //     i = b.setFirstEmpty(4);
        let i4 = b.set_first_empty_with(4).unwrap();
        //     ASSERT_EQ(i, 4ULL);
        assert_eq!(i4, 4);
        //     i = b.setFirstEmpty(111);
        let i5 = b.set_first_empty_with(111).unwrap();
        //     ASSERT_EQ(i, 111ULL);
        assert_eq!(i5, 111);
        // }
    }

    #[test]
    fn iter() {
        // TEST(BitmapTest, Iter) {
        //     mesh::internal::RelaxedBitmap b{512};
        let mut b = Bitmap::new_relaxed(512);
        //     b.tryToSet(0);
        b.try_set(0).unwrap();
        //     b.tryToSet(200);
        b.try_set(200).unwrap();
        //     b.tryToSet(500);
        b.try_set(500).unwrap();
        //     std::unordered_map<size_t, bool> bits;
        let mut bits = HashMap::new();

        //     ASSERT_EQ(bits.size(), 0ULL);
        //     size_t n = 0;
        let mut n = 0;

        //     for (auto const &off : b) {
        for bit_idx in b.into_iter() {
            //         bits[off] = true;
            bits.insert(bit_idx, true);
            //         n++;
            n += 1;
            //     }
        }

        //     ASSERT_EQ(n, 3ULL);
        assert_eq!(n, 3);
        //     ASSERT_EQ(bits.size(), 3ULL);
        assert_eq!(bits.len(), 3);
        //     ASSERT_EQ(bits[0], true);
        assert_eq!(bits.get(&0), Some(&true));
        //     ASSERT_EQ(bits[200], true);
        assert_eq!(bits.get(&200), Some(&true));
        //     ASSERT_EQ(bits[500], true);
        assert_eq!(bits.get(&500), Some(&true));
        //     ASSERT_EQ(bits[1], false);
        assert_eq!(bits.get(&1), None);
        //     }
    }

    // TEST(BitmapTest, Iter2) {
    #[test]
    fn iter2() {
        //     mesh::internal::RelaxedBitmap b{512};
        let mut b = Bitmap::new_relaxed(512);
        //     b.tryToSet(200);
        b.try_set(200).unwrap();
        //     b.tryToSet(500);
        b.try_set(500).unwrap();

        //     std::unordered_map<size_t, bool> bits;
        let mut bits = HashMap::new();

        //     ASSERT_EQ(bits.size(), 0ULL);

        //     size_t n = 0;
        let mut n = 0;
        //     for (auto const &off : b) {
        for off in b.into_iter() {
            //       bits[off] = true;
            bits.insert(off, true);
            //       n++;
            n += 1;
            //     }
        }

        //     ASSERT_EQ(n, 2ULL);
        assert_eq!(n, 2);
        //     ASSERT_EQ(bits.size(), 2ULL);
        assert_eq!(bits.len(), 2);

        //     ASSERT_EQ(bits[200], true);
        assert_eq!(bits.get(&200), Some(&true));
        //     ASSERT_EQ(bits[500], true);
        assert_eq!(bits.get(&500), Some(&true));

        //     ASSERT_EQ(bits.find(0), bits.end());
        assert_eq!(bits.iter().find(|(_, b)| !*b), None);
        //   }
    }

    //   TEST(BitmapTest, SetHalf) {
    #[test]
    fn set_half() {
        //     for (size_t i = 2; i <= 2048; i *= 2) {
        let mut i = 2;
        while i <= 2048 {
            //       const auto nBits = i;
            let n_bits = i;
            //       mesh::internal::RelaxedBitmap bitmap{nBits};
            let mut bm = Bitmap::new_relaxed(n_bits);
            //       ASSERT_TRUE(bitmap.byteCount() >= nBits / 8);
            assert!(bm.byte_ct() >= n_bits / 8);
            //       for (size_t i = 0; i < nBits / 2; i++) {
            for i in 0..(n_bits / 2) {
                //         bitmap.tryToSet(i);
                bm.try_set(i).unwrap();
                //         ASSERT_TRUE(bitmap.isSet(i));
                assert!(bm.is_set(i));
                //         ASSERT_TRUE(bitmap.inUseCount() == i + 1);
                assert!(bm.in_use_ct() == i + 1);
                //       }
            }
            //       ASSERT_TRUE(bitmap.isSet(0));
            assert!(bm.is_set(0));
            //       ASSERT_TRUE(bitmap.inUseCount() == nBits / 2);
            assert!(bm.in_use_ct() == n_bits / 2);
            //     }
            i *= 2;
        }
        //   }
    }
}
