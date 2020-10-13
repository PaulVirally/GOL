#[derive(Copy, Clone)]
pub enum Cell {
    Dead = 0,
    Alive = 1
}

impl From<u8> for Cell {
    fn from(rhs: u8) -> Self {
        match rhs {
            0 => Cell::Dead,
            _ => Cell::Alive
        }
    }
}

impl From<Cell> for u8 {
    fn from(rhs: Cell) -> u8 {
        match rhs {
            Cell::Dead => 0,
            _ => 1
        }
    }
}
