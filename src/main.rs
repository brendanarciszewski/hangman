///    ^——————
///    |     |
///    O     |
///  \-|-/   |
///  _/\_    |
///      ____|__
///
use bevy::{app::AppExit, prelude::*};
use bevy_crossterm::prelude::*;
use crossterm::event::KeyCode;

#[derive(Debug, Default)]
struct FailedGuesses(u8);

struct LetterGuess(char);

#[derive(Debug, Default)]
struct Word(String);

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

	App::build()
		.add_plugins(DefaultPlugins)
		.add_plugin(CrosstermPlugin)
		.add_resource(settings)
		.add_resource(bevy::core::DefaultTaskPoolOptions::with_num_threads(1))
		.add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
			std::time::Duration::from_millis(50), // 20 FPS
		))
		.add_event::<LetterGuess>()
		.init_resource::<Word>()
		.init_resource::<FailedGuesses>()
		.add_startup_system(create_hanger_system.system())
		.add_startup_system(create_word_system.system())
		.add_system(get_input.system())
		.add_system(was_correct_letter.system())
		.add_system(was_wrong_letter.system())
		.run();
}

fn create_hanger_system(
	commands: &mut Commands,
	window: Res<CrosstermWindow>,
	mut sprites: ResMut<Assets<Sprite>>,
	mut stylemaps: ResMut<Assets<StyleMap>>,
) {
	commands.spawn(SpriteBundle {
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
	commands: &mut Commands,
	mut sprites: ResMut<Assets<Sprite>>,
	mut stylemaps: ResMut<Assets<StyleMap>>,
	mut word: ResMut<Word>,
) {
	*word = Word("hello".to_string());
	for (i, ch) in word.0.char_indices() {
		commands
			.spawn(SpriteBundle {
				sprite: sprites.add(Sprite::new('_')),
				stylemap: stylemaps.add(StyleMap::default()),
				position: Position {
					x: i as i32,
					y: 0,
					z: 0,
				},
				..Default::default()
			})
			.with(LetterPosition(ch));
	}
}

fn was_correct_letter(
	mut guess_reader: Local<EventReader<LetterGuess>>,
	guesses: Res<Events<LetterGuess>>,
	word: Res<Word>,
	mut q: Query<(&mut Handle<Sprite>, &LetterPosition)>,
	mut sprites: ResMut<Assets<Sprite>>,
) {
	for letter in guess_reader.iter(&guesses).filter(|g| word.0.contains(g.0)) {
		for mut sprite in
			q.iter_mut().filter_map(
				|(sprite, ch)| {
					if ch.0 == letter.0 {
						Some(sprite)
					} else {
						None
					}
				},
			) {
			*sprite = sprites.add(Sprite::new(letter.0))
		}
	}
}

#[allow(clippy::too_many_arguments)]
fn was_wrong_letter(
	mut guess_reader: Local<EventReader<LetterGuess>>,
	guesses: Res<Events<LetterGuess>>,
	word: Res<Word>,
	commands: &mut Commands,
	window: Res<CrosstermWindow>,
	mut sprites: ResMut<Assets<Sprite>>,
	mut stylemaps: ResMut<Assets<StyleMap>>,
	mut failed_guesses: ResMut<FailedGuesses>, // keep a separate tally
	mut app_exit: ResMut<Events<AppExit>>,
) {
	for letter in guess_reader
		.iter(&guesses)
		.filter(|g| !word.0.contains(g.0))
	{
		if contains_letter(&guesses, letter.0) {
			continue;
		}
		let part = &BODY_PARTS[failed_guesses.0 as usize];
		commands.spawn(SpriteBundle {
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
	mut reader: Local<EventReader<KeyEvent>>,
	keys: Res<Events<KeyEvent>>,
	mut letters: ResMut<Events<LetterGuess>>,
) {
	for key in reader.iter(&keys) {
		if let KeyCode::Char(c) = key.code {
			letters.send(LetterGuess(c));
		}
	}
}

fn contains_letter(guesses: &Events<LetterGuess>, letter: char) -> bool {
	let mut iter = guesses.get_reader().iter(&guesses);
	iter.next_back();
	iter.any(|g| g.0 == letter)
}
