use bevy::prelude::*;
use clap::Parser;
use crake::{
    board::{Board, Colour, MailboxBoard, Move, Piece, PieceKind, RawMove},
    engine::Engine,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    fen: Option<String>,

    #[arg(short, long, default_value_t = false)]
    engine_colour: bool,

    // TODO: Change search depth default
    #[arg(short, long, default_value_t = 3)]
    search_depth: u8,
}

#[derive(Resource)]
struct GameData {
    board: MailboxBoard,
    engine: Engine<MailboxBoard>,
    engine_colour: Colour,
    selected_square: Option<usize>,
    players_move: bool,
}

impl GameData {
    fn new(fen: Option<&str>, engine_colour: bool, search_depth: u8) -> Self {
        let board = if let Some(fen_str) = fen {
            MailboxBoard::from_fen(fen_str)
        } else {
            MailboxBoard::new()
        };
        let turn = board.turn;
        let engine_colour = Colour::from(engine_colour);

        GameData {
            board,
            engine: Engine::new(fen, search_depth),
            engine_colour: engine_colour,
            selected_square: None,
            players_move: engine_colour != turn,
        }
    }
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
        .insert_resource(GameData::new(
            args.fen.as_deref(),
            args.engine_colour,
            args.search_depth,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse)
        .run();
}

fn square_to_transform(square: usize) -> Transform {
    let x = ((square % 8) * 100) as f32 - 350.0;
    let y = ((square / 8) * 100) as f32 - 350.0;

    Transform::from_xyz(x, y, 0.0)
}

fn make_move(
    from: usize,
    to: usize,
    commands: &mut Commands,
    pieces: &mut Query<(Entity, &mut Sprite, &mut Transform, &mut PieceData)>,
) {
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

    let board = &game_data.board.squares;

    for i in 0..64 {
        if let Some(piece) = board[i] {
            let mut filename = String::from(if piece.colour == Colour::White {
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

// TODO: Split this into various functions
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
            let mut rmove = RawMove(previous_square, square_index);
            if game_data.engine_colour == Colour::White {
                rmove = rmove.rotate();
            }

            if let Some(cmove) = game_data
                .board
                .valid_move(RawMove(previous_square, square_index))
            {
                game_data.engine.player_move(cmove);
                make_move(previous_square, square_index, &mut commands, &mut pieces);
                game_data.board.make_move(cmove);
                game_data.players_move = false;

                // TODO: Fine to block here?
                match game_data.engine.engine_move() {
                    Move::Standard(_, RawMove(from, to), _) => {
                        make_move(from, to, &mut commands, &mut pieces)
                    }
                    _ => unimplemented!(),
                }

                game_data.players_move = true;
            }

            let mut lens = pieces.transmute_lens::<&mut Sprite>();
            reset_colouring(lens.query());
            game_data.selected_square = None;
        } else {
            if let Some(piece) = game_data.board.squares[square_index] {
                if piece.colour != game_data.engine_colour {
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
