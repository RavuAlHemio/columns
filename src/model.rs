use crate::{FIELD_BLOCK_COUNT, FIELD_WIDTH_BLOCKS};


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FieldBlock {
    #[default] Background,
    Block { color_index: u8, falling: bool },
}
impl FieldBlock {
    pub fn color_index(&self) -> Option<u8> {
        match self {
            Self::Background => None,
            Self::Block { color_index, .. } => Some(*color_index),
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            Self::Block { .. } => true,
            _ => false,
        }
    }

    pub fn is_falling_block(&self) -> bool {
        match self {
            Self::Block { falling, .. } => *falling,
            _ => false,
        }
    }
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Field {
    blocks: [FieldBlock; FIELD_BLOCK_COUNT],
}
impl Field {
    pub fn block_by_coord(&self, x: u32, y: u32) -> &FieldBlock {
        &self.blocks[usize::try_from(y * FIELD_WIDTH_BLOCKS + x).unwrap()]
    }

    pub fn block_by_coord_mut(&mut self, x: u32, y: u32) -> &mut FieldBlock {
        &mut self.blocks[usize::try_from(y * FIELD_WIDTH_BLOCKS + x).unwrap()]
    }

    pub fn coords_and_blocks<'a>(&'a self) -> CoordsAndBlocks<'a> {
        CoordsAndBlocks { field: self, index: 0 }
    }

    pub fn coords_and_blocks_mut<'a>(&'a mut self) -> CoordsAndBlocksMut<'a> {
        CoordsAndBlocksMut { field: self, index: 0 }
    }
}
impl Default for Field {
    fn default() -> Self {
        Self {
            blocks: [FieldBlock::Background; FIELD_BLOCK_COUNT],
        }
    }
}


macro_rules! define_coords_blocks_iterator {
    ($name:ident $(, $mut:tt)?) => {
        pub(crate) struct $name<'a> {
            field: &'a $($mut)? Field,
            index: usize,
        }
        impl<'a> Iterator for $name<'a> {
            type Item = (i32, i32, &'a $($mut)? FieldBlock);

            fn next(&mut self) -> Option<Self::Item> {
                if self.index >= self.field.blocks.len() {
                    return None;
                }

                let index_i32 = i32::try_from(self.index).unwrap();
                let width_i32 = i32::try_from(FIELD_WIDTH_BLOCKS).unwrap();
        
                let x = index_i32 % width_i32;
                let y = index_i32 / width_i32;
                let item = & $($mut)? self.field.blocks[self.index];

                self.index += 1;

                Some((x, y, item))
            }
        }
    };
}
define_coords_blocks_iterator!(CoordsAndBlocks);
define_coords_blocks_iterator!(CoordsAndBlocksMut, mut);
