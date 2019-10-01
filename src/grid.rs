use itertools::Itertools;
use rand::{thread_rng, Rng};

use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub enum Cell {
  Empty,
  Tagged,
  Ship,
  Wreckage,
}

pub struct Grid(pub [[Cell; 10]; 10]);

impl Grid {
  pub fn new_empty() -> Self {
    Grid([[Cell::Empty; 10]; 10])
  }

  pub fn new_random() -> Self {
    random_grid()
  }

  pub fn at(&self, x: i32, y: i32) -> Cell {
    self.0[x as usize][y as usize]
  }

  pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
    self.0[x as usize][y as usize] = cell;
  }
}

fn random_grid() -> Grid {
  let mut grid = Grid::new_empty();
  let mut rng = thread_rng();

  for ship_length in [5, 4, 4, 3, 3, 3].iter() {
    for _try in 0..100 {
      let x = rng.gen_range(0, 10);
      let y = rng.gen_range(0, 10);
      let d: bool = rand::random();
      if place_ship(&mut grid, x, y, *ship_length, d).is_ok() {
        break;
      };
    }
  }

  grid
}

fn place_ship(
  grid: &mut Grid,
  mut x: i32,
  mut y: i32,
  length: i32,
  horizontal: bool,
) -> Result<(), &str> {
  if !is_valid_placement(grid, x, y, length, horizontal) {
    return Err("ship placement is incorrect");
  };

  for _ in 0..length {
    grid.0[x as usize][y as usize] = Cell::Ship;
    if horizontal {
      y += 1;
    } else {
      x += 1;
    };
  }

  Ok(())
}

fn is_valid_placement(g: &Grid, x: i32, y: i32, length: i32, horizontal: bool) -> bool {
  let (left_x, top_y, right_x, bottom_y, end) = if horizontal {
    (x - 1, y - 1, x + 1, y + length, y + length)
  } else {
    (x - 1, y - 1, x + length, y + 1, x + length)
  };

  if end > 10 {
    return false;
  };

  let mut iter = (left_x..right_x + 1).cartesian_product(top_y..bottom_y + 1);

  if iter.any(|v| !is_free(g, v)) {
    return false;
  };

  true
}

fn is_free(g: &Grid, (x, y): (i32, i32)) -> bool {
  if x > 9 || y > 9 || x < 0 || y < 0 {
    return true;
  }

  if g.0[x as usize][y as usize] == Cell::Empty {
    return true;
  }

  false
}

impl fmt::Display for Grid {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut s = "  1 2 3 4 5 6 7 8 9 10".to_string();

    for (i, row) in self.0.iter().enumerate() {
      s.push((i as u8 + 65) as char);

      for c in row.iter() {
        s.push_str(match c {
          Cell::Empty => "  ",
          Cell::Ship => "██",
          Cell::Tagged => "»«",
          Cell::Wreckage => "▒▒",
        });
      }

      s.push('\n');
    }

    write!(f, "{}", s)
  }
}
