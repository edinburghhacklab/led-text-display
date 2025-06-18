use logic::DisplayLogic;
use rpi_led_panel::{Canvas, RGBMatrix};

pub struct Display {
    matrix: RGBMatrix,
    canvas: Box<Canvas>,
    logic: DisplayLogic<Canvas>,
}

impl Display {
    pub fn new(matrix: RGBMatrix, canvas: Box<Canvas>, logic: DisplayLogic<Canvas>) -> Self {
        Self {
            matrix,
            canvas,
            logic,
        }
    }

    pub fn main_loop(mut self) -> Result<(), &'static str> {
        loop {
            self.logic
                .draw(&mut self.canvas)
                .map_err(|_| "error drawing")?;
            self.canvas = self.matrix.update_on_vsync(self.canvas);
        }
    }
}
