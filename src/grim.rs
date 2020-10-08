use serenity::model::user::User;

pub struct Player {
    inner: User,
}

pub enum CardType {
    Pip,
    Face,
    Ace,
    Joker,
}

impl CardType {
    pub fn description(&self) -> &'static str {
        match self {
            CardType::Pip => "a pip card",
            CardType::Face => "a face card",
            CardType::Ace => "an ace",
            CardType::Joker => "a joker",
        }
    }
}

pub struct Deck {
    inner: Vec<CardType>,
}

impl Deck {
    pub fn new(player_count: usize) -> Self {
        let mut inner = Vec::with_capacity(player_count * 13);
        for _ in 0..player_count {
            inner.push(CardType::Ace);
            for _ in 0..9 {
                inner.push(CardType::Pip);
            }
            for _ in 0..3 {
                inner.push(CardType::Face);
            }
        }
        inner.push(CardType::Joker);

        Self { inner }
    }

    pub fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        rand::seq::SliceRandom::shuffle(self.inner.as_mut_slice(), rng)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct Grim {
    pub admin: User,
    pub players: Vec<Player>,
    pub deck: Deck,
    pub discard: Vec<CardType>,
    rng: rand::rngs::StdRng,
}

impl Grim {
    pub fn new(admin: User, players: Vec<User>) -> Self {
        let mut rng = rand::SeedableRng::from_entropy();
        let mut deck = Deck::new(players.len());
        deck.shuffle(&mut rng);

        Self {
            admin,
            players: players.into_iter().map(|p| Player { inner: p }).collect(),
            deck,
            discard: vec![],
            rng,
        }
    }

    pub fn is_player(&self, user: &User) -> bool {
        self.players.iter().find(|p| p.inner.id == user.id).is_some()
    }

    pub fn draw(&mut self) -> Option<CardType> {
        self.deck.inner.pop()
    }

    pub fn reset(&mut self) {
        self.deck = Deck::new(self.players.len());
        self.deck.shuffle(&mut self.rng);
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
