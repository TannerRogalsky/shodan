use serenity::model::user::User;

pub struct Player {
    inner: User,
}

pub struct Grim {
    pub admin: User,
    pub players: Vec<Player>,
}

impl Grim {
    pub fn new(admin: User, players: Vec<User>) -> Self {
        Self {
            admin,
            players: players.into_iter().map(|p| Player { inner: p }).collect(),
        }
    }
}

pub struct Builder {
    pub creator: User,
    pub max_players: Option<usize>,
    players: Vec<User>,
}

impl Builder {
    pub fn new(creator: User) -> Self {
        Self {
            players: vec![creator.clone()],
            creator,
            max_players: None,
        }
    }

    pub fn with_max_players(creator: User, max_players: usize) -> Self {
        Self {
            players: vec![creator.clone()],
            creator,
            max_players: Some(max_players),
        }
    }

    pub fn add_player(&mut self, user: &User) {
        if let Some(max) = self.max_players {
            if max == self.players.len() {
                return; // REACHED MAX PLAYERS
            }
        }

        if self
            .players
            .iter()
            .find(|other| user.id == other.id)
            .is_some()
        {
            return; // PLAYER ALREADY JOINED
        }

        self.players.push(user.clone())
    }

    pub fn players(&self) -> &[User] {
        &self.players
    }

    pub fn ready(self) -> Grim {
        Grim::new(self.creator, self.players)
    }
}
