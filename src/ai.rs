use std::collections::VecDeque;

use crate::{FIELD_HEIGHT_BLOCKS, FIELD_WIDTH_BLOCKS, MINIMUM_SEQUENCE};
use crate::model::{BlockState, Field, FieldBlock};


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BestMove {
    pub column: u32,
    pub rotate_count: usize,
}


fn rotate_descending_blocks(field: &mut Field, count: usize) {
    let desc_block_coords = field
        .block_coords_with_predicate(|b| b.is_descending());
    if desc_block_coords.len() == 0 {
        return;
    }

    for _ in 0..count {
        let mut desc_blocks: VecDeque<FieldBlock> = desc_block_coords.iter()
            .map(|&(x, y)| field.block_by_coord(x, y).clone())
            .collect();
        let front_block = desc_blocks.pop_front().unwrap();
        desc_blocks.push_back(front_block);
        for (&(x, y), block) in desc_block_coords.iter().zip(desc_blocks.into_iter()) {
            *field.block_by_coord_mut(x, y) = block;
        }
    }
}


fn drop_descending_blocks(field: &mut Field) {
    let desc_block_coords = field
        .block_coords_with_predicate(|b| b.is_descending());
    // deepest blocks are returned first

    for (x, y) in desc_block_coords {
        let mut new_y = y;
        while !field.block_at_coord_hit_bottom_or_stationary_block(x, new_y) {
            new_y += 1;
        }
        field.swap_blocks(
            x, y,
            x, new_y,
        );
        field.block_by_coord_mut(x, new_y)
            .as_block_mut().unwrap()
            .state = BlockState::Stationary;
    }
}


fn rate_field(field: &Field) -> i64 {
    const SCORING_SEQUENCE_WEIGHT: i64 = 3;
    const EXTENSIBLE_SEQUENCE_WEIGHT: i64 = 1;
    const MAX_TOWER_HEIGHT_WEIGHT: i64 = -3;

    let mut total_rating = 0;

    // find all actual sequences (more than one)
    let sequences = field
        .get_coordinates_of_sequences(|seq| seq.coordinates.len() > 1);
    for seq in &sequences {
        if seq.coordinates.len() >= MINIMUM_SEQUENCE {
            total_rating += i64::try_from(seq.coordinates.len()).unwrap() * SCORING_SEQUENCE_WEIGHT;
        } else if seq.extensible {
            // can we at least continue the sequence with a later block?
            total_rating += i64::try_from(seq.coordinates.len()).unwrap() * EXTENSIBLE_SEQUENCE_WEIGHT;
        }
    }

    // check tower heights
    let mut max_tower_height = 0;
    for x in 0..FIELD_WIDTH_BLOCKS {
        let mut tower_height = 0;
        for y in (0..FIELD_HEIGHT_BLOCKS).rev() {
            if field.block_by_coord(x, y).is_background() {
                // top of tower; go to the next one
                break;
            } else {
                tower_height += 1;
            }
        }
        max_tower_height = max_tower_height.max(tower_height);
    }
    total_rating += max_tower_height * MAX_TOWER_HEIGHT_WEIGHT;

    total_rating
}


pub(crate) fn pick_best_move(base_field: &Field) -> Option<BestMove> {
    let desc_blocks = base_field
        .block_coords_with_predicate(|b| b.is_descending());
    if desc_blocks.len() == 0 {
        return None;
    }

    let mut fields_ratings = Vec::new();
    for rotate_count in 0..desc_blocks.len() {
        let mut rotated_field = base_field.clone();
        rotate_descending_blocks(&mut rotated_field, rotate_count);

        for column in 0..FIELD_WIDTH_BLOCKS {
            // move descending blocks to that column
            let mut columned_field = rotated_field.clone();

            // ... unless those fields are already filled
            let mut already_filled = false;
            for &(_x, y) in &desc_blocks {
                if !columned_field.block_by_coord(column, y).is_background() {
                    already_filled = true;
                    break;
                }
            }
            if already_filled {
                // this column is not an option
                continue;
            }

            for &(x, y) in &desc_blocks {
                columned_field.swap_blocks(x, y, column, y);
            }

            // now, drop the descending blocks
            drop_descending_blocks(&mut columned_field);

            // how good is this state?
            let rating = rate_field(&columned_field);

            fields_ratings.push((
                columned_field, 
                BestMove {
                    column,
                    rotate_count,
                },
                rating
            ));
        }
    }

    // pick the best field by rating
    fields_ratings.into_iter()
        .max_by_key(|(_field, _best_move, rating)| *rating)
        .map(|(_field, best_move, _rating)| best_move)
}
