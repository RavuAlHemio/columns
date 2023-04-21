use std::fmt;

use crate::{FIELD_BLOCK_COUNT, FIELD_HEIGHT_BLOCKS, FIELD_WIDTH_BLOCKS};


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum BlockState {
    #[default] Stationary,
    Descending,
    Gravity,
    Disappearing { counter: usize, sequence: Vec<(u32, u32)> },
}
impl BlockState {
    pub fn is_stationary(&self) -> bool {
        match self {
            Self::Stationary => true,
            _ => false,
        }
    }

    pub fn is_descending(&self) -> bool {
        match self {
            Self::Descending => true,
            _ => false,
        }
    }

    pub fn is_pulled_by_gravity(&self) -> bool {
        match self {
            Self::Gravity => true,
            _ => false,
        }
    }

    pub fn is_disappearing(&self) -> bool {
        match self {
            Self::Disappearing { .. } => true,
            _ => false,
        }
    }

    pub fn disappearing_counter(&self) -> Option<usize> {
        match self {
            Self::Disappearing { counter, .. } => Some(*counter),
            _ => None,
        }
    }

    pub fn disappearing_counter_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Disappearing { counter, .. } => Some(counter),
            _ => None,
        }
    }

    pub fn disappearing_sequence(&self) -> Option<&[(u32, u32)]> {
        match self {
            Self::Disappearing { sequence, .. } => Some(sequence.as_slice()),
            _ => None,
        }
    }
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Block {
    pub color_index: u8,
    pub state: BlockState,
}


#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FieldBlock {
    #[default] Background,
    Block(Block),
}
impl FieldBlock {
    pub fn color_index(&self) -> Option<u8> {
        match self {
            Self::Background => None,
            Self::Block(block) => Some(block.color_index),
        }
    }

    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Self::Block(block) => Some(block),
            _ => None,
        }
    }

    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        match self {
            Self::Block(block) => Some(block),
            _ => None,
        }
    }

    pub fn is_background(&self) -> bool {
        match self {
            Self::Background => true,
            _ => false,
        }
    }

    pub fn is_stationary_block(&self) -> bool {
        match self {
            Self::Block(block) => match block.state {
                BlockState::Stationary => true,
                BlockState::Disappearing { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Field {
    blocks: [FieldBlock; FIELD_BLOCK_COUNT],
}
impl Field {
    pub fn new() -> Self {
        let blocks = std::array::from_fn(|_| FieldBlock::Background);
        Self {
            blocks,
        }
    }

    pub fn block_by_coord(&self, x: u32, y: u32) -> &FieldBlock {
        &self.blocks[usize::try_from(y * FIELD_WIDTH_BLOCKS + x).unwrap()]
    }

    pub fn block_by_coord_mut(&mut self, x: u32, y: u32) -> &mut FieldBlock {
        &mut self.blocks[usize::try_from(y * FIELD_WIDTH_BLOCKS + x).unwrap()]
    }

    /// Returns a reference to the field's blocks.
    pub fn blocks(&self) -> &[FieldBlock] {
        &self.blocks
    }

    /// Returns a mutable reference to the field's blocks.
    pub fn blocks_mut(&mut self) -> &mut [FieldBlock] {
        &mut self.blocks
    }

    /// Returns an iterator over all the (x, y) coordinates of the field.
    pub fn coords() -> FieldCoords { FieldCoords::new() }

    /// Returns a vector of coordinates of the blocks that have the given state, in reverse order.
    pub fn block_coords_with_predicate<F: FnMut(&BlockState) -> bool>(&self, mut pred: F) -> Vec<(u32, u32)> {
        self
            .blocks()
            .iter()
            .zip(Self::coords())
            .rev()
            .filter_map(|(field_block, coords)| field_block.as_block().map(|b| (b, coords)))
            .filter(|(block, _)| pred(&block.state))
            .map(|(_, coords)| coords)
            .collect()
    }

    /// Returns whether the block at the given coordinate hit the bottom of the field or fell on top
    /// of a stationary block.
    pub fn block_at_coord_hit_bottom_or_stationary_block(&self, x: u32, y: u32) -> bool {
        (y == FIELD_HEIGHT_BLOCKS - 1)
        || self.block_by_coord(x, y + 1).is_stationary_block()
    }

    /// Swaps two blocks in the field.
    pub fn swap_blocks(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        unsafe {
            let ptr1 = self.block_by_coord_mut(x1, y1) as *mut _;
            let ptr2 = self.block_by_coord_mut(x2, y2) as *mut _;
            std::ptr::swap(ptr1, ptr2);
        }
    }

    /// Returns the coordinates of the next block if the sequence started by the given block
    /// continues in the given direction.
    pub fn sequence_continues(&self, x: u32, y: u32, dx: i32, dy: i32) -> Option<(u32, u32)> {
        let this_color = self.block_by_coord(x, y).as_block()?.color_index;

        let next_x = i32::try_from(x).unwrap() + dx;
        let next_y = i32::try_from(y).unwrap() + dy;

        if next_x < 0 {
            return None;
        }
        if next_x >= FIELD_WIDTH_BLOCKS.try_into().unwrap() {
            return None;
        }

        if next_y < 0 {
            return None;
        }
        if next_y >= FIELD_HEIGHT_BLOCKS.try_into().unwrap() {
            return None;
        }

        let x2 = u32::try_from(next_x).unwrap();
        let y2 = u32::try_from(next_y).unwrap();

        let neighbor_color = match self.block_by_coord(x2, y2).as_block() {
            Some(block) => block.color_index,
            None => return None,
        };

        if this_color == neighbor_color {
            Some((x2, y2))
        } else {
            None
        }
    }

    /// Finds all the coordinates of the sequence beginning at the given block and continuing in the
    /// given direction.
    pub fn find_sequence(&self, x: u32, y: u32, dx: i32, dy: i32) -> Sequence {
        assert!(dx != 0 || dy != 0);
        assert!(x < FIELD_WIDTH_BLOCKS && y < FIELD_HEIGHT_BLOCKS);

        let mut coords = Vec::new();
        if self.block_by_coord(x, y).as_block().is_none() {
            return Sequence::new(coords, true); // no block here
        };
        coords.push((x, y));
        loop {
            let (last_x, last_y) = *coords.last().unwrap();
            if let Some((x2, y2)) = self.sequence_continues(last_x, last_y, dx, dy) {
                coords.push((x2, y2));
            } else {
                break;
            }
        }

        // check if our sequence can be extended on either side
        let mut sequence_extensible = false;
        let (last_x, last_y) = *coords.last().unwrap();
        let more_x = i32::try_from(last_x).unwrap() + dx;
        let more_y = i32::try_from(last_y).unwrap() + dy;
        if more_x >= 0 && more_x < i32::try_from(FIELD_WIDTH_BLOCKS).unwrap() {
            if more_y >= 0 && more_y < i32::try_from(FIELD_HEIGHT_BLOCKS).unwrap() {
                if self.block_by_coord(more_x.try_into().unwrap(), more_y.try_into().unwrap()).is_background() {
                    sequence_extensible = true;
                }
            }
        }
        if !sequence_extensible {
            // try the beginning
            let (first_x, first_y) = *coords.first().unwrap();
            let less_x = i32::try_from(first_x).unwrap() - dx;
            let less_y = i32::try_from(first_y).unwrap() - dy;
            if less_x >= 0 && less_x < i32::try_from(FIELD_WIDTH_BLOCKS).unwrap() {
                if less_y >= 0 && less_y < i32::try_from(FIELD_HEIGHT_BLOCKS).unwrap() {
                    if self.block_by_coord(less_x.try_into().unwrap(), less_y.try_into().unwrap()).is_background() {
                        sequence_extensible = true;
                    }
                }
            }
        }

        Sequence::new(coords, sequence_extensible)
    }

    /// Gets all sequences on the field as vectors of their blocks' coordinates.
    pub fn get_coordinates_of_sequences<P: FnMut(&Sequence) -> bool>(&self, mut predicate: P) -> Vec<Sequence> {
        let settled_blocks = self.block_coords_with_predicate(|bs| bs.is_stationary());

        let mut sequences = Vec::with_capacity(4);
        for &(x, y) in &settled_blocks {
            // when looking for new sequences, we only look in four directions;
            // to ensure we don't count a sequence multiple times, we ensure there isn't a sequence in
            // the other direction as well
            if self.sequence_continues(x, y, -1, 0).is_none() { // left
                sequences.push(self.find_sequence(x, y, 1, 0)); // right
            }
            if self.sequence_continues(x, y, -1, -1).is_none() { // up-left
                sequences.push(self.find_sequence(x, y, 1, 1)); // down-right
            }
            if self.sequence_continues(x, y, 0, -1).is_none() { // up
                sequences.push(self.find_sequence(x, y, 0, 1)); // down
            }
            if self.sequence_continues(x, y, 1, -1).is_none() { // up-right
                sequences.push(self.find_sequence(x, y, -1, 1)); // down-left
            }

            // ensure our sequences are long enough
            sequences.retain(&mut predicate);
        }

        sequences
    }
}
impl Default for Field {
    fn default() -> Self {
        Field::new()
    }
}
impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\u{250C}")?;
        for _ in 0..FIELD_WIDTH_BLOCKS {
            write!(f, "\u{2500}")?;
        }
        writeln!(f, "\u{2510}")?;

        for y in 0..FIELD_HEIGHT_BLOCKS {
            write!(f, "\u{2502}")?;
            for x in 0..FIELD_WIDTH_BLOCKS {
                match self.block_by_coord(x, y) {
                    FieldBlock::Background => write!(f, " ")?,
                    FieldBlock::Block(block) => write!(f, "{}", block.color_index)?,
                }
            }
            writeln!(f, "\u{2502}")?;
        }

        write!(f, "\u{2514}")?;
        for _ in 0..FIELD_WIDTH_BLOCKS {
            write!(f, "\u{2500}")?;
        }
        writeln!(f, "\u{2518}")?;

        Ok(())
    }
}

pub(crate) struct FieldCoords {
    index: usize,
    length: usize,
    field_width: u32,
}
impl FieldCoords {
    pub fn new() -> Self {
        Self {
            index: 0,
            length: FIELD_BLOCK_COUNT,
            field_width: FIELD_WIDTH_BLOCKS,
        }
    }

    fn coords_for_index(&self, index: u32) -> (u32, u32) {
        let x = index % self.field_width;
        let y = index / self.field_width;
        (x, y)
    }
}
impl Iterator for FieldCoords {
    type Item = (u32, u32);

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.index >= self.length {
            (0, Some(0))
        } else {
            let remaining = self.length - self.index;
            (remaining, Some(remaining))
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }

        let coords = self.coords_for_index(self.index.try_into().unwrap());
        self.index += 1;
        Some(coords)
    }
}
impl ExactSizeIterator for FieldCoords {
}
impl DoubleEndedIterator for FieldCoords {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }

        self.length -= 1;
        let coords = self.coords_for_index(self.length.try_into().unwrap());
        Some(coords)
    }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sequence {
    pub coordinates: Vec<(u32, u32)>,
    pub extensible: bool,
}
impl Sequence {
    pub fn new(
        coordinates: Vec<(u32, u32)>,
        extensible: bool,
    ) -> Self {
        Self {
            coordinates,
            extensible,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::FieldCoords;

    #[test]
    fn test_field_coords() {
        {
            // iterate from beginning
            let mut fc = FieldCoords {
                index: 0,
                length: 8,
                field_width: 2,
            };

            assert_eq!(fc.next(), Some((0, 0)));
            assert_eq!(fc.next(), Some((1, 0)));
            assert_eq!(fc.next(), Some((0, 1)));
            assert_eq!(fc.next(), Some((1, 1)));
            assert_eq!(fc.next(), Some((0, 2)));
            assert_eq!(fc.next(), Some((1, 2)));
            assert_eq!(fc.next(), Some((0, 3)));
            assert_eq!(fc.next(), Some((1, 3)));
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next(), None);
        }

        {
            // iterate from end
            let mut fc = FieldCoords {
                index: 0,
                length: 8,
                field_width: 2,
            };

            assert_eq!(fc.next_back(), Some((1, 3)));
            assert_eq!(fc.next_back(), Some((0, 3)));
            assert_eq!(fc.next_back(), Some((1, 2)));
            assert_eq!(fc.next_back(), Some((0, 2)));
            assert_eq!(fc.next_back(), Some((1, 1)));
            assert_eq!(fc.next_back(), Some((0, 1)));
            assert_eq!(fc.next_back(), Some((1, 0)));
            assert_eq!(fc.next_back(), Some((0, 0)));
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next(), None);
        }

        {
            // iterate from both ends
            let mut fc = FieldCoords {
                index: 0,
                length: 8,
                field_width: 2,
            };

            assert_eq!(fc.next(), Some((0, 0)));
            assert_eq!(fc.next(), Some((1, 0)));
            assert_eq!(fc.next(), Some((0, 1)));
            assert_eq!(fc.next(), Some((1, 1)));
            assert_eq!(fc.next_back(), Some((1, 3)));
            assert_eq!(fc.next_back(), Some((0, 3)));
            assert_eq!(fc.next_back(), Some((1, 2)));
            assert_eq!(fc.next_back(), Some((0, 2)));
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next_back(), None);
            assert_eq!(fc.next_back(), None);
            assert_eq!(fc.next_back(), None);
        }

        {
            // iterate from both ends, intercalated
            let mut fc = FieldCoords {
                index: 0,
                length: 8,
                field_width: 2,
            };

            assert_eq!(fc.next(), Some((0, 0)));
            assert_eq!(fc.next_back(), Some((1, 3)));
            assert_eq!(fc.next(), Some((1, 0)));
            assert_eq!(fc.next_back(), Some((0, 3)));
            assert_eq!(fc.next(), Some((0, 1)));
            assert_eq!(fc.next_back(), Some((1, 2)));
            assert_eq!(fc.next(), Some((1, 1)));
            assert_eq!(fc.next_back(), Some((0, 2)));
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next_back(), None);
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next_back(), None);
            assert_eq!(fc.next(), None);
            assert_eq!(fc.next_back(), None);
        }
    }
}
