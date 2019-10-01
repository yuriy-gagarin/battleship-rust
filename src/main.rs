use env_logger;
use getopts::Options;
use regex::Regex;
use serde_json::from_str;
use websocket::client::ClientBuilder;
use websocket::sender::Writer;
use websocket::sync::Server;
use websocket::OwnedMessage;

use std::env;
use std::error::Error;
use std::io::{self, BufRead};
use std::net::TcpStream;

mod game;
mod grid;
mod misc;

use game::{Game, GameMessage};
use misc::take_one;

fn client(addr: String) {
  let conn = ClientBuilder::new(format!("ws://{}", addr.as_str()).as_str())
    .expect("unable to connect")
    .connect_insecure()
    .expect("unable to connect");

  let conn_addr = conn.local_addr().unwrap();
  println!("connected to {}", conn_addr);

  let (mut ws_receiver, mut ws_sender) = conn.split().unwrap();
  let mut messages = ws_receiver
    .incoming_messages()
    .filter_map(game_message_filter);

  match game_loop(messages.by_ref(), &mut ws_sender, false) {
    Ok(_) => {}
    Err(e) => println!("game loop exited with error {}", e),
  };

  ws_sender.shutdown_all().unwrap();
}

fn server(addr: String) {
  let server = Server::bind(addr.as_str()).expect("unable to bind socket");
  println!("listening on {}\nwaiting for the connection...", addr);

  let conn = server
    .filter_map(Result::ok)
    .filter_map(|c| c.accept().ok())
    .next()
    .expect("can't get the connection");

  let conn_addr = conn.local_addr().unwrap();
  println!("connection from {} ", conn_addr);

  let (mut ws_receiver, mut ws_sender) = conn.split().unwrap();
  let mut messages = ws_receiver
    .incoming_messages()
    .filter_map(game_message_filter);

  match game_loop(messages.by_ref(), &mut ws_sender, true) {
    Ok(_) => {}
    Err(e) => println!("game loop exited with error: {}", e),
  };

  ws_sender.shutdown_all().unwrap();
}

fn game_loop(
  messages: &mut dyn Iterator<Item = GameMessage>,
  sender: &mut Writer<TcpStream>,
  server: bool,
) -> Result<(), Box<dyn Error>> {
  let mut game = Game::new();
  let mut state = GameMessage::Ack;

  println!("entered game loop");

  // order of messages
  // -> attack
  // <- answer
  // -> ack
  // <- attack
  // -> answer
  // <- ack
  // repeat

  // server goes second, so it has to wait for an attack
  if server {
    state = take_one(messages, |m| m.is_attack() || m.is_stop())?;
  }

  loop {
    state = match state {
      GameMessage::Attack { x, y } => {
        println!("got attack {} {}", x, y);
        let msg = game.receive_attack(x, y);
        println!("{}", game);
        let msg = msg.to_ws_msg();
        sender.send_message(&msg)?;
        take_one(messages, |m| m.is_ack() || m.is_stop())?
      }
      GameMessage::Answer { x, y, r } => {
        println!("got answer {} {} {}", x, y, r);
        let msg = game.acknowledge_answer(x, y, r);
        println!("{}", game);
        let msg = msg.to_ws_msg();
        sender.send_message(&msg)?;
        take_one(messages, |m| m.is_attack() || m.is_stop())?
      }
      GameMessage::Ack => {
        println!("got ack");
        let msg = game.make_attack(get_input());
        let msg = msg.to_ws_msg();
        sender.send_message(&msg)?;
        take_one(messages, |m| m.is_answer() || m.is_stop())?
      }
      GameMessage::Stop => {
        return Err("got stop message".into());
      }
    };
  }
}

fn game_message_filter<E>(message: Result<OwnedMessage, E>) -> Option<GameMessage> {
  match message.ok()? {
    OwnedMessage::Close(_) => Some(GameMessage::Stop),
    OwnedMessage::Text(t) => from_str(t.as_str()).ok()?,
    _ => None,
  }
}

fn get_input() -> (i32, i32) {
  let stdin = io::stdin();
  let (x, y) = stdin
    .lock()
    .lines()
    .filter_map(Result::ok)
    .filter_map(parse_input)
    .next()
    .expect("io error");

  println!("{} {}", x, y);
  (x, y)
}

fn parse_input(s: String) -> Option<(i32, i32)> {
  let re = Regex::new(r"^(?P<letter>[A-Ja-j])(?P<number>10|[1-9])$").expect("regex error");

  let c = re.captures(s.trim())?;
  let letter = c.name("letter")?.as_str().to_lowercase().chars().nth(0)?;
  let number = c.name("number")?.as_str().parse::<i32>().ok()?;

  Some((letter as i32 - 97, number - 1))
}

fn parse_args() -> (bool, String) {
  let args: Vec<String> = env::args().collect();

  let mut opts = Options::new();
  opts.optflag("s", "server", "run as a server");
  opts.optopt("a", "addr", "addr", "");

  let matches = opts.parse(&args[1..]).expect("couldn't parse arguments");

  let is_server = matches.opt_present("s");
  let addr = matches.opt_str("a").unwrap_or("127.0.0.1:8080".to_string());

  (is_server, addr)
}

fn main() {
  env_logger::init();
  let (is_server, addr) = parse_args();

  if is_server {
    server(addr)
  } else {
    client(addr)
  }
}
