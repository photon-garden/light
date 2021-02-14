mod image;
mod manifest;
pub mod rand;
mod seed;
mod source_code;

use rand::Rand;
use seed::Seed;

pub struct Checkpoint {
    pub rand: Rand,
    pub name: String,
    pub seed: u64,
}

impl Checkpoint {
    pub fn clean_up(&self, _app: &nannou::prelude::App) {
        image::symlink_into_checkpoints_directory(self);
    }

    fn create(frame_number: u64) -> Checkpoint {
        let name = Checkpoint::get_name(frame_number);

        let seed = Seed::load();
        seed.save_to_file();
        source_code::save_current_version(&name);
        seed.clean_up_file();

        dbg!(&name);
        dbg!(seed.value);

        Checkpoint {
            name,
            seed: seed.value,
            rand: Rand::from_seed(seed.value),
        }
    }

    fn get_name(frame_number: u64) -> String {
        let name: String = frame_number.to_string();
        let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        format!("{} {}", current_time, name)
    }
}

pub fn save(app: &nannou::prelude::App) -> Checkpoint {
    let checkpoint = Checkpoint::create(app.elapsed_frames());
    image::capture_frame(&checkpoint, app);

    checkpoint
}

pub fn exit(app: &nannou::prelude::App, _model: crate::Model) {
    image::clean_up(app);
}
