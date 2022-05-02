use rand::seq::SliceRandom;
use spirits_awaken::Connection;

pub struct Character {
    pub spirits: spirits_awaken::SpiritSelection,
    pub stats: spirits_awaken::BaseStats,
}

impl Character {
    pub fn format(&self) -> String {
        let mut out = String::new();
        use std::fmt::Write;
        writeln!(&mut out, "Spirits:").unwrap();
        for (spirit, connection) in &self.spirits {
            writeln!(&mut out, "- {:?}: {}", spirit, connection).unwrap();
        }
        writeln!(&mut out, "Stats:").unwrap();
        writeln!(&mut out, "- discipline:  {}", self.stats.discipline).unwrap();
        writeln!(&mut out, "- knowledge:   {}", self.stats.knowledge).unwrap();
        writeln!(&mut out, "- proficiency: {}", self.stats.proficiency).unwrap();
        out
    }
}

pub fn generate() -> Character {
    let ref mut rng = rand::thread_rng();
    let mut spirits = spirits_awaken::Spirit::LIST.clone();
    spirits.shuffle(rng);
    let connections = (0..spirits_awaken::SpiritSelection::MASTERY_COUNT)
        .map(|_| Connection::Mastery)
        .chain((0..spirits_awaken::SpiritSelection::EXPERTISE_COUNT).map(|_| Connection::Expertise))
        .chain(
            (0..spirits_awaken::SpiritSelection::COMPETENCE_COUNT).map(|_| Connection::Competence),
        )
        .chain(
            (0..spirits_awaken::SpiritSelection::INEPTITUDE_COUNT).map(|_| Connection::Ineptitude),
        );
    let selection =
        spirits_awaken::SpiritSelection::try_from_iter(spirits.iter().copied().zip(connections))
            .unwrap();
    let stats = spirits_awaken::BaseStats::new(&selection);
    Character {
        spirits: selection,
        stats,
    }
}
