use bevy::prelude::*;
use clap::Parser;
use crake::{
    board::{Colour, Move, Piece, PieceKind},
    engine::Engine,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    fen: Option<String>,

    // TODO: Change search depth default
    #[arg(short, long, default_value_t = 3)]
    search_depth: u8,

    #[arg(short, long, default_value_t = false)]
    engine_colour: bool,
}

#[derive(Resource)]
struct GameData {
    engine: Engine,
    selected_square: Option<usize>,
    players_move: bool,
}

#[derive(Component)]
struct PieceData {
    index: usize,
    piece: Piece,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Crake".into(),
                resolution: (800, 800).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameData {
            // TODO: Change this to new function for GameData
            engine: Engine::new(
                args.fen.as_deref(),
                Colour::from(args.engine_colour),
                args.search_depth,
            ),
            selected_square: None,
            players_move: true,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse)
        .run();
}

fn square_to_transform(square: usize) -> Transform {
    let x = ((square % 8) * 100) as f32 - 350.0;
    let y = ((square / 8) * 100) as f32 - 350.0;

    Transform::from_xyz(x, y, 0.0)
}

// TODO: Better variable names
fn make_move(
    from: usize,
    to: usize,
    commands: &mut Commands,
    pieces: &mut Query<(Entity, &mut Sprite, &mut Transform, &mut PieceData)>,
) {
    // TODO: Is the & required here? Think about it
    // Each piece sprite appears in the loop once, so the moved piece won't be deleted
    for (entity, _, mut transform, mut piece_data) in pieces {
        // Delete the captured piece, if capture
        if piece_data.index == to {
            commands.entity(entity).despawn();
        }

        // Move the piece
        if piece_data.index == from {
            piece_data.index = to;
            *transform = square_to_transform(to);
        }
    }
}

fn reset_colouring(sprites: Query<&mut Sprite>) {
    for mut sprite in sprites {
        sprite.color = Color::WHITE;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, game_data: Res<GameData>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("board.png")),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    let board = game_data.engine.board();

    for i in 0..64 {
        if let Some(piece) = board[i] {
            let mut filename = String::from(if let Colour::White = piece.colour {
                "w"
            } else {
                "b"
            });
            filename.push(piece.kind.to_algebraic());
            filename.push_str(".png");

            commands.spawn((
                Sprite::from_image(asset_server.load(&filename)),
                square_to_transform(i),
                PieceData { index: i, piece },
            ));
        }
    }
}

// TODO: See what cargo fmt does with this
fn handle_mouse(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut game_data: ResMut<GameData>,
    mut pieces: Query<(Entity, &mut Sprite, &mut Transform, &mut PieceData)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) && game_data.players_move {
        // TODO: Don't just unwrap?
        let window = windows.single().unwrap();
        let position = window.cursor_position().unwrap();

        let square_index = (7 - (position.y / 100.0) as usize) * 8 + (position.x / 100.0) as usize;

        if let Some(previous_square) = game_data.selected_square {
            let cmove = Move::Standard(previous_square, square_index, None);

            if game_data.engine.valid_move(cmove) {
                game_data.engine.player_move(cmove);
                make_move(previous_square, square_index, &mut commands, &mut pieces);
                game_data.players_move = false;

                // TODO: Fine to block here?
                match game_data.engine.engine_move() {
                    Move::Standard(from, to, _) => make_move(from, to, &mut commands, &mut pieces),
                    _ => unimplemented!(),
                }

                game_data.players_move = true;
            }

            let mut lens = pieces.transmute_lens::<&mut Sprite>();
            reset_colouring(lens.query());
            game_data.selected_square = None;
        } else {
            if let Some(piece) = game_data.engine.get_square(square_index) {
                if piece.colour != game_data.engine.engine_colour {
                    game_data.selected_square = Some(square_index);

                    for (_, mut sprite, _, piece_data) in &mut pieces {
                        if piece_data.piece.colour == piece.colour
                            && piece_data.index != square_index
                        {
                            // TODO: Tweak this colour, create once?
                            sprite.color = Color::from(Srgba::new(0.9, 0.9, 0.9, 1.0));
                        }
                    }
                }
            }
        }
    }
}
