use rand::Rng;

const ADJECTIVES: &[&str] = &[
    "Amber", "Blessed", "Bright", "Calm", "Coral", "Cosmic", "Crimson", "Crystal",
    "Dancing", "Drifting", "Dusky", "Emerald", "Fading", "Fierce", "Gentle", "Golden",
    "Hidden", "Hollow", "Ivory", "Jade", "Kindred", "Lazy", "Lunar", "Midnight",
    "Misty", "Noble", "Opal", "Pale", "Quiet", "Radiant", "Rolling", "Ruby",
    "Sacred", "Silent", "Silver", "Solar", "Tender", "Velvet", "Violet", "Wandering",
];

const NOUNS: &[&str] = &[
    "Acorn", "Basil", "Birch", "Canyon", "Cedar", "Clover", "Compass", "Coral",
    "Cricket", "Crystal", "Dagger", "Dawn", "Falcon", "Finch", "Flint", "Harbor",
    "Heron", "Lantern", "Lotus", "Maple", "Meadow", "Nebula", "Orchid", "Pebble",
    "Phoenix", "Prism", "Quartz", "Raspberry", "Raven", "River", "Sequoia", "Sparrow",
    "Storm", "Summit", "Thunder", "Tide", "Tulip", "Wren", "Willow", "Zenith",
];

pub fn generate_name() -> String {
    let mut rng = rand::thread_rng();
    let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
    let noun = NOUNS[rng.gen_range(0..NOUNS.len())];
    format!("{adj} {noun}")
}
