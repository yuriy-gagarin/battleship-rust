use serde::{Deserialize, Serialize};
use websocket::Message;

use std::fmt;

use crate::grid::{Cell, Grid};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GameMessage {
  Attack { x: i32, y: i32 },
  Answer { x: i32, y: i32, r: bool },
  Ack,
  Stop,
}

impl GameMessage {
  pub fn to_ws_msg(&self) -> Message {
    let j = serde_json::to_string(self).expect("serialization error");
    Message::text(j)
  }

  pub fn is_attack(&self) -> bool {
    match *self {
      GameMessage::Attack { .. } => true,
      _ => false,
    }
  }

  pub fn is_answer(&self) -> bool {
    match *self {
      GameMessage::Answer { .. } => true,
      _ => false,
    }
  }

  pub fn is_ack(&self) -> bool {
    *self == GameMessage::Ack
  }

  pub fn is_stop(&self) -> bool {
    *self == GameMessage::Stop
  }
}

pub struct Game {
  player: Grid,
  enemy: Grid,
}

impl Game {
  pub fn new() -> Self {
    return Game {
      player: Grid::new_random(),
      enemy: Grid::new_empty(),
    };
  }

  pub fn receive_attack(&mut self, x: i32, y: i32) -> GameMessage {
    let r = match self.player.at(x, y) {
      Cell::Empty | Cell::Tagged | Cell::Wreckage => false,
      Cell::Ship => {
        self.player.set(x, y, Cell::Wreckage);
        true
      }
    };
    GameMessage::Answer { x, y, r }
  }

  pub fn acknowledge_answer(&mut self, x: i32, y: i32, r: bool) -> GameMessage {
    match r {
      true => self.enemy.set(x, y, Cell::Wreckage),
      false => self.enemy.set(x, y, Cell::Tagged),
    };
    GameMessage::Ack
  }

  pub fn make_attack(&self, (x, y): (i32, i32)) -> GameMessage {
    GameMessage::Attack { x, y }
  }
}

impl fmt::Display for Game {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut s = "  1 2 3 4 5 6 7 8 9 10\t  1 2 3 4 5 6 7 8 9 10\n".to_string();

    for (i, (p, e)) in self.player.0.iter().zip(self.enemy.0.iter()).enumerate() {
      s.push((i as u8 + 65) as char);

      for c in p.iter() {
        s.push_str(match c {
          Cell::Empty => "  ",
          Cell::Ship => "██",
          Cell::Tagged => "»«",
          Cell::Wreckage => "▒▒",
        });
      }

      s.push('\t');
      s.push((i as u8 + 65) as char);

      for c in e.iter() {
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
