use bevy_prng::WyRand;
use bevy_rand::component::EntropyComponent;
use bevy_rand::resource::GlobalEntropy;

pub type GlobalRng = GlobalEntropy<WyRand>;
pub type RngComponent = EntropyComponent<WyRand>;
