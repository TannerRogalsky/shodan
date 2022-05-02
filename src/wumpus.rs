use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;

#[derive(Debug, Copy, Clone)]
enum State {
    Running,
    Won,
    GameOverShot,
    GameOverWumpus,
    GameOverPit,
    GameOverArrows,
}

const ROOMS: [[usize; 3]; 20] = [
    [1, 4, 7],
    [0, 2, 9],
    [1, 3, 11],
    [2, 4, 13],
    [0, 3, 5],
    [4, 6, 14],
    [5, 7, 16],
    [0, 6, 8],
    [7, 9, 17],
    [1, 8, 10],
    [9, 11, 18],
    [2, 10, 12],
    [11, 13, 19],
    [3, 12, 14],
    [5, 13, 15],
    [14, 16, 19],
    [6, 15, 17],
    [8, 16, 18],
    [10, 17, 19],
    [12, 15, 18],
];

#[derive(Debug, Copy, Clone)]
pub struct HuntTheWumpus {
    player: Player,
    rooms: [Room; 20],
    wumpus: Wumpus,
    pits: [usize; 2],
    bats: [usize; 2],
    arrows: usize,
    state: State,
}

impl HuntTheWumpus {
    pub fn new() -> Self {
        let ref mut rng = rand::thread_rng();
        let mut points = (0..20).choose_multiple(rng, 6).into_iter();
        Self {
            player: Player {
                room_index: points.next().unwrap(),
            },
            rooms: ROOMS.map(|connections| Room { connections }),
            wumpus: Wumpus {
                room_index: points.next().unwrap(),
            },
            pits: [points.next().unwrap(), points.next().unwrap()],
            bats: [points.next().unwrap(), points.next().unwrap()],
            arrows: 5,
            state: State::Running,
        }
    }

    pub fn status(&self) -> String {
        match self.state {
            State::Running => {
                let connections = self.rooms[self.player.room_index].connections;
                let [a, b, c] = connections;
                let mut out = String::new();
                use std::fmt::Write;
                writeln!(&mut out, "You're in room {}.", self.player.room_index).unwrap();
                writeln!(&mut out, "Tunnels lead to {}, {}, {}", a, b, c).unwrap();
                if connections.contains(&self.wumpus.room_index) {
                    writeln!(&mut out, "I smell a Wumpus.").unwrap();
                }
                if self.bats.iter().any(|b| connections.contains(b)) {
                    writeln!(&mut out, "Bats nearby!").unwrap();
                }
                if self.pits.iter().any(|p| connections.contains(p)) {
                    writeln!(&mut out, "I feel a draft.").unwrap();
                }
                out
            }
            State::Won => format!("You killed the Wumpus!"),
            State::GameOverShot => format!("You shoot yourself!"),
            State::GameOverWumpus => format!("HE HE HE! The Wumpus got ya!"),
            State::GameOverPit => format!("YIIIEEEE... fell in a pit."),
            State::GameOverArrows => format!("Out of arrows!"),
        }
    }

    pub fn move_player_to(&mut self, new_room: usize) -> bool {
        let current = &self.rooms[self.player.room_index];
        if current.connections.contains(&new_room) {
            self.player.room_index = new_room;
            self.update_state();
            true
        } else {
            false
        }
    }

    pub fn shoot(&mut self, at_room: usize, room_travel: usize) -> bool {
        let current = &self.rooms[self.player.room_index];
        if current.connections.contains(&at_room) {
            self.arrows -= 1;
            let ref mut rng = rand::thread_rng();
            let mut current = at_room;
            for _ in 0..room_travel {
                if self.wumpus.room_index == current {
                    self.state = State::Won;
                    return true;
                }
                if self.player.room_index == current {
                    self.state = State::GameOverShot;
                    return true;
                }
                current = self.rooms[current]
                    .connections
                    .choose(rng)
                    .copied()
                    .unwrap();
            }
            if self.arrows == 0 {
                self.state = State::GameOverArrows;
            }
            true
        } else {
            false
        }
    }

    pub fn is_running(&self) -> bool {
        std::matches!(self.state, State::Running)
    }

    fn update_state(&mut self) {
        if self.wumpus.room_index == self.player.room_index {
            self.state = State::GameOverWumpus
        } else if self.bats.contains(&self.player.room_index) {
            self.player.room_index = rand::thread_rng().gen_range(0..self.rooms.len());
            self.update_state();
        } else if self.pits.contains(&self.player.room_index) {
            self.state = State::GameOverPit
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Player {
    pub room_index: usize,
}

#[derive(Debug, Copy, Clone)]
struct Wumpus {
    pub room_index: usize,
}

#[derive(Debug, Copy, Clone)]
struct Room {
    pub connections: [usize; 3],
}
