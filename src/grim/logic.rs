use serenity::model::user::User;

pub struct Player {
    pub user: User,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

    pub fn reposition_joker<R: rand::Rng>(&mut self, ratio: f32, rng: &mut R) {
        if (0f32..=1f32).contains(&ratio) {
            let high = (ratio * self.inner.len() as f32).floor() as usize;
            let low = 0;
            let new_position = if high == low {
                low
            } else {
                rng.gen_range(low..high)
            };
            if let Some(position) = self.inner.iter().position(|c| *c == CardType::Joker) {
                self.inner.swap(position, new_position);
            }
        }
    }
}

pub struct Game {
    pub admin: User,
    pub players: Vec<Player>,
    pub deck: Deck,
    pub discard: Vec<CardType>,
    pub custom_id: Option<String>,
    rng: rand::rngs::StdRng,
}

impl Game {
    pub fn new(admin: User, players: Vec<User>, custom_id: Option<String>) -> Self {
        let mut rng = rand::SeedableRng::from_entropy();
        let mut deck = Deck::new(players.len());
        deck.shuffle(&mut rng);
        deck.reposition_joker(0.5, &mut rng);

        Self {
            admin,
            players: players.into_iter().map(|p| Player { user: p }).collect(),
            deck,
            discard: vec![],
            custom_id,
            rng,
        }
    }

    pub fn player_position(&self, user: &User) -> Option<usize> {
        self.players.iter().position(|p| p.user.id == user.id)
    }

    pub fn draw(&mut self) -> Option<CardType> {
        self.deck.inner.pop()
    }

    pub fn reset(&mut self) {
        self.deck = Deck::new(self.players.len());
        self.deck.shuffle(&mut self.rng);
    }

    pub fn reposition_joker(&mut self, ratio: f32) {
        self.deck.reposition_joker(ratio, &mut self.rng)
    }
}

pub struct Builder {
    pub creator: User,
    pub custom_id: Option<String>,
    players: Vec<User>,
}

impl Builder {
    pub fn new(creator: User) -> Self {
        Self {
            players: vec![creator.clone()],
            creator,
            custom_id: None,
        }
    }

    pub fn new_with_custom_id(creator: User, custom_id: String) -> Self {
        Self {
            players: vec![creator.clone()],
            creator,
            custom_id: Some(custom_id),
        }
    }

    pub fn add_player(&mut self, user: &User) {
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

    pub fn ready(self) -> Game {
        Game::new(self.creator, self.players, self.custom_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reposition_joker_test() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let mut deck = Deck::new(1);
            deck.shuffle(&mut rng);
            assert!(deck.inner.contains(&CardType::Joker));

            deck.reposition_joker(0., &mut rng);
            assert_eq!(deck.inner[0], CardType::Joker);

            deck.reposition_joker(0.5, &mut rng);
            assert!(deck.inner.contains(&CardType::Joker));
            assert_ne!(deck.inner.last().unwrap(), &CardType::Joker);

            deck.reposition_joker(1., &mut rng);
            assert!(deck.inner.contains(&CardType::Joker));
        }
    }
}
