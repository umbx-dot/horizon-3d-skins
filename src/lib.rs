pub mod voxels;

use mod_api::{
    self, ClientHost, ClientTickPostEvent, Hooks, Registrar, RegistrationContext, SettingSpec,
};
use std::sync::atomic::{AtomicU32, Ordering};

const ID: &str = "skins_3d";
const REACHES: [u32; 4] = [8, 16, 32, 64];
const DEFAULT_REACH: usize = 2;

static REACH: AtomicU32 = AtomicU32::new(REACHES[DEFAULT_REACH]);

pub fn reach() -> u32 {
    REACH.load(Ordering::Relaxed)
}

fn reach_of(choice: Option<u32>) -> u32 {
    choice
        .and_then(|i| REACHES.get(i as usize).copied())
        .unwrap_or(REACHES[DEFAULT_REACH])
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        let options = REACHES.map(|r| format!("{ID}.reach.{r}")).to_vec();
        let spec = SettingSpec::choice("reach", "skins_3d.setting.reach", options, DEFAULT_REACH);
        if let Err(error) = r.register_setting(spec) {
            eprintln!("[{ID}] setting refused: {error:?}");
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _: &mut ClientTickPostEvent) {
        REACH.store(reach_of(host.setting_choice("reach")), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reach_comes_from_the_choice_or_its_default() {
        assert_eq!(reach_of(Some(0)), 8);
        assert_eq!(reach_of(Some(3)), 64);
        assert_eq!(reach_of(Some(9)), 32);
        assert_eq!(reach_of(None), 32);
    }
}
