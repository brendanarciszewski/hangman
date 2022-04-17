///    ^——————
///    |     |
///    O     |
///  \-|-/   |
///  _/\_    |
///      ____|__
///
use bevy::{app::AppExit, prelude::*, utils::HashSet};
use bevy_crossterm::prelude::*;
use crossterm::event::KeyCode;

#[derive(Debug, Default)]
struct FailedGuesses(u8);

struct LetterGuessRight(char);
struct LetterGuessWrong(char);

#[derive(Debug, Default)]
struct Guesses(HashSet<char>);

#[derive(Debug, Default)]
struct Word(String);

#[derive(Component)]
struct LetterPosition(char);

struct BodyPart {
	x: i32,
	y: i32,
	sprite: &'static str,
}

const HANGER_TOP: i32 = 2;
const HANGER: &str = r#"^——————
|     |
      |
      |
      |
  ____|__"#;
const BODY_PARTS: [BodyPart; 6] = [
	BodyPart {
		x: 0,
		y: 1,
		sprite: "O",
	},
	BodyPart {
		x: 0,
		y: 2,
		sprite: "|",
	},
	BodyPart {
		x: -2,
		y: 2,
		sprite: r#"\-"#,
	},
	BodyPart {
		x: 1,
		y: 2,
		sprite: "-/",
	},
	BodyPart {
		x: 0,
		y: 3,
		sprite: r#"\_"#,
	},
	BodyPart {
		x: -2,
		y: 3,
		sprite: "_/",
	},
];

#[bevy_main]
fn main() {
	let mut settings = CrosstermWindowSettings::default();
	settings.set_title("Hello, World!");

	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(CrosstermPlugin)
		.insert_resource(settings)
		.insert_resource(bevy::core::DefaultTaskPoolOptions::with_num_threads(1))
		.insert_resource(bevy::app::ScheduleRunnerSettings::run_loop(
			std::time::Duration::from_millis(50), // 20 FPS
		))
		.add_event::<LetterGuessRight>()
		.add_event::<LetterGuessWrong>()
		.init_resource::<Word>()
		.add_startup_system(create_hanger_system.system())
		.add_startup_system(create_word_system.system())
		.add_system(get_input.system())
		.add_system(was_correct_letter.system())
		.add_system(was_wrong_letter.system())
		.run();
}

fn create_hanger_system(
	mut commands: Commands,
	window: Res<CrosstermWindow>,
	mut sprites: ResMut<Assets<Sprite>>,
	mut stylemaps: ResMut<Assets<StyleMap>>,
) {
	commands.spawn().insert_bundle(SpriteBundle {
		sprite: sprites.add(Sprite::new(HANGER)),
		stylemap: stylemaps.add(StyleMap::default()),
		position: Position {
			x: window.width() as i32 / 5,
			y: HANGER_TOP,
			z: 0,
		},
		..Default::default()
	});
}

fn create_word_system(
	mut commands: Commands,
	mut sprites: ResMut<Assets<Sprite>>,
	mut stylemaps: ResMut<Assets<StyleMap>>,
	mut word: ResMut<Word>,
) {
	*word = Word("hello".to_string());
	for (i, ch) in word.0.char_indices() {
		commands
			.spawn().insert_bundle(SpriteBundle {
				sprite: sprites.add(Sprite::new('_')),
				stylemap: stylemaps.add(StyleMap::default()),
				position: Position {
					x: i as i32,
					y: 0,
					z: 0,
				},
				..Default::default()
			})
			.insert(LetterPosition(ch));
	}
}

fn was_correct_letter(
	mut guess_reader: EventReader<LetterGuessRight>,
	mut q: Query<(&Handle<Sprite>, &LetterPosition)>,
	mut sprites: ResMut<Assets<Sprite>>,
) {
	for letter in guess_reader.iter() {
		for sprite in q.iter_mut().filter_map(
			|(sprite, ch)| {
				if ch.0 == letter.0 {
					Some(sprite)
				} else {
					None
				}
			},
		) {
			if let Some(sprite) = sprites.get_mut(sprite) {
				sprite.update(letter.0);
			}
		}
	}
}

#[allow(clippy::too_many_arguments)]
fn was_wrong_letter(
	mut guess_reader: EventReader<LetterGuessWrong>,
	mut commands: Commands,
	window: Res<CrosstermWindow>,
	mut sprites: ResMut<Assets<Sprite>>,
	mut stylemaps: ResMut<Assets<StyleMap>>,
	mut failed_guesses: Local<FailedGuesses>, // keep a separate tally
	mut app_exit: EventWriter<AppExit>,
) {
	for _letter in guess_reader.iter() {
		let part = &BODY_PARTS[failed_guesses.0 as usize];
		commands.spawn().insert_bundle(SpriteBundle {
			sprite: sprites.add(Sprite::new(part.sprite)),
			stylemap: stylemaps.add(StyleMap::default()),
			position: Position {
				x: window.width() as i32 / 5 + part.x,
				y: HANGER_TOP + 1 + part.y,
				z: 0,
			},
			..Default::default()
		});
		failed_guesses.0 += 1;
		if failed_guesses.0 as usize >= BODY_PARTS.len() {
			app_exit.send(AppExit);
		}
	}
}

fn get_input(
	mut reader: EventReader<KeyEvent>,
	word: Res<Word>,
	mut guesses: Local<Guesses>,
	mut right_letters: EventWriter<LetterGuessRight>,
	mut wrong_letters: EventWriter<LetterGuessWrong>,
) {
	for key in reader.iter() {
		if let KeyCode::Char(c) = key.code {
			if guesses.0.insert(c) {
				if word.0.contains(c) {
					right_letters.send(LetterGuessRight(c));
				} else {
					wrong_letters.send(LetterGuessWrong(c));
				}
			}
		}
	}
}
