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
}
impl Default for Field {
    fn default() -> Self {
        Field::new()
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
