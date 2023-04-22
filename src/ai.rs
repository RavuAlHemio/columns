use crate::{FIELD_WIDTH_BLOCKS, MINIMUM_SEQUENCE};
use crate::model::Field;


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BestMove {
    pub column: u32,
    pub rotate_count: usize,
}


fn rotate_descending_blocks(field: &mut Field, count: usize) {
    for _ in 0..count {
        field.rotate_descending_blocks();
    }
}



fn rate_field(field: &Field) -> Vec<i64> {
    let mut criteria: Vec<i64> = Vec::new();

    // the first criterion is the score
    let mut field_score = 0;
    let scoring_sequences = field
        .get_coordinates_of_sequences(|seq| seq.coordinates.len() >= MINIMUM_SEQUENCE);
    if scoring_sequences.len() > 0 {
        // simulate what this would do
        let mut scoring_field = field.clone();
        while scoring_field.disappear_scoring_sequences(&mut field_score) {
            scoring_field.immediately_remove_disappearing_blocks();
            scoring_field.immediately_drop_gravity_blocks();
        }
    }
    criteria.push(field_score.try_into().unwrap());

    // the next criterion is the number of extensible sequences
    let ext_seq_count = field
        .get_coordinates_of_sequences(|seq| seq.coordinates.len() > 1)
        .iter()
        .filter(|seq| seq.extensible)
        .count();
    criteria.push(ext_seq_count.try_into().unwrap());

    // the next criterion is the height of the highest tower
    // (negated to ensure lowest = best)
    let mut max_tower_height: i64 = 0;
    for x in 0..FIELD_WIDTH_BLOCKS {
        let tower_height: i64 = field.tower_height(x).try_into().unwrap();
        max_tower_height = max_tower_height.max(tower_height);
    }
    criteria.push(-max_tower_height);

    criteria
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
                let block = columned_field.block_by_coord(column, y);
                if !block.is_background() && !block.as_block().unwrap().state.is_descending() {
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
            columned_field.hand_descending_blocks_to_gravity();
            columned_field.immediately_drop_gravity_blocks();

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
        .max_by_key(|(_field, _best_move, rating)| rating.clone())
        .map(|(_field, best_move, _rating)| best_move)
}
