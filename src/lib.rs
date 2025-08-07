use np_utils::file_watch;

use rand::seq::IndexedRandom;

use std::{
    error::Error,
    path::PathBuf,
    collections::HashMap,
    sync::{Arc, Mutex}
};

const SEPARATOR: &str = ",";
const CARDS: &[&str] = &[
    "0: The Fool",
    "I: Magician",
    "II: High Priestess",
    "III: Empress",
    "IV: Emperor",
    "V: Hierophant",
    "VI: Lovers",
    "VII: Chariot",
    "VIII: Strength",
    "IX: Hermit",
    "X: Wheel of Fortune",
    "XI: Justice",
    "XII: The Hanged Man",
    "XIII: Death",
    "XIV: Temperance",
    "XV: Devil",
    "XVI: The Tower",
    "XVII: The Star",
    "XVIII: The Moon",
    "XIX: The Sun",
    "XX: Judgement",
    "XXI: The World"
];

pub struct Tarot {
    affinity: Arc<Mutex<HashMap<String, i32>>>,
}

impl Tarot {
    pub fn new(affinity_file: PathBuf) -> Result<Tarot, Box<dyn Error>> {
        let tarot = Tarot { 
            affinity: Arc::new(Mutex::new(HashMap::new()))
        };
        log::debug!("Initializing cards affinity...");
        let affinity_data = std::fs::read_to_string(&affinity_file)?;
        log::debug!("Starting affinity watcher...");
        update_affinity(affinity_data, Arc::clone(&tarot.affinity));
        tarot.start_watch(affinity_file);
        Ok(tarot)
    }

    pub fn draw_many(&self, num: u16) -> Result<Vec<(String, i32)>, Box<dyn Error>> {
        if num > 22 {
            return Err("Drew too many, there's only 22 cards in deck!".into());
        }
        let mut result = Vec::new();
        let draw = CARDS.choose_multiple(&mut rand::rng(), num as usize);
        for card in draw {
            let affinity = self.get_affinity(card)?;
            result.push((card.to_string(), affinity));
        }
        Ok(result)
    }

    pub fn draw(&self) -> Result<(String, i32), Box<dyn Error>> {
        let card = CARDS
            .choose(&mut rand::rng())
            .expect("my const array is never empty");
        let card = if rand::random::<u8>() > 16 {
            card.to_string()
        } else {
            format!("{} (Reversed)", card)
        };
        let affinity = self.get_affinity(&card)?;
        Ok((card, affinity))
    }

    fn get_affinity(&self, card: &str) -> Result<i32, Box<dyn Error>> {
        let affinity = self.affinity.lock();
        if let Err(e) = affinity {
            log::warn!("Error obtaining affinity: {}", e);
            return Ok(0);
        }
        let affinity = affinity.unwrap();
        if !affinity.contains_key(card) {
            return Err(format!("Couldn't get affinity for {}", card).into())
        };
        Ok(*affinity.get(card).unwrap())
    }

    fn start_watch(&self, affinity_file: PathBuf) {
        let reference = Arc::clone(&self.affinity);
        file_watch(
            affinity_file,
            1000 * 60,
            move |data| update_affinity(data, Arc::clone(&reference)));
    }
}

fn update_affinity(data: String, affinity: Arc<Mutex<HashMap<String, i32>>>) {
    let affinity = affinity.lock();
    if let Err(e) = affinity {
        log::error!("Failed to update card affinity");
        return;
    }
    let mut affinity = affinity.unwrap();
    data.lines().enumerate().for_each(|(indx, line)| {
        let entry = line.split(SEPARATOR).collect::<Vec<_>>();
        if entry.len() < 2 {
            log::warn!("Not enough entries for affinity at line {}: {}", indx+1, line);
        } else {
            if let Ok(value) = str::parse::<i32>(entry[1]) {
                affinity.insert(entry[0].to_owned(), value);
            } else {
                log::warn!("Error parsing affinity to i32 at line {}: {}", indx+1, entry[1])
            }
        }
    });
}

