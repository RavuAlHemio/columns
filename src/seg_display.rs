use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::video::Window;


pub(crate) const SEGMENT_LENGTH: u32 = 24;
pub(crate) const SEGMENT_THICKNESS: u32 = 8;
pub(crate) const DIGIT_SPACING: i32 = 4;

pub(crate) const DIGIT_OFFSET: i32 = (SEGMENT_LENGTH as i32) + DIGIT_SPACING;


// ###### _____# ###### ###### #____# ###### ###### ###### ###### ######
// #    # _    # _    # _    # #    # #    _ #    _ _    # #    # #    #
// #____# _____# ###### ###### ###### ###### ###### _____# ###### ######
// #    # _    # #    _ _    # _    # _    # #    # _    # #    # _    #
// ###### _____# ###### ###### _____# ###### ###### _____# ###### ######


pub(crate) struct SegmentedDisplay {
    x: i32,
    y: i32,
    color: Color,
    value: u8,
}
impl SegmentedDisplay {
    pub fn new<C: Into<Color>>(x: i32, y: i32, color: C, value: u8) -> Self {
        Self {
            x,
            y,
            color: color.into(),
            value,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        assert!(self.value < 10);

        canvas.set_draw_color(self.color);

        let space_over: i32 = (SEGMENT_LENGTH - SEGMENT_THICKNESS).try_into().unwrap();

        if self.value != 1 && self.value != 4 {
            // top bar
            canvas.fill_rect(Rect::new(self.x, self.y, SEGMENT_LENGTH, SEGMENT_THICKNESS))
                .unwrap();
        }
        if self.value != 1 && self.value != 2 && self.value != 3 && self.value != 7 {
            // top-left bar
            canvas.fill_rect(Rect::new(self.x, self.y, SEGMENT_THICKNESS, SEGMENT_LENGTH))
                .unwrap();
        }
        if self.value != 5 && self.value != 6 {
            // top-right bar
            canvas.fill_rect(Rect::new(self.x + space_over, self.y, SEGMENT_THICKNESS, SEGMENT_LENGTH))
                .unwrap();
        }
        if self.value != 0 && self.value != 1 && self.value != 7 {
            // middle bar
            canvas.fill_rect(Rect::new(self.x, self.y + space_over, SEGMENT_LENGTH, SEGMENT_THICKNESS))
                .unwrap();
        }
        if self.value == 0 || self.value == 2 || self.value == 6 || self.value == 8 {
            // bottom-left bar
            canvas.fill_rect(Rect::new(self.x, self.y + space_over, SEGMENT_THICKNESS, SEGMENT_LENGTH))
                .unwrap();
        }
        if self.value != 2 {
            // bottom-right bar
            canvas.fill_rect(Rect::new(self.x + space_over, self.y + space_over, SEGMENT_THICKNESS, SEGMENT_LENGTH))
                .unwrap();
        }
        if self.value != 1 && self.value != 4 && self.value != 7 {
            // bottom bar
            canvas.fill_rect(Rect::new(self.x, self.y + 2*space_over, SEGMENT_LENGTH, SEGMENT_THICKNESS))
                .unwrap();
        }
    }

    pub fn set_value(&mut self, new_value: u8) {
        assert!(new_value < 10);
        self.value = new_value;
    }
}
